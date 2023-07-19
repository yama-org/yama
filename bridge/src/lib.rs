pub mod cache;
pub mod subscription;

use backend::Backend;
use cache::{Cache, EpisodeCache, TitleCache};

use iced::futures::channel::mpsc;
use iced::widget::pane_grid::{Direction, ResizeEvent};
use std::sync::Arc;
use std::time::Instant;

type TitleIndex = usize;
type EpisodeNumber = usize;
type Refresh = bool;

/// Messages to be sended to the [`Backend`][Backend] thread.
#[derive(Debug, Clone)]
pub enum BackendMessage {
    /// Args: (Title index, Should refresh)
    LoadEpisodes(TitleIndex, Refresh),
    /// Args: (Title index, Episode number)
    WatchEpisode(TitleIndex, EpisodeNumber),
    /// Args: (Title index, Episode number)
    MarkEpisode(TitleIndex, EpisodeNumber),
    /// Args: (Title index, Episode number)
    MarkPreviousEpisodes(TitleIndex, EpisodeNumber),
}

/// Messages to be sended to the [Frontend] thread.
#[derive(Debug, Clone)]
pub enum FrontendMessage {
    Ready(mpsc::Sender<BackendMessage>, Cache),
    PaneAction(PanelAction),
    Loading(Instant),
    MenuBar(MenuBar),
    HideMenubar,
    FileDialog,
    Exit,
    Error(Arc<str>),
}

/// Action to be sended to the Panels frontend thread.
#[derive(Debug, Clone)]
pub enum PanelAction {
    /// Args: (Title index, Title cache, Should refresh)
    EpisodesLoaded(TitleIndex, TitleCache),
    /// Args: (Title index)
    UpdateEpisode(TitleIndex, Vec<EpisodeCache>),
    MarkPreviousEpisodes,
    MarkEpisode,
    FocusItem(Direction),
    Refresh,
    Enter,
    Back,
    Resized(ResizeEvent),
}

/// [yama] menus
#[derive(Debug, Clone)]
pub enum MenuBar {
    About,
    Config,
    Yama,
}
