use super::{FocusedType, InnerData};

use crate::widgets::Element;

use bridge::{cache::*, FrontendMessage};

use iced::widget::{column, container, image, scrollable, text};
use iced::Length;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

/// The [`Panels`][super::Panels] renderable [`elements`][iced::Element].
#[derive(Debug)]
pub enum InnerPanel {
    Listdata(FocusedType),
    Metadata(Arc<MetaCache>),
}

impl InnerPanel {
    /// Returns the [`Pane`][iced::widget::pane_grid::Pane] elements.
    pub fn view<'a>(&self, data: &InnerData) -> Element<'a, FrontendMessage> {
        match self {
            Self::Listdata(ftype) => match ftype {
                FocusedType::Title(_) => container(
                    scrollable(data.view())
                        .height(Length::Shrink)
                        .id(SCROLLABLE_ID.clone()),
                )
                .width(Length::Fill)
                .padding(5)
                .center_y()
                .into(),

                FocusedType::Episode(_, _) => container(
                    scrollable(data.view())
                        .height(Length::Shrink)
                        .id(SCROLLABLE_ID.clone()),
                )
                .width(Length::Fill)
                .padding(5)
                .center_y()
                .into(),
            },

            Self::Metadata(meta) => {
                let font = iced::Font::External {
                    name: "Kumbh Sans Bold",
                    bytes: include_bytes!("../../../../res/fonts/KumbhSans-Bold.ttf"),
                };

                let handle = match &meta.thumbnail {
                    Some(path) => image::Handle::from_path(path.as_ref()),
                    None => image::Handle::from_memory(super::NO_TUMBNAIL),
                };

                container(scrollable(
                    column![
                        image::Image::new(handle),
                        text(meta.description.clone()).font(font)
                    ]
                    .spacing(10),
                ))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(15)
                .into()
            }
        }
    }
}
