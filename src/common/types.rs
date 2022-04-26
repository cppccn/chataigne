use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use config::Value;
use serde_derive::Deserialize;

use crate::settings::Settings;

#[derive(Deserialize)]
pub struct _PkgFile {
    pub package: _Package,
    pub dependencies: Option<HashMap<String, Value>>,
    pub dev_dependencies: Option<HashMap<String, Value>>,
    pub test_dependencies: Option<HashMap<String, Value>>,
    // todo: we can also have a shared library or dynamic.
    pub lib: Option<StaticLib>,
}

#[derive(Deserialize)]
pub struct _Package {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub src: Option<Value>,
    pub repostory: Option<String>,
}

#[derive(Deserialize)]
pub struct StaticLib {
    #[serde(default)]
    pub headers: Vec<String>,
    #[serde(default)]
    pub builds: Vec<String>,
    pub opt: Option<Vec<String>>,
}

#[derive(Deserialize, Clone)]
/// Dependency parsed in package files. Simplifyed into that
/// enum.
pub enum DepVal {
    Version(String),
    Path(PathBuf),
    Git(GitTarget),
}

#[derive(Deserialize, Clone)]
pub enum SrcVal {
    Local(LocalTarget),
    Git(GitTarget),
}

#[derive(Deserialize, Clone)]
pub struct LocalTarget {
    pub path: String,
}

/// Git target that can be deserialized from config value in the settings.toml
/// and in the dependencies in the package files.
///
/// ```toml
/// # in src of a package in a versionning description file
/// [package]
/// src={git = "...", commit="..."}
///
/// # in a dependency description
/// [dependencies]
/// sample={git = "...", commit="..."}
///
/// # as a layer in settings.toml
/// layers=["local_path", {git = "...", commit="..."}]
/// ```
#[derive(Deserialize, Clone)]
pub struct GitTarget {
    /// Git url
    pub git: String,
    pub commit: Option<String>,
}

/// Structured representation of dependencies in PkgFile,
/// used in PkgFile implementation.
///
/// [PkgFile::get_dependencies]
pub struct Dependency {
    pub name: String,
    pub desc: DepVal,
}

pub struct Package {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub src: Option<SrcVal>,
    pub repostory: Option<String>,
}

pub struct PkgFile {
    pub package: Package,
    pub dependencies: Option<HashMap<String, DepVal>>,
    pub dev_dependencies: Option<HashMap<String, DepVal>>,
    pub test_dependencies: Option<HashMap<String, DepVal>>,
    // todo: we can also have a shared library or dynamic.
    pub lib: Option<StaticLib>,
}

pub struct PackagePaths {
    pub header_folders: HashSet<PathBuf>,
    pub source_files: HashSet<PathBuf>,
}

impl Package {
    /// Return an owned pathbuf to the target directory.
    ///
    /// Target is computed with format ${cache_dir}/${pkg_name}_${pkg_version}
    pub fn target_dir(&self, settings: &Settings) -> PathBuf {
        // todo: memoization
        let mut p = settings.project_dirs.cache_dir().to_path_buf();
        p.push(format!("{}_{}", self.name, self.version));
        p
    }
}
