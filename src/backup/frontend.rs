use crate::backend::{Backend, Episode};
use iced::alignment::Alignment;
use iced::theme::Theme;
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{button, column, container, image, scrollable, text};
use iced::{alignment, executor, keyboard};
use iced::{Application, Command, Element, Length, Subscription};
use iced_native::{event, subscription, Event};

mod list;

enum PaneKind {
    None,
    Titles(usize),
    Episodes(usize, usize),
    Metadata(usize, usize, bool),
}

impl PaneKind {
    pub fn up(&mut self, len: usize) -> usize {
        match *self {
            PaneKind::Episodes(_, ref mut focused) | PaneKind::Titles(ref mut focused) => {
                if *focused == 0 {
                    *focused = len - 1;
                } else {
                    *focused -= 1;
                }
                *focused
            }
            PaneKind::None | PaneKind::Metadata(_, _, _) => 0,
        }
    }

    pub fn down(&mut self, len: usize) -> usize {
        match *self {
            PaneKind::Episodes(_, ref mut focused) | PaneKind::Titles(ref mut focused) => {
                if *focused == len - 1 {
                    *focused = 0;
                } else {
                    *focused += 1;
                }
                *focused
            }
            PaneKind::None | PaneKind::Metadata(_, _, _) => 0,
        }
    }
}

struct Pane {
    kind: PaneKind,
}

impl Pane {
    fn new(kind: PaneKind) -> Self {
        Self { kind }
    }
}

pub struct GUI {
    panes: pane_grid::State<Pane>,
    focus: Option<pane_grid::Pane>,
    backend: Backend,
    theme: Theme,
    current_thumbnail: Option<image::Handle>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FocusAdjacent(pane_grid::Direction),
    FocusItem(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Enter,
}

impl Application for GUI {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut focused = Option::None;

        let (mut panes, pane) = pane_grid::State::new(Pane::new(PaneKind::None));
        let result = panes.split(
            pane_grid::Axis::Vertical,
            &pane,
            Pane::new(PaneKind::Titles(0)),
        );

        if let Some((pane, split)) = result {
            focused = Some(pane);
            panes.resize(&split, 0.25);

            let result = panes.split(pane_grid::Axis::Vertical, &pane, Pane::new(PaneKind::None));
            if let Some((_, split)) = result {
                panes.resize(&split, 0.35);
            }
        }

        (
            GUI {
                panes,
                focus: focused,
                backend: Backend::new(),
                theme: Theme::Light,
                current_thumbnail: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("YAMA")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FocusAdjacent(direction) => {
                if let Some(pane) = self.focus {
                    if let Some(adjacent) = self.panes.adjacent(&pane, direction) {
                        self.focus = Some(adjacent);
                    }
                }
            }

            Message::FocusItem(direction) => {
                if let Some(pane) = self.focus {
                    let panel = self.panes.get_mut(&pane).unwrap();

                    match panel.kind {
                        PaneKind::Episodes(title, _) => {
                            let mut focused = 0;

                            if let pane_grid::Direction::Up = direction {
                                focused = panel.kind.up(self.backend.titles[title].count);
                            } else if let pane_grid::Direction::Down = direction {
                                focused = panel.kind.down(self.backend.titles[title].count);
                            }

                            if let Some(adj) =
                                self.panes.adjacent(&pane, pane_grid::Direction::Right)
                            {
                                let ad_panel = self.panes.get_mut(&adj).unwrap();
                                *ad_panel = Pane::new(PaneKind::Metadata(title, focused, false));

                                let ep =
                                    &self.backend.titles[title].episodes[focused].thumbnail_path;
                                self.current_thumbnail = Some(image::Handle::from(ep));
                            }
                        }

                        PaneKind::Titles(_) => {
                            if let pane_grid::Direction::Up = direction {
                                panel.kind.up(self.backend.count);
                            } else if let pane_grid::Direction::Down = direction {
                                panel.kind.down(self.backend.count);
                            }
                        }

                        PaneKind::None | PaneKind::Metadata(_, _, _) => (),
                    }
                }
            }

            Message::Clicked(pane) => {
                self.focus = Some(pane);
            }

            Message::Enter => {
                if let Some(pane) = self.focus {
                    let panel = self.panes.get(&pane).unwrap();

                    match panel.kind {
                        PaneKind::Titles(focused) => {
                            if let Some(adjacent) =
                                self.panes.adjacent(&pane, pane_grid::Direction::Left)
                            {
                                let ad_panel = self.panes.get_mut(&adjacent).unwrap();
                                *ad_panel = Pane::new(PaneKind::Titles(focused));

                                let panel = self.panes.get_mut(&pane).unwrap();
                                *panel = Pane::new(PaneKind::Episodes(focused, 0));

                                if let Some(adjacent) =
                                    self.panes.adjacent(&pane, pane_grid::Direction::Right)
                                {
                                    let ad_panel = self.panes.get_mut(&adjacent).unwrap();
                                    *ad_panel = Pane::new(PaneKind::Metadata(focused, 0, false));

                                    let ep =
                                        &self.backend.titles[focused].episodes[0].thumbnail_path;
                                    self.current_thumbnail = Some(image::Handle::from(ep));
                                }
                            } else if let Some(adjacent) =
                                self.panes.adjacent(&pane, pane_grid::Direction::Right)
                            {
                                let ad_panel = self.panes.get_mut(&adjacent).unwrap();
                                *ad_panel = Pane::new(PaneKind::Episodes(focused, 0));

                                if let Some(adj) =
                                    self.panes.adjacent(&adjacent, pane_grid::Direction::Right)
                                {
                                    let ad_panel = self.panes.get_mut(&adj).unwrap();
                                    *ad_panel = Pane::new(PaneKind::Metadata(focused, 0, false));

                                    let ep =
                                        &self.backend.titles[focused].episodes[0].thumbnail_path;
                                    self.current_thumbnail = Some(image::Handle::from(ep));
                                }
                            }
                        }
                        PaneKind::Episodes(_, _) | PaneKind::Metadata(_, _, _) | PaneKind::None => {
                            ()
                        }
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
            let content = pane_grid::Content::new(view_content(
                &pane.kind,
                &self.backend,
                &self.theme,
                self.current_thumbnail.clone(),
            ))
            .style(if is_focused {
                style::pane_focused
            } else {
                style::pane_active
            });

            match pane.kind {
                PaneKind::Episodes(title, _) => {
                    let title = self.backend.titles[title].name.as_str();
                    let title_bar =
                        pane_grid::TitleBar::new(title)
                            .padding(10)
                            .style(if is_focused {
                                style::title_bar_focused
                            } else {
                                style::title_bar_active
                            });

                    content.title_bar(title_bar)
                }

                PaneKind::Titles(_) => {
                    let title = "Titles";
                    let title_bar =
                        pane_grid::TitleBar::new(title)
                            .padding(10)
                            .style(if is_focused {
                                style::title_bar_focused
                            } else {
                                style::title_bar_active
                            });

                    content.title_bar(title_bar)
                }

                PaneKind::Metadata(_, _, _) | PaneKind::None => content,
            }
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(1)
        .on_click(Message::Clicked);

        let help = text("Up and Down to Move, Right or Enter to Accept")
            .width(Length::Fill)
            .size(24)
            .horizontal_alignment(alignment::Horizontal::Right);

        let conent = column![pane_grid, help].spacing(10);
        container(conent)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }
}

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    use keyboard::KeyCode;
    use pane_grid::Direction;

    let direction = match key_code {
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    };

    match key_code {
        KeyCode::Up => Some(Message::FocusItem(Direction::Up)),
        KeyCode::Down => Some(Message::FocusItem(Direction::Down)),
        KeyCode::Enter => Some(Message::Enter),
        _ => direction.map(Message::FocusAdjacent),
    }
}

fn set_title_focus<'a>(focused: usize, backend: &Backend) -> iced::widget::Column<'a, Message> {
    let mut arr: Vec<Element<'a, Message, _>> = Vec::new();

    for (i, title) in backend.titles.iter().enumerate() {
        arr.push(
            button(text(format!("{}", title.name)).size(24))
                .style(if i == focused {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Text
                })
                .width(Length::Fill)
                .into(),
        );
    }

    iced::widget::Column::with_children(arr)
}

fn set_episode_focus<'a>(
    episodes: &Vec<Episode>,
    focused: usize,
    theme: &Theme,
) -> iced::widget::Column<'a, Message> {
    let mut arr: Vec<Element<'a, Message, _>> = Vec::new();

    for ep in episodes.iter() {
        arr.push(
            text(format!("{} - {}", ep.number, ep.name))
                .style(if ep.number - 1 == focused {
                    style::title_focused(theme)
                } else {
                    style::title_unfocused(theme)
                })
                .size(24)
                .into(),
        );
    }

    iced::widget::Column::with_children(arr)
}

fn view_content<'a>(
    kind: &PaneKind,
    backend: &Backend,
    theme: &Theme,
    thumbnail: Option<image::Handle>,
) -> Element<'a, Message> {
    match kind {
        PaneKind::Episodes(title, focused) => {
            let content = set_episode_focus(&backend.titles[*title].episodes, *focused, theme)
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(10)
                .align_items(Alignment::Start);

            container(scrollable(content))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(5)
                .center_y()
                .into()
        }

        PaneKind::Titles(focused) => {
            let content = set_title_focus(*focused, backend)
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(10)
                .align_items(Alignment::Start);

            container(scrollable(content))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(5)
                .center_y()
                .into()
        }

        PaneKind::Metadata(title, episode, drawed) => {
            let ep = &backend.titles[*title].episodes[*episode];
            //let mut content = column![];

            if let Some(th) = thumbnail {
                //let image = image::Handle::from(&ep.thumbnail_path);
                //content = column![text(ep.description()).size(24)].spacing(10);
                container(image::Image::new(th))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(15)
                    .into()
            } else {
                //content = column![text(ep.description()).size(24)].spacing(10);
                container(column![])
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(15)
                    .into()
            }
        }

        PaneKind::None => container(column![])
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    }
}

mod style {
    use iced::widget::container;
    use iced::Color;
    use iced::Theme;

    pub fn title_bar_active(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.background.strong.text),
            background: Some(palette.background.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn title_bar_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.primary.strong.text),
            background: Some(palette.primary.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn pane_active(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(palette.background.weak.color.into()),
            border_width: 2.0,
            border_color: palette.background.strong.color,
            ..Default::default()
        }
    }

    pub fn pane_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(palette.background.weak.color.into()),
            border_width: 2.0,
            border_color: palette.primary.strong.color,
            ..Default::default()
        }
    }

    pub fn title_unfocused(theme: &Theme) -> Color {
        let palette = theme.extended_palette();
        palette.background.strong.text
    }

    pub fn title_focused(theme: &Theme) -> Color {
        let palette = theme.extended_palette();
        palette.primary.strong.color
    }
}
