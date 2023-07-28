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
    Restart,
}

/// Messages to be sended to the [Frontend] thread.
#[derive(Debug, Clone)]
pub enum FrontendMessage {
    Recovery(mpsc::Sender<BackendMessage>, Arc<str>),
    Ready(mpsc::Sender<BackendMessage>, Cache),
    UpdateConfig(ConfigChange),
    PaneAction(PanelAction),
    Loading(Instant),
    MenuBar(Modals),
    Error(Arc<str>),
    HideMenubar,
    ToLoad,
    Exit,
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
    JumpTo(usize),
    Plus(isize),
    Refresh,
    Enter,
    Start,
    End,
    Back,
    Resized(ResizeEvent),
}

/// [yama] floating windows
#[derive(Debug, Clone)]
pub enum Modals {
    Help,
    About,
    Config,
    Yama,
    Error(Arc<str>),
}

#[derive(Debug, Clone)]
pub enum ConfigChange {
    SeriesPath,
    ThemePath,
    MinTime(f32),
}
