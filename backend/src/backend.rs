pub mod episode;
pub mod meta;
pub mod title;
pub mod video_metadata;

use crate::Config;
use crate::Discord;
use crate::Result;
use crate::Title;

use anyhow::bail;
use core::fmt::Debug;
use discord_sdk as ds;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::{env, fs, path::PathBuf, process::Command};
use tracing::warn;

static SCRIPT_PATH: Lazy<PathBuf> = Lazy::new(|| {
    confy::get_configuration_file_path("yama", "config")
        .expect("No configuration path found.")
        .parent()
        .expect("No valid configuration path found.")
        .join("scripts/save_info.lua")
});

/// [yama's] Backend, contains all the [`Titles`][Title] and utils to run this application.
///
/// _So, did you do some good deeds?_
#[derive(Debug)]
pub struct Backend {
    pub ds_client: Option<Discord>,
    pub titles: Vec<Title>,
    /// Number of [`titles`][Title] this [`Backend`][Backend] has.
    pub count: usize,
    title_cache: Arc<[Arc<str>]>,
}

impl Backend {
    /// Creates a new [`Backend`][Backend] instance, it will find all the [`titles`][Title]
    /// in the folder specified in the [`Config`][Config] file, and download their [metadata]
    /// with [`Anilist`][crate::Anilist] API.
    pub async fn new() -> Result<Self> {
        let mut titles = Self::load_titles()?;
        Self::download_titles_data(titles.as_mut_slice()).await;

        let ds_client = match Discord::new(ds::Subscriptions::ACTIVITY).await {
            Ok(user) => {
                user.idle_activity().await;
                Some(user)
            }
            Err(e) => {
                tracing::error!("Discord client failed to connect: {e}");
                None
            }
        };

        Ok(Self {
            title_cache: titles
                .iter()
                .map(|t| match &t.data {
                    Some(data) => data.media.title.english.as_str().into(),
                    None => t.name.clone(),
                })
                .collect(),
            count: titles.len(),
            titles,
            ds_client,
        })
    }

    fn load_titles() -> Result<Vec<Title>> {
        let cfg: Config = confy::load("yama", "config")?;

        if cfg.series_path.is_none() {
            warn!("No series path found.");
            bail!("No Titles found.");
        }

        let mut series: Vec<Title> = Self::get_files(&cfg.series_path.unwrap())?
            .filter(|x| match fs::metadata(x) {
                Ok(f) => f.is_dir(),
                Err(_) => false,
            })
            .flat_map(Title::new)
            .collect();

        series.sort_by(|a, b| alphanumeric_sort::compare_str(&a.name, &b.name));
        Ok(series)
    }

    async fn download_titles_data(titles: &mut [Title]) {
        use crate::Anilist;
        use iced::futures::future;

        let api = Anilist::default();

        let mut futs: Vec<_> = titles
            .iter_mut()
            .enumerate()
            .map(|(id, t)| api.try_query(t, id))
            .map(Box::pin)
            .collect();

        while !futs.is_empty() {
            match future::select_all(futs).await {
                (Ok(_), _, remaining) => {
                    futs = remaining;
                }
                (Err(e), _, remaining) => {
                    warn!("{e}");
                    futs = remaining;
                }
            }
        }
    }

    /// **[`Backend`][Backend] util:** Runs a secondary process given by a command.
    /// (Windows and Linux compatibility only!)
    pub fn run_process(cmd: &str) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            let mut cmd = cmd.split(',');
            let output = Command::new(cmd.next().unwrap())
                .current_dir(env::current_dir()?)
                .args(cmd)
                .output()?;

            if !output.status.success() {
                bail!("Command failed: {}", String::from_utf8(output.stdout)?);
            }

            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            let output = Command::new("sh")
                .current_dir(env::current_dir()?)
                .args(["-c", cmd])
                .output()?;

            if !output.status.success() {
                bail!("Command failed: {}", String::from_utf8(output.stdout)?);
            }

            Ok(())
        }
    }

    /// **[`Backend`][Backend] util:** Runs an instance of mpv with the given command.
    /// The [`Episode`][crate::Episode] and it's starting time should be passes as a command.
    pub fn run_mpv(command: &str) -> Result<()> {
        let cfg: Config = confy::load("yama", "config")?;

        let cmd = if cfg!(target_os = "windows") {
            format!(
                "mpv,--script={},--script-opts=save_info-min_time={},{command}",
                SCRIPT_PATH.display(),
                cfg.min_time
            )
        } else {
            format!(
                "mpv --script={} --script-opts=save_info-min_time={} {command}",
                SCRIPT_PATH.display(),
                cfg.min_time
            )
        };

        Backend::run_process(&cmd)
    }

    // #[cfg(not(target_os = "windows"))]
    /// **(Linux Version) [`Backend`][Backend] util:** Returns an [`Iterator`][Iterator] with all the _(non-hidden)_ paths inside a given directory.
    fn get_files(path: &PathBuf) -> Result<impl Iterator<Item = PathBuf>> {
        Ok(fs::read_dir(path)?
            .flatten()
            .filter_map(|x| match x.path().file_name() {
                Some(filename) => {
                    if !filename.to_str()?.starts_with('.') {
                        Some(x.path())
                    } else {
                        None
                    }
                }
                None => None,
            }))
    }

    /*#[cfg(target_os = "windows")]
    /// **(Windows Version) [`Backend`][Backend] util:** Returns an [`Iterator`][Iterator] with all the _(non-hidden)_ paths inside a given directory.
    fn get_files(path: &PathBuf) -> Result<impl Iterator<Item = PathBuf>> {
        use std::os::windows::fs::MetadataExt;

        Ok(fs::read_dir(path)?
            .flatten()
            .filter_map(|x| match x.metadata() {
                Ok(metadata) => {
                    if (metadata.file_attributes() & 0x2) > 0 {
                        Some(x.path())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }))
    }*/

    /// Returns the specified [`Episode`][crate::Episode] or [`None`][None] if it doesn't exist.
    pub fn get_episode(
        &mut self,
        title_number: usize,
        episode_number: usize,
    ) -> Option<&mut crate::Episode> {
        self.titles
            .get_mut(title_number)?
            .get_episode(episode_number)
    }

    /// Returns a copy of the [`Titles`][Title] names to be shared with the [frontend] thread.
    pub fn cache(&self) -> Arc<[Arc<str>]> {
        self.title_cache.clone()
    }

    /// Returns the name of the indexed [`Title`][Title].
    /// ## Panics
    /// May panic if `title_number` is out of bounds.
    pub fn get_title_name(&self, title_number: usize) -> Arc<str> {
        self.title_cache[title_number].clone()
    }

    /// Returns the data of the indexed [`Episode`][crate::Episode].
    /// ## Panics
    /// May panic if `title_number` or `episode_number` is out of bounds.
    pub fn get_episode_data(
        &self,
        title_number: usize,
        episode_number: usize,
    ) -> Option<(Arc<str>, f64)> {
        self.titles[title_number].episodes.as_ref().map(|episodes| {
            let ep = &episodes[episode_number];
            (
                ep.name.clone(),
                if ep.metadata.watched || ep.metadata.remaining == 0.00 {
                    ep.metadata.duration
                } else {
                    ep.metadata.remaining
                },
            )
        })
    }

    /// Takes a closure, applies it to the [`Titles`][Title] vector
    /// and returns a [`Vec`][Vec] with the results.
    pub fn map<F, T>(&self, f: F) -> Vec<T>
    where
        F: Fn(&Title) -> T,
    {
        self.titles.iter().map(f).collect()
    }
}
