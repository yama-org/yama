mod inner_data;
mod inner_panel;
mod pointer;

use self::inner_data::{FocusedType, InnerData};
use self::inner_panel::{InnerPanel, SCROLLABLE_ID};

use crate::embedded::*;
use crate::frontend::State;
use crate::widgets::{theme, Element};

use bridge::{cache::*, FrontendMessage};
use bridge::{BackendMessage, PanelAction as Message};

use iced::font::Family;
use iced::futures::channel::mpsc::Sender;
use iced::widget::pane_grid::{self, Direction, PaneGrid};
use iced::widget::{button, row, scrollable, svg, text, tooltip};
use iced::{Command, Font, Length, Settings};
use std::vec;

#[derive(Debug)]
pub struct Panels {
    panes: pane_grid::State<InnerPanel>,
    focus: pane_grid::Pane,
    sender: Sender<BackendMessage>,
    data: InnerData,
}

impl Panels {
    pub fn new(cache: Cache, sender: Sender<BackendMessage>) -> Panels {
        let title_meta = cache.get_title_cache(0);

        let (mut panes, focus) = pane_grid::State::new(InnerPanel::Listdata(FocusedType::Title(0)));
        panes.split(
            pane_grid::Axis::Vertical,
            &focus,
            InnerPanel::Metadata(title_meta),
        );

        Panels {
            panes,
            sender,
            focus,
            data: InnerData::new(cache),
        }
    }

    fn next(&mut self, mut y: f32) -> Command<FrontendMessage> {
        //Fixes top items being cut-out
        if y > 0.1 {
            //Fixes bottom items being cut-out
            y += 1.0 / Settings::<()>::default().default_text_size;
        }

        let metadata = self.data.get_metacache();

        if let Some(adj) = self.panes.adjacent(&self.focus, Direction::Right) {
            *self.panes.get_mut(&adj).unwrap() = InnerPanel::Metadata(metadata);
        }

        scrollable::snap_to(
            SCROLLABLE_ID.clone(),
            scrollable::RelativeOffset { x: 0.0, y },
        )
    }

    pub fn update(&mut self, message: Message, state: &mut State) -> Command<FrontendMessage> {
        match message {
            Message::EpisodesLoaded(title_number, title_cache) => {
                *state = State::Normal;

                self.data.set_title_cache(title_cache, title_number);
                *self.panes.get_mut(&self.focus).unwrap() =
                    InnerPanel::Listdata(FocusedType::Episode(0, 0));

                return Command::perform(
                    async { Message::FocusItem(Direction::Left) },
                    FrontendMessage::PaneAction,
                );
            }

            Message::FocusItem(direction) => {
                let y = self.data.update(direction);
                return self.next(y);
            }

            Message::JumpTo(to) => {
                let y = self.data.jump_to(to);
                return self.next(y);
            }

            Message::Plus(to_add) => {
                let y = self.data.plus(to_add);
                return self.next(y);
            }

            Message::Start => {
                let y = self.data.start();
                return self.next(y);
            }

            Message::End => {
                let y = self.data.end();
                return self.next(y);
            }

            Message::Enter => match self.data.get_type() {
                FocusedType::Title(title_number) => {
                    *state = State::Loading;
                    let _ = self
                        .sender
                        .try_send(BackendMessage::LoadEpisodes(title_number, false));
                }

                FocusedType::Episode(title_number, episode_number) => {
                    *state = State::Watching;
                    let _ = self
                        .sender
                        .try_send(BackendMessage::WatchEpisode(title_number, episode_number));
                }
            },

            Message::Back => {
                self.data.back();

                if let Some(adj) = self.panes.adjacent(&self.focus, Direction::Right) {
                    let metadata = self.data.get_metacache();
                    *self.panes.get_mut(&adj).unwrap() = InnerPanel::Metadata(metadata);
                }

                *self.panes.get_mut(&self.focus).unwrap() =
                    InnerPanel::Listdata(FocusedType::Title(self.data.focused));
            }

            Message::Refresh => {
                if let FocusedType::Episode(title_number, _) = self.data.get_type() {
                    *state = State::Loading;
                    let _ = self
                        .sender
                        .try_send(BackendMessage::LoadEpisodes(title_number, true));
                }
            }

            Message::UpdateEpisode(title_number, episodes_cache) => {
                *state = State::Normal;
                self.data.set_episodes_cache(title_number, episodes_cache);

                return Command::perform(
                    async { Message::FocusItem(Direction::Left) },
                    FrontendMessage::PaneAction,
                );
            }

            Message::MarkEpisode => {
                if let FocusedType::Episode(title_number, episode_number) = self.data.get_type() {
                    let _ = self
                        .sender
                        .try_send(BackendMessage::MarkEpisode(title_number, episode_number));
                }
            }

            Message::MarkPreviousEpisodes => {
                if let FocusedType::Episode(title_number, episode_number) = self.data.get_type() {
                    let _ = self.sender.try_send(BackendMessage::MarkPreviousEpisodes(
                        title_number,
                        episode_number,
                    ));
                }
            }

            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(&split, ratio);
            }
        }

        Command::none()
    }

    pub fn view(&self) -> Element<FrontendMessage> {
        let focus = self.focus;

        PaneGrid::new(&self.panes, |id, pane, _| {
            let is_focused = focus == id;

            let content = pane_grid::Content::new(pane.view(&self.data)).style(if is_focused {
                theme::Container::Focused
            } else {
                theme::Container::Unfocused
            });

            if let InnerPanel::Listdata(focused_type) = pane {
                match focused_type {
                    FocusedType::Title(_) => {
                        let title_bar = pane_grid::TitleBar::new(
                            row![
                                text("Titles")
                                    .font(Font {
                                        family: Family::Name("Kumbh Sans"),
                                        weight: iced::font::Weight::Semibold,
                                        ..Default::default()
                                    })
                                    .size(26)
                                    .width(Length::Fill),
                                button(
                                    svg(svg::Handle::from_memory(EMPTY_SVG))
                                        .width(Length::Fixed(25.0))
                                        .height(Length::Fixed(25.0))
                                )
                            ]
                            .align_items(iced::Alignment::Center)
                            .width(Length::Fill),
                        )
                        .padding(15);
                        content.title_bar(title_bar)
                    }

                    FocusedType::Episode(_, _) => {
                        let svg_size = 25.0;

                        let reload_svg = svg(svg::Handle::from_memory(RELOAD_SVG))
                            .width(Length::Fixed(svg_size))
                            .height(Length::Fixed(svg_size));

                        let checkmark_svg = svg(svg::Handle::from_memory(CHECKMARK_SVG))
                            .width(Length::Fixed(svg_size))
                            .height(Length::Fixed(svg_size));

                        let checkmark_previous_svg = svg(svg::Handle::from_memory(CHECKMARK_P_SVG))
                            .width(Length::Fixed(svg_size))
                            .height(Length::Fixed(svg_size));

                        let title_bar = pane_grid::TitleBar::new(
                            row![
                                text("Episodes")
                                    .font(Font {
                                        family: Family::Name("Kumbh Sans"),
                                        weight: iced::font::Weight::Semibold,
                                        ..Default::default()
                                    })
                                    .size(26)
                                    .width(Length::Fill),
                                tooltip(
                                    button(reload_svg)
                                        .on_press(FrontendMessage::PaneAction(Message::Refresh))
                                        .style(theme::Button::Menu),
                                    "Refresh current title",
                                    tooltip::Position::Top,
                                )
                                .style(theme::Container::Tooltip),
                                tooltip(
                                    button(checkmark_svg)
                                        .on_press(FrontendMessage::PaneAction(Message::MarkEpisode))
                                        .style(theme::Button::Menu),
                                    "Mark episode as watched",
                                    tooltip::Position::Top,
                                )
                                .style(theme::Container::Tooltip),
                                tooltip(
                                    button(checkmark_previous_svg)
                                        .on_press(FrontendMessage::PaneAction(
                                            Message::MarkPreviousEpisodes
                                        ))
                                        .style(theme::Button::Menu),
                                    "Mark previous episodes as watched",
                                    tooltip::Position::Top,
                                )
                                .style(theme::Container::Tooltip),
                            ]
                            .align_items(iced::Alignment::Center)
                            .width(Length::Fill),
                        )
                        .padding(15);

                        content.title_bar(title_bar)
                    }
                }
            } else {
                let title_bar = pane_grid::TitleBar::new("");
                content.title_bar(title_bar)
            }
        })
        .on_resize(10, |r| FrontendMessage::PaneAction(Message::Resized(r)))
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .into()
    }
}
