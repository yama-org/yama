use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing_unwrap::ResultExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub series_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        std::fs::create_dir_all("./series").unwrap_or_log();

        Self {
            series_path: PathBuf::from("./series"),
        }
    }
}
