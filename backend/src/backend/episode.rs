use anyhow::{anyhow, bail};

use crate::Result;
use crate::{Backend, VideoMetadata};

use core::fmt::Debug;
use std::{fs, path::PathBuf, sync::Arc};

/// Contains all the information necessary to display an episode in [yama].
#[derive(Debug)]
pub struct Episode {
    pub number: u16,
    pub name: Arc<str>,
    pub metadata: VideoMetadata,
    pub thumbnail_path: PathBuf,
    pub metadata_path: PathBuf,
    pub path: PathBuf,
}

impl Episode {
    /// Creates a new [`Episode`][Episode] from the file specified by the _path_,
    /// the episode number should also be specified.
    ///
    /// If it returns an [`Error`][Error] then it's not a valid video file.
    pub async fn new(path: &PathBuf, number: u16) -> Result<Episode> {
        if !path.is_file() {
            bail!("The path {} is not a valid file.", path.display());
        }

        // Now we are sure is a valid path, so...
        let name = Arc::from(unsafe {
            path.file_stem()
                .unwrap_unchecked()
                .to_str()
                .unwrap_unchecked()
        });

        // We know it has a parent folder, its the title folder.
        let dir = path.parent().unwrap();

        let md_folder = format!(".metadata/episode_{number}");
        fs::create_dir_all(format!("{}/{}", dir.display(), &md_folder))?;

        let metadata_path = dir.join(format!("{}/{}.md", &md_folder, name));
        let thumbnail_path = dir.join(format!("{}/thumbnail.jpg", &md_folder));

        if fs::metadata(&metadata_path).is_err() {
            let duration: f64 = ffprobe::ffprobe(path)
                .map_err(
                    |_| match fs::remove_dir(format!("{}/{}", dir.display(), &md_folder)) {
                        Ok(_) => anyhow!("{} is not a valid video file.", path.display()),
                        Err(e) => e.into(),
                    },
                )?
                .format
                .get_duration()
                .ok_or_else(
                    || match fs::remove_dir(format!("{}/{}", dir.display(), &md_folder)) {
                        Ok(_) => anyhow!("{} is not a valid video file.", path.display()),
                        Err(e) => e.into(),
                    },
                )?
                .as_secs_f64();

            VideoMetadata::default_file(duration, &metadata_path)?
        }

        if fs::metadata(&thumbnail_path).is_err() {
            let cmd = if cfg!(target_os = "windows") {
                format!(
                    "ffmpeg,-i,{},-vf,thumbnail,-frames:v,1,{},-f,mjpeg,-hide_banner,-nostdin,-nostats,-loglevel,quiet",
                    path.display(),
                    thumbnail_path.display(),
                )
            } else {
                format!(
                    "ffmpeg -i \"{}\" -vf \"thumbnail\" -frames:v 1 \"{}\" -f mjpeg -hide_banner -nostdin -nostats -loglevel quiet",
                    path.display(),
                    thumbnail_path.display(),
                )
            };

            Backend::run_process(&cmd)?;
        }

        Ok(Episode {
            number,
            name,
            metadata: VideoMetadata::new(&metadata_path)?,
            thumbnail_path,
            metadata_path,
            path: path.to_owned(),
        })
    }

    /// Updates the [`VideoMetadata`][VideoMetadata] of this [`Episode`][Episode].
    fn update(&mut self) -> Result<()> {
        fs::rename(self.path.with_extension("md"), &self.metadata_path)?;
        self.metadata = VideoMetadata::new(&self.metadata_path)?;
        Ok(())
    }

    /// Marks the [`Episode`][Episode] as watched or not.
    pub fn as_watched(&mut self) -> Result<()> {
        self.metadata.as_watched();
        VideoMetadata::create_file(&self.metadata, &self.metadata_path)
    }

    /// Runs the [`Episode`][Episode] in [mpv] on the current time,
    /// or from the start if it has been already watched.
    ///
    /// Returns [`Error`][Error] if [mpv] fails to launch it or
    /// the [`VideoMetadata`][VideoMetadata] can not be updated.
    pub fn run(&mut self) -> Result<()> {
        let cmd = if cfg!(target_os = "windows") {
            format!(
                "--start={},{}",
                if self.metadata.watched {
                    0.00
                } else {
                    self.metadata.current
                },
                self.path.display()
            )
        } else {
            format!(
                "--start={} \"{}\"",
                if self.metadata.watched {
                    0.00
                } else {
                    self.metadata.current
                },
                self.path.display()
            )
        };

        Backend::run_mpv(&cmd)?;
        self.update()
    }

    pub fn change_number(mut self, idx: usize) -> Self {
        self.number = idx as u16;
        self
    }
}
