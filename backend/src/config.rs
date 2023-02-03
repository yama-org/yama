use std::path::PathBuf;

pub struct Config {
    pub series_path: PathBuf,
}

impl Config {
    pub fn new(path: PathBuf) -> Result<Config, &'static str> {
        Ok(Config { series_path: path })
    }
}
