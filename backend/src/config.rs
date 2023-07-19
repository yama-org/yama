use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// [yama's] Config
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub series_path: PathBuf,
    pub min_time: f32,
}

impl Default for Config {
    fn default() -> Self {
        //TODO! - 0.8.0: Ask user for default path.
        std::fs::create_dir_all("./series").unwrap();

        Self {
            series_path: PathBuf::from("./series"),
            min_time: 10.0,
        }
    }
}
