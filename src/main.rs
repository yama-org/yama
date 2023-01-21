pub mod backend;
pub mod frontend;

use crate::frontend::GUI;
use iced::{window, Application, Settings};

pub fn main() -> iced::Result {
    GUI::run(Settings {
        id: None,
        window: window::Settings {
            size: (1920, 1080),
            ..window::Settings::default()
        },
        //default_font: Some(include_bytes!("../res/linuxBiolinum.ttf")),
        ..Settings::default()
    })
}
