use std::format;

use crate::theme::{self, widget::Element};
use backend::config::Config;
use bridge::FrontendMessage;
use iced::widget::{button, column, row, text};
use iced::{alignment, Length};
use iced_native::widget::svg;
use tracing::{info, warn};
use tracing_unwrap::ResultExt;

static FOLDER_SVG: &[u8] = include_bytes!("../../res/folder.svg");

pub struct GUIConfig;

impl GUIConfig {
    pub fn view<'a>(cfg: &Config) -> Element<'a, super::Message> {
        let folder_path = format!("{}", cfg.series_path.display());
        let folder_svg = svg(svg::Handle::from_memory(FOLDER_SVG))
            .width(Length::Fixed(25.0))
            .height(Length::Fixed(25.0));

        column![
            text("Configs"),
            row![
                button(folder_svg)
                    .on_press(FrontendMessage::FileDialog)
                    .style(theme::Button::Menu),
                button(text(folder_path))
                    .on_press(FrontendMessage::FileDialog)
                    .style(theme::Button::Focused),
            ]
            .spacing(5)
            .align_items(alignment::Alignment::Center),
        ]
        .spacing(25)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(alignment::Alignment::Center)
        .into()
    }

    pub fn file_dialog(cfg: &mut Config) {
        let path = std::env::current_dir().unwrap_or_log();
        let res = rfd::FileDialog::new().set_directory(path).pick_folder();
        info!("The user choose: {:#?}", res);

        if let Some(path) = res {
            if std::fs::metadata(&path).is_ok() {
                cfg.series_path = path;
                if let Err(error) = confy::store("yama", "config", cfg) {
                    warn!("Could not save config because: {:#?}", error)
                }

                return;
            }
        }

        warn!("Invalid path");
    }
}
