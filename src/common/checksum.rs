/*
todo: use checksum for local dependencies

use std::path::Path;
use config::{Config, File};

/// Check sum of dependency headers and sources
#[derive(Deserialize)]
pub struct CheckSum {
    #[serde(default)]
    pub(super) headers: Vec<u32>,
    #[serde(default)]
    pub(super) sources: Vec<u32>,
}

pub fn checksum() {}

impl CheckSum {
    pub fn load(dep_path: &Path) -> Self {
        let mut cfg = Config::default();
        let mut cs_file = dep_path.to_path_buf();
        cs_file.push("cs.toml");
        match cfg.merge(File::from(cs_file)) {
            Ok(cs) => return cs.try_into(),
            Err(_) => todo!(),
        }
    }
}

pub fn load_checksum_lock(dep_path: &Path) {}

*/
