mod config;
mod frontend;
mod widgets;

pub type Result<T> = anyhow::Result<T>;
pub use frontend::Frontend;
