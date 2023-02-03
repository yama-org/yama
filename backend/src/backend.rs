pub mod episode;
pub mod title;
pub mod video_metadata;

use crate::api::Data;
use crate::backend::title::Title;
use crate::config::Config;
use crate::Media;

use core::fmt::Debug;
use once_cell::sync::OnceCell;
use std::{
    env, fs,
    io::{self, Error, ErrorKind},
    path::PathBuf,
    process::{Command, Output},
    sync::{Arc, Mutex},
};
use tracing::warn;

#[derive(Debug)]
pub struct Backend {
    pub titles: Vec<Media<Title>>,
    pub count: usize,
    cache: OnceCell<Vec<String>>,
}

impl Default for Backend {
    fn default() -> Backend {
        Backend::new()
    }
}

impl Backend {
    pub fn new() -> Backend {
        //TODO: Read config from a file
        let config = Config::new(PathBuf::from("./series")).expect("[ERROR] - Configuration file.");
        let titles = Backend::load_titles(config)
            .expect("[ERROR] - No valid titles inside selected folder.");

        Backend {
            cache: OnceCell::default(),
            count: titles.len(),
            titles,
        }
    }

    pub fn run_process(cmd: &str) -> io::Result<Output> {
        let output = Command::new("sh")
            .current_dir(
                env::current_dir().expect("[ERROR] - YAMA can not work on this invalid directory."),
            )
            .args(["-c", cmd])
            .output()?;

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
        Backend::run_process(format!("mpv --script=./scripts/save_info.lua {command}").as_str())
    }

    fn get_files(path: &PathBuf) -> io::Result<Vec<PathBuf>> {
        let files: Vec<PathBuf> = fs::read_dir(path)?
            .into_iter()
            .flatten()
            .map(|x| x.path())
            .collect();

        Ok(files)
    }

    fn load_titles(config: Config) -> io::Result<Vec<Media<Title>>> {
        let mut series: Vec<Title> = Backend::get_files(&config.series_path)?
            .into_iter()
            .filter(|x| match fs::metadata(x) {
                Ok(f) => f.is_dir(),
                Err(_) => false,
            })
            .flat_map(Title::new)
            .collect();

        series.sort_by(|a, b| alphanumeric_sort::compare_str(&a.name, &b.name));

        Ok(series
            .into_iter()
            .map(|t| Arc::new(Mutex::new(t)))
            .collect())
    }

    pub fn get_title(&self, number: usize) -> Media<Title> {
        Arc::clone(&self.titles[number])
    }

    pub fn view(&self) -> &[String] {
        self.cache.get_or_init(|| {
            self.titles
                .iter()
                .map(|t| t.lock().unwrap_or_else(|t| t.into_inner()).name.clone())
                .collect()
        })
    }

    pub async fn batch_titles_data(titles: Vec<(PathBuf, String)>) -> Vec<Data> {
        use crate::api::Api;
        use iced::futures::future;

        let api = Api::default();

        let data_fut: Vec<_> = titles
            .iter()
            .enumerate()
            .map(|(id, (path, name))| api.query(path, name, id))
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
}
