use std::path::Path;

use anyhow::{bail, Result};
use config::{Config, File, Value};
use directories::ProjectDirs;
use serde_derive::Deserialize;

use crate::common::types::{GitTarget, LocalTarget};

/// An overlay in a folder or a git repository that have a folder architecture
/// like:
///
/// pkg_name1/
///     pkg_version1.toml
///     pkg_version2.toml
///     ...
/// pkg_name2/
///     pkg_version1.toml
///     pkg_version2.toml
///     ...
/// ...
pub enum Layer {
    Git(GitTarget),
    Dir(LocalTarget),
}

#[derive(Deserialize)]
struct SettingFile {
    // todo: g++ binary direcory
    #[serde(default)]
    pub layers: Vec<Value>,
    // todo: conservation of .o, cache management
    // todo: compiler default flags
}

impl SettingFile {
    pub fn get_layers(&self) -> Vec<Layer> {
        self.layers
            .iter()
            .map(|s| {
                if let Ok(git) = s.clone().try_into::<GitTarget>() {
                    Layer::Git(git)
                } else if let Ok(local) = s.clone().try_into::<LocalTarget>() {
                    Layer::Dir(local)
                } else {
                    panic!("unknown layer format")
                }
            })
            .collect()
    }
}

pub struct Settings {
    pub layers: Vec<Layer>,
    pub project_dirs: ProjectDirs,
}

/// Initialisation for the first use of chataigne
fn init(config: &Path, cache: &Path) {
    if config.is_dir() && cache.is_dir() {
        return;
    }
    std::fs::create_dir_all(config).unwrap();
    std::fs::create_dir_all(cache).unwrap();
    let mut settings = config.to_path_buf();
    settings.push("settings.toml");
    std::fs::write(
        settings,
        r#"layers = [{git="https://github.com/adrien-zinger/layer_test_ch.git"}]"#,
    )
    .unwrap();
}

impl Settings {
    pub fn new() -> Result<Self> {
        let project_dirs = match ProjectDirs::from("com", "cppccn", "chataigne") {
            Some(project_dirs) => project_dirs,
            // todo: create default file instead of just returning default setting
            None => bail!("Failed to instanciate or get project directories"),
        };
        let mut config = Config::default();
        let mut config_path = project_dirs.config_dir().to_path_buf();
        init(&config_path, project_dirs.cache_dir());
        config_path.push("settings.toml");
        config.merge(File::with_name(&config_path.to_string_lossy()))?;
        let setting_file = config.try_into::<SettingFile>()?;

        Ok(Settings {
            layers: setting_file.get_layers(),
            project_dirs,
        })
    }
}
