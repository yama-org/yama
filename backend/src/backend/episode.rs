use tracing::info;

use crate::backend::{video_metadata::VideoMetadata, Backend};
use crate::Meta;

use core::fmt::Debug;
use std::{fs, io, path::PathBuf};

#[derive(Debug)]
pub struct Episode {
    pub name: String,
    pub path: PathBuf,
    pub number: usize,
    pub thumbnail_path: PathBuf,
    pub md_path: PathBuf,
    pub metadata: VideoMetadata,
}

impl Episode {
    pub fn new(path: PathBuf, number: usize) -> io::Result<Episode> {
        // It's safe to just unwrap parent(), file_name() and to_str()
        // because we made sure befor that this path is a valid file.
        let dir = path.parent().unwrap();
        fs::create_dir_all(format!("{}/.metadata/episode_{number}/", dir.display()))?;

        let mut old_md_path = path.clone();
        old_md_path.set_extension("md");

        let md_path = dir
            .join(format!(".metadata/episode_{number}/"))
            .join(old_md_path.file_name().unwrap());

        if fs::metadata(&md_path).is_err() {
            let command = format!("--no-video --end=0.0001 \"{}\"", path.display());
            Backend::run_mpv(&command)?;
        }

        if fs::metadata(&old_md_path).is_ok() {
            fs::rename(old_md_path, &md_path)?;
        }

        let thumbnail_path = dir
            .join(format!(".metadata/episode_{number}/"))
            .join("thumbnail.jpg");

        if fs::metadata(&thumbnail_path).is_err() {
            Backend::run_process(
                format!(
                    "ffmpegthumbnailer -i \"{}\" -o \"{}\" -s 0",
                    path.display(),
                    thumbnail_path.display(),
                )
                .as_str(),
            )?;
        }

        Ok(Episode {
            name: path.file_name().unwrap().to_str().unwrap().to_string(),
            metadata: VideoMetadata::new(&md_path)?,
            md_path,
            number,
            path,
            thumbnail_path,
        })
    }

    pub fn update_metadata(&mut self) -> io::Result<()> {
        let mut new_md_path = self.path.clone();
        new_md_path.set_extension("md");

        fs::rename(new_md_path, &self.md_path)?;
        self.metadata = VideoMetadata::new(&self.md_path)?;
        Ok(())
    }

    /// Runs episode on the current time (Restart it if it has been already watched)
    ///
    /// Returns [`io::Error`] if the mpv fails to launch it or the metadata can not be updated.
    pub fn run(&mut self) -> io::Result<()> {
        info!("Running {}", self.name);

        Backend::run_mpv(
            format!(
                "--start={} \"{}\"",
                if self.metadata.watched {
                    0.0
                } else {
                    self.metadata.current
                },
                self.path.display()
            )
            .as_str(),
        )?;

        self.update_metadata()
    }
}

impl Meta for Episode {
    fn thumbnail(&self) -> PathBuf {
        self.thumbnail_path.clone()
    }

    fn description(&self) -> String {
        format!(
            "Name: {}\n\nDuration: {}{}\nWatched: {}",
            self.name,
            VideoMetadata::to_time(self.metadata.duration),
            if !self.metadata.watched && self.metadata.current > 1.0 {
                format!(
                    "\nCurrent: {}",
                    VideoMetadata::to_time(self.metadata.current)
                )
            } else {
                "".to_string()
            },
            if self.metadata.watched { "Yes" } else { "No" },
        )
    }
}
