use std::path::Path;

use config::Config;

mod config;
mod exports;
mod patches;
mod version;

fn init_dll(dll_path: &Path) -> bool {
    let config = Config::read_or_create_default(dll_path);

    version::verify() && patches::place_all(&config).is_ok()
}
