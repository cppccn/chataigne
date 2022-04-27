use super::internal_run;
use crate::{
    common::types::{DepVal, Dependency, PkgFile, SrcVal},
    settings::Settings,
};
use anyhow::{bail, Result};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tracing::debug;

/// Checkout source folder if needed from `Src` object.
///
/// - `git` param is Some, checkout git repository if not in cache
/// - `commit` param is Some, process command `git checkout $param`
/// - `path` param is Some, print an error if `git` is set (incompatible)
///
/// Return an error if git not found or any command failed.
///
/// The source path is returned anyway to trace where the sources are.
pub fn checkout_dependency(
    dependency: &Dependency,
    pkg_file: &PkgFile,
    settings: &Settings,
) -> Result<PathBuf> {
    match &dependency.desc {
        // if version, clone git repository
        DepVal::Version(_) => {
            // todo: force reload with a parameter, also add a `clear` param
            let mut dep_path = settings.project_dirs.cache_dir().to_path_buf();
            dep_path.push(&dependency.name);
            match &pkg_file.package.src {
                Some(SrcVal::Local(_)) => {
                    todo!("soon")
                }
                Some(SrcVal::Git(src)) => git_clone(&src.git, &src.commit, &dep_path),
                None => {} // nothing to do
            };
            Ok(dep_path)
        }
        DepVal::Path(path) => Ok(path.to_owned()),
        _ => bail!("cannot get path dependency of {}", pkg_file.package.name),
    }

    // todo, implement for other depvals
    // todo manage each case in specific function, implement checkout in another file.
}

/// Clone a repository into `dest`, optionnaly checkout something, a tag, a
/// commit...
pub fn git_clone(url: &str, checkout: &Option<String>, dest_path: &Path) {
    debug!("check repository at {}", dest_path.to_string_lossy());
    let dest = dest_path.file_name().unwrap().to_string_lossy();
    let parent = dest_path.parent().unwrap();
    if !dest_path.is_dir() {
        debug!("clone repository {url} at {}", dest_path.to_string_lossy());
        let mut cmd = Command::new("git");
        cmd.args(vec!["clone", url, &dest]).current_dir(parent);
        internal_run(cmd);
    }
    if let Some(h) = checkout {
        debug!("repository cloned, checkout {h}");
        let mut cmd = Command::new("git");
        cmd.args(vec!["checkout", h]).current_dir(dest_path);
        internal_run(cmd);
    }
}
