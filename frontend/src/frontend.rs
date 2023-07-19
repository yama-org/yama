use crate::config::GUIConfig;
use crate::widgets::*;
use crate::Result;

use backend::Config;
use bridge::{FrontendMessage as Message, MenuBar, PanelAction};

use iced::widget::{button, canvas, column, container, image, pane_grid::Direction, row, text};
use iced::{alignment, executor, keyboard, mouse, window};
use iced::{event, subscription, Event};
use iced::{Application, Command, Length, Settings, Subscription};
//use iced_native::{event, subscription, Event};
use tracing::error;

static YAMA_ICON: &[u8] = include_bytes!("../../res/yama_logo.ico");
static YAMA_PNG: &[u8] = include_bytes!("../../res/yama.png");

#[derive(Debug)]
pub enum State {
    Normal,
    Loading,
    Watching,
    ShowingMenu(MenuBar),
}

#[derive(Debug)]
pub struct Frontend {
    cfg: Config,
    state: State,
    pane: Option<Panels>,
    loading: LoadingCircle,
}

impl Frontend {
    pub fn execute(cfg: Config) -> Result<()> {
        Ok(Frontend::run(Settings {
            flags: cfg,
            antialiasing: true,
            window: window::Settings {
                size: (1600, 900),
                position: window::Position::Centered,
                icon: Some(window::icon::from_file_data(YAMA_ICON, None)?),
                ..window::Settings::default()
            },
            default_font: Some(include_bytes!("../../res/fonts/KumbhSans-Regular.ttf")),
            ..Settings::default()
        })?)
    }
}

impl Application for Frontend {
    type Message = Message;
    type Theme = theme::Theme;
    type Executor = executor::Default;
    type Flags = Config;

    fn new(cfg: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                cfg,
                pane: None,
                state: State::Loading,
                loading: LoadingCircle::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("yama")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self.state {
            State::Loading => match message {
                Message::Loading(instant) => self.loading.update(instant),
                Message::Ready(sender, cache) => {
                    self.pane = Some(Panels::new(cache, sender));
                    self.state = State::Normal;

                    return Command::perform(
                        async { PanelAction::FocusItem(Direction::Left) },
                        Message::PaneAction,
                    );
                }
                Message::PaneAction(message) => {
                    if let Some(pane) = &mut self.pane {
                        return pane.update(message, &mut self.state);
                    }
                }
                Message::Error(err) => {
                    error!("yama has encounter an error: {err}");
                    return window::close::<Message>();
                }
                _ => (),
            },
            _ => {
                if let Some(pane) = &mut self.pane {
                    if let Message::PaneAction(message) = message {
                        return pane.update(message, &mut self.state);
                    }
                }

                match message {
                    Message::MenuBar(menu) => self.state = State::ShowingMenu(menu),
                    Message::HideMenubar => self.state = State::Normal,
                    Message::FileDialog => GUIConfig::file_dialog(&mut self.cfg),
                    Message::Exit => {
                        return window::close::<Message>();
                    }
                    Message::Error(err) => {
                        error!("yama has encounter an error: {err}");
                        return window::close::<Message>();
                    }
                    _ => (),
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.state {
            //Loading subscription, disables input
            State::Loading => Subscription::batch(vec![
                bridge::subscription::start(),
                window::frames().map(Message::Loading),
            ]),

            //Input subscription
            State::Normal => Subscription::batch(vec![
                bridge::subscription::start(),
                subscription::events_with(|event, status| {
                    if let event::Status::Captured = status {
                        return None;
                    }

                    match event {
                        Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                            handle_mousewheel(delta)
                        }

                        Event::Keyboard(keyboard::Event::KeyPressed {
                            key_code,
                            modifiers,
                        }) if modifiers.shift() && key_code == keyboard::KeyCode::W => {
                            Some(Message::PaneAction(PanelAction::MarkPreviousEpisodes))
                        }

                        Event::Keyboard(keyboard::Event::KeyPressed {
                            key_code,
                            modifiers: _,
                        }) => handle_hotkey(key_code),

                        Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                            handle_mousebutton(button)
                        }

                        _ => None,
                    }
                }),
            ]),

            State::Watching | State::ShowingMenu(_) => bridge::subscription::start(),
        }
    }

    fn view(&self) -> Element<Message> {
        //let font_size = Settings::<()>::default().default_text_size;

        /*let title = text("Y.A.M.A - Your Anime Manager Automata")
        .size(font_size + 4.0)
        .vertical_alignment(alignment::Vertical::Center);*/

        let pane_view = if let Some(pane) = &self.pane {
            pane.view()
        } else {
            text("").into()
        };

        let content = container(
            column![
                row![
                    button("Config")
                        .on_press(Message::MenuBar(MenuBar::Config))
                        .style(theme::Button::Menu),
                    button("About")
                        .on_press(Message::MenuBar(MenuBar::About))
                        .style(theme::Button::Menu),
                ],
                /*button(text(" ").size(0))
                .padding(0)
                .style(theme::Button::Separator)
                .width(Length::Fill)
                .height(Length::Fixed(1.0)),*/
                //title,
                pane_view,
            ]
            .spacing(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(35);

        match &self.state {
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
                .width(Length::Fixed(600.0))
                .height(Length::Fixed(300.0))
                .padding(10);

                Modal::new(content, modal).into()
            }
            State::ShowingMenu(menu) => {
                let modal = match menu {
                    MenuBar::About => container(
                        column![
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
                    .style(theme::Container::Box),

                    MenuBar::Config => container(GUIConfig::view(&self.cfg))
                        .width(Length::Fixed(300.0))
                        .height(Length::Fixed(300.0))
                        .padding(25)
                        .style(theme::Container::Box),

                    MenuBar::Yama => {
                        let img = image::Handle::from_memory(YAMA_PNG);
                        container(image::Image::new(img))
                            .center_x()
                            .center_y()
                            .width(Length::Fixed(300.0))
                            .height(Length::Fixed(300.0))
                            .padding(25)
                            .style(theme::Container::Box)
                    }
                };

                Modal::new(content, modal)
                    .on_blur(Message::HideMenubar)
                    .into()
            }
            _ => content.into(),
        }
    }
}

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    use keyboard::KeyCode;

    match key_code {
        // Bridge Messages
        KeyCode::Up | KeyCode::K => {
            Some(Message::PaneAction(PanelAction::FocusItem(Direction::Up)))
        }
        KeyCode::Down | KeyCode::J => {
            Some(Message::PaneAction(PanelAction::FocusItem(Direction::Down)))
        }
        KeyCode::Right | KeyCode::L => Some(Message::PaneAction(PanelAction::Enter)),
        KeyCode::Enter => Some(Message::PaneAction(PanelAction::Enter)),
        KeyCode::Left | KeyCode::H => Some(Message::PaneAction(PanelAction::Back)),
        KeyCode::R => Some(Message::PaneAction(PanelAction::Refresh)),
        KeyCode::W | KeyCode::Space => Some(Message::PaneAction(PanelAction::MarkEpisode)),

        // Messages
        KeyCode::Q => Some(Message::Exit),
        _ => None,
    }
}

fn handle_mousewheel(delta: mouse::ScrollDelta) -> Option<Message> {
    if let mouse::ScrollDelta::Lines { x: _, y } = delta {
        if y > 0.0 {
            Some(Message::PaneAction(PanelAction::FocusItem(Direction::Up)))
        } else {
            Some(Message::PaneAction(PanelAction::FocusItem(Direction::Down)))
        }
    } else {
        None
    }
}

fn handle_mousebutton(button: mouse::Button) -> Option<Message> {
    match button {
        mouse::Button::Right | mouse::Button::Other(8) => {
            Some(Message::PaneAction(PanelAction::Back))
        }
        mouse::Button::Other(9) => Some(Message::PaneAction(PanelAction::Enter)),
        _ => None,
    }
}
