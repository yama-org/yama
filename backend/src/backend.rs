pub mod episode;
pub mod title;
pub mod video_metadata;

use crate::api::Data;
use crate::backend::title::Title;
use crate::config::Config;

use core::fmt::Debug;
use once_cell::sync::Lazy;
use std::{
    env, fs,
    io::{self, Error, ErrorKind},
    path::PathBuf,
    process::{Command, Output},
};
use tracing::warn;
use tracing_unwrap::ResultExt;

static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    confy::get_configuration_file_path("yama", "config")
        .unwrap()
        .parent()
        .unwrap()
        .join("scripts/save_info.lua")
});

#[derive(Debug)]
pub struct Backend {
    pub titles: Vec<Title>,
    pub count: usize,
    pub cfg: Config,
}

#[allow(clippy::new_without_default)]
impl Backend {
    pub fn new() -> Backend {
        let config = confy::load("yama", "config").expect_or_log("[ERROR] - Configuration file.");
        let titles = Backend::load_titles(&config)
            .expect_or_log("[ERROR] - No valid titles inside selected folder.");

        Backend {
            count: titles.len(),
            titles,
            cfg: config,
        }
    }

    pub fn run_process(cmd: &str) -> io::Result<Output> {
        let output = if cfg!(target_os = "windows") {
            let mut cmd = cmd.split(',');

            Command::new(cmd.next().unwrap())
                .current_dir(
                    env::current_dir()
                        .expect_or_log("[ERROR] - YAMA can not work on this invalid directory."),
                )
                .args(cmd)
                .output()?
        } else {
            Command::new("sh")
                .current_dir(
                    env::current_dir()
                        .expect_or_log("[ERROR] - YAMA can not work on this invalid directory."),
                )
                .args(["-c", cmd])
                .output()?
        };

        if !output.status.success() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("[ERROR] - Exit code failure.\n{}", unsafe {
                    String::from_utf8_unchecked(output.stderr)
                }),
            ));
        }

        Ok(output)
    }

    pub fn run_mpv(command: &str) -> io::Result<Output> {
        let cmd = if cfg!(target_os = "windows") {
            format!("mpv,--script={},{command}", CONFIG_PATH.display())
        } else {
            format!("mpv --script={} {command}", CONFIG_PATH.display())
        };

        Backend::run_process(&cmd)
    }

    fn get_files(path: &PathBuf) -> io::Result<impl Iterator<Item = PathBuf>> {
        Ok(fs::read_dir(path)?.flatten().map(|x| x.path()))
    }

    fn load_titles(config: &Config) -> io::Result<Vec<Title>> {
        let mut series: Vec<Title> = Backend::get_files(&config.series_path)?
            .filter(|x| match fs::metadata(x) {
                Ok(f) => f.is_dir(),
                Err(_) => false,
            })
            .flat_map(Title::new)
            .collect();

        series.sort_by(|a, b| alphanumeric_sort::compare_str(&a.name, &b.name));
        Ok(series)
    }

    pub async fn download_title_data(&self) -> Vec<Data> {
        use crate::api::Api;
        use iced::futures::future;

        let api = Api::default();

        let data_fut: Vec<_> = self
            .titles
            .iter()
            .enumerate()
            .map(|(id, t)| api.try_query(&t.path, &t.name, id))
            .map(Box::pin)
            .collect();

        let mut rc = Vec::with_capacity(data_fut.len());
        let mut futs = data_fut;

        while !futs.is_empty() {
            match future::select_all(futs).await {
                (Ok(data), _, remaining) => {
                    rc.push(data);
                    futs = remaining;
                }
                (Err(e), _, remaining) => {
                    warn!("{e}");
                    futs = remaining;
                }
            }
        }

        rc
    }

    pub fn view(&self) -> Vec<String> {
        self.titles.iter().map(|t| t.name.clone()).collect()
    }
}
