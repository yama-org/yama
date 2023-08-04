use super::{FocusedType, InnerData};

use crate::widgets::Element;

use backend::MetaType;
use bridge::{cache::*, FrontendMessage};

use iced::font::Family;
use iced::widget::{column as col, container, image, scrollable, text};
use iced::{Font, Length};
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
                FocusedType::Title(_) => {
                    container(scrollable(data.view()).id(SCROLLABLE_ID.clone()))
                        .width(Length::Fill)
                        .padding(15)
                        .center_y()
                        .into()
                }

                FocusedType::Episode(_, _) => {
                    container(scrollable(data.view()).id(SCROLLABLE_ID.clone()))
                        .width(Length::Fill)
                        .padding(15)
                        .center_y()
                        .into()
                }
            },

            Self::Metadata(meta) => {
                let handle = match &meta.thumbnail {
                    Some(path) => image::Handle::from_path(path.as_ref()),
                    None => image::Handle::from_memory(crate::embedded::NO_TUMBNAIL),
                };

                let thumbnail = match meta.mtype {
                    MetaType::Title => container(
                        scrollable(image::Image::new(handle)).height(Length::Fixed(167.0)),
                    ),
                    MetaType::Episode => container(image::Image::new(handle)),
                };

                container(scrollable(
                    col![
                        thumbnail,
                        text(meta.title.clone()).font(Font {
                            family: Family::Name("Kumbh Sans"),
                            weight: iced::font::Weight::Semibold,
                            ..Default::default()
                        }),
                        text(meta.description.clone())
                    ]
                    .spacing(20),
                ))
                .width(Length::Fill)
                .padding(15)
                .into()
            }
        }
    }
}
