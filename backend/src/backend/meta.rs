use crate::Episode;
use crate::Title;

use std::path::Path;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum MetaType {
    Title,
    Episode,
}

/// A [trait] for displaying a [thumbnail] and [description] in the [metadata panel] of the [frontend] crate.
pub trait Meta {
    /// Returns a path to the [`Meta`][Meta] thumbnail.
    fn thumbnail(&self) -> Option<Arc<Path>>;

    /// Returns a [str] description of the [`Meta`][Meta] element.
    fn description(&self) -> Arc<str>;

    /// Returns a [str] with the title of the [`Meta`][Meta] element.
    fn title(&self) -> Arc<str>;

    fn mtype(&self) -> MetaType;
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
                "{}\n\nStudio: {}",
                data.media.to_str(),
                data.studio
            ));
        }

        Arc::from("No description found...")
    }

    fn title(&self) -> Arc<str> {
        if let Some(data) = &self.data {
            return Arc::from(data.media.title.english.clone());
        }

        self.name.clone()
    }

    fn mtype(&self) -> MetaType {
        MetaType::Title
    }
}

impl Meta for Episode {
    fn thumbnail(&self) -> Option<Arc<Path>> {
        Some(Arc::from(self.thumbnail_path.as_path()))
    }

    fn description(&self) -> Arc<str> {
        Arc::from(self.metadata.to_str())
    }

    fn title(&self) -> Arc<str> {
        self.name.clone()
    }

    fn mtype(&self) -> MetaType {
        MetaType::Episode
    }
}
