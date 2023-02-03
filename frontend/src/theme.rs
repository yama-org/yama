use iced::widget::{button, container, pane_grid, scrollable, text};
use iced::{application, color, Color};

// Always import widget types from this module since it
// uses our custom theme instead of the built-in iced::Theme.
// Otherwise you will get compilation errors since iced::Element
// expects use of iced::Theme by default.
pub mod widget {
    #![allow(dead_code)]
    use crate::theme::Theme;

    pub type Renderer = iced::Renderer<Theme>;
    pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
    pub type Container<'a, Message> = iced::widget::Container<'a, Message, Renderer>;
    pub type Button<'a, Message> = iced::widget::Button<'a, Message, Renderer>;
    pub type PaneGrid<'a, Message> = iced::widget::pane_grid::PaneGrid<'a, Message, Renderer>;
    pub type Content<'a, Message> = iced::widget::pane_grid::Content<'a, Message, Renderer>;
    pub type TitleBar<'a, Message> = iced::widget::pane_grid::TitleBar<'a, Message, Renderer>;
    pub type Scrollable<'a, Message> = iced::widget::scrollable::Scrollable<'a, Message, Renderer>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Theme;

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: BACKGROUND,
            text_color: TEXT,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Text {
    #[default]
    Default,
    Focused,
    Watched,
    WatchedFocus,
    Color(Color),
}

impl text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        match style {
            Text::Default => text::Appearance { color: TEXT.into() },
            Text::Focused => text::Appearance {
                color: FOCUS.into(),
            },
            Text::Watched => text::Appearance {
                color: WATCHED.into(),
            },
            Text::WatchedFocus => text::Appearance {
                color: TEXT.inverse().into(),
            },
            Text::Color(c) => text::Appearance { color: Some(c) },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Container {
    #[default]
    Default,
    Unfocused,
    Focused,
    TitleBar,
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Default => container::Appearance::default(),
            Container::Unfocused => container::Appearance {
                border_color: UNFOCUS,
                border_width: 2.0,
                border_radius: 5.0,
                ..Default::default()
            },
            Container::Focused => container::Appearance {
                border_color: FOCUS,
                border_width: 2.0,
                border_radius: 5.0,
                ..Default::default()
            },
            Container::TitleBar => container::Appearance::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Button {
    Focused,
    #[default]
    Default,
}

impl button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Focused => button::Appearance {
                background: FOCUS.inverse().into(),
                border_radius: 5.0,
                ..Default::default()
            },
            Button::Default => button::Appearance::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Scrollable {
    #[default]
    Primary,
}

impl scrollable::StyleSheet for Theme {
    type Style = Scrollable;

    fn active(&self, style: &Self::Style) -> scrollable::Scrollbar {
        match style {
            Scrollable::Primary => scrollable::Scrollbar {
                background: Color::TRANSPARENT.into(),
                border_radius: 4.0,
                border_width: 1.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: UNFOCUS,
                    border_radius: 4.0,
                    border_width: 1.0,
                    border_color: Color::TRANSPARENT,
                },
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> scrollable::Scrollbar {
        match style {
            Scrollable::Primary => scrollable::Scrollbar {
                background: Color::TRANSPARENT.into(),
                border_radius: 4.0,
                border_width: 1.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: FOCUS,
                    border_radius: 4.0,
                    border_width: 1.0,
                    border_color: Color::TRANSPARENT,
                },
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum PaneGrid {
    #[default]
    Default,
}

impl pane_grid::StyleSheet for Theme {
    type Style = PaneGrid;

    fn picked_split(&self, style: &Self::Style) -> Option<pane_grid::Line> {
        match style {
            PaneGrid::Default => Some(pane_grid::Line {
                color: color!(0x45, 0x85, 0x88),
                width: 5.0,
            }),
        }
    }

    fn hovered_split(&self, style: &Self::Style) -> Option<pane_grid::Line> {
        match style {
            PaneGrid::Default => Some(pane_grid::Line {
                color: color!(0x45, 0x85, 0x88),
                width: 5.0,
            }),
        }
    }
}

pub const BACKGROUND: Color = Color::from_rgb(
    0x16 as f32 / 255.0,
    0x1b as f32 / 255.0,
    0x24 as f32 / 255.0,
);

pub const TEXT: Color = Color::from_rgb(
    0xf3 as f32 / 255.0,
    0xf3 as f32 / 255.0,
    0xf3 as f32 / 255.0,
);

pub const FOCUS: Color = Color::from_rgb(
    0x61 as f32 / 255.0,
    0xa3 as f32 / 255.0,
    0xff as f32 / 255.0,
);

pub const UNFOCUS: Color = Color::from_rgb(
    0x1d as f32 / 255.0,
    0x6a as f32 / 255.0,
    0xd5 as f32 / 255.0,
);

pub const WATCHED: Color = Color::from_rgb(
    0x27 as f32 / 255.0,
    0x2d as f32 / 255.0,
    0x3a as f32 / 255.0,
);
