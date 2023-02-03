use core::fmt::Debug;
use std::{fs, io, path::PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct VideoMetadata {
    pub duration: f64,
    pub current: f64,
    pub remaining: f64,
    pub watched: bool,
}

impl VideoMetadata {
    pub fn new(path: &PathBuf) -> io::Result<VideoMetadata> {
        let metadata = fs::read_to_string(path)?;

        let mut duration = 0.0;
        let mut current: f64 = 0.0;
        let mut remaining = 0.0;
        let mut watched = false;

        for line in metadata.lines() {
            if line.contains("Duration") {
                duration = line
                    .replace("Duration:", "")
                    .trim()
                    .parse()
                    .unwrap_or_default();
            } else if line.contains("Current") {
                current = line
                    .replace("Current:", "")
                    .trim()
                    .parse()
                    .unwrap_or_default();
            } else if line.contains("Remaining") {
                remaining = line
                    .replace("Remaining:", "")
                    .trim()
                    .parse()
                    .unwrap_or_default();
            } else if line.contains("Status") {
                watched = line
                    .replace("Status:", "")
                    .trim()
                    .parse()
                    .unwrap_or_default();
            }
        }

        if current < 1.0 {
            current = 0.0;
        }

        Ok(VideoMetadata {
            duration,
            current,
            remaining,
            watched,
        })
    }

    pub fn to_time(duration: f64) -> String {
        let minutes = (duration / 60.0).trunc();
        let seconds = (((duration / 60.0) - minutes) * 60.0).floor();
        format!("{minutes:02.0}:{seconds:02.0}")
    }
}
