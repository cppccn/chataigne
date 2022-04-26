use anyhow::{bail, Result};
use config::{Config, File};
use directories::ProjectDirs;
use serde_derive::Deserialize;

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
    Git(String),
    Dir(String),
}

#[derive(Deserialize)]
struct SettingFile {
    // todo: g++ binary direcory
    #[serde(default)]
    pub layers: Vec<String>,
    // todo: conservation of .o, cache management
    // todo: compiler default flags
}

impl SettingFile {
    pub fn get_layers(&self) -> Vec<Layer> {
        self.layers
            .iter()
            .map(|s| {
                if s.starts_with("git:") {
                    Layer::Git(s.clone().drain(4..).collect())
                } else if s.starts_with("dir:") {
                    Layer::Dir(s.clone().drain(4..).collect())
                } else {
                    Layer::Dir(s.clone())
                }
            })
            .collect()
    }
}

pub struct Settings {
    pub layers: Vec<Layer>,
    pub project_dirs: ProjectDirs,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let project_dirs = match ProjectDirs::from("com", "cppccn", "luc") {
            Some(project_dirs) => project_dirs,
            // todo: create default file instead of just returning default setting
            None => bail!("Failed to instanciate or get project directories"),
        };
        let mut config = Config::default();
        let mut config_path = project_dirs.config_dir().to_path_buf();
        config_path.push("settings.toml");
        config.merge(File::with_name(&config_path.to_string_lossy()))?;
        let setting_file = config.try_into::<SettingFile>()?;

        Ok(Settings {
            layers: setting_file.get_layers(),
            project_dirs,
        })
    }
}
