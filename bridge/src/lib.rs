pub mod cache;

use cache::{Cache, EpisodeCache, TitleCache};

use backend::backend::Backend;
use iced::futures::channel::mpsc;
use iced::subscription::{self, Subscription};
use iced::widget::pane_grid::Direction;
use std::time::Instant;
use tracing::info;

#[derive(Debug, Clone)]
pub enum BackendMessage {
    LoadTitleEpisodes(usize),
    GettingTitleEpisodes(usize),
    WatchEpisode(usize, usize),
}

#[derive(Debug, Clone)]
pub enum FrontendMessage {
    Bridge(BridgeMessage),
    Loading(Instant),
}

#[derive(Debug, Clone)]
pub enum BridgeMessage {
    Ready(mpsc::Sender<BackendMessage>, Cache),
    PaneAction(PanelsMessage),
    None,
}

#[derive(Debug, Clone)]
pub enum PanelsMessage {
    EpisodesLoaded(usize, TitleCache),
    LoadingEpisodes(usize),
    FocusItem(Direction),
    SavingEpisode(usize, usize, EpisodeCache),
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
                        Some(BridgeMessage::Ready(sender, cache)),
                        State::Ready(receiver, backend),
                    )
                }
                State::Ready(mut receiver, mut backend) => {
                    use iced::futures::StreamExt;
                    let msg = receiver.select_next_some().await;

                    match msg {
                        BackendMessage::LoadTitleEpisodes(number) => {
                            //let title = backend.titles.get_mut(number).unwrap();

                            /*if title.is_loaded() {
                                title.load_episodes(false);
                                let title_cache = TitleCache::new(title);

                                (
                                    Some(BridgeMessage::PaneAction(PanelsMessage::EpisodesLoaded(
                                        number,
                                        title_cache,
                                    ))),
                                    State::Ready(receiver, backend),
                                )
                            } else {*/
                            (
                                Some(BridgeMessage::PaneAction(PanelsMessage::LoadingEpisodes(
                                    number,
                                ))),
                                State::Ready(receiver, backend),
                            )
                            //}
                        }
                        BackendMessage::GettingTitleEpisodes(number) => {
                            let title = backend.titles.get_mut(number).unwrap();
                            info!("Loading episodes of {}.", title.name);

                            title.load_episodes(false);
                            let title_cache = TitleCache::new(title);

                            (
                                Some(BridgeMessage::PaneAction(PanelsMessage::EpisodesLoaded(
                                    number,
                                    title_cache,
                                ))),
                                State::Ready(receiver, backend),
                            )
                        }
                        BackendMessage::WatchEpisode(title, number) => {
                            let episode = backend
                                .titles
                                .get_mut(title)
                                .unwrap()
                                .episodes
                                .get_mut(number)
                                .unwrap();

                            episode.run().expect("[ERROR] - Cannot run episode.");
                            let episode_cache = EpisodeCache::new(episode);

                            (
                                Some(BridgeMessage::PaneAction(PanelsMessage::SavingEpisode(
                                    title,
                                    number,
                                    episode_cache,
                                ))),
                                State::Ready(receiver, backend),
                            )
                        }
                    }
                }
            }
        },
    )
}
