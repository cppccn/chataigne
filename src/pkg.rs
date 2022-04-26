use crate::{
    common::types::{
        DepVal, Dependency, GitTarget, LocalTarget, Package, PkgFile, SrcVal, _Package, _PkgFile,
    },
    settings::Settings,
};
use anyhow::{bail, Result};
use config::{Config, File, Value};
use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
};

impl DepVal {
    fn adapt(dependencies: Option<HashMap<String, Value>>) -> Option<HashMap<String, DepVal>> {
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

// todo: put all `deserializable` things into a sub package dedicated.
//       - PkgFileInternal
//       - LocalDependency
//       - GitDependency

impl PkgFile {
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
        if self.dev_dependencies.is_some() && compile_level >= 2 {
            map.extend(
                self.dev_dependencies
                    .clone()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone())),
            );
        }
        if self.test_dependencies.is_some() && compile_level == 3 {
            map.extend(
                self.test_dependencies
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
}

impl TryFrom<_Package> for Package {
    type Error = anyhow::Error;
    fn try_from(p: _Package) -> Result<Self, Self::Error> {
        let src = match p.src {
            Some(t) => {
                if let Ok(git) = t.clone().try_into() {
                    Some(SrcVal::Git(git))
                } else if let Ok(local) = t.try_into() {
                    Some(SrcVal::Local(local))
                } else {
                    bail!("unexpected src value")
                }
            }
            None => None,
        };
        Ok(Self {
            name: p.name,
            version: p.version,
            description: p.description,
            src,
            repostory: p.repostory,
        })
    }
}

impl From<_PkgFile> for PkgFile {
    fn from(i: _PkgFile) -> Self {
        Self {
            package: i.package.try_into().unwrap(),
            dependencies: DepVal::adapt(i.dependencies),
            dev_dependencies: DepVal::adapt(i.dev_dependencies),
            test_dependencies: DepVal::adapt(i.test_dependencies),
            lib: i.lib,
        }
    }
}

pub fn read(path: Option<String>) -> Result<PkgFile> {
    let mut pkg_cfg = Config::default();
    pkg_cfg.merge(File::with_name(
        &path.unwrap_or_else(|| "luc.toml".to_string()),
    ))?;
    //todo: if lib, shared lib, dyn lib a defined in the same pkg file, print warning
    let internal: _PkgFile = pkg_cfg.try_into()?;
    Ok(internal.into())
}

impl Package {
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
}
