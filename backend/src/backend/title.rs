use super::{episode::Episode, Backend};
use crate::api::Data;
use crate::Meta;

use core::fmt::Debug;
use std::{
    fs,
    io::{self, Error, ErrorKind, Write},
    path::PathBuf,
};
use tracing::error;
use tracing_unwrap::{OptionExt, ResultExt};

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
                    "[WARNING] - Invalid Directory Name",
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

    /// <b>Optimize this: Do not check is_video insted remove from list all files that fail to build an Episode.</b>
    /// Loads a list of video files as Episodes for a Title.
    pub fn load_episodes(&mut self, refresh: bool) {
        if refresh || self.episodes.is_empty() {
            let episodes: Vec<Episode> = if refresh || !self.is_loaded() {
                let mut paths: Vec<PathBuf> = Backend::get_files(&self.path)
                    .unwrap_or_log()
                    .filter(|x| match x.metadata() {
                        Ok(f) => f.is_file(),
                        Err(_) => false,
                    })
                    /*.filter(|x| match fs::read(x) {
                        Ok(file) => infer::is_video(&file),
                        Err(_) => false,
                    })*/
                    .collect();

                paths.sort_by(|a, b| alphanumeric_sort::compare_path(a, b));

                let mut file =
                    fs::File::create(self.path.join(".metadata").join("files.md")).unwrap_or_log();

                paths
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, path)| match Episode::new(&path, i + 1) {
                        Ok(ep) => {
                            writeln!(file, "{}", path.display()).unwrap_or_log();
                            Some(ep)
                        }
                        Err(e) => {
                            error!("{}", e);
                            None
                        }
                    })
                    .collect()
            } else {
                let paths: Vec<PathBuf> =
                    fs::read_to_string(self.path.join(".metadata").join("files.md"))
                        .unwrap_or_log()
                        .lines()
                        .map(PathBuf::from)
                        .collect();

                paths
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, path)| match Episode::new(&path, i + 1) {
                        Ok(ep) => Some(ep),
                        Err(e) => {
                            error!("{}", e);
                            None
                        }
                    })
                    .collect()
            };

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
        let cant_episodes = match fs::read_to_string(self.path.join(".metadata").join("files.md")) {
            Ok(paths) => paths.lines().count(),
            Err(_) => return false,
        };

        let dir = self.path.join(".metadata");
        let mut metafolders: usize = 0;
        let mut metafiles: usize = 0;

        if dir.is_dir() {
            match fs::read_dir(dir) {
                Ok(files) => {
                    for file in files.flatten() {
                        if let Ok(f) = file.metadata() {
                            if f.is_dir() {
                                metafolders += 1;
                            } else if matches!(
                                file.file_name().to_str().unwrap_or_log(),
                                "thumbnail.jpg" | "data.json" | "files.md"
                            ) {
                                metafiles += 1;
                            }
                        }
                    }
                }
                Err(_) => return false,
            }

            metafolders == cant_episodes && metafiles == 3
        } else {
            false
        }
    }

    pub fn as_watched(&mut self, to: usize) -> io::Result<()> {
        for episode in &mut self.episodes[..to] {
            episode.as_watched()?
        }

        Ok(())
    }
}

impl Meta for Title {
    fn thumbnail(&self) -> Option<PathBuf> {
        self.data.as_ref().map(|data| data.thumbnail_path.clone())
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
