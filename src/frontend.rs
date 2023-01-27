use crate::backend::{Backend, Episode};
use iced::widget::pane_grid::{self, Direction, PaneGrid};
use iced::widget::{canvas, column, container, image, scrollable, text};
use iced::{alignment, executor, keyboard, window};
use iced::{Application, Command, Length, Subscription};
use iced_native::{event, subscription, Event};
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::Instant;

mod list;
use self::list::List;

mod theme;
use self::theme::widget::Element;
use self::theme::Theme;

mod loading;
use self::loading::LoadingCircle;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

#[derive(Debug)]
enum PaneKind {
    None,
    Titles,
    Episodes(usize),
    Metadata(Episode),
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    Normal,
    Loading,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    FocusItem(Direction),
    Loading(Instant),
    EpisodesLoaded,
    Enter,
    Back,
}

#[derive(Debug)]
struct Pane {
    kind: PaneKind,
    list: Option<List>,
}

pub struct GUI {
    panes: pane_grid::State<Pane>,
    focus: Option<pane_grid::Pane>,
    loading: LoadingCircle,
    backend: Backend,
    state: State,
}

impl Pane {
    fn new(kind: PaneKind, list: Option<List>) -> Self {
        Self { kind, list }
    }
}

impl Application for GUI {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let backend = Backend::new();
        let mut focus = Option::None;
        let title_list = List::new(0, backend.count);

        let (mut panes, pane) = pane_grid::State::new(Pane::new(PaneKind::None, None));
        let result = panes.split(
            pane_grid::Axis::Vertical,
            &pane,
            Pane::new(PaneKind::Titles, Some(title_list)),
        );

        if let Some((pane, split)) = result {
            focus = Some(pane);
            panes.resize(&split, 0.25);

            let result = panes.split(
                pane_grid::Axis::Vertical,
                &pane,
                Pane::new(PaneKind::None, None),
            );
            if let Some((_, split)) = result {
                panes.resize(&split, 0.35);
            }
        }

        (
            GUI {
                panes,
                focus,
                backend,
                state: State::Normal,
                loading: LoadingCircle::new(),
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
                Message::EpisodesLoaded => {
                    self.state = State::Normal;

                    if let Some(pane) = self.focus {
                        let panel = self.panes.get(&pane).unwrap();
                        let focused = panel.list.as_ref().unwrap().focused;
                        let title = &mut self.backend.titles[focused].lock().unwrap();

                        if let Some(adj) = self.panes.adjacent(&pane, Direction::Left) {
                            let episodes_list = List::new(0, title.count);

                            *self.panes.get_mut(&adj).unwrap() =
                                Pane::new(PaneKind::Episodes(focused), Some(episodes_list));

                            self.panes.swap(&pane, &adj);
                            self.focus = Some(adj);

                            return Command::perform(async { Direction::Left }, Message::FocusItem);
                        }
                    }
                }
                _ => (),
            },
            State::Normal => match message {
                Message::FocusItem(direction) => {
                    if let Some(pane) = self.focus {
                        let panel = self.panes.get_mut(&pane).unwrap();

                        if let Some(list) = panel.list.as_mut() {
                            list.update(direction);
                            let focus = list.focused;
                            let mut y = (1.0 / list.size as f32) * list.focused as f32;

                            //Fixes top items being cut-out
                            if y > 0.1 {
                                y += 1.0 / list.font_size as f32; //Fixes bottom items being cut-out
                            }

                            if let PaneKind::Episodes(title) = panel.kind {
                                if let Some(adj) = self.panes.adjacent(&pane, Direction::Right) {
                                    *self.panes.get_mut(&adj).unwrap() = Pane::new(
                                        PaneKind::Metadata(
                                            self.backend.titles[title]
                                                .lock()
                                                .unwrap()
                                                .get_episode(focus),
                                        ),
                                        None,
                                    );
                                }
                            }

                            return scrollable::snap_to(
                                SCROLLABLE_ID.clone(),
                                scrollable::RelativeOffset { x: 0.0, y },
                            );
                        }
                    }
                }
                Message::Enter => {
                    if let Some(pane) = self.focus {
                        let panel = self.panes.get(&pane).unwrap();

                        match panel.kind {
                            PaneKind::Titles => {
                                let focused = panel.list.as_ref().unwrap().focused;
                                let mut title = self.backend.titles[focused].lock().unwrap();

                                if title.is_loaded() {
                                    title.get_or_init(0);
                                    return Command::perform(async {}, move |_| {
                                        Message::EpisodesLoaded
                                    });
                                } else {
                                    drop(title); //We release the lock before sending it to the thread
                                    let title = Arc::clone(&self.backend.titles[focused]); //We remove mut from ref but CHECK
                                    self.state = State::Loading;
                                    return Command::perform(
                                        async move {
                                            let mut title = title.lock().unwrap();
                                            title.get_or_init(0);
                                        },
                                        move |_| Message::EpisodesLoaded,
                                    );
                                }
                            }
                            PaneKind::Episodes(title) => {
                                let number = panel.list.as_ref().unwrap().focused;
                                self.backend.titles[title]
                                    .lock()
                                    .unwrap()
                                    .get_or_init(number)
                                    .run()
                                    .unwrap();

                                return Command::perform(
                                    async { Direction::Left },
                                    Message::FocusItem,
                                );
                            }
                            _ => (),
                        }
                    }
                }
                Message::Back => {
                    if let Some(pane) = self.focus {
                        let panel = self.panes.get(&pane).unwrap();

                        if let PaneKind::Episodes(_) = panel.kind {
                            if let Some(adj) = self.panes.adjacent(&pane, Direction::Right) {
                                *self.panes.get_mut(&adj).unwrap() =
                                    Pane::new(PaneKind::None, None);
                            }

                            if let Some(adj) = self.panes.adjacent(&pane, Direction::Left) {
                                self.panes.swap(&pane, &adj);
                                self.focus = Some(adj);

                                let panel = self.panes.get_mut(&pane).unwrap();
                                *panel = Pane::new(PaneKind::None, None);
                            }
                        }
                    }
                }
                Message::EpisodesLoaded => {
                    if let Some(pane) = self.focus {
                        let panel = self.panes.get(&pane).unwrap();
                        let focused = panel.list.as_ref().unwrap().focused;
                        let title = &mut self.backend.titles[focused].lock().unwrap();

                        if let Some(adj) = self.panes.adjacent(&pane, Direction::Left) {
                            let episodes_list = List::new(0, title.count);

                            *self.panes.get_mut(&adj).unwrap() =
                                Pane::new(PaneKind::Episodes(focused), Some(episodes_list));

                            self.panes.swap(&pane, &adj);
                            self.focus = Some(adj);

                            return Command::perform(async { Direction::Left }, Message::FocusItem);
                        }
                    }
                }
                _ => (),
            },
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
        }
    }

    fn view(&self) -> Element<Message> {
        let focus = self.focus;

        let pane_grid = PaneGrid::new(&self.panes, |id, pane, _| {
            let is_focused = focus == Some(id);

            let content = pane_grid::Content::new(view_content(pane, self)).style(if is_focused {
                theme::Container::Focused
            } else {
                theme::Container::Unfocused
            });

            match pane.kind {
                PaneKind::Titles => {
                    let title_bar = pane_grid::TitleBar::new("Titles").padding(10);
                    content.title_bar(title_bar)
                }
                PaneKind::Episodes(_) => {
                    //let title = self.backend.titles[title].lock().unwrap().name.as_str();
                    let title_bar = pane_grid::TitleBar::new("Episodes").padding(10);
                    content.title_bar(title_bar)
                }
                _ => content,
            }
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10);

        let help = text("Up and Down to Move, Right or Enter to Accept")
            .width(Length::Fill)
            .size(20)
            .horizontal_alignment(alignment::Horizontal::Right);

        let title = text("Y.A.M.A - Your Anime Manager Automata")
            .size(26)
            .vertical_alignment(alignment::Vertical::Center);

        let conent = column![title, pane_grid, help].spacing(10);
        container(conent)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(35)
            .into()
    }
}

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    use keyboard::KeyCode;

    match key_code {
        KeyCode::Up => Some(Message::FocusItem(Direction::Up)),
        KeyCode::Down => Some(Message::FocusItem(Direction::Down)),
        KeyCode::Right => Some(Message::Enter),
        KeyCode::Enter => Some(Message::Enter),
        KeyCode::Left => Some(Message::Back),
        _ => None,
    }
}

fn view_content<'a>(pane: &Pane, gui: &'a GUI) -> Element<'a, Message> {
    match &pane.kind {
        PaneKind::Titles => {
            if let State::Loading = gui.state {
                container(
                    canvas(&gui.loading)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_y()
                .into()
            } else {
                container(
                    scrollable(
                        pane.list
                            .as_ref()
                            .unwrap()
                            .view(gui.backend.view(), |_, _, _| theme::Text::Default),
                    )
                    .height(Length::Shrink)
                    .id(SCROLLABLE_ID.clone()),
                )
                .width(Length::Fill)
                .padding(5)
                .center_y()
                .into()
            }
        }
        PaneKind::Episodes(title) => {
            let title = gui.backend.titles[*title].lock().unwrap();

            container(
                scrollable(
                    pane.list
                        .as_ref()
                        .unwrap()
                        .view(title.view(), |focused, id, _| {
                            let watched = title.get_episode_ref(id).metadata.watched;
                            match id == focused {
                                true if watched => theme::Text::WatchedFocus,
                                false if watched => theme::Text::Watched,
                                true | false => theme::Text::Default,
                            }
                        }),
                )
                .height(Length::Shrink)
                .id(SCROLLABLE_ID.clone()),
            )
            .width(Length::Fill)
            .padding(5)
            .center_y()
            .into()
        }
        PaneKind::Metadata(episode) => container(
            column![
                image::Image::new(&episode.thumbnail_path),
                text(episode.description()).size(24)
            ]
            .spacing(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(15)
        .into(),

        PaneKind::None => container(column![])
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    }
}
