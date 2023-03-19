use crate::api::Data;
use crate::backend::{episode::Episode, Backend};
use crate::Meta;

use core::fmt::Debug;
use std::{
    fs,
    io::{self, Error, ErrorKind},
    path::PathBuf,
};
use tracing::error;

#[derive(Debug)]
pub struct Title {
    pub path: PathBuf,
    pub name: String,
    pub data: Option<Data>,
    pub episodes: Vec<Episode>,
    pub count: usize,
}

impl Title {
    pub fn new(path: PathBuf) -> io::Result<Title> {
        fs::metadata(&path)?; // We check if the dir is valid
        fs::create_dir_all(format!("{}/.metadata/", path.display()))?;

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
            path,
            name,
            data: None,
            episodes: Vec::new(),
            count: 0,
        })
    }

    /// Loads a list of video files as Episodes for a Title.
    pub fn load_episodes(&mut self, skip_checking: bool) {
        // We made sure before creating this Title that the path is correct.
        // So we can just unwrap it.
        if self.episodes.is_empty() {
            let mut paths: Vec<PathBuf> = Backend::get_files(&self.path)
                .unwrap()
                .into_iter()
                .filter(|x| match fs::metadata(x) {
                    Ok(f) => f.is_file(),
                    Err(_) => false,
                })
                .filter(|x| {
                    skip_checking
                        || match fs::read(x) {
                            Ok(file) => infer::is_video(&file),
                            Err(_) => false,
                        }
                })
                .collect();

            paths.sort_by(|a, b| {
                alphanumeric_sort::compare_str(a.to_str().unwrap(), b.to_str().unwrap())
            });

            let episodes: Vec<Episode> = paths
                .into_iter()
                .enumerate()
                .filter_map(|(i, path)| match Episode::new(path, i + 1) {
                    Ok(ep) => Some(ep),
                    Err(e) => {
                        error!("{}", e);
                        None
                    }
                })
                .collect();

            self.count = episodes.len();
            self.episodes = episodes
        }
    }

    pub fn view(&self) -> Vec<String> {
        self.episodes.iter().map(|e| e.name.clone()).collect()
    }

    pub fn get_episode(&self, number: usize) -> &Episode {
        &self.episodes[number]
    }

    pub fn is_loaded(&self) -> bool {
        let dir = self.path.join(".metadata");
        let mut folders: usize = 0;

        if dir.is_dir() {
            let files: Vec<bool> = match fs::read_dir(dir) {
                Ok(files) => files
                    .into_iter()
                    .flatten()
                    .map(|file| match fs::metadata(file.path()) {
                        Ok(f) => {
                            if f.is_dir() {
                                match fs::read_dir(file.path()) {
                                    Ok(f) => {
                                        folders += 1;
                                        f.count() == 2
                                    }
                                    Err(_) => false,
                                }
                            } else if f.is_file() {
                                file.file_name() == "thumbnail.jpg"
                                    || file.file_name() == "data.json"
                            } else {
                                false
                            }
                        }
                        Err(_) => false,
                    })
                    .collect(),
                Err(_) => vec![false],
            };

            !files.contains(&false) && folders > 0
        } else {
            false
        }
    }
}

impl Meta for Title {
    fn thumbnail(&self) -> PathBuf {
        if let Some(data) = &self.data {
            data.thumbnail_path.clone()
        } else {
            PathBuf::from("./res/no_thumbnail.jpg")
        }
    }

    fn description(&self) -> String {
        if let Some(data) = &self.data {
            format!(
                "{}\n\nDescription: {}\n\nGenres: {:?}\n\nStudio: {}",
                data.media.title.english, data.media.description, data.media.genres, data.studio
            )
        } else {
            "No Data".to_string()
        }
    }
}
