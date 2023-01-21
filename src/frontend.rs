use crate::backend::{Backend, Episode};
use iced::widget::pane_grid::{self, Direction, PaneGrid};
use iced::widget::{column, container, image, scrollable, text};
use iced::{alignment, executor, keyboard};
use iced::{Application, Command, Length, Subscription};
use iced_native::{event, subscription, Event};

mod list;
use self::list::List;

mod theme;
use self::theme::widget::Element;
use self::theme::Theme;

enum PaneKind {
    None,
    Titles,
    Episodes(usize),
    Metadata(Episode),
}

struct Pane {
    kind: PaneKind,
    list: Option<List>,
}

pub struct GUI {
    panes: pane_grid::State<Pane>,
    focus: Option<pane_grid::Pane>,
    backend: Backend,
}

#[derive(Debug, Clone)]
pub enum Message {
    FocusItem(Direction),
    Enter,
    Back,
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
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("YAMA")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FocusItem(direction) => {
                if let Some(pane) = self.focus {
                    let panel = self.panes.get_mut(&pane).unwrap();

                    if let Some(list) = panel.list.as_mut() {
                        list.update(direction);
                    }

                    match panel.kind {
                        PaneKind::Episodes(title) => {
                            let focused = panel.list.as_ref().unwrap().focused;

                            if let Some(adj) = self.panes.adjacent(&pane, Direction::Right) {
                                *self.panes.get_mut(&adj).unwrap() = Pane::new(
                                    PaneKind::Metadata(self.backend.get_episode(title, focused)),
                                    None,
                                );
                            }
                        }
                        _ => (),
                    }
                }
            }
            Message::Enter => {
                if let Some(pane) = self.focus {
                    let panel = self.panes.get(&pane).unwrap();

                    match panel.kind {
                        PaneKind::Titles => {
                            let focused = panel.list.as_ref().unwrap().focused;
                            let title = &self.backend.titles[focused];
                            let episodes_list = List::new(0, title.count);

                            if let Some(adj) = self.panes.adjacent(&pane, Direction::Left) {
                                *self.panes.get_mut(&adj).unwrap() =
                                    Pane::new(PaneKind::Episodes(focused), Some(episodes_list));

                                self.panes.swap(&pane, &adj);
                                self.focus = Some(adj);

                                return Command::perform(
                                    async { Direction::Left },
                                    Message::FocusItem,
                                );
                            }
                        }
                        PaneKind::Episodes(title) => {
                            let number = panel.list.as_ref().unwrap().focused;
                            self.backend.get_episode_mut(title, number).run().unwrap();

                            return Command::perform(async { Direction::Left }, Message::FocusItem);
                        }
                        _ => (),
                    }
                }
            }
            Message::Back => {
                if let Some(pane) = self.focus {
                    let panel = self.panes.get(&pane).unwrap();

                    match panel.kind {
                        PaneKind::Episodes(_) => {
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
                        _ => (),
                    }
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| {
            if let event::Status::Captured = status {
                return None;
            }

            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code,
                    modifiers,
                }) if modifiers.is_empty() => handle_hotkey(key_code),
                _ => None,
            }
        })
    }

    fn view(&self) -> Element<Message> {
        let focus = self.focus;

        let pane_grid = PaneGrid::new(&self.panes, |id, pane, _| {
            let is_focused = focus == Some(id);
            let content =
                pane_grid::Content::new(view_content(&pane, &self.backend)).style(if is_focused {
                    theme::Container::Focused
                } else {
                    theme::Container::Unfocused
                });

            match pane.kind {
                PaneKind::Titles => {
                    let title = "Titles";
                    let title_bar = pane_grid::TitleBar::new(title).padding(10);

                    content.title_bar(title_bar)
                }
                PaneKind::Episodes(title) => {
                    let title = self.backend.titles[title].name.as_str();
                    let title_bar = pane_grid::TitleBar::new(title).padding(10);

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

fn view_content<'a>(pane: &Pane, backend: &Backend) -> Element<'a, Message> {
    match &pane.kind {
        PaneKind::Titles => container(scrollable(
            pane.list
                .as_ref()
                .unwrap()
                .view(backend.view(), |_, _, _| theme::Text::Default),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .center_y()
        .into(),

        PaneKind::Episodes(title) => container(scrollable(pane.list.as_ref().unwrap().view(
            backend.titles[*title].view(),
            |focused, id, _| {
                let watched = backend.get_episode(*title, id).metadata.watched;

                match id == focused {
                    true if watched => theme::Text::WatchedFocus,
                    false if watched => theme::Text::Watched,
                    true | false => theme::Text::Default,
                }
            },
        )))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .center_y()
        .into(),
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
