use std::vec;

use super::theme::{self, widget::Element};
use super::{List, State};

use bridge::BridgeMessage;
use bridge::{cache::*, FrontendMessage};
use bridge::{BackendMessage, PanelsMessage as Message};
use iced::futures::channel::mpsc::Sender;
use iced::widget::pane_grid::{self, Direction, PaneGrid};
use iced::widget::{button, column, container, image, row, scrollable, svg, text, tooltip};
use iced::{Command, Length, Settings};
use once_cell::sync::Lazy;
use tracing_unwrap::ResultExt;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static RELOAD_SVG: &[u8] = include_bytes!("../../res/reload.svg");
static CHECKMARK_SVG: &[u8] = include_bytes!("../../res/checkmark.svg");
static CHECKMARK_P_SVG: &[u8] = include_bytes!("../../res/checkmark_previous.svg");
static NO_TUMBNAIL: &[u8] = include_bytes!("../../res/no_thumbnail.jpg");

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

        if count == 0 {
            return Pane {
                focus: None,
                panes,
                sender,
                cache,
            };
        }

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
            Message::LoadingEpisodes(focus, refresh) => {
                *state = State::Loading;
                self.sender
                    .try_send(BackendMessage::GettingTitleEpisodes(focus, refresh))
                    .expect_or_log("[ERROR] - Can not communicate with background thread.")
            }
            Message::EpisodesLoaded(title_number, title_cache, refresh) => {
                self.cache.set_title_cache(title_cache, title_number);
                *state = State::Normal;

                if let Some(pane) = self.focus {
                    let count = self.cache.titles_map[title_number].1.len();
                    let episodes_list = List::new(0, count);

                    if refresh {
                        *self.panes.get_mut(&pane).unwrap() =
                            PaneData::new(PaneKind::Episodes(title_number), Some(episodes_list));
                    } else if let Some(adj) = self.panes.adjacent(&pane, Direction::Left) {
                        *self.panes.get_mut(&adj).unwrap() =
                            PaneData::new(PaneKind::Episodes(title_number), Some(episodes_list));

                        self.panes.swap(&pane, &adj);
                        self.focus = Some(adj);
                    }
                }

                return Command::perform(
                    async { BridgeMessage::PaneAction(Message::FocusItem(Direction::Left)) },
                    FrontendMessage::Bridge,
                );
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
                            //Fixes bottom items being cut-out
                            y += 1.0 / Settings::<()>::default().default_text_size;
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
                                .try_send(BackendMessage::LoadTitleEpisodes(focus, false))
                                .expect_or_log(
                                    "[ERROR] - Can not communicate with background thread.",
                                )
                        }
                        PaneKind::Episodes(title) => {
                            let list = panel.list.as_ref().unwrap();

                            if !list.empty {
                                *state = State::Watching;

                                self.sender
                                    .try_send(BackendMessage::WatchEpisode(title, list.focused))
                                    .expect_or_log(
                                        "[ERROR] - Can not communicate with background thread.",
                                    )
                            }
                        }
                        _ => (),
                    }
                }
            }
            Message::Refresh => {
                if let Some(pane) = self.focus {
                    let panel = self.panes.get(&pane).unwrap();

                    if let PaneKind::Episodes(title) = panel.kind {
                        self.sender
                            .try_send(BackendMessage::LoadTitleEpisodes(title, true))
                            .expect_or_log("[ERROR] - Can not communicate with background thread.")
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
            Message::SavingEpisode(title, episode_cache) => {
                for ep in episode_cache {
                    self.cache.set_episode_cache(ep, title);
                }

                *state = State::Normal;
                return Command::perform(
                    async { BridgeMessage::PaneAction(Message::FocusItem(Direction::Left)) },
                    FrontendMessage::Bridge,
                );
            }
            Message::MarkEpisode => {
                if let Some(pane) = self.focus {
                    let panel = self.panes.get(&pane).unwrap();
                    let list = panel.list.as_ref().unwrap();

                    if let PaneKind::Episodes(title) = panel.kind {
                        self.sender
                            .try_send(BackendMessage::MarkEpisode(title, list.focused))
                            .expect_or_log("[ERROR] - Can not communicate with background thread.")
                    }
                }
            }
            Message::MarkTitleEpisodes => {
                if let Some(pane) = self.focus {
                    let panel = self.panes.get(&pane).unwrap();
                    let list = panel.list.as_ref().unwrap();

                    if let PaneKind::Episodes(title) = panel.kind {
                        self.sender
                            .try_send(BackendMessage::MarkTitleEpisodes(title, list.focused))
                            .expect_or_log("[ERROR] - Can not communicate with background thread.")
                    }
                }
            }
        }

        Command::none()
    }

    pub fn view(&self) -> Element<FrontendMessage> {
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
                    let reload_svg = svg(svg::Handle::from_memory(RELOAD_SVG))
                        .width(Length::Fixed(25.0))
                        .height(Length::Fixed(25.0));

                    let checkmark_svg = svg(svg::Handle::from_memory(CHECKMARK_SVG))
                        .width(Length::Fixed(25.0))
                        .height(Length::Fixed(25.0));

                    let checkmark_previous_svg = svg(svg::Handle::from_memory(CHECKMARK_P_SVG))
                        .width(Length::Fixed(25.0))
                        .height(Length::Fixed(25.0));

                    let title_bar = pane_grid::TitleBar::new(
                        row![
                            text("Episodes").width(Length::Fill),
                            tooltip(
                                button(reload_svg)
                                    .on_press(FrontendMessage::Bridge(BridgeMessage::PaneAction(
                                        Message::Refresh
                                    )))
                                    .style(theme::Button::Menu),
                                "Refresh current title",
                                tooltip::Position::Top,
                            )
                            .style(theme::Container::Tooltip),
                            tooltip(
                                button(checkmark_svg)
                                    .on_press(FrontendMessage::Bridge(BridgeMessage::PaneAction(
                                        Message::MarkEpisode
                                    )))
                                    .style(theme::Button::Menu),
                                "Mark episode as watched",
                                tooltip::Position::Top,
                            )
                            .style(theme::Container::Tooltip),
                            tooltip(
                                button(checkmark_previous_svg)
                                    .on_press(FrontendMessage::Bridge(BridgeMessage::PaneAction(
                                        Message::MarkTitleEpisodes
                                    )))
                                    .style(theme::Button::Menu),
                                "Mark previous episodes as watched",
                                tooltip::Position::Top,
                            )
                            .style(theme::Container::Tooltip),
                        ]
                        .align_items(iced_native::Alignment::Center)
                        .width(Length::Fill),
                    )
                    .padding(10);
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

    fn view_content<'a>(pane_data: &'a PaneData, pane: &Pane) -> Element<'a, super::Message> {
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
            PaneKind::Metadata(meta) => {
                let handle = match &meta.thumbnail {
                    Some(path) => image::Handle::from_path(path),
                    None => image::Handle::from_memory(NO_TUMBNAIL),
                };

                container(scrollable(
                    column![image::Image::new(handle), text(meta.description.clone())].spacing(10),
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
