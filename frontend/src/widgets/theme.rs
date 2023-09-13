use backend::Config;

use iced::theme::TextInput;
use iced::widget::{button, container, pane_grid, scrollable, text};
use iced::widget::{svg, text_input};
use iced::{application, color, Color};
use iced_aw::style::{number_input, NumberInputStyles};
use serde::{Deserialize, Serialize};

// Always import widget types from this module since it
// uses our custom theme instead of the built-in iced::Theme.
// Otherwise you will get compilation errors since iced::Element
// expects use of iced::Theme by default.
pub mod widget {
    #![allow(dead_code)]
    use super::Theme;

    pub type Renderer = iced::Renderer<Theme>;
    pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
    pub type Container<'a, Message> = iced::widget::Container<'a, Message, Renderer>;
    pub type Button<'a, Message> = iced::widget::Button<'a, Message, Renderer>;
    pub type PaneGrid<'a, Message> = iced::widget::pane_grid::PaneGrid<'a, Message, Renderer>;
    pub type Content<'a, Message> = iced::widget::pane_grid::Content<'a, Message, Renderer>;
    pub type TitleBar<'a, Message> = iced::widget::pane_grid::TitleBar<'a, Message, Renderer>;
    pub type Scrollable<'a, Message> = iced::widget::scrollable::Scrollable<'a, Message, Renderer>;
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Color,
    pub text: Color,
    pub focus: Color,
    pub unfocus: Color,
    pub watched: Color,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct ThemeParser {
    background: [u8; 3],
    text: [u8; 3],
    focus: [u8; 3],
    unfocus: [u8; 3],
    watched: [u8; 3],
}

impl Default for Theme {
    fn default() -> Self {
        let cfg: Config = confy::load("yama", "config").expect("Could not load config file.");
        let content = std::fs::read_to_string(cfg.theme_path).unwrap();
        let tp: ThemeParser = serde_json::from_str(&content).unwrap();

        Self {
            background: Color::from_rgb8(tp.background[0], tp.background[1], tp.background[2]),
            text: Color::from_rgb8(tp.text[0], tp.text[1], tp.text[2]),
            focus: Color::from_rgb8(tp.focus[0], tp.focus[1], tp.focus[2]),
            unfocus: Color::from_rgb8(tp.unfocus[0], tp.unfocus[1], tp.unfocus[2]),
            watched: Color::from_rgb8(tp.watched[0], tp.watched[1], tp.watched[2]),
        }
    }
}

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: self.background,
            text_color: self.text,
        }
    }
}

impl svg::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> svg::Appearance {
        svg::Appearance {
            color: Some(self.text),
        }
    }
}

impl iced_aw::number_input::StyleSheet for Theme {
    type Style = NumberInputStyles;

    fn active(&self, _style: &Self::Style) -> number_input::Appearance {
        number_input::Appearance {
            button_background: None,
            icon_color: self.unfocus,
        }
    }

    fn pressed(&self, _style: &Self::Style) -> number_input::Appearance {
        number_input::Appearance {
            button_background: None,
            icon_color: self.focus,
        }
    }

    fn disabled(&self, _style: &Self::Style) -> number_input::Appearance {
        number_input::Appearance {
            button_background: None,
            icon_color: self.watched,
        }
    }
}

impl text_input::StyleSheet for Theme {
    type Style = TextInput;

    /// Produces the style of an active text input.
    fn active(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: self.background.into(),
            border_color: self.unfocus,
            border_radius: 2.0.into(),
            border_width: 2.0,
            icon_color: self.unfocus,
        }
    }

    /// Produces the style of a focused text input.
    fn focused(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: self.background.into(),
            border_color: self.focus,
            border_radius: 2.0.into(),
            border_width: 2.0,
            icon_color: self.focus,
        }
    }

    /// Produces the style of a disabled text input.
    fn disabled(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: self.background.into(),
            border_color: self.watched,
            border_radius: 2.0.into(),
            border_width: 2.0,
            icon_color: self.watched,
        }
    }

    /// Produces the [`Color`] of the placeholder of a text input.
    fn placeholder_color(&self, _: &Self::Style) -> Color {
        self.watched
    }

    /// Produces the [`Color`] of the value of a text input.
    fn value_color(&self, _: &Self::Style) -> Color {
        self.text
    }

    /// Produces the [`Color`] of the value of a disabled text input.
    fn disabled_color(&self, _: &Self::Style) -> Color {
        self.watched
    }

    /// Produces the [`Color`] of the selection of a text input.
    fn selection_color(&self, _: &Self::Style) -> Color {
        self.focus
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
            Text::Default => text::Appearance {
                color: self.text.into(),
            },
            Text::Focused => text::Appearance {
                color: self.focus.into(),
            },
            Text::Watched => text::Appearance {
                color: self.watched.into(),
            },
            Text::WatchedFocus => text::Appearance {
                color: self.text.inverse().into(),
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
    Box,
    Tooltip,
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Default | Container::TitleBar => container::Appearance::default(),
            Container::Unfocused => container::Appearance {
                border_radius: 5.0.into(),
                border_width: 2.0,
                border_color: self.unfocus,
                ..Default::default()
            },
            Container::Focused => container::Appearance {
                border_color: self.focus,
                border_radius: 5.0.into(),
                border_width: 2.0,
                ..Default::default()
            },
            Container::Box => container::Appearance {
                background: Some(self.background.into()),
                border_radius: 5.0.into(),
                border_width: 5.0,
                border_color: self.focus,
                ..Default::default()
            },
            Container::Tooltip => container::Appearance {
                background: Some(self.background.into()),
                border_radius: 5.0.into(),
                border_width: 2.0,
                border_color: self.focus,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Button {
    Focused,
    #[default]
    Default,
    Menu,
    Input,
}

impl button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Focused => button::Appearance {
                background: Some(self.focus.inverse().into()),
                border_radius: 5.0.into(),
                ..Default::default()
            },
            Button::Input => button::Appearance {
                background: Some(self.background.into()),
                border_radius: 2.0.into(),
                border_width: 2.0,
                border_color: self.unfocus,
                ..Default::default()
            },
            Button::Default | Button::Menu => button::Appearance::default(),
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        match style {
            Button::Input => button::Appearance {
                border_color: self.focus,
                ..active
            },
            Button::Focused => button::Appearance { ..active },
            Button::Default => button::Appearance::default(),
            Button::Menu => button::Appearance {
                background: Some(self.unfocus.into()),
                text_color: self.focus,
                ..active
            },
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
                background: Some(Color::TRANSPARENT.into()),
                border_radius: 4.0.into(),
                border_width: 1.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: self.unfocus,
                    border_radius: 4.0.into(),
                    border_width: 1.0,
                    border_color: Color::TRANSPARENT,
                },
            },
        }
    }

    fn hovered(&self, style: &Self::Style, is_mouse_over_scrollbar: bool) -> scrollable::Scrollbar {
        match style {
            Scrollable::Primary => scrollable::Scrollbar {
                background: Some(Color::TRANSPARENT.into()),
                border_radius: 4.0.into(),
                border_width: 1.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: if is_mouse_over_scrollbar {
                        self.focus
                    } else {
                        self.unfocus
                    },
                    border_radius: 4.0.into(),
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

    fn hovered_region(&self, style: &Self::Style) -> pane_grid::Appearance {
        match style {
            PaneGrid::Default => pane_grid::Appearance {
                background: self.background.into(),
                border_radius: 5.0.into(),
                border_width: 2.0,
                border_color: self.focus,
            },
        }
    }
}
