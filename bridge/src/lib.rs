pub mod cache;

use cache::{Cache, EpisodeCache, TitleCache};

use backend::backend::Backend;
use iced::futures::channel::mpsc;
use iced::subscription::{self, Subscription};
use iced::widget::pane_grid::Direction;
use std::time::Instant;
use tracing::info;
use tracing_unwrap::{OptionExt, ResultExt};

#[derive(Debug, Clone)]
pub enum BackendMessage {
    LoadTitleEpisodes(usize, bool),
    GettingTitleEpisodes(usize, bool),
    MarkTitleEpisodes(usize, usize),
    WatchEpisode(usize, usize),
    MarkEpisode(usize, usize),
}

#[derive(Debug, Clone)]
pub enum MenuBar {
    About,
    Config,
    Yama,
}

#[derive(Debug, Clone)]
pub enum FrontendMessage {
    Bridge(BridgeMessage),
    Loading(Instant),
    MenuBar(MenuBar),
    HideMenubar,
    FileDialog,
    Exit,
}

#[derive(Debug, Clone)]
pub enum BridgeMessage {
    Ready(mpsc::Sender<BackendMessage>, Cache),
    PaneAction(PanelsMessage),
}

#[derive(Debug, Clone)]
pub enum PanelsMessage {
    EpisodesLoaded(usize, TitleCache, bool),
    LoadingEpisodes(usize, bool),
    FocusItem(Direction),
    SavingEpisode(usize, Vec<EpisodeCache>),
    MarkTitleEpisodes,
    MarkEpisode,
    Refresh,
    Enter,
    Back,
}

enum State {
    Starting,
    Ready(mpsc::Receiver<BackendMessage>, Backend),
}

pub fn start() -> Subscription<BridgeMessage> {
    subscription::unfold(
        std::any::TypeId::of::<Backend>(),
        State::Starting,
        |state| async move {
            match state {
                State::Starting => {
                    info!("Starting up backend...");

                    let mut backend = Backend::new();
                    let (sender, receiver) = mpsc::channel(512);
                    let batched_data = backend.download_title_data().await;

                    for data in batched_data {
                        let id = data.id;
                        backend.titles[id].data = Some(data);
                    }

                    let cache = Cache::new(&backend);

                    (
                        BridgeMessage::Ready(sender, cache),
                        State::Ready(receiver, backend),
                    )
                }
                State::Ready(mut receiver, mut backend) => {
                    use iced::futures::StreamExt;
                    let msg = receiver.select_next_some().await;

                    match msg {
                        BackendMessage::LoadTitleEpisodes(number, refresh) => (
                            BridgeMessage::PaneAction(PanelsMessage::LoadingEpisodes(
                                number, refresh,
                            )),
                            State::Ready(receiver, backend),
                        ),
                        BackendMessage::GettingTitleEpisodes(number, refresh) => {
                            let title = backend.titles.get_mut(number).unwrap_or_log();
                            info!("Loading episodes of {}.", title.name);

                            title.load_episodes(refresh);
                            let title_cache = TitleCache::new(title);

                            (
                                BridgeMessage::PaneAction(PanelsMessage::EpisodesLoaded(
                                    number,
                                    title_cache,
                                    refresh,
                                )),
                                State::Ready(receiver, backend),
                            )
                        }
                        BackendMessage::WatchEpisode(title, number) => {
                            let episode = backend
                                .titles
                                .get_mut(title)
                                .unwrap_or_log()
                                .episodes
                                .get_mut(number)
                                .unwrap_or_log();

                            episode.run().expect_or_log("[ERROR] - Cannot run episode.");
                            let episode_cache = EpisodeCache::new(episode);

                            (
                                BridgeMessage::PaneAction(PanelsMessage::SavingEpisode(
                                    title,
                                    vec![episode_cache],
                                )),
                                State::Ready(receiver, backend),
                            )
                        }
                        BackendMessage::MarkEpisode(title, number) => {
                            let episode = backend
                                .titles
                                .get_mut(title)
                                .unwrap_or_log()
                                .episodes
                                .get_mut(number)
                                .unwrap_or_log();

                            episode
                                .as_watched()
                                .expect_or_log("[ERROR] - Cannot mark episode.");
                            let episode_cache = EpisodeCache::new(episode);

                            (
                                BridgeMessage::PaneAction(PanelsMessage::SavingEpisode(
                                    title,
                                    vec![episode_cache],
                                )),
                                State::Ready(receiver, backend),
                            )
                        }
                        BackendMessage::MarkTitleEpisodes(title_number, number) => {
                            let title = backend.titles.get_mut(title_number).unwrap_or_log();
                            let mut episodes_cache = Vec::with_capacity(number);

                            title
                                .as_watched(number)
                                .expect_or_log("[ERROR] - Cannot mark episode.");

                            for episode in &title.episodes[..number] {
                                episodes_cache.push(EpisodeCache::new(episode));
                            }

                            (
                                BridgeMessage::PaneAction(PanelsMessage::SavingEpisode(
                                    title_number,
                                    episodes_cache,
                                )),
                                State::Ready(receiver, backend),
                            )
                        }
                    }
                }
            }
        },
    )
}
