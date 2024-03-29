mod backend;
mod config;
mod networking;

pub use self::config::Config;
pub use backend::episode::Episode;
pub use backend::meta::Meta;
pub use backend::meta::MetaType;
pub use backend::title::Title;
pub use backend::video_metadata::VideoMetadata;
pub use backend::Backend;
pub use networking::anilist::Anilist;
pub use networking::discord::{Discord, APP_ID};

pub type Result<T> = anyhow::Result<T>;
