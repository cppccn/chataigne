use config::Value;
use serde_derive::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

/// Parse the build options in package files. Will be adapted into
/// [BuildOption], don't use that structure outside deserialization.
#[derive(Deserialize, Default)]
pub struct ConfigBuildOption {
    /// Paths that will be ignored
    #[serde(default)]
    pub ignore: Vec<String>,
    pub dependencies: Option<HashMap<String, Value>>,
    /// Overriding the auto sources loader
    #[serde(default)]
    pub sources: Vec<String>,
    /// Overriding the auto includes loader
    #[serde(default)]
    pub includes: Vec<String>,
    /// Simply add options (flags) to the build
    #[serde(default)]
    pub opt: Vec<String>,
}

/// Same as [ConfigBuildOption] but after a little adaptation to be used in rust
/// code.
pub struct BuildOption {
    /// Paths that will be ignored, default change for test, and dev/release
    pub ignore: Vec<String>,
    /// Build dependencies
    pub dependencies: Option<HashMap<String, DepVal>>,
    /// Overriding auto detection of build sources (path and regex)
    pub sources: Vec<String>,
    /// Overriding auto detection of includes/headers (path and regex)
    pub includes: Vec<String>,
    /// Simply add options (flags) to the build
    pub opt: Vec<String>,
}

/// Package deduced for a toml file. It will be changed into a [Package]
/// for a better usage in the code.
#[derive(Deserialize)]
pub struct ConfigPackage {
    pub package: ConfigPkgDescription,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub opt: Vec<String>,
    #[serde(default)]
    pub dev: ConfigBuildOption,
    #[serde(default)]
    pub test: ConfigBuildOption,

    pub dependencies: Option<HashMap<String, Value>>,
    // todo: we can also have a shared library or dynamic.
    pub lib: Option<StaticLib>,
}

/// Package as describen in the package part in the package file. Don't use that struct in general.
/// It's used only for deserialization. [Package] structure has a form simplier to use.
#[derive(Deserialize)]
pub struct ConfigPkgDescription {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub src: Option<Value>,
    pub repostory: Option<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub includes: Vec<String>,
}

#[derive(Deserialize)]
pub struct StaticLib {
    #[serde(default)]
    pub headers: Vec<String>,
    // todo make `builds` deprecated and use `sources` instead
    #[serde(default)]
    pub builds: Vec<String>,
    #[serde(default)]
    pub opt: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
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

pub struct PkgDescription {
    /// Pure text name of the package
    pub name: String,
    /// Information about the version of the package
    pub version: String,
    /// Description pure text of the package
    pub description: Option<String>,
    /// Define in a package description (for dependencies) where the user can find
    /// the repository where sources are stored
    pub src: Option<SrcVal>,
    /// Information about the repository. Nothing very usefull
    pub repostory: Option<String>,
}

pub struct Package {
    pub pkg_description: PkgDescription,
    pub dependencies: Option<HashMap<String, DepVal>>,
    /// Simply add options (flags) to the build
    pub opt: Vec<String>,
    pub ignore: Vec<String>,
    pub dev: BuildOption,
    pub test: BuildOption,
    // todo: we can also have a shared library or dynamic.
    pub lib: Option<StaticLib>,
    /// Overriding auto detection of sources (path and regex)
    pub sources: Vec<String>,
    /// Overriding auto detection of includes/headers (path and regex)
    pub includes: Vec<String>,
}

pub struct PackagePaths {
    pub header_folders: HashSet<PathBuf>,
    pub source_files: HashSet<PathBuf>,
}
