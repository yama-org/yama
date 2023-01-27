use crate::frontend::theme::{self, widget::Element};
use iced::widget::{button, pane_grid::Direction, text, Column};
use iced::Length;

#[derive(Debug, Clone)]
pub struct List {
    pub focused: usize,
    pub size: usize,
    pub font_size: u16,
}

impl List {
    pub fn new(focused: usize, size: usize) -> Self {
        List {
            focused,
            size,
            font_size: 24,
        }
    }

    pub fn update(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.increment(),
            Direction::Down => self.decrement(),
            _ => (),
        }
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

    pub fn view<'a>(
        &self,
        content: &[String],
        style: impl Fn(usize, usize, bool) -> <theme::Theme as text::StyleSheet>::Style,
    ) -> Element<'a, crate::frontend::Message> {
        let mut arr: Vec<Element<'a, crate::frontend::Message>> = Vec::new();

        for (id, cont) in content.iter().enumerate() {
            arr.push(
                button(
                    text(cont)
                        .size(self.font_size)
                        .style(style(self.focused, id, false)),
                )
                .style(if id == self.focused {
                    theme::Button::Focused
                } else {
                    theme::Button::Default
                })
                .width(Length::Fill)
                .into(),
            );
        }

        Column::with_children(arr)
            .width(Length::Fill)
            .spacing(10)
            .into()
    }
}
