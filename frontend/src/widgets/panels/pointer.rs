use crate::widgets::{mouse_area, theme, Element};

use bridge::{FrontendMessage, PanelAction};

use iced::widget::{button, pane_grid::Direction, text, Column};
use iced::Length;

pub type FocusedElement = usize;
pub type CurrentElement = usize;

/// A pointer to the current selected element of a list.
///
/// Made for a more accessible way to keep a notion of what element is the user currently sitting on.
#[derive(Debug, Clone, Copy)]
pub struct Pointer {
    pub focused: FocusedElement,
    pub size: usize,
}

impl Pointer {
    /// Creates a new [`Pointer`] with the size of the pointed list.
    pub fn new(size: usize) -> Self {
        Self { size, focused: 0 }
    }

    /// Changes the current [`FocusedElement`] according to the passed [`Direction`]
    /// (only [`Direction::Up`] or [`Direction::Down`] are accepted).
    ///
    /// Returns a vertical offset to align a [`scrollable`][iced::widget::scrollable].
    pub fn update(&mut self, direction: Direction) -> f32 {
        if self.size > 0 {
            match direction {
                Direction::Up => self.increment(),
                Direction::Down => self.decrement(),
                _ => (),
            }
            (1.0 / self.size as f32) * self.focused as f32
        } else {
            0.0
        }
    }

    pub fn jump_to(&mut self, to: usize) -> f32 {
        if to < self.size {
            self.focused = to;
            (1.0 / self.size as f32) * self.focused as f32
        } else {
            0.0
        }
    }

    pub fn plus(&mut self, to_add: isize) -> f32 {
        let mut to_add = to_add + self.focused as isize;
        to_add %= self.size as isize;

        if to_add < 0 {
            self.focused = self.size.saturating_add_signed(to_add);
        } else {
            self.focused = to_add as usize;
        }

        (1.0 / self.size as f32) * self.focused as f32
    }

    pub fn start(&mut self) -> f32 {
        self.focused = 0;
        (1.0 / self.size as f32) * self.focused as f32
    }

    pub fn end(&mut self) -> f32 {
        self.focused = self.size - 1;
        (1.0 / self.size as f32) * self.focused as f32
    }

    fn increment(&mut self) {
        self.focused = match self.focused == 0 {
            true => self.size - 1,
            false => self.focused - 1,
        }
    }

    fn decrement(&mut self) {
        self.focused = match self.focused == self.size - 1 {
            true => 0,
            false => self.focused + 1,
        }
    }

    /// Returns a [`Column`][iced_native::widget::Column] with a [`Button`][iced_native::widget::Button] for each element in _content_,
    /// and the [`Text`][iced_native::widget::Text] styled with the passed closure.
    ///
    /// The closure takes two arguments, the focused element in the [`Pointer`] and the current element
    /// being processed at the moment.
    pub fn view<'a, F>(
        &self,
        content: &[impl ToString + std::fmt::Display],
        style_text: F,
    ) -> Element<'a, FrontendMessage>
    where
        F: Fn(FocusedElement, CurrentElement) -> <theme::Theme as text::StyleSheet>::Style,
    {
        let mut arr: Vec<Element<'a, FrontendMessage>> = Vec::new();

        for (id, cont) in content.iter().enumerate() {
            arr.push(
                mouse_area(
                    button(text(cont).style(style_text(self.focused, id)))
                        .on_press(FrontendMessage::PaneAction(PanelAction::Enter))
                        .style(if id == self.focused {
                            theme::Button::Focused
                        } else {
                            theme::Button::Default
                        })
                        .width(Length::Fill),
                )
                .on_area(FrontendMessage::PaneAction(PanelAction::JumpTo(id)))
                .into(),
            );
        }

        Column::with_children(arr)
            .width(Length::Fill)
            .spacing(10)
            .into()
    }
}
