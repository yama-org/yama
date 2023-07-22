use crate::{
    config::GUIConfig,
    widgets::{theme, Element},
};

use backend::Config;
use bridge::{FrontendMessage as Message, MenuBar};

use iced::widget::{button, column as col, container, text, image};
use iced_native::{alignment, Length};

pub fn about<'a>() -> Element<'a, Message> {
    container(
        col![
            button(text("yama").style(theme::Text::Color(theme::FOCUS)))
                .on_press(Message::MenuBar(MenuBar::Yama)),
            text(format!(
                "\nVersion: {}\n\nBy {}",
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_AUTHORS")
            ))
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Center)
            .width(Length::Fill)
            .height(Length::Fill)
        ]
        .height(Length::Fixed(150.0))
        .align_items(alignment::Alignment::Center),
    )
    .center_y()
    .width(Length::Fixed(600.0))
    .height(Length::Fixed(300.0))
    .padding(10)
    .style(theme::Container::Box)
    .into()
}

pub fn config<'a>(cfg: &Config) -> Element<'a, Message> {
    container(GUIConfig::view(cfg))
        .width(Length::Fixed(300.0))
        .height(Length::Fixed(300.0))
        .padding(25)
        .style(theme::Container::Box)
        .into()
}

pub fn yama<'a>() -> Element<'a, Message> {
    let img = image::Handle::from_memory(crate::embedded::YAMA_PNG);
    container(image::Image::new(img))
        .center_x()
        .center_y()
        .width(Length::Fixed(300.0))
        .height(Length::Fixed(300.0))
        .padding(25)
        .style(theme::Container::Box)
        .into()
}
