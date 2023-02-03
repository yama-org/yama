use super::theme::{self, widget::Element};
use super::{List, State};
use backend::{api::Data, backend::Backend, Media, Meta};
use iced::widget::pane_grid::{self, Direction, PaneGrid};
use iced::widget::{column, container, image, scrollable, text};
use iced::{Command, Length};
use once_cell::sync::Lazy;
use std::sync::Arc;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

#[derive(Debug)]
enum PaneKind {
    None,
    Titles,
    Episodes(usize),
    Metadata(Media<dyn Meta>),
}

#[derive(Debug)]
struct PaneData {
    kind: PaneKind,
    list: Option<List>,
}

#[derive(Debug)]
pub struct Pane {
    panes: pane_grid::State<PaneData>,
    focus: Option<pane_grid::Pane>,
    backend: Backend,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadingTitles,
    TitlesLoaded(Vec<Data>),
    LoadingEpisode,
    EpisodesLoaded,
    FocusItem(Direction),
    Enter,
    Back,
}

impl PaneData {
    fn new(kind: PaneKind, list: Option<List>) -> PaneData {
        PaneData { kind, list }
    }
}

impl Default for Pane {
    fn default() -> Pane {
        Pane::new()
    }
}

impl Pane {
    pub fn new() -> Pane {
        let backend = Backend::new();
        let mut focus = Option::None;
        let title_list = List::new(0, backend.count);

        let (mut panes, pane) = pane_grid::State::new(PaneData::new(PaneKind::None, None));
        let result = panes.split(
            pane_grid::Axis::Vertical,
            &pane,
            PaneData::new(PaneKind::Titles, Some(title_list)),
        );

        if let Some((pane, split)) = result {
            focus = Some(pane);
            panes.resize(&split, 0.25);

            let result = panes.split(
                pane_grid::Axis::Vertical,
                &pane,
                PaneData::new(PaneKind::Metadata(backend.get_title(0)), None),
            );
            if let Some((_, split)) = result {
                panes.resize(&split, 0.35);
            }
        }

        Pane {
            panes,
            focus,
            backend,
        }
    }

    pub fn update(&mut self, message: Message, state: &mut State) -> Command<super::Message> {
        match message {
            Message::LoadingTitles => {
                let titles = self
                    .backend
                    .titles
                    .iter()
                    .map(|t| {
                        let t = t.lock().unwrap_or_else(|t| t.into_inner());
                        (t.path.clone(), t.name.clone())
                    })
                    .collect();

                return Command::perform(
                    async move {
                        let data = Backend::batch_titles_data(titles).await;
                        Message::TitlesLoaded(data)
                    },
                    super::Message::PaneAction,
                );
            }
            Message::TitlesLoaded(data) => {
                *state = State::Normal;

                for data in data {
                    let id = data.id;
                    let mut title = self.backend.titles[id]
                        .lock()
                        .unwrap_or_else(|l| l.into_inner());
                    title.data = Some(data);
                }

                return Command::perform(
                    async { Message::FocusItem(Direction::Left) },
                    super::Message::PaneAction,
                );
            }
            Message::FocusItem(direction) => {
                *state = State::Normal;

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
                                *self.panes.get_mut(&adj).unwrap() = PaneData::new(
                                    PaneKind::Metadata(
                                        self.backend.titles[title]
                                            .lock()
                                            .unwrap_or_else(|l| l.into_inner())
                                            .get_episode(focus),
                                    ),
                                    None,
                                );
                            }
                        } else if let Some(adj) = self.panes.adjacent(&pane, Direction::Right) {
                            *self.panes.get_mut(&adj).unwrap() = PaneData::new(
                                PaneKind::Metadata(self.backend.get_title(focus)),
                                None,
                            );
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
                            let mut title = self.backend.titles[focused]
                                .lock()
                                .unwrap_or_else(|l| l.into_inner());

                            if title.is_loaded() {
                                title.get_or_init(0);

                                return Command::perform(
                                    async { Message::EpisodesLoaded },
                                    super::Message::PaneAction,
                                );
                            } else {
                                *state = State::Loading;

                                drop(title); //We release the lock before sending it to the thread
                                let title = Arc::clone(&self.backend.titles[focused]);

                                return Command::perform(
                                    async move {
                                        let mut title =
                                            title.lock().unwrap_or_else(|l| l.into_inner());
                                        title.get_or_init(0);
                                        Message::EpisodesLoaded
                                    },
                                    super::Message::PaneAction,
                                );
                            }
                        }
                        PaneKind::Episodes(_) => {
                            *state = State::Watching;

                            return Command::perform(
                                async { Message::LoadingEpisode },
                                super::Message::PaneAction,
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
                                PaneData::new(PaneKind::None, None);
                        }

                        if let Some(adj) = self.panes.adjacent(&pane, Direction::Left) {
                            self.panes.swap(&pane, &adj);
                            self.focus = Some(adj);

                            let panel = self.panes.get_mut(&pane).unwrap();
                            *panel = PaneData::new(PaneKind::None, None);
                        }
                    }
                }
            }
            Message::EpisodesLoaded => {
                *state = State::Normal;

                if let Some(pane) = self.focus {
                    let panel = self.panes.get(&pane).unwrap();
                    let focused = panel.list.as_ref().unwrap().focused;
                    let title_count = self.backend.titles[focused]
                        .lock()
                        .unwrap_or_else(|l| l.into_inner())
                        .count;

                    if let Some(adj) = self.panes.adjacent(&pane, Direction::Left) {
                        let episodes_list = List::new(0, title_count);

                        *self.panes.get_mut(&adj).unwrap() =
                            PaneData::new(PaneKind::Episodes(focused), Some(episodes_list));

                        self.panes.swap(&pane, &adj);
                        self.focus = Some(adj);

                        return Command::perform(
                            async { Message::FocusItem(Direction::Left) },
                            super::Message::PaneAction,
                        );
                    }
                }
            }
            Message::LoadingEpisode => {
                if let Some(pane) = self.focus {
                    let panel = self.panes.get(&pane).unwrap();

                    if let PaneKind::Episodes(title) = panel.kind {
                        let number = panel.list.as_ref().unwrap().focused;
                        let episode = self.backend.titles[title]
                            .lock()
                            .unwrap_or_else(|l| l.into_inner())
                            .get_episode(number);

                        return Command::perform(
                            async move {
                                if let Err(error) =
                                    episode.lock().unwrap_or_else(|l| l.into_inner()).run()
                                {
                                    eprintln!("{error}");
                                }
                                Message::FocusItem(Direction::Left)
                            },
                            super::Message::PaneAction,
                        );
                    }
                }
            }
        }

        Command::none()
    }

    pub fn view(&self) -> Element<super::Message> {
        let focus = self.focus;

        PaneGrid::new(&self.panes, |id, pane, _| {
            let is_focused = focus == Some(id);

            let content =
                pane_grid::Content::new(Pane::view_content(pane, self)).style(if is_focused {
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
                    let title_bar = pane_grid::TitleBar::new("Episodes").padding(10);
                    content.title_bar(title_bar)
                }
                _ => content,
            }
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .into()
    }

    fn view_content<'a>(pane_data: &PaneData, pane: &Pane) -> Element<'a, super::Message> {
        match &pane_data.kind {
            PaneKind::Titles => container(
                scrollable(
                    pane_data
                        .list
                        .as_ref()
                        .unwrap()
                        .view(pane.backend.view(), |_, _, _| theme::Text::Default),
                )
                .height(Length::Shrink)
                .id(SCROLLABLE_ID.clone()),
            )
            .width(Length::Fill)
            .padding(5)
            .center_y()
            .into(),
            PaneKind::Episodes(title) => {
                let title = pane.backend.titles[*title]
                    .lock()
                    .unwrap_or_else(|l| l.into_inner());

                container(
                    scrollable(pane_data.list.as_ref().unwrap().view(
                        title.view(),
                        |focused, id, _| {
                            let watched = title
                                .get_episode(id)
                                .lock()
                                .unwrap_or_else(|l| l.into_inner())
                                .metadata
                                .watched;
                            match id == focused {
                                true if watched => theme::Text::WatchedFocus,
                                false if watched => theme::Text::Watched,
                                true | false => theme::Text::Default,
                            }
                        },
                    ))
                    .height(Length::Shrink)
                    .id(SCROLLABLE_ID.clone()),
                )
                .width(Length::Fill)
                .padding(5)
                .center_y()
                .into()
            }
            PaneKind::Metadata(meta) => {
                let meta = meta.lock().unwrap_or_else(|l| l.into_inner());
                container(scrollable(
                    column![
                        image::Image::new(meta.thumbnail()),
                        text(meta.description())
                    ]
                    .spacing(10),
                ))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(15)
                .into()
            }

            PaneKind::None => container(column![])
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        }
    }
}
