use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub patch_character_limit: bool,
    pub patch_soundbank_limit: bool,
    pub heap_size_multiplier: u32,
    pub heap_sizes: HeapSizeConfig,
}

#[derive(Serialize, Deserialize)]
pub struct HeapSizeConfig {
    pub debug: u32,
    pub facegen: u32,
    pub file_data: u32,
    pub global: u32,
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
            patch_character_limit: true,
            patch_soundbank_limit: true,
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
            global: 1,
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
            sound: 3,
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
    pub fn read_or_create_default(dll_path: &Path) -> Self {
        Self::read_or_create(dll_path).normalize()
    }

    fn read(config_path: &Path) -> Result<Self, ConfigError> {
        let raw_config = match fs::read_to_string(config_path) {
            Ok(contents) => contents,
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => return Err(ConfigError::FileNotFound),
                _ => return Err(ConfigError::IoError),
            },
        };

        match toml::from_str::<Self>(&raw_config) {
            Ok(config) => Ok(config.normalize()),
            Err(_) => Err(ConfigError::InvalidToml),
        }
    }

    pub fn read_or_create(dll_path: &Path) -> Self {
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

    fn normalize(self) -> Self {
        let heap_size_multiplier = self.heap_size_multiplier.max(1);

        Self {
            heap_size_multiplier: 1,
            heap_sizes: HeapSizeConfig {
                debug: self
                    .heap_sizes
                    .debug
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                facegen: self
                    .heap_sizes
                    .facegen
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                file_data: self
                    .heap_sizes
                    .file_data
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                global: self
                    .heap_sizes
                    .global
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                graphics: self
                    .heap_sizes
                    .graphics
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                gui: self
                    .heap_sizes
                    .gui
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                havok: self
                    .heap_sizes
                    .havok
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                menu: self
                    .heap_sizes
                    .menu
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                morpheme: self
                    .heap_sizes
                    .morpheme
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                network: self
                    .heap_sizes
                    .network
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                player: self
                    .heap_sizes
                    .player
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                regulation: self
                    .heap_sizes
                    .regulation
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                scene_graph: self
                    .heap_sizes
                    .scene_graph
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                sfx: self
                    .heap_sizes
                    .sfx
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                sound: self
                    .heap_sizes
                    .sound
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                string_data: self
                    .heap_sizes
                    .string_data
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                system: self
                    .heap_sizes
                    .system
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                temp: self
                    .heap_sizes
                    .temp
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
                temp2: self
                    .heap_sizes
                    .temp2
                    .max(1)
                    .saturating_mul(heap_size_multiplier),
            },
            ..self
        }
    }
}

fn dll_dir_from_path(dll_path: &Path) -> Option<PathBuf> {
    let dirname = dll_path.parent()?;

    dirname.canonicalize().ok()
}
