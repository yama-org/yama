use std::sync::Arc;

use crate::config::GUIConfig;
use crate::widgets::*;
use crate::{keybindings, Result};

use backend::Config;
use bridge::{BackendMessage, ConfigChange, FrontendMessage as Message, Modals, PanelAction};

use iced::futures::channel::mpsc::Sender;
use iced::widget::{
    button, canvas, column, container, horizontal_space, pane_grid::Direction, row, text,
};
use iced::{alignment, executor, font, keyboard, mouse, window, Font};
use iced::{event, subscription, Event};
use iced::{Application, Command, Length, Settings, Subscription};
use tracing::{error, info};

#[derive(Debug)]
pub enum State {
    Normal,
    Loading,
    Watching,
    ShowingMenu(Modals),
}

#[derive(Debug)]
pub struct Frontend {
    cfg: Config,
    state: State,
    pane: Option<Panels>,
    loading: LoadingCircle,
    sender: Option<Sender<BackendMessage>>,
}

impl Frontend {
    pub fn execute(cfg: Config) -> Result<()> {
        Ok(Frontend::run(Settings {
            flags: cfg,
            antialiasing: true,
            window: window::Settings {
                size: (1600, 900),
                position: window::Position::Centered,
                icon: Some(window::icon::from_file_data(
                    crate::embedded::YAMA_ICON,
                    None,
                )?),
                ..window::Settings::default()
            },
            default_font: Font::with_name("Kumbh Sans"),
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
                sender: None,
            },
            Command::batch(vec![
                font::load(crate::embedded::REGULAR_FONT_BYTES).map(Message::FontLoaded),
                font::load(crate::embedded::BOLD_FONT_BYTES).map(Message::FontLoaded),
                font::load(iced_aw::graphics::icons::ICON_FONT_BYTES).map(Message::FontLoaded),
            ]),
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
                    self.sender = Some(sender.clone());
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
                    self.state = State::Normal;
                    return Command::perform(async { Modals::Error(err) }, Message::MenuBar);
                }
                Message::Recovery(sender, err) => {
                    self.sender = Some(sender);
                    return Command::perform(async { err }, Message::Error);
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
                    Message::ToLoad => self.state = State::Loading,
                    Message::MenuBar(menu) => self.state = State::ShowingMenu(menu),
                    Message::HideMenubar => self.state = State::Normal,
                    Message::UpdateConfig(change) => match change {
                        ConfigChange::SeriesPath => {
                            let res = GUIConfig::change_series_path(&mut self.cfg);

                            match res {
                                Ok(()) => {
                                    if let Some(sender) = &mut self.sender {
                                        let _ = sender.try_send(BackendMessage::Restart);
                                    }
                                }
                                Err(err) => {
                                    return Command::perform(
                                        async move { Arc::from(err.to_string()) },
                                        Message::Error,
                                    );
                                }
                            }
                        }
                        ConfigChange::ThemePath => GUIConfig::change_theme_path(&mut self.cfg),
                        ConfigChange::MinTime(new_time) => {
                            GUIConfig::change_min_time(&mut self.cfg, new_time)
                        }
                    },
                    Message::CleanUp => {
                        if let Some(sender) = &mut self.sender {
                            let _ = sender.try_send(BackendMessage::CleanUp);
                        }
                    }
                    Message::Exit => {
                        info!("Bye-bye~");
                        return window::close::<Message>();
                    }
                    Message::Error(err) => {
                        error!("yama has encounter an error: {err}");
                        return Command::perform(async { Modals::Error(err) }, Message::MenuBar);
                    }
                    Message::Recovery(sender, err) => {
                        self.sender = Some(sender);
                        return Command::perform(async { err }, Message::Error);
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
                            keybindings::handle_mousewheel(delta)
                        }

                        Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                            keybindings::handle_mousebutton(button)
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
                        }) => keybindings::handle_hotkey(key_code),

                        _ => None,
                    }
                }),
            ]),

            State::Watching | State::ShowingMenu(_) => bridge::subscription::start(),
        }
    }

    fn view(&self) -> Element<Message> {
        let pane_view = if let Some(pane) = &self.pane {
            pane.view()
        } else {
            horizontal_space(Length::Shrink).into()
        };

        let content = container(
            column![
                row![
                    button("Config")
                        .on_press(Message::MenuBar(Modals::Config))
                        .style(theme::Button::Menu),
                    button("About")
                        .on_press(Message::MenuBar(Modals::About))
                        .style(theme::Button::Menu),
                    horizontal_space(Length::Fill),
                    button("  ?  ")
                        .on_press(Message::MenuBar(Modals::Help))
                        .style(theme::Button::Menu)
                ]
                .align_items(iced::Alignment::Center)
                .width(Length::Fill),
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
                .padding(15);

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
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(15);

                Modal::new(content, modal).into()
            }

            State::ShowingMenu(menu) => {
                let modal = match menu {
                    Modals::Help => menus::help(),
                    Modals::About => menus::about(),
                    Modals::Config => menus::config(&self.cfg),
                    Modals::Yama => menus::yama(),
                    Modals::Error(err) => menus::error(err.clone()),
                };

                Modal::new(content, modal)
                    .on_blur(Message::HideMenubar)
                    .into()
            }
            _ => content.into(),
        }
    }
}
