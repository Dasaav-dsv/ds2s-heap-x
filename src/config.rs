use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub heap_size_multiplier: u32,
    pub heap_sizes: HeapSizeConfig,
}

#[derive(Serialize, Deserialize)]
pub struct HeapSizeConfig {
    pub debug: u32,
    pub facegen: u32,
    pub file_data: u32,
    pub graphics: u32,
    pub gui: u32,
    pub havok: u32,
    pub menu: u32,
    pub morpheme: u32,
    pub network: u32,
    pub player: u32,
    pub regulation: u32,
    pub scene_graph: u32,
    pub sfx: u32,
    pub sound: u32,
    pub string_data: u32,
    pub system: u32,
    pub temp: u32,
    pub temp2: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            heap_size_multiplier: 2,
            heap_sizes: Default::default(),
        }
    }
}

impl Default for HeapSizeConfig {
    fn default() -> Self {
        Self {
            debug: 1,
            facegen: 1,
            file_data: 2,
            graphics: 1,
            gui: 1,
            havok: 4,
            menu: 1,
            morpheme: 4,
            network: 1,
            player: 1,
            regulation: 2,
            scene_graph: 1,
            sfx: 4,
            sound: 2,
            string_data: 2,
            system: 2,
            temp: 1,
            temp2: 1,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound,
    IoError,
    InvalidToml,
}

impl Config {
    fn read(config_path: &Path) -> Result<Self, ConfigError> {
        let raw_config = match fs::read_to_string(config_path) {
            Ok(contents) => contents,
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => return Err(ConfigError::FileNotFound),
                _ => return Err(ConfigError::IoError),
            },
        };

        toml::from_str(&raw_config).map_err(|_| ConfigError::InvalidToml)
    }

    pub fn read_or_create_default(dll_path: &Path) -> Self {
        let Some(config_path) = dll_dir_from_path(dll_path).map(|mut path| {
            path.push("ds2s_heap_x.toml");
            path
        }) else {
            return Self::default();
        };

        use ConfigError::*;

        match Self::read(&config_path) {
            Ok(config) => config,
            Err(FileNotFound) | Err(InvalidToml) => {
                let default = Self::default();

                let contents = toml::to_string(&default).expect("valid default toml");

                let _ = fs::write(config_path, contents);

                default
            }
            Err(IoError) => Self::default(),
        }
    }
}

fn dll_dir_from_path(dll_path: &Path) -> Option<PathBuf> {
    let dirname = dll_path.parent()?;

    dirname.canonicalize().ok()
}
