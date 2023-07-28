use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

static CFG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    confy::get_configuration_file_path("yama", "config")
        .expect("No configuration path found.")
        .parent()
        .expect("No valid configuration path found.")
        .to_path_buf()
});

/// [yama's] Config
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub series_path: Option<PathBuf>,
    pub theme_path: PathBuf,
    pub min_time: f32,
}

impl Default for Config {
    fn default() -> Self {
        //TODO! - 0.8.0: Ask user for default path.
        //std::fs::create_dir_all("./series").unwrap();

        Self {
            series_path: None,
            theme_path: CFG_PATH.join("themes/iced.json"),
            min_time: 10.0,
        }
    }
}
