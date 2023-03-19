use std::vec;

use super::theme::{self, widget::Element};
use super::{List, State};

use bridge::BridgeMessage;
use bridge::{cache::*, FrontendMessage};
use bridge::{BackendMessage, PanelsMessage as Message};
use iced::futures::channel::mpsc::Sender;
use iced::widget::pane_grid::{self, Direction, PaneGrid};
use iced::widget::{column, container, image, scrollable, text};
use iced::{Command, Length, Settings, Size};
use iced_lazy::responsive;
use once_cell::sync::Lazy;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

#[derive(Debug)]
enum PaneKind {
    None,
    Titles,
    Episodes(usize),
    Metadata(MetaCache),
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
    cache: Cache,
    sender: Sender<BackendMessage>,
}

impl PaneData {
    fn new(kind: PaneKind, list: Option<List>) -> PaneData {
        PaneData { kind, list }
    }
}

impl Pane {
    pub fn new(cache: Cache, sender: Sender<BackendMessage>) -> Pane {
        let mut focus = Option::None;
        let count = cache.titles_map.len();
        let title_list = List::new(0, count);

        let (mut panes, pane) = pane_grid::State::new(PaneData::new(PaneKind::None, None));
        let result = panes.split(
            pane_grid::Axis::Vertical,
            &pane,
            PaneData::new(PaneKind::Titles, Some(title_list)),
        );

        if let Some((pane, split)) = result {
            focus = Some(pane);
            panes.resize(&split, 0.25);

            let title = cache.titles_map[0].0.clone();

            let result = panes.split(
                pane_grid::Axis::Vertical,
                &pane,
                PaneData::new(PaneKind::Metadata(title), None),
            );

            if let Some((_, split)) = result {
                panes.resize(&split, 0.35);
            }
        }

        Pane {
            panes,
            focus,
            sender,
            cache,
        }
    }

    pub fn update(&mut self, message: Message, state: &mut State) -> Command<FrontendMessage> {
        match message {
            Message::LoadingEpisodes(focus) => {
                *state = State::Loading;
                self.sender
                    .try_send(BackendMessage::GettingTitleEpisodes(focus))
                    .expect("[ERROR] - Can not communicate with background thread.")
            }
            Message::EpisodesLoaded(title_number, title_cache) => {
                self.cache.set_title_cache(title_cache, title_number);
                *state = State::Normal;

                if let Some(pane) = self.focus {
                    if let Some(adj) = self.panes.adjacent(&pane, Direction::Left) {
                        let count = self.cache.titles_map[title_number].1.len();
                        let episodes_list = List::new(0, count);

                        *self.panes.get_mut(&adj).unwrap() =
                            PaneData::new(PaneKind::Episodes(title_number), Some(episodes_list));

                        self.panes.swap(&pane, &adj);
                        self.focus = Some(adj);

                        return Command::perform(
                            async {
                                BridgeMessage::PaneAction(Message::FocusItem(Direction::Left))
                            },
                            FrontendMessage::Bridge,
                        );
                    }
                }
            }
            Message::FocusItem(direction) => {
                if let Some(pane) = self.focus {
                    let panel = self.panes.get_mut(&pane).unwrap();

                    if let Some(list) = panel.list.as_mut() {
                        list.update(direction);
                        let focus = list.focused;

                        let mut y = (1.0 / list.size as f32) * list.focused as f32;
                        //Fixes top items being cut-out
                        if y > 0.1 {
                            y += 1.0 / Settings::<()>::default().default_text_size;
                            //Fixes bottom items being cut-out
                        }

                        let metacache = if let PaneKind::Episodes(title) = panel.kind {
                            if let Some(data) = self.cache.titles_map[title].1.get(focus) {
                                data.clone()
                            } else {
                                MetaCache::empty()
                            }
                        } else {
                            self.cache.titles_map[focus].0.clone()
                        };

                        if let Some(adj) = self.panes.adjacent(&pane, Direction::Right) {
                            *self.panes.get_mut(&adj).unwrap() =
                                PaneData::new(PaneKind::Metadata(metacache), None);
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
                            let focus = panel.list.as_ref().unwrap().focused;
                            self.sender
                                .try_send(BackendMessage::LoadTitleEpisodes(focus))
                                .expect("[ERROR] - Can not communicate with background thread.")
                        }
                        PaneKind::Episodes(title) => {
                            let list = panel.list.as_ref().unwrap();

                            if !list.empty {
                                *state = State::Watching;

                                self.sender
                                    .try_send(BackendMessage::WatchEpisode(title, list.focused))
                                    .expect("[ERROR] - Can not communicate with background thread.")
                            }
                        }
                        _ => (),
                    }
                }
            }
            Message::Back => {
                if let Some(pane) = self.focus {
                    let panel = self.panes.get(&pane).unwrap();

                    if let PaneKind::Episodes(title) = panel.kind {
                        if let Some(adj) = self.panes.adjacent(&pane, Direction::Right) {
                            let metacache = self.cache.titles_map[title].0.clone();

                            *self.panes.get_mut(&adj).unwrap() =
                                PaneData::new(PaneKind::Metadata(metacache), None);
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
            Message::SavingEpisode(title, number, episode_cache) => {
                self.cache.set_episode_cache(episode_cache, title, number);
                *state = State::Normal;
                return Command::perform(
                    async { BridgeMessage::PaneAction(Message::FocusItem(Direction::Left)) },
                    FrontendMessage::Bridge,
                );
            }
        }

        Command::none()
    }

    pub fn view(&self) -> Element<FrontendMessage> {
        let focus = self.focus;

        PaneGrid::new(&self.panes, |id, pane, _| {
            let is_focused = focus == Some(id);

            let content = pane_grid::Content::new(responsive(move |size| {
                Pane::view_content(pane, self, size)
            }))
            .style(if is_focused {
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

    fn view_content<'a>(
        pane_data: &'a PaneData,
        pane: &Pane,
        _size: Size,
    ) -> Element<'a, super::Message> {
        match &pane_data.kind {
            PaneKind::Titles => container(
                scrollable(
                    pane_data
                        .list
                        .as_ref()
                        .unwrap()
                        .view(&pane.cache.titles_names, |_, _| theme::Text::Default),
                )
                .height(Length::Shrink)
                .id(SCROLLABLE_ID.clone()),
            )
            .width(Length::Fill)
            .padding(5)
            .center_y()
            .into(),
            PaneKind::Episodes(title) => container(
                scrollable(pane_data.list.as_ref().unwrap().view(
                    &pane.cache.episodes_names[*title],
                    |focused, id| {
                        let watched = pane.cache.episodes_watched[*title][id];
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
            .into(),
            PaneKind::Metadata(meta) => container(scrollable(
                column![
                    image::Image::new(meta.thumbnail.clone()),
                    text(meta.description.clone())
                ]
                .spacing(10),
            ))
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
}
