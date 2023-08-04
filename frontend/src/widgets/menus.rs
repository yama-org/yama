use crate::{
    config::GUIConfig,
    widgets::{theme, Element},
};

use backend::Config;
use bridge::{FrontendMessage as Message, Modals};

use iced::widget::{button, column as col, container, image, scrollable, text};
use iced::{alignment, Length};
use std::sync::Arc;

pub fn about<'a>() -> Element<'a, Message> {
    container(
        col![
            button(
                text("yama")
                    .style(theme::Text::Focused)
                    .vertical_alignment(alignment::Vertical::Top)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .width(Length::Fill)
            )
            .on_press(Message::MenuBar(Modals::Yama)),
            text(format!(
                "Version: {}\n\nBy {}",
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_AUTHORS")
            ))
            .width(Length::Fill)
            .height(Length::Fill)
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Center)
        ]
        .align_items(alignment::Alignment::Center)
        .padding(15),
    )
    .center_x()
    .center_y()
    .width(Length::Fixed(600.0))
    .height(Length::Fixed(300.0))
    .style(theme::Container::Box)
    .padding(15)
    .into()
}

pub fn config<'a>(cfg: &Config) -> Element<'a, Message> {
    container(GUIConfig::view(cfg))
        .width(Length::Fixed(600.0))
        .height(Length::Fixed(400.0))
        .style(theme::Container::Box)
        .padding(15)
        .into()
}

pub fn yama<'a>() -> Element<'a, Message> {
    let img = image::Handle::from_memory(crate::embedded::YAMA_PNG);
    container(image::Image::new(img))
        .center_x()
        .center_y()
        .width(Length::Fixed(300.0))
        .height(Length::Fixed(300.0))
        .style(theme::Container::Box)
        .padding(25)
        .into()
}

pub fn error<'a>(err: Arc<str>) -> Element<'a, Message> {
    container(
        col![
            text("yama oops!")
                .style(theme::Text::Focused)
                .vertical_alignment(alignment::Vertical::Top)
                .horizontal_alignment(alignment::Horizontal::Center)
                .width(Length::Fill),
            text(err)
                .size(32)
                .vertical_alignment(alignment::Vertical::Center)
                .horizontal_alignment(alignment::Horizontal::Center)
                .width(Length::Fill)
                .height(Length::Fill),
            button("  Ok?  ")
                .on_press(Message::HideMenubar)
                .style(theme::Button::Menu)
        ]
        .align_items(alignment::Alignment::Center)
        .padding(15),
    )
    .center_x()
    .center_y()
    .width(Length::Fixed(600.0))
    .height(Length::Fixed(300.0))
    .style(theme::Container::Box)
    .padding(15)
    .into()
}

pub fn help<'a>() -> Element<'a, Message> {
    container(
        col![
            text("Keybindings:")
                .style(theme::Text::Focused)
                .vertical_alignment(alignment::Vertical::Top)
                .horizontal_alignment(alignment::Horizontal::Left)
                .width(Length::Fill),
            scrollable(text(HELP_KEYBINDINGS)).width(Length::Fill),
        ]
        .align_items(alignment::Alignment::Center)
        .spacing(15)
        .padding(15),
    )
    .center_x()
    .center_y()
    .width(Length::Fixed(615.0))
    .height(Length::Fixed(350.0))
    .style(theme::Container::Box)
    .padding(15)
    .into()
}

const HELP_KEYBINDINGS: &str = "
-- General:
K / UpArrow -> Move one item up
J / DownArror -> Move one item down
PageUp -> Move 5 items up
PageDown -> Move 5 items down
Home -> Go to the first item
End -> Go to the last item
5th MB ->
L / Enter / RightArrow -> Enter to Title/Watch Episode
Right MB / 4th MB ->
H / LeftArrow -> Go back to Titles
Q -> Exit yama

-- Episodes:
R -> Refresh Title episodes list
W -> Mark selected episode as watched/unwatched
Shift + W -> Mark previous episodes to the selected as watched/unwatched
";
