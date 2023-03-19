mod list;
mod loading;
mod modal;
pub mod panels;
mod theme;

use crate::list::List;
use crate::loading::LoadingCircle;
use crate::modal::Modal;
use crate::panels::Pane;
use crate::theme::widget::Element;
use crate::theme::Theme;

use bridge::BridgeMessage;
use bridge::FrontendMessage as Message;
use bridge::PanelsMessage;
use iced::widget::{canvas, column, container, pane_grid::Direction, text};
use iced::{alignment, executor, keyboard, window};
use iced::{Application, Command, Length, Settings, Subscription};
use iced_native::{event, subscription, Event};

pub type Result = std::result::Result<(), iced::Error>;

#[derive(Debug)]
pub enum State {
    Normal,
    Loading,
    Watching,
}

#[derive(Debug)]
pub struct GUI {
    state: State,
    pane: Option<Pane>,
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

    fn new(_: Self::Flags) -> (Self, Command<Message>) {
        (
            GUI {
                state: State::Loading,
                loading: LoadingCircle::new(),
                pane: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("YAMA")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self.state {
            State::Loading => match message {
                Message::Loading(instant) => self.loading.update(instant),
                Message::Bridge(msg) => match msg {
                    BridgeMessage::Ready(sender, cache) => {
                        self.pane = Some(Pane::new(cache, sender));
                        self.state = State::Normal;

                        return Command::perform(
                            async {
                                BridgeMessage::PaneAction(PanelsMessage::FocusItem(Direction::Left))
                            },
                            Message::Bridge,
                        );
                    }
                    BridgeMessage::PaneAction(message) => {
                        if let Some(pane) = &mut self.pane {
                            return pane.update(message, &mut self.state);
                        }
                    }
                    _ => (),
                },
            },
            State::Normal | State::Watching => {
                if let Some(pane) = &mut self.pane {
                    if let Message::Bridge(BridgeMessage::PaneAction(message)) = message {
                        return pane.update(message, &mut self.state);
                    }
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.state {
            //Loading subscription, disables input
            State::Loading => Subscription::batch(vec![
                bridge::start().map(Message::Bridge),
                window::frames().map(Message::Loading),
            ]),

            //Input subscription
            State::Normal => Subscription::batch(vec![
                bridge::start().map(Message::Bridge),
                subscription::events_with(|event, status| {
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
            ]),

            State::Watching => bridge::start().map(Message::Bridge),
        }
    }

    fn view(&self) -> Element<Message> {
        let help = text("Up and Down to Move, Right or Enter to Accept")
            .width(Length::Fill)
            .size(16)
            .horizontal_alignment(alignment::Horizontal::Right);

        let title = text("Y.A.M.A - Your Anime Manager Automata")
            .size(24)
            .vertical_alignment(alignment::Vertical::Center);

        let pane_view = if let Some(pane) = &self.pane {
            pane.view()
        } else {
            text("").into()
        };

        let content = container(column![title, pane_view, help].spacing(10))
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
                .width(Length::Fixed(300.0))
                .height(Length::Fixed(300.0))
                .padding(10);

                Modal::new(content, modal).into()
            }
            State::Watching => {
                let modal = container(
                    text("Watching episode...")
                        .size(42)
                        .vertical_alignment(alignment::Vertical::Center)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .width(Length::Fixed(300.0))
                .height(Length::Fixed(300.0))
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
        KeyCode::Up => PanelsMessage::FocusItem(Direction::Up),
        KeyCode::Down => PanelsMessage::FocusItem(Direction::Down),
        KeyCode::Right => PanelsMessage::Enter,
        KeyCode::Enter => PanelsMessage::Enter,
        KeyCode::Left => PanelsMessage::Back,
        _ => return None,
    };

    Some(Message::Bridge(BridgeMessage::PaneAction(msg)))
}
