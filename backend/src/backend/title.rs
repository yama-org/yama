use crate::networking::anilist::Data;
use crate::Result;
use crate::{Backend, Episode};

use anyhow::bail;
use core::fmt::Debug;
use std::{fs, path::PathBuf, sync::Arc};
use tracing::error;

/// Contains all the information necessary to display a title in [yama].
#[derive(Debug)]
pub struct Title {
    /// Number of [`episodes`][Episode] this [`Title`][Title] has.
    pub count: u16,
    pub name: Arc<str>,
    /// [Metadata] of this [`Title`][Title],
    pub data: Option<Data>,
    pub episodes: Option<Vec<Episode>>,
    episodes_cache: Option<Arc<[Arc<str>]>>,
    pub path: PathBuf,
}

impl Title {
    /// Creates a new [`Title`][Title] for the folder specified by the _path_,
    ///
    /// If it returns an [`Error`][Error] then it's not a valid folder.
    pub fn new(path: PathBuf) -> Result<Title> {
        if !path.is_dir() {
            bail!("The path {} is not a valid folder.", path.display());
        }

        // Now we are sure is a valid path, so...
        let name = Arc::from(unsafe {
            path.file_name()
                .unwrap_unchecked()
                .to_str()
                .unwrap_unchecked()
        });

        fs::create_dir_all(format!("{}/.metadata/", path.display()))?;

        Ok(Title {
            name,
            path,
            count: 0,
            data: None,
            episodes: None,
            episodes_cache: None,
        })
    }

    #[allow(dead_code)]
    /// Checks if this [`Title`][Title] was properly loaded or its missing some meta-files.
    fn is_loaded(&self) -> bool {
        let cant_episodes = match fs::read_to_string(self.path.join(".metadata/files.md")) {
            Ok(paths) => paths.lines().count(),
            Err(_) => return false,
        };

        let dir = self.path.join(".metadata");
        let mut metafolders: usize = 0;
        let mut metafiles: usize = 0;

        match fs::read_dir(dir) {
            Ok(files) => {
                for file in files.flatten() {
                    if let Ok(f) = file.metadata() {
                        if f.is_dir() {
                            metafolders += 1;
                        } else if matches!(
                            file.file_name().to_str().unwrap(),
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
    }

    /// **Asynchronously** loads a list of video files as [`Episode`][Episode] for this [`Title`][Title].
    /// With a _refresh_ option to force the reloading of the [`Episode`][Episode] list.
    pub async fn load_episodes(&mut self, refresh: bool) -> Result<()> {
        use iced::futures::future;

        if refresh || self.episodes.is_none() {
            let episodes: Vec<Episode> = {
                let mut paths: Vec<PathBuf> = Backend::get_files(&self.path)?
                    .filter(|x| match x.metadata() {
                        Ok(f) => f.is_file(),
                        Err(_) => false,
                    })
                    .collect();

                paths.sort_by(|a, b| alphanumeric_sort::compare_path(a, b));

                let mut episodes = Vec::with_capacity(paths.len());
                let mut futs: Vec<_> = paths
                    .iter()
                    .enumerate()
                    .map(|(i, path)| Episode::new(path, i as u16))
                    .map(Box::pin)
                    .collect();

                while !futs.is_empty() {
                    match future::select_all(futs).await {
                        (Ok(ep), _, remaining) => {
                            episodes.push(ep);
                            futs = remaining;
                        }
                        (Err(e), _, remaining) => {
                            error!("{e}");
                            futs = remaining;
                        }
                    }
                }

                episodes.sort_by(|a, b| a.number.cmp(&b.number));
                episodes = episodes
                    .into_iter()
                    .enumerate()
                    .map(|(idx, ep)| ep.change_number(idx))
                    .collect();
                episodes
            };

            self.count = episodes.len() as u16;
            self.episodes_cache = Some(episodes.iter().map(|e| e.name.clone()).collect());
            self.episodes = Some(episodes)
        }

        Ok(())
    }

    /// Returns a copy of this title [`Episodes`][Episode] names to be shared with the [frontend] thread.
    pub fn cache(&self) -> Arc<[Arc<str>]> {
        match &self.episodes_cache {
            Some(episodes_cache) => episodes_cache.clone(),
            None => Arc::from([Arc::from("")]),
        }
    }

    /// Takes a closure, applies it to this title [`Episodes`][Episode] vector
    /// and returns a [`Vec`][Vec] with the results.
    ///
    /// If this title [`Episodes`][Episode] vector is empty it will return an empty [`Vec`][Vec].
    pub fn map<F, T>(&self, f: F) -> Vec<T>
    where
        F: Fn(&Episode) -> T,
    {
        match &self.episodes {
            Some(episodes) => episodes.iter().map(f).collect(),
            None => Vec::new(),
        }
    }

    /// Returns the specified [`Episode`][Episode] or [`None`][None] if it doesn't exist.
    pub fn get_episode(&mut self, number: usize) -> Option<&mut Episode> {
        self.episodes.as_mut()?.get_mut(number)
    }

    /// Marks all previous [`Episodes`][Episode] of this title [`Episode`][Episode] as watched or not.
    pub fn as_watched(&mut self, to: usize) -> Result<()> {
        if let Some(episodes_vec) = self.episodes.as_mut() {
            if let Some(episodes_slice) = episodes_vec.get_mut(..to) {
                for episode in episodes_slice {
                    episode.as_watched()?
                }
            }
        }

        Ok(())
    }
}
