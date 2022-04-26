use crate::{
    common::types::DepVal,
    pkg::read,
    settings::{self, Settings},
};

use super::types::{Dependency, PackagePaths, PkgFile, StaticLib};
use anyhow::{bail, Result};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const DEFAULT_PACKAGE_FILE_NAME: &str = "luc.toml";

/// Return include paths and source files path from a root file.
///
/// By default, find all `.cpp` files as `source_files` and all parent folder
/// of any `.h` file. A package can overides these extensions with the `build`
/// and `include` parameters. Inverse is also possible, all path containing
/// something in the `ignore` parameter in package file will be really ignored.
pub fn package_paths(path: &Path) -> Result<PackagePaths> {
    // todo: return an error if a cyclic path found.
    // todo: add `ignore` parameter in pkg_file.
    // todo: find any file that end with the given extensions from pkg_file.
    println!("package path walk from {}", path.to_string_lossy());
    let mut source_files = HashSet::new();
    let mut header_folders = HashSet::new();
    let base = path.to_path_buf();
    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = entry.file_name().to_string_lossy();
        if f_name.ends_with(".cpp") {
            source_files.insert(entry.path().strip_prefix(&base).unwrap().to_path_buf());
        }

        // todo: don't add sub folders into result. add to ignore
        if f_name.ends_with(".h") {
            let e = entry.path().parent().unwrap().strip_prefix(&base).unwrap();
            println!("include folder found at {}", e.to_string_lossy());
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
pub fn lib_package_path(lib: &StaticLib, src_path: &Path) -> Result<PackagePaths> {
    println!("compile paths for {}", src_path.to_string_lossy());
    // todo: replace unwraps with error management
    let mut source_files = HashSet::from_iter(lib.builds.iter().map(|src| src.into()));
    let mut header_folders = HashSet::from_iter(lib.headers.iter().map(|src| src.into()));

    if source_files.is_empty() || header_folders.is_empty() {
        let pkg_paths = package_paths(src_path).unwrap();
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
    println!("Load dependency {} pkgfile", dependency.name);
    match &dependency.desc {
        DepVal::Version(version) => {
            for layer in &settings.layers {
                // todo, by default check filesystem for layers, but if
                // git repository, clone the layer before
                let dir = match layer {
                    settings::Layer::Git(_) => todo!("clone repo if not done and return path"),
                    settings::Layer::Dir(dir) => {
                        if std::fs::metadata(dir).is_ok() {
                            dir
                        } else {
                            bail!("Broken link to layer {dir}")
                        }
                    }
                };
                let mut p = PathBuf::from(dir);
                p.push(dir);
                p.push(&dependency.name);
                p.push(format!("{version}.toml"));
                if p.is_file() {
                    return read(Some(p.to_str().unwrap().to_string()));
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
