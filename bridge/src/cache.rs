use std::path::PathBuf;

use backend::backend::{episode::Episode, title::Title, Backend};
use backend::Meta;

#[derive(Debug, Default, Clone)]
pub struct Cache {
    pub titles_map: Vec<(MetaCache, Vec<MetaCache>)>,
    pub titles_names: Vec<String>,
    pub episodes_names: Vec<Vec<String>>,
    pub episodes_watched: Vec<Vec<bool>>,
}

impl Cache {
    pub fn new(backend: &Backend) -> Cache {
        Cache {
            titles_map: backend
                .titles
                .iter()
                .map(|t| (MetaCache::from(t), Vec::new()))
                .collect(),
            titles_names: backend.titles.iter().map(|t| t.name.clone()).collect(),
            episodes_names: vec![Vec::new(); backend.count],
            episodes_watched: vec![Vec::new(); backend.count],
        }
    }

    pub fn set_title_cache(&mut self, title_cache: TitleCache, number: usize) {
        self.titles_map[number].1 = title_cache.episodes_cache;
        self.episodes_names[number] = title_cache.episodes_names;
        self.episodes_watched[number] = title_cache.episodes_watched;
    }

    pub fn set_episode_cache(&mut self, episode_cache: EpisodeCache, title_number: usize) {
        self.titles_map[title_number].1[episode_cache.number - 1] = episode_cache.cache;
        self.episodes_watched[title_number][episode_cache.number - 1] = episode_cache.watched;
    }

    pub fn cache_episodes(title: &Title) -> Vec<MetaCache> {
        title.episodes.iter().map(MetaCache::from).collect()
    }

    pub fn cache_episodes_names(title: &Title) -> Vec<String> {
        title.episodes.iter().map(|e| e.name.clone()).collect()
    }

    pub fn cache_episodes_watch_status(title: &Title) -> Vec<bool> {
        title.episodes.iter().map(|e| e.metadata.watched).collect()
    }
}

#[derive(Debug, Default, Clone)]
pub struct TitleCache {
    pub episodes_cache: Vec<MetaCache>,
    pub episodes_names: Vec<String>,
    pub episodes_watched: Vec<bool>,
}

impl TitleCache {
    pub fn new(title: &Title) -> TitleCache {
        TitleCache {
            episodes_cache: Cache::cache_episodes(title),
            episodes_names: Cache::cache_episodes_names(title),
            episodes_watched: Cache::cache_episodes_watch_status(title),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EpisodeCache {
    pub cache: MetaCache,
    pub watched: bool,
    pub number: usize,
}

impl EpisodeCache {
    pub fn new(episode: &Episode) -> EpisodeCache {
        EpisodeCache {
            cache: MetaCache::from(episode),
            watched: episode.metadata.watched,
            number: episode.number,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MetaCache {
    pub thumbnail: Option<PathBuf>,
    pub description: String,
}

impl From<&Title> for MetaCache {
    fn from(title: &Title) -> MetaCache {
        MetaCache {
            thumbnail: title.thumbnail(),
            description: title.description(),
        }
    }
}

impl From<&Episode> for MetaCache {
    fn from(episode: &Episode) -> MetaCache {
        MetaCache {
            thumbnail: episode.thumbnail(),
            description: episode.description(),
        }
    }
}

impl MetaCache {
    pub fn empty() -> MetaCache {
        MetaCache {
            thumbnail: None,
            description: "No Data".to_string(),
        }
    }
}
