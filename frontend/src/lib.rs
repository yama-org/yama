mod list;
mod loading;
mod modal;
mod panels;
mod theme;

use crate::list::List;
use crate::loading::LoadingCircle;
use crate::modal::Modal;
use crate::panels::Pane;
use crate::theme::widget::Element;
use crate::theme::Theme;
use iced::widget::{canvas, column, container, pane_grid::Direction, text};
use iced::{alignment, executor, keyboard, window};
use iced::{Application, Command, Length, Settings, Subscription};
use iced_native::{event, subscription, Event};
use std::time::Instant;

pub type Result = std::result::Result<(), iced::Error>;

#[derive(Debug)]
pub enum State {
    Normal,
    Loading,
    Watching,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loading(Instant),
    PaneAction(panels::Message),
}

#[derive(Debug)]
pub struct GUI {
    state: State,
    pane: Pane,
    loading: LoadingCircle,
}

impl GUI {
    pub fn execute() -> Result {
        GUI::run(Settings {
            id: None,
            antialiasing: true,
            window: window::Settings {
                size: (1920, 1080),
                ..window::Settings::default()
            },
            //default_font: Some(include_bytes!("../res/linuxBiolinum.ttf")),
            ..Settings::default()
        })
    }
}

impl Application for GUI {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_: ()) -> (Self, Command<Message>) {
        (
            GUI {
                state: State::Loading,
                pane: Pane::new(),
                loading: LoadingCircle::new(),
            },
            Command::perform(
                async { panels::Message::LoadingTitles },
                Message::PaneAction,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("YAMA")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self.state {
            State::Loading => match message {
                Message::Loading(instant) => self.loading.update(instant),
                Message::PaneAction(message) => return self.pane.update(message, &mut self.state),
            },
            State::Normal | State::Watching => {
                if let Message::PaneAction(message) = message {
                    return self.pane.update(message, &mut self.state);
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.state {
            //Loading subscription, disables input
            State::Loading => window::frames().map(Message::Loading),

            //Input subscription
            State::Normal => subscription::events_with(|event, status| {
                if let event::Status::Captured = status {
                    return None;
                }

                match event {
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key_code,
                        modifiers: _,
                    }) => handle_hotkey(key_code),
                    _ => None,
                }
            }),
            _ => Subscription::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        let help = text("Up and Down to Move, Right or Enter to Accept")
            .width(Length::Fill)
            .size(20)
            .horizontal_alignment(alignment::Horizontal::Right);

        let title = text("Y.A.M.A - Your Anime Manager Automata")
            .size(26)
            .vertical_alignment(alignment::Vertical::Center);

        let content = container(column![title, self.pane.view(), help].spacing(10))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(35);

        match self.state {
            State::Loading => {
                let modal = container(
                    canvas(&self.loading)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .width(Length::Units(300))
                .height(Length::Units(300))
                .padding(10);

                Modal::new(content, modal).into()
            }
            State::Watching => {
                let modal = container(
                    text("Watching episode...")
                        .size(48)
                        .vertical_alignment(alignment::Vertical::Center)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .width(Length::Units(300))
                .height(Length::Units(300))
                .padding(10);

                Modal::new(content, modal).into()
            }
            _ => content.into(),
        }
    }
}

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    use keyboard::KeyCode;

    let msg = match key_code {
        KeyCode::Up => panels::Message::FocusItem(Direction::Up),
        KeyCode::Down => panels::Message::FocusItem(Direction::Down),
        KeyCode::Right => panels::Message::Enter,
        KeyCode::Enter => panels::Message::Enter,
        KeyCode::Left => panels::Message::Back,
        _ => return None,
    };

    Some(Message::PaneAction(msg))
}
