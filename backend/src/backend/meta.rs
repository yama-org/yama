use crate::Episode;
use crate::Title;

use std::path::Path;
use std::sync::Arc;

/// A [trait] for displaying a [thumbnail] and [description] in the [metadata panel] of the [frontend] crate.
pub trait Meta {
    /// Returns a path to the [`Meta`][Meta] thumbnail.
    fn thumbnail(&self) -> Option<Arc<Path>>;

    /// Returns a [str] description of the [`Meta`][Meta] element.
    fn description(&self) -> Arc<str>;
}

impl Meta for Title {
    fn thumbnail(&self) -> Option<Arc<Path>> {
        if let Some(data) = self.data.as_ref() {
            return Some(Arc::from(data.thumbnail_path.as_path()));
        }

        None
    }

    fn description(&self) -> Arc<str> {
        if let Some(data) = &self.data {
            return Arc::from(format!(
                "{}\n\nDescription: {}\n\nGenres: {:?}\n\nStudio: {}",
                data.media.title.english, data.media.description, data.media.genres, data.studio
            ));
        }

        Arc::from("No description found...")
    }
}

impl Meta for Episode {
    fn thumbnail(&self) -> Option<Arc<Path>> {
        Some(Arc::from(self.thumbnail_path.as_path()))
    }

    fn description(&self) -> Arc<str> {
        Arc::from(format!("Name: {}\n\n{}", self.name, self.metadata.to_str(),))
    }
}
