mod config;
pub mod embedded;
mod frontend;
pub mod keybindings;
mod widgets;

pub type Result<T> = anyhow::Result<T>;
pub use frontend::Frontend;
