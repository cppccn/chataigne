use crate::common::tools::unwrap_path_patterns;
use crate::common::types::{
    ConfigPackage, DepVal, Dependency, GitTarget, LocalTarget, Package, PackagePaths, StaticLib,
};
use crate::settings::Settings;
use crate::DEFAULT_PACKAGE_FILE_NAME;
use anyhow::{bail, Result};
use config::{Config, File, Value};
use glob::Pattern;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use tracing::debug;
use walkdir::WalkDir;

impl DepVal {
    pub fn adapt(dependencies: Option<HashMap<String, Value>>) -> Option<HashMap<String, DepVal>> {
        dependencies.as_ref()?;

        let mut ret = HashMap::new();
        for (k, v) in dependencies.unwrap() {
            if let Ok(version) = v.clone().try_into::<String>() {
                ret.insert(k, DepVal::Version(version));
            } else if let Ok(local) = v.clone().try_into::<LocalTarget>() {
                ret.insert(k, DepVal::Path(PathBuf::from(local.path)));
            } else if let Ok(git) = v.try_into::<GitTarget>() {
                ret.insert(k, DepVal::Git(git));
            }
        }
        Some(ret)
    }
}

impl Package {
    /// Read a package file with name [DEFAULT_PACKAGE_FILE_NAME] or with the
    /// given `path`.
    pub fn read(path: Option<String>) -> Result<Self> {
        let mut pkg_cfg = Config::default();
        pkg_cfg.merge(File::with_name(
            &path.unwrap_or_else(|| DEFAULT_PACKAGE_FILE_NAME.to_string()),
        ))?;
        //todo: if lib, shared lib, dyn lib a defined in the same pkg file, print warning
        let internal: ConfigPackage = pkg_cfg.try_into()?;
        Ok(internal.into())
    }

    /// Return an owned pathbuf to the target directory.
    ///
    /// Target is computed with format ${cache_dir}/${pkg_name}_${pkg_version}
    pub fn target_dir(&self, settings: &Settings) -> PathBuf {
        // todo: memoization
        let mut p = settings.project_dirs.cache_dir().to_path_buf();
        p.push(format!(
            "{}_{}",
            self.pkg_description.name, self.pkg_description.version
        ));
        p
    }

    /// Create all directories and return the .o path respectiv to the given
    /// `src_file`.
    ///
    /// Example:
    /// ${cache_dir}/${pkg_name}/${rel_src_file_path.cpp} =>
    /// ${cache_dir}/${target_dir}/${rel_src_file_path.o}
    pub fn object_path(&self, src_file: &Path, settings: &Settings) -> PathBuf {
        let mut target_dir = self.target_dir(settings);
        target_dir.push(src_file);
        let ret = target_dir.with_extension("o");
        target_dir.pop();
        std::fs::create_dir_all(target_dir).unwrap();
        ret
    }

    /// Get the dependences of the packages corresponding to the
    /// given compile level.
    ///
    /// Note: dependencies are comulative, e.g. `test` level contains
    ///       also `dev` and `release` dependencies.
    pub fn get_dependencies(&self, compile_level: usize) -> VecDeque<Dependency> {
        let mut map = HashMap::new();
        if self.dependencies.is_some() && compile_level >= 1 {
            map.extend(
                self.dependencies
                    .clone()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone())),
            )
        }
        if self.dev.dependencies.is_some() && compile_level >= 2 {
            map.extend(
                self.dev
                    .dependencies
                    .clone()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone())),
            );
        }
        if self.test.dependencies.is_some() && compile_level == 3 {
            map.extend(
                self.test
                    .dependencies
                    .clone()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone())),
            );
        }
        VecDeque::from_iter(map.iter().map(|(pkg_name, value)| Dependency {
            name: pkg_name.clone(),
            desc: value.clone(),
        }))
    }

    /// Get patterns of files and folders we want to ignores corresponding to
    /// the compilation level. 1: release, 2: dev, 3: test. Ignore doesn't
    /// inherits like the dependencies.
    pub fn get_ignore(&self, compile_level: usize) -> &Vec<String> {
        match compile_level {
            2 => &self.dev.ignore,
            3 => &self.test.ignore,
            _ => &self.ignore,
        }
    }

    /// Return a library or an error if the lib is None
    pub fn get_lib(&self) -> Result<&StaticLib> {
        match &self.lib {
            Some(lib) => Ok(lib),
            None => bail!(
                "error {} doesn't contains a [lib] description",
                self.pkg_description.name
            ),
        }
    }

    /// Get the opt of the current package. Ignore doesn't
    /// inherits like the dependencies.
    pub fn get_opt(&self, compile_level: usize) -> Vec<String> {
        match compile_level {
            2 => self.dev.opt.clone(),
            3 => self.test.opt.clone(),
            _ => {
                if let Some(lib) = &self.lib {
                    lib.opt.clone()
                } else {
                    self.opt.clone()
                }
            }
        }
    }

    pub fn get_sources(&self, compile_level: usize) -> &Vec<String> {
        match compile_level {
            2 => &self.dev.sources,
            3 => &self.test.sources,
            _ => &self.sources,
        }
    }

    pub fn get_includes(&self, compile_level: usize) -> &Vec<String> {
        match compile_level {
            2 => &self.dev.includes,
            3 => &self.test.includes,
            _ => &self.includes,
        }
    }

    /// Determine the headers folder that need to be included and the sources
    /// that need to be build. Take the `local_path` that is the deduced path
    /// of the package repository (after being loaded if necessary).
    /// Use the `package` to override sources and headers if necessary depending
    /// on the given `compile_level`. Ignore all path matching with something
    /// in the `ignore`.
    ///
    /// By default, find all `.cpp` files as `source_files` and all parent folder
    /// of any `.h` file. A package can overides these extensions with the `build`
    /// and `include` parameters. Inverse is also possible, all path containing
    /// something in the `ignore` parameter in package file will be really ignored.
    fn get_paths(&self, local_path: &Path, compile_level: usize) -> Result<PackagePaths> {
        // todo: return an error if a cyclic path found.
        // todo: add `ignore` parameter in pkg_file.
        // todo: find any file that end with the given extensions from pkg_file.
        debug!("package path walk from {}", local_path.to_string_lossy());
        let mut source_files = HashSet::new();
        let mut header_folders = HashSet::new();
        // note, I don't remember why I removed the base :( but for
        // now I need to try without.
        // let base = local_path.to_path_buf();
        let ignore = self.get_ignore(compile_level);
        'walk: for entry in WalkDir::new(local_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            for ig in ignore {
                if Pattern::new(ig).unwrap().matches_path(entry.path()) {
                    debug!("ignoring {:?}", entry.path());
                    continue 'walk;
                }
            }
            let f_name = entry.file_name().to_string_lossy();
            debug!("path {:?}", entry.path());
            if f_name.ends_with(".cpp") {
                source_files.insert(entry.path().to_path_buf());
            }
            // todo: don't add sub folders into result. add to ignore
            if f_name.ends_with(".h") {
                let e = entry.path().parent().unwrap();
                debug!("include folder found at '{}'", e.to_string_lossy(),);
                header_folders.insert(e.to_path_buf());
            }
        }
        Ok(PackagePaths {
            header_folders,
            source_files,
        })
    }

    pub fn root_package_paths(&self, compile_level: usize) -> Result<PackagePaths> {
        let local_path = Path::new(".");
        let mut source_files = unwrap_path_patterns(local_path, self.get_sources(compile_level));
        let mut header_folders = unwrap_path_patterns(local_path, self.get_includes(compile_level));
        if source_files.is_empty() || header_folders.is_empty() {
            let pkg_paths = self.get_paths(local_path, compile_level).unwrap();
            if source_files.is_empty() {
                source_files = pkg_paths.source_files;
            }
            if header_folders.is_empty() {
                header_folders = pkg_paths.header_folders;
            }
        }

        Ok(PackagePaths {
            header_folders,
            source_files,
        })
    }

    /// Basically the same thing as [get_path] but used when the package is
    /// deduced from a library.
    pub fn lib_package_path(&self, local_path: &Path) -> Result<PackagePaths> {
        debug!("compile paths for {}", local_path.to_string_lossy());
        let lib = match &self.lib {
            Some(lib) => lib,
            None => bail!(
                "error: {} has no [lib] description",
                self.pkg_description.name
            ),
        };

        let mut source_files = unwrap_path_patterns(local_path, &lib.builds);
        let mut header_folders = unwrap_path_patterns(local_path, &lib.headers);
        // todo: replace unwraps with error management

        if source_files.is_empty() || header_folders.is_empty() {
            let pkg_paths = self.get_paths(local_path, 0)?;
            if source_files.is_empty() {
                source_files = pkg_paths.source_files;
            }
            if header_folders.is_empty() {
                header_folders = pkg_paths.header_folders;
            }
        }

        println!("headers lib: {:?}", header_folders);

        Ok(PackagePaths {
            header_folders,
            source_files,
        })
    }
}
