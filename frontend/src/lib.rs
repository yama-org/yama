mod gui_config;
mod list;
mod loading;
mod modal;
pub mod panels;
mod theme;

use crate::gui_config::GUIConfig;
use crate::list::List;
use crate::loading::LoadingCircle;
use crate::modal::Modal;
use crate::panels::Pane;
use crate::theme::{widget::Element, Theme};

use bridge::{BridgeMessage, FrontendMessage as Message, MenuBar, PanelsMessage};

use iced::widget::{button, canvas, column, container, image, pane_grid::Direction, row, text};
use iced::{alignment, executor, keyboard, mouse, window};
use iced::{Application, Command, Length, Settings, Subscription};
use iced_native::{event, subscription, Event};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use tracing_unwrap::ResultExt;

pub type Result = std::result::Result<(), iced::Error>;

static YAMA_ICON: &[u8] = include_bytes!("../../res/yama_icon.ico");
static YAMA_PNG: &[u8] = include_bytes!("../../res/yama.png");
static YAMA_WAV: &[u8] = include_bytes!("../../res/hai_yama.wav");

#[derive(Debug)]
pub enum State {
    Normal,
    Loading,
    Watching,
    ShowMenu(MenuBar),
}

struct Sound {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    sink: Sink,
}

impl std::fmt::Debug for Sound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sound Debug")
    }
}

#[derive(Debug)]
pub struct GUI {
    state: State,
    pane: Option<Pane>,
    loading: LoadingCircle,
    cfg: backend::config::Config,
    sound: Sound,
}

impl GUI {
    pub fn execute() -> Result {
        GUI::run(Settings {
            id: None,
            antialiasing: true,
            window: window::Settings {
                size: (1920, 1080),
                icon: Some(window::icon::from_file_data(YAMA_ICON, None).unwrap_or_log()),
                ..window::Settings::default()
            },
            default_font: Some(include_bytes!("../../res/KumbhSans-Regular.ttf")),
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
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap_or_log();
        let sink = rodio::Sink::try_new(&handle).unwrap_or_log();
        sink.set_volume(0.5);

        let sound = Sound {
            _stream,
            _handle: handle,
            sink,
        };

        (
            GUI {
                sound,
                state: State::Loading,
                loading: LoadingCircle::new(),
                pane: None,
                cfg: confy::load("yama", "config").expect_or_log("[ERROR] - Configuration file."),
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
                },
                _ => (),
            },
            _ => {
                if let Some(pane) = &mut self.pane {
                    if let Message::Bridge(BridgeMessage::PaneAction(message)) = message {
                        return pane.update(message, &mut self.state);
                    }
                }

                match message {
                    Message::MenuBar(menu) => self.state = State::ShowMenu(menu),
                    Message::HideMenubar => self.state = State::Normal,
                    Message::FileDialog => GUIConfig::file_dialog(&mut self.cfg),
                    Message::Exit => {
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
                        Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                            handle_mousewheel(delta)
                        }

                        Event::Keyboard(keyboard::Event::KeyPressed {
                            key_code,
                            modifiers,
                        }) if modifiers.shift() && key_code == keyboard::KeyCode::W => {
                            Some(Message::Bridge(BridgeMessage::PaneAction(
                                PanelsMessage::MarkTitleEpisodes,
                            )))
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

            State::Watching | State::ShowMenu(_) => bridge::start().map(Message::Bridge),
        }
    }

    fn view(&self) -> Element<Message> {
        let font_size = Settings::<()>::default().default_text_size;

        let title = text("Y.A.M.A - Your Anime Manager Automata")
            .size(font_size + 4.0)
            .vertical_alignment(alignment::Vertical::Center);

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
                button(text(" ").size(0))
                    .padding(0)
                    .style(theme::Button::Separator)
                    .width(Length::Fill)
                    .height(Length::Fixed(0.0)),
                title,
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
            State::ShowMenu(menu) => {
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
                        //let file = std::fs::File::open("./res/hai_yama.wav").unwrap_or_log();
                        let sound_file = std::io::Cursor::new(YAMA_WAV);
                        self.sound
                            .sink
                            .append(rodio::Decoder::new(sound_file).unwrap_or_log());

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
        KeyCode::Up | KeyCode::J => Some(Message::Bridge(BridgeMessage::PaneAction(
            PanelsMessage::FocusItem(Direction::Up),
        ))),
        KeyCode::Down | KeyCode::K => Some(Message::Bridge(BridgeMessage::PaneAction(
            PanelsMessage::FocusItem(Direction::Down),
        ))),
        KeyCode::Right | KeyCode::L => Some(Message::Bridge(BridgeMessage::PaneAction(
            PanelsMessage::Enter,
        ))),
        KeyCode::Enter => Some(Message::Bridge(BridgeMessage::PaneAction(
            PanelsMessage::Enter,
        ))),
        KeyCode::Left | KeyCode::H => Some(Message::Bridge(BridgeMessage::PaneAction(
            PanelsMessage::Back,
        ))),
        KeyCode::R => Some(Message::Bridge(BridgeMessage::PaneAction(
            PanelsMessage::Refresh,
        ))),
        KeyCode::W => Some(Message::Bridge(BridgeMessage::PaneAction(
            PanelsMessage::MarkEpisode,
        ))),

        // Messages
        KeyCode::Q => Some(Message::Exit),
        _ => None,
    }
}

fn handle_mousewheel(delta: mouse::ScrollDelta) -> Option<Message> {
    if let mouse::ScrollDelta::Lines { x: _, y } = delta {
        if y > 0.0 {
            Some(Message::Bridge(BridgeMessage::PaneAction(
                PanelsMessage::FocusItem(Direction::Up),
            )))
        } else {
            Some(Message::Bridge(BridgeMessage::PaneAction(
                PanelsMessage::FocusItem(Direction::Down),
            )))
        }
    } else {
        None
    }
}

fn handle_mousebutton(button: mouse::Button) -> Option<Message> {
    match button {
        mouse::Button::Right => Some(Message::Bridge(BridgeMessage::PaneAction(
            PanelsMessage::Back,
        ))),
        _ => None,
    }
}
