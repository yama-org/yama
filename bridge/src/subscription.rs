use super::*;

use iced::subscription::{self, Subscription};
use tracing::{error, info};

/// States of the [`Backend`][Backend] [`Subscription`][Subscription].
#[derive(Debug)]
enum State {
    Starting,
    Ready(mpsc::Receiver<BackendMessage>, Backend),
}

/// Subscribes to the [`Backend`][Backend] thread of the [yama] application.
///
/// At the start it will return a [`FrontendMessage::Ready`][FrontendMessage::Ready] with:
/// - [`mpsc::Sender`][mpsc::Sender] to send [`BackendMessage`][BackendMessage] to this thread.
/// - [`Cache`][Cache] with just the initial data loaded (just Titles and no Episodes).
pub fn start() -> Subscription<FrontendMessage> {
    subscription::unfold(
        std::any::TypeId::of::<Backend>(),
        State::Starting,
        |state| async move {
            match state {
                State::Starting => {
                    info!("Starting up the backend thread...");

                    let (sender, receiver) = mpsc::channel(1);
                    let backend = match Backend::new().await {
                        Ok(b) => b,
                        Err(e) => {
                            error!("Failed to create backend: {e}");
                            std::process::exit(1)
                        }
                    };

                    let cache = Cache::new(&backend);

                    (
                        FrontendMessage::Ready(sender, cache),
                        State::Ready(receiver, backend),
                    )
                }

                State::Ready(mut receiver, mut backend) => {
                    use iced::futures::StreamExt;
                    let msg = receiver.select_next_some().await;

                    let msg = match msg {
                        BackendMessage::LoadEpisodes(title_number, refresh) => match backend
                            .titles
                            .get_mut(title_number)
                        {
                            Some(title) => {
                                info!("Loading episodes of: {}.", title.name);

                                match title.load_episodes(refresh).await {
                                    Ok(_) => {
                                        let title_cache = TitleCache::with_episodes(title);

                                        FrontendMessage::PaneAction(PanelAction::EpisodesLoaded(
                                            title_number,
                                            title_cache,
                                        ))
                                    }
                                    Err(e) => {
                                        error!("{}", e);
                                        FrontendMessage::Error(Arc::from("Could not load title!"))
                                    }
                                }
                            }
                            None => {
                                error!("No title found at the index {}", title_number);
                                FrontendMessage::Error(Arc::from("No title found!"))
                            }
                        },

                        BackendMessage::WatchEpisode(title_number, episode_number) => match backend
                            .get_episode(title_number, episode_number)
                        {
                            Some(episode) => {
                                info!("Loading episode: {}.", episode.name);

                                match episode.run() {
                                    Ok(_) => {
                                        let episode_cache = EpisodeCache::new(episode);

                                        FrontendMessage::PaneAction(PanelAction::UpdateEpisode(
                                            title_number,
                                            vec![episode_cache],
                                        ))
                                    }
                                    Err(e) => {
                                        error!("{}", e);
                                        FrontendMessage::Error(Arc::from("Could not load episode!"))
                                    }
                                }
                            }
                            None => {
                                error!("No episode found at the index {}", episode_number);
                                FrontendMessage::Error(Arc::from("No episode found!"))
                            }
                        },

                        BackendMessage::MarkEpisode(title_number, episode_number) => match backend
                            .get_episode(title_number, episode_number)
                        {
                            Some(episode) => {
                                info!(
                                    "Mark {} as {}.",
                                    episode.name,
                                    if episode.metadata.watched {
                                        "unwatched"
                                    } else {
                                        "watched"
                                    }
                                );

                                match episode.as_watched() {
                                    Ok(_) => {
                                        let episode_cache = EpisodeCache::new(episode);

                                        FrontendMessage::PaneAction(PanelAction::UpdateEpisode(
                                            title_number,
                                            vec![episode_cache],
                                        ))
                                    }
                                    Err(e) => {
                                        error!("{}", e);
                                        FrontendMessage::Error(Arc::from("Could not load episode!"))
                                    }
                                }
                            }
                            None => {
                                error!("No episode found at the index {}", episode_number);
                                FrontendMessage::Error(Arc::from("No episode found!"))
                            }
                        },

                        BackendMessage::MarkPreviousEpisodes(title_number, episode_number) => {
                            match backend.titles.get_mut(title_number) {
                                Some(title) => {
                                    info!(
                                        "Mark all previous episodes of {} from episode {} as watched/unwatched.",
                                        title.name,
                                        episode_number + 1
                                    );

                                    match title.as_watched(episode_number) {
                                        Ok(_) => {
                                            // We can safely unwrap because 'as_watched' checked the bounds for us.
                                            let episodes_cache: Vec<EpisodeCache> =
                                                title.episodes.as_ref().unwrap()[..episode_number]
                                                    .iter()
                                                    .map(EpisodeCache::new)
                                                    .collect();

                                            FrontendMessage::PaneAction(PanelAction::UpdateEpisode(
                                                title_number,
                                                episodes_cache,
                                            ))
                                        }
                                        Err(e) => {
                                            error!("{}", e);
                                            FrontendMessage::Error(Arc::from(
                                                "Could not load episode!",
                                            ))
                                        }
                                    }
                                }
                                None => {
                                    error!("No title found at the index {}", title_number);
                                    FrontendMessage::Error(Arc::from("No title found!"))
                                }
                            }
                        }
                    };

                    (msg, State::Ready(receiver, backend))
                }
            }
        },
    )
}
