use crate::widgets::theme::{self, widget::Element};

use anyhow::{bail, Ok};
use backend::Config;
use bridge::{ConfigChange, FrontendMessage};

use iced::widget::{button, column, row, text, tooltip, vertical_space};
use iced::{alignment, Length};
use iced_aw::NumberInput;
use tracing::{info, warn};

pub struct GUIConfig;

impl GUIConfig {
    pub fn view<'a>(cfg: &Config) -> Element<'a, FrontendMessage> {
        let theme_name = capitalize(unsafe {
            cfg.theme_path
                .file_stem()
                .unwrap_unchecked()
                .to_str()
                .unwrap_unchecked()
        });

        let series_path = match &cfg.series_path {
            Some(path) => path.display().to_string(),
            None => "No path found".to_string(),
        };

        column![
            text("Configs")
                .style(theme::Text::Focused)
                .vertical_alignment(alignment::Vertical::Top)
                .horizontal_alignment(alignment::Horizontal::Center)
                .width(Length::Fill),
            tooltip(
                row![
                    text("Series Path: ")
                        .width(Length::Fixed(100.0))
                        .horizontal_alignment(alignment::Horizontal::Right),
                    button(text(series_path))
                        .on_press(FrontendMessage::UpdateConfig(ConfigChange::SeriesPath))
                        .style(theme::Button::Input)
                        .width(Length::Fill),
                ]
                .align_items(alignment::Alignment::Center)
                .spacing(15),
                "Folder to scan for Titles",
                tooltip::Position::Top,
            )
            .style(theme::Container::Tooltip),
            tooltip(
                row![
                    text("Min Time: ")
                        .width(Length::Fixed(100.0))
                        .horizontal_alignment(alignment::Horizontal::Right),
                    NumberInput::new(cfg.min_time, f32::MAX, |new_time| {
                        FrontendMessage::UpdateConfig(ConfigChange::MinTime(new_time))
                    })
                ]
                .align_items(alignment::Alignment::Center)
                .spacing(15),
                "Seconds remaining before the episode is considered watched",
                tooltip::Position::Top,
            )
            .style(theme::Container::Tooltip),
            tooltip(
                row![
                    text("Theme Path: ")
                        .width(Length::Fixed(100.0))
                        .horizontal_alignment(alignment::Horizontal::Right),
                    button(text(theme_name))
                        .on_press(FrontendMessage::UpdateConfig(ConfigChange::ThemePath))
                        .style(theme::Button::Input)
                        .width(Length::Fill),
                ]
                .align_items(alignment::Alignment::Center)
                .spacing(15),
                "Theme file",
                tooltip::Position::Top,
            )
            .style(theme::Container::Tooltip),
            vertical_space(Length::Fill),
            button(
                text("  Ok?  ")
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .width(Length::Fill)
            )
            .on_press(FrontendMessage::HideMenubar)
            .style(theme::Button::Menu)
            .width(Length::Fill)
        ]
        .width(Length::Fill)
        .spacing(25)
        .padding(15)
        .into()
    }

    pub fn change_series_path(cfg: &mut Config) -> crate::Result<()> {
        let path = std::env::current_dir()?;
        let res = rfd::FileDialog::new().set_directory(path).pick_folder();
        info!("The user choose: {:#?}", res);

        if let Some(path) = res {
            if std::fs::metadata(&path).is_ok() {
                std::env::set_current_dir(path.clone())?;
                cfg.series_path = Some(path);

                if let Err(error) = confy::store("yama", "config", cfg) {
                    bail!("Could not save config because: {:#?}", error)
                }

                return Ok(());
            }
        }

        bail!("Invalid path")
    }

    pub fn change_theme_path(cfg: &mut Config) {
        let path = confy::get_configuration_file_path("yama", "config")
            .unwrap()
            .parent()
            .unwrap()
            .join("themes");
        let res = rfd::FileDialog::new().set_directory(path).pick_file();
        info!("The user choose: {:#?}", res);

        if let Some(path) = res {
            if std::fs::metadata(&path).is_ok() {
                cfg.theme_path = path;
                if let Err(error) = confy::store("yama", "config", cfg) {
                    warn!("Could not save config because: {:#?}", error)
                }

                return;
            }
        }

        warn!("Invalid path");
    }

    pub fn change_min_time(cfg: &mut Config, new_time: f32) {
        cfg.min_time = new_time;

        if let Err(error) = confy::store("yama", "config", cfg) {
            warn!("Could not save config because: {:#?}", error)
        }
    }
}

/// Utils
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
