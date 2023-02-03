use crate::api::Data;
use crate::backend::{episode::Episode, Backend};
use crate::{Media, Meta};

use core::fmt::Debug;
use once_cell::sync::OnceCell;
use std::{
    fs,
    io::{self, Error, ErrorKind},
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct Title {
    pub name: String,
    pub path: PathBuf,
    pub count: usize,
    episodes: Vec<Media<Episode>>,
    cache: OnceCell<Vec<String>>,
    pub data: Option<Data>,
}

impl Title {
    pub fn new(path: PathBuf) -> io::Result<Title> {
        fs::metadata(&path)?; //We check if the dir is valid

        let name = match path.file_name() {
            Some(path) => path.to_owned(),
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "[WARNING] - Invalid Directory",
                ))
            }
        };

        let name = match name.into_string() {
            Ok(name) => name,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "[WARNING] - Invalid Directory",
                ))
            }
        };

        Ok(Title {
            //name: path.display().to_string().replace("./series/", ""),
            name,
            path,
            episodes: Vec::new(),
            count: 0,
            cache: OnceCell::default(),
            data: None,
        })
    }

    /// Loads a list of video files as Episodes for a Title.
    fn load_episodes(path: &PathBuf) -> Vec<Media<Episode>> {
        //TODO: Improve video file detection
        //const VIDEO_FORMATS: &[&str] = &[".mp4", ".mkv"];

        //We made sure before creating this Title that the path is correct.
        //So we can just unwrap it.
        let mut episodes: Vec<Episode> = Backend::get_files(path)
            .unwrap()
            .into_iter()
            .filter(|x| match fs::metadata(x) {
                Ok(f) => f.is_file(),
                Err(_) => false,
            })
            .filter(|x| {
                match fs::read(x) {
                    Ok(file) => infer::is_video(&file),
                    Err(_) => false,
                }
                //let k = x.display().to_string();
                //VIDEO_FORMATS.iter().any(|y| k.to_lowercase().contains(y))
            })
            .enumerate()
            .flat_map(|(i, path)| Episode::new(path, i + 1))
            .collect();

        episodes.sort_by(|a, b| alphanumeric_sort::compare_str(&a.name, &b.name));

        episodes
            .into_iter()
            .map(|e| Arc::new(Mutex::new(e)))
            .collect()
    }

    pub fn get_or_init(&mut self, number: usize) -> Media<Episode> {
        if self.episodes.is_empty() {
            self.episodes = Title::load_episodes(&self.path);
            self.count = self.episodes.len();
        }
        Arc::clone(&self.episodes[number])
    }

    pub fn view(&self) -> &[String] {
        self.cache.get_or_init(|| {
            self.episodes
                .iter()
                .map(|e| e.lock().unwrap_or_else(|e| e.into_inner()).name.clone())
                .collect()
        })
    }

    pub fn get_episode(&self, number: usize) -> Media<Episode> {
        Arc::clone(&self.episodes[number])
    }

    pub fn is_loaded(&self) -> bool {
        self.path.join(".metadata/").is_dir()
    }
}

impl Meta for Title {
    fn thumbnail(&self) -> &PathBuf {
        if let Some(data) = &self.data {
            &data.thumbnail_path
        } else {
            &self.path //TEMPORARY
        }
    }

    fn description(&self) -> String {
        if let Some(data) = &self.data {
            format!(
                "{}\n\nDescription: {}\n\nGenres: {:?}",
                data.media.title.english, data.media.description, data.media.genres
            )
        } else {
            "No data".to_string()
        }
    }
}
