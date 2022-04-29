use crate::{
    cmd::git_clone,
    common::types::DepVal,
    pkg::read,
    settings::{self, Layer, Settings},
};

use super::types::{Dependency, PackagePaths, PkgFile, StaticLib};
use anyhow::{bail, Result};
use glob::Pattern;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use tracing::debug;
use walkdir::WalkDir;

const DEFAULT_PACKAGE_FILE_NAME: &str = "luc.toml";

/// Return include paths and source files path from a root file.
///
/// By default, find all `.cpp` files as `source_files` and all parent folder
/// of any `.h` file. A package can overides these extensions with the `build`
/// and `include` parameters. Inverse is also possible, all path containing
/// something in the `ignore` parameter in package file will be really ignored.
pub fn package_paths(path: &Path, ignore: &Vec<String>) -> Result<PackagePaths> {
    // todo: return an error if a cyclic path found.
    // todo: add `ignore` parameter in pkg_file.
    // todo: find any file that end with the given extensions from pkg_file.
    debug!("package path walk from {}", path.to_string_lossy());
    let mut source_files = HashSet::new();
    let mut header_folders = HashSet::new();
    let base = path.to_path_buf();
    'walk: for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        debug!("ignore paths {:?}", ignore);

        for ig in ignore {
            if Pattern::new(ig).unwrap().matches_path(entry.path()) {
                debug!("ignoring {:?}", entry.path());
                continue 'walk;
            }
        }
        let f_name = entry.file_name().to_string_lossy();
        debug!("path {:?}", entry.path());

        if f_name.ends_with(".cpp") {
            source_files.insert(entry.path().strip_prefix(&base).unwrap().to_path_buf());
        }

        // todo: don't add sub folders into result. add to ignore
        if f_name.ends_with(".h") {
            let e = entry.path().parent().unwrap().strip_prefix(&base).unwrap();
            debug!("include folder found at {}", e.to_string_lossy());
            header_folders.insert(e.to_path_buf());
        }
    }
    Ok(PackagePaths {
        header_folders,
        source_files,
    })
}

// todo: better manage that. Maybe everywhere we could use a Pkg instead of
//       PkgFile that implement all of that stuff
pub fn lib_package_path(
    lib: &StaticLib,
    src_path: &Path,
    ignore: &Vec<String>,
) -> Result<PackagePaths> {
    debug!("compile paths for {}", src_path.to_string_lossy());
    // todo: replace unwraps with error management
    let mut source_files = HashSet::from_iter(lib.builds.iter().map(|src| src.into()));
    let mut header_folders = HashSet::from_iter(lib.headers.iter().map(|src| src.into()));

    if source_files.is_empty() || header_folders.is_empty() {
        let pkg_paths = package_paths(src_path, ignore).unwrap();
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

pub fn concat(root: &Path, file: &str) -> String {
    let mut ret = root.to_path_buf();
    ret.push(file);
    ret.to_str().unwrap().to_string()
}

/// Find layer directory, in case of a git repository, clone
pub fn get_layer_directory(layer: &Layer, settings: &Settings) -> Result<PathBuf> {
    match layer {
        settings::Layer::Git(git) => {
            let mut p = settings.project_dirs.cache_dir().to_path_buf();
            p.push(sha256::digest(&git.git));
            git_clone(&git.git, &None, &p);
            Ok(p)
        }
        settings::Layer::Dir(dir) => {
            if std::fs::metadata(&dir.path).is_ok() {
                Ok(PathBuf::from(dir.path.clone()))
            } else {
                bail!("Broken link to layer {}", dir.path)
            }
        }
    }
}

// todo: return better errors, readable by final user
// todo, clone git layers
// todo, clone git dependency

/// Return the pkg file of the dependency. Done for each dependencies in
/// the full project including [sub]+ dependencies.
///
/// The search of the sources of the dependencies can be done here if the
/// dependency value is a git target. Otherwise, we just check in the layer or
/// in the local if toml exist.
///
/// In cases where we don't clone the sources. It's done later just before
/// running the compilation of each lib. It's done in [compile_lib] with the
/// usefull function [checkout].
pub fn find_pkg(dependency: &Dependency, settings: &Settings) -> Result<PkgFile> {
    debug!("Load dependency {} pkgfile", dependency.name);
    match &dependency.desc {
        DepVal::Version(version) => {
            for layer in &settings.layers {
                // todo, by default check filesystem for layers, but if
                // git repository, clone the layer before
                let mut dir = get_layer_directory(layer, settings)?;
                dir.push(&dependency.name);
                dir.push(format!("{version}.toml"));
                if dir.is_file() {
                    return read(Some(dir.to_str().unwrap().to_string()));
                }
            }
        }
        DepVal::Path(path) => {
            let pkg_file_path = {
                let mut p = path.clone();
                p.push(DEFAULT_PACKAGE_FILE_NAME);
                p
            };
            if pkg_file_path.is_file() {
                return read(Some(pkg_file_path.to_str().unwrap().to_string()));
            }
        }
        DepVal::Git(_) => todo!("will be implemented soon"),
    }
    bail!("unable to read dependency")
}
