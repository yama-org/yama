use backend::{Backend, Episode, Meta, Title};
use std::{path::Path, sync::Arc};

/// A cached copy of backend data to be shared with the frontend thread without the need of lockers.
///
/// It uses some [`Arc`][Arc] pointers to actually avoid making a deep-copy of some values,
/// but still allows the mutability of [`TitleCache`][TitleCache].
#[derive(Debug, Clone)]
pub struct Cache {
    pub size: usize,
    pub titles_names: Arc<[Arc<str>]>,
    titles_cache: Vec<TitleCache>,
}

impl Cache {
    /// Creates a new instance of [`Cache`][Cache], requires a reference to [`Backend`][Backend].
    pub fn new(backend: &Backend) -> Self {
        Self {
            titles_names: backend.cache(),
            titles_cache: backend.map(TitleCache::without_episodes),
            size: backend.count,
        }
    }

    /// Returns a reference to the indexed [`TitleCache`][TitleCache].
    /// ## Panics
    /// May panic if `number` is out of bounds.
    pub fn get_title(&self, number: usize) -> &TitleCache {
        &self.titles_cache[number]
    }

    /// Returns a mutable reference to the indexed [`TitleCache`][TitleCache].
    /// ## Panics
    /// May panic if `number` is out of bounds.
    pub fn get_mut_title(&mut self, number: usize) -> &mut TitleCache {
        &mut self.titles_cache[number]
    }

    /// Returns the number of episodes that the indexed [`TitleCache`][TitleCache] has.
    /// ## Panics
    /// May panic if `number` is out of bounds.
    pub fn get_title_size(&self, number: usize) -> usize {
        self.titles_cache[number].size
    }

    /// Returns a reference to the [`MetaCache`][MetaCache] of the indexed [`TitleCache`][TitleCache].
    ///
    /// It's wrapped in an [`Arc`][Arc] pointer to avoid unnecessaries copies.
    /// ## Panics
    /// May panic if `number` is out of bounds.
    pub fn get_title_cache(&self, number: usize) -> Arc<MetaCache> {
        self.titles_cache[number].cache.clone()
    }

    /// Sets the [`TitleCache`][TitleCache] of the indexed [`TitleCache`][TitleCache].
    /// ## Panics
    /// May panic if `number` is out of bounds.
    pub fn set_title_cache(&mut self, title_cache: TitleCache, number: usize) {
        self.titles_cache[number] = title_cache;
    }
}

/// A cached copy of a title data to be shared with the frontend thread without the need of lockers.
///
/// It uses some [`Arc`][Arc] pointers to actually avoid making a deep-copy of some values,
/// but still allows the mutability of [`EpisodeCache`][EpisodeCache].
#[derive(Debug, Clone, Default)]
pub struct TitleCache {
    pub size: usize,
    cache: Arc<MetaCache>,
    pub episodes_names: Option<Arc<[Arc<str>]>>,
    episodes_cache: Option<Vec<EpisodeCache>>,
}

impl TitleCache {
    /// Creates a new instance of [`TitleCache`][TitleCache] without episodes loaded, requires a reference to [`Title`][Title].
    pub fn without_episodes(title: &Title) -> Self {
        Self {
            size: title.count as usize,
            cache: Arc::from(MetaCache::from(title as &dyn Meta)),
            episodes_names: None,
            episodes_cache: None,
        }
    }

    /// Creates a new instance of [`TitleCache`][TitleCache] with its episodes loaded, requires a reference to [`Title`][Title].
    pub fn with_episodes(title: &Title) -> Self {
        Self {
            size: title.count as usize,
            cache: Arc::from(MetaCache::from(title as &dyn Meta)),
            episodes_names: Some(title.cache()),
            episodes_cache: Some(title.map(EpisodeCache::new)),
        }
    }

    /// Returns a reference to the indexed [`EpisodeCache`][EpisodeCache] or [`None`][None] if its empty.
    pub fn get_episode(&self, number: usize) -> Option<&EpisodeCache> {
        self.episodes_cache.as_ref()?.get(number)
    }

    /// Returns a reference to the [`MetaCache`][MetaCache] of the indexed [`EpisodeCache`][EpisodeCache] or [`None`][None] if its empty.
    ///
    /// It's wrapped in an [`Arc`][Arc] pointer to avoid unnecessaries copies.
    pub fn get_episode_cache(&self, number: usize) -> Option<Arc<MetaCache>> {
        Some(self.episodes_cache.as_ref()?.get(number)?.cache.clone())
    }

    /// Sets the [`EpisodeCache`][EpisodeCache] of the indexed [`EpisodeCache`][EpisodeCache].
    pub fn set_episode_cache(&mut self, episode_cache: EpisodeCache) {
        let number = episode_cache.number as usize;

        if let Some(eps) = self.episodes_cache.as_mut() {
            if let Some(ep) = eps.get_mut(number) {
                *ep = episode_cache
            }
        }
    }
}

/// A cached copy of an episode data to be shared with the frontend thread without the need of lockers.
///
/// It uses some [`Arc`][Arc] pointers to actually avoid making a deep-copy of some values.
/// Instead of mutate an instance it's recommended to just replace it.
#[derive(Debug, Clone, Default)]
pub struct EpisodeCache {
    pub number: u16,
    pub watched: bool,
    cache: Arc<MetaCache>,
}

impl EpisodeCache {
    /// Creates a new instance of [`EpisodeCache`][EpisodeCache], requires a reference to [`Episode`][Episode].
    pub fn new(episode: &Episode) -> Self {
        Self {
            number: episode.number,
            watched: episode.metadata.watched,
            cache: Arc::from(MetaCache::from(episode as &dyn Meta)),
        }
    }
}

/// A cached copy of metadata to be shared with the frontend thread without the need of lockers.
/// Used for displaying [`Title`][Title] and [`Episode`][Episode] thumbnails and description.
///
/// It uses some [`Arc`][Arc] pointers to actually avoid making a deep-copy of some values.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MetaCache {
    pub thumbnail: Option<Arc<Path>>,
    pub description: Arc<str>,
}

impl MetaCache {
    /// Creates an empty instance of [`MetaCache`][MetaCache].
    pub fn empty() -> Self {
        Self {
            thumbnail: None,
            description: Arc::from("No description found..."),
        }
    }
}

impl Default for MetaCache {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<&dyn Meta> for MetaCache {
    fn from(title: &dyn Meta) -> Self {
        Self {
            thumbnail: title.thumbnail(),
            description: title.description(),
        }
    }
}
