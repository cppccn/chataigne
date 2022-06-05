use crate::{
    cmd::git_clone,
    common::types::{DepVal, Dependency, Package},
    settings::{self, Layer, Settings},
};
use anyhow::{bail, Result};
use glob::Pattern;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use tracing::debug;
use walkdir::WalkDir;

const DEFAULT_PACKAGE_FILE_NAME: &str = "chataigne.toml";

// todo: better manage that. Maybe everywhere we could use a Pkg instead of
//       PkgFile that implement all of that stuff

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

pub fn unwrap_path_patterns(path: &Path, patterns: &[String]) -> HashSet<PathBuf> {
    debug!("unwrap patterns {:?}", patterns);
    let mut ret = HashSet::new();
    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        for p in patterns {
            if Pattern::new(p).unwrap().matches_path(entry.path()) {
                ret.insert(entry.path().to_path_buf());
                break;
            }
            if Pattern::new(&format!("./{p}"))
                .unwrap()
                .matches_path(entry.path())
            {
                ret.insert(entry.path().to_path_buf());
                break;
            }
        }
    }
    ret
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
pub fn find_pkg(dependency: &Dependency, settings: &Settings) -> Result<Package> {
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
                    return Package::read(Some(dir.to_str().unwrap().to_string()));
                }
            }
        }
        DepVal::Path(path) => {
            let pkg_file_path = {
                let mut p = path.clone();
                p.push(DEFAULT_PACKAGE_FILE_NAME);
                p
            };
            debug!(
                "Find pkg {} in path {}",
                &dependency.name,
                pkg_file_path.to_string_lossy()
            );
            if pkg_file_path.is_file() {
                return Package::read(Some(pkg_file_path.to_str().unwrap().to_string()));
            }
        }
        DepVal::Git(_) => todo!("will be implemented soon"),
    }
    bail!("unable to read dependency")
}
