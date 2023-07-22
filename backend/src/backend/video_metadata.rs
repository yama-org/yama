use crate::Result;

use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{fs, path::PathBuf};

/// [`Episode`][crate::Episode] information. Serialized for easy parsing.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub struct VideoMetadata {
    pub duration: f64,
    pub current: f64,
    pub remaining: f64,
    pub watched: bool,
}

impl VideoMetadata {
    /// Creates a new [`VideoMetadata`][VideoMetadata] from a [`PathBuf`][PathBuf].
    ///
    /// The _path_ should point to a valid _.md_ json-formatted file.
    pub fn new(path: &PathBuf) -> Result<VideoMetadata> {
        let metadata = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&metadata)?)
    }

    /// Formats a [f64] value to a XX:YY time format.
    /// Where X: Minutes and Y: Seconds.
    fn format_time(time: f64) -> Box<str> {
        let minutes = (time / 60.0).trunc();
        let seconds = (((time / 60.0) - minutes) * 60.0).floor();
        format!("{minutes:02.0}:{seconds:02.0}").into_boxed_str()
    }

    /// Formats [`VideoMetadata`][VideoMetadata] into a pretty [`str`][str].
    pub fn to_str(&self) -> Box<str> {
        format!(
            "Duration: {}{}\nWatched: {}",
            Self::format_time(self.duration),
            if !self.watched && self.current > 1.0 {
                format!("\nCurrent: {}", Self::format_time(self.current))
            } else {
                String::new()
            },
            if self.watched { "Yes" } else { "No" }
        )
        .into_boxed_str()
    }

    /// Creates a valid _.md_ json-formatted file, with a default [`VideoMetadata`][VideoMetadata]
    /// to the referenced _path_.
    ///
    /// The only altered value is the _duration_ of the video.
    pub fn default_file(duration: f64, path: &PathBuf) -> Result<()> {
        let mut file = fs::File::create(path)?;
        let vm = VideoMetadata {
            duration,
            ..Default::default()
        };

        let parsed = serde_json::to_string_pretty(&vm)?;
        Ok(file.write_all(parsed.as_bytes())?)
    }

    /// Creates a valid _.md_ json-formatted file, from the [`VideoMetadata`][VideoMetadata]
    /// passed to the referenced _path_.
    pub fn create_file(metadata: &VideoMetadata, path: &PathBuf) -> Result<()> {
        let mut file = fs::File::create(path)?;
        let parsed = serde_json::to_string_pretty(metadata)?;
        Ok(file.write_all(parsed.as_bytes())?)
    }

    /// Marks the [`VideoMetadata`][VideoMetadata] as watched or not.
    pub fn as_watched(&mut self) {
        self.watched = !self.watched;
        self.current = if !self.watched { 0.0 } else { self.duration }
    }
}
