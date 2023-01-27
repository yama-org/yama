use once_cell::unsync::OnceCell;
use std::{
    env, fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    process::Command,
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct Config {
    pub series_path: PathBuf,
}

impl Config {
    pub fn new(path: PathBuf) -> Result<Config, &'static str> {
        Ok(Config { series_path: path })
    }
}

#[derive(Debug)]
pub struct Backend {
    pub titles: Vec<Arc<Mutex<Title>>>,
    pub count: usize,
    cache: OnceCell<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Title {
    pub name: String,
    pub path: PathBuf,
    pub count: usize,
    episodes: Vec<Episode>,
    cache: OnceCell<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Episode {
    pub number: usize,
    pub name: String,
    pub path: PathBuf,
    pub md_path: PathBuf,
    pub metadata: VideoMetadata,
    pub thumbnail_path: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub struct VideoMetadata {
    pub duration: f64,
    pub current: f64,
    pub remaining: f64,
    pub watched: bool,
}

impl Default for Backend {
    fn default() -> Self {
        Backend::new()
    }
}

impl Backend {
    pub fn new() -> Self {
        //TODO: Read config from a file
        let config = Config::new(PathBuf::from("./series")).unwrap();
        let titles = Self::get_titles(config).unwrap();

        Backend {
            cache: OnceCell::default(),
            count: titles.len(),
            titles,
        }
    }

    pub fn run_process(cmd: &str) -> std::io::Result<std::process::Output> {
        let output = Command::new("sh")
            .current_dir(env::current_dir().unwrap())
            .args(["-c", cmd])
            .output()?;

        if !output.status.success() {
            return Err(Error::new(ErrorKind::Other, "[ERROR] - Exit code failure."));
        }
        Ok(output)
    }

    pub fn run_mpv(command: &str) -> std::io::Result<std::process::Output> {
        Backend::run_process(format!("mpv --script=./mpv_scripts/save_info.lua {command}").as_str())
    }

    fn get_files_from_dir(path: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
        let files: Vec<PathBuf> = fs::read_dir(path)?
            .into_iter()
            .flatten()
            .map(|x| x.path())
            .collect();

        Ok(files)
    }

    fn get_titles(config: Config) -> std::io::Result<Vec<Arc<Mutex<Title>>>> {
        let mut series: Vec<Title> = Self::get_files_from_dir(&config.series_path)?
            .into_iter()
            .filter(|x| match fs::metadata(x) {
                Ok(f) => f.is_dir(),
                Err(_) => false,
            })
            .map(Title::new)
            .collect();

        series.sort_by(|a, b| alphanumeric_sort::compare_str(&a.name, &b.name));

        Ok(series
            .into_iter()
            .map(|t| Arc::new(Mutex::new(t)))
            .collect())
    }

    pub fn view(&self) -> &[String] {
        self.cache.get_or_init(|| {
            self.titles
                .iter()
                .map(|t| {
                    let t = t.lock().unwrap();
                    t.name.clone()
                })
                .collect()
        })
    }
}

impl Title {
    pub fn new(path: PathBuf) -> Self {
        Title {
            name: path.display().to_string().replace("./series/", ""),
            cache: OnceCell::default(),
            episodes: Vec::new(),
            count: 0,
            path,
        }
    }

    fn get_episodes(path: &PathBuf) -> std::io::Result<Vec<Episode>> {
        //TODO: Improve video file detection
        const VIDEO_FORMATS: &[&str] = &[".mp4", ".mkv"];

        let mut episodes: Vec<PathBuf> = Backend::get_files_from_dir(path)?
            .into_iter()
            .filter(|x| match fs::metadata(x) {
                Ok(f) => f.is_file(),
                Err(_) => false,
            })
            .filter(|x| {
                let k = x.display().to_string();
                VIDEO_FORMATS.iter().any(|y| k.to_lowercase().contains(y))
            })
            .collect();

        episodes.sort_by(|a, b| {
            a.to_str()
                .unwrap()
                .to_lowercase()
                .cmp(&b.to_str().unwrap().to_lowercase())
        });

        let episodes = episodes
            .into_iter()
            .enumerate()
            .map(|(i, path)| {
                let (tx, rx) = mpsc::channel();

                thread::spawn(move || {
                    let ep = Episode::new(path, i + 1).unwrap();
                    tx.send(ep).unwrap();
                });

                rx.recv().unwrap()
            })
            .collect();

        Ok(episodes)
    }

    pub fn get_or_init(&mut self, number: usize) -> &mut Episode {
        if self.episodes.is_empty() {
            self.episodes = Self::get_episodes(&self.path).unwrap();
            self.count = self.episodes.len();
        }
        &mut self.episodes[number]
    }

    pub fn view(&self) -> &[String] {
        self.cache
            .get_or_init(|| self.episodes.iter().map(|e| e.name.clone()).collect())
    }

    pub fn get_episode_ref(&self, number: usize) -> &Episode {
        &self.episodes[number]
    }

    pub fn get_episode(&self, number: usize) -> Episode {
        self.episodes[number].clone()
    }

    pub fn is_loaded(&self) -> bool {
        self.path.join(".metadata/").is_dir()
    }
}

impl Episode {
    pub fn new(path: PathBuf, number: usize) -> std::io::Result<Self> {
        Self::create_cache_folder(number, path.parent().unwrap())?;

        let mut old_md_path = path.clone();
        old_md_path.set_extension("md");

        let md_path = old_md_path
            .parent()
            .unwrap()
            .join(format!(".metadata/episode_{number}/"))
            .join(old_md_path.file_name().unwrap());

        if fs::metadata(&md_path).is_err() {
            let command = format!("--no-video --end=0.0001 \"{}\"", path.display());
            Backend::run_mpv(&command)?;
        }

        if fs::metadata(&old_md_path).is_ok() {
            fs::rename(old_md_path, &md_path)?;
        }

        let thumbnail_path = path
            .parent()
            .unwrap()
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
            metadata: VideoMetadata::new(&md_path).unwrap(),
            md_path,
            number,
            path,
            thumbnail_path,
        })
    }

    fn create_cache_folder(number: usize, title_path: &Path) -> std::io::Result<()> {
        fs::create_dir_all(format!(
            "{}/.metadata/episode_{number}",
            title_path.display()
        ))?;
        Ok(())
    }

    pub fn update_metadata(&mut self) -> std::io::Result<()> {
        let mut new_md_path = self.path.clone();
        new_md_path.set_extension("md");

        fs::rename(new_md_path, &self.md_path)?;
        self.metadata = VideoMetadata::new(&self.md_path).unwrap();
        Ok(())
    }

    pub fn description(&self) -> String {
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

    pub fn run(&mut self) -> std::io::Result<()> {
        Backend::run_mpv(
            format!(
                "--start={} \"{}\"",
                self.metadata.current,
                self.path.display()
            )
            .as_str(),
        )?;
        self.update_metadata()
    }
}

impl VideoMetadata {
    pub fn new(path: &PathBuf) -> std::io::Result<VideoMetadata> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn series_titles() {
        let backend = Backend::new();
        let series: Vec<String> = backend
            .titles
            .into_iter()
            .map(|t| t.lock().unwrap().name.clone())
            .collect();

        assert_eq!(
            vec!["Akiba Maid Wars", "Bocchi the Rock", "Girls Last Tour"],
            series
        );
    }

    #[test]
    #[serial]
    fn serie_episodes() {
        let backend = Backend::new();
        let episodes: Vec<String> = backend.titles[0]
            .lock()
            .unwrap()
            .episodes
            .clone()
            .into_iter()
            .map(|e| e.name)
            .collect();

        let base_name = &backend.titles[0].lock().unwrap().name;
        let mut episodes_test: Vec<String> = Vec::new();

        for i in 1..4 {
            episodes_test.push(format!("{base_name} - 0{i}.mkv"));
        }

        assert_eq!(episodes_test, *episodes)
    }

    #[test]
    #[serial]
    fn open_episode() {
        let backend = Backend::new();
        let mut episodes: Vec<Episode> = backend.titles[0].lock().unwrap().episodes.clone();

        let command = format!(
            "--start={} --end={} \"{}\"",
            episodes[0].metadata.duration - 5.0,
            episodes[0].metadata.duration - 4.0,
            episodes[0].path.display()
        );
        let output = Backend::run_mpv(&command).expect("[ERROR] - Failed to execute process.");
        assert!(output.status.success());

        episodes[0].update_metadata().unwrap();
        assert_eq!(1417.0, episodes[0].metadata.current.ceil());
    }

    #[test]
    #[serial]
    fn is_watched() {
        let backend = Backend::new();
        let mut episodes: Vec<Episode> = backend.titles[0].lock().unwrap().episodes.clone();

        // Running video with mpv
        let command = format!(
            "--start={} --end={} \"{}\"",
            episodes[1].metadata.duration - 5.0,
            episodes[1].metadata.duration - 4.0,
            episodes[1].path.display()
        );

        let output = Backend::run_mpv(&command).expect("[ERROR] - Failed to execute process.");
        assert!(output.status.success());
        episodes[1].update_metadata().unwrap();
        assert!(episodes[1].metadata.watched);
    }

    #[test]
    #[serial]
    fn is_serie_watched() {
        let backend = Backend::new();
        let mut episodes: Vec<Episode> = backend.titles[1].lock().unwrap().episodes.clone();
        let mut episodes_watched: Vec<bool> = Vec::new();

        for ep in episodes.iter_mut() {
            Backend::run_mpv(&format!(
                "--no-video --start={} --end={} \"{}\"",
                ep.metadata.duration - 5.0,
                ep.metadata.duration - 4.0,
                ep.path.display()
            ))
            .expect("[ERROR] - Failed to execute process.");
            ep.update_metadata().unwrap();
            episodes_watched.push(ep.metadata.watched);
        }

        assert_eq!(vec![true; episodes_watched.len()], episodes_watched);
    }
}
