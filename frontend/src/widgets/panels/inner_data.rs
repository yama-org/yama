use super::pointer::{FocusedElement, Pointer};

use crate::widgets::{theme, Element};

use bridge::{
    cache::{Cache, EpisodeCache, MetaCache, TitleCache},
    FrontendMessage,
};

use iced::widget::pane_grid::Direction;
use std::sync::Arc;

/// The [`FocusedElement`] type.
#[derive(Debug)]
pub enum FocusedType {
    /// Args: (Title Index)
    Title(usize),
    /// Args: (Title Index, Episode Number)
    Episode(usize, usize),
}

/// The data shared between the [`Panels`][super::Panels] and the [`Backend`][backend::Backend].
#[derive(Debug)]
pub struct InnerData {
    pub focused: FocusedElement,
    /// First element is root.
    pointers: Vec<(Pointer, FocusedType)>,
    data: Cache,
}

impl InnerData {
    /// Creates a new [`InnerData`] with the passed [`Cache`].
    pub fn new(data: Cache) -> Self {
        let mut pointers = Vec::with_capacity(data.size + 1);
        pointers.push((Pointer::new(data.size), FocusedType::Title(0)));

        for _ in 0..data.size {
            pointers.push((Pointer::new(0), FocusedType::Episode(0, 0)))
        }

        Self {
            pointers,
            data,
            focused: 0,
        }
    }

    /// Changes the current [`FocusedElement`] according to the passed [`Direction`]
    /// (only [`Direction::Up`] or [`Direction::Down`] are accepted).
    ///
    /// Returns a vertical offset to align a [`scrollable`][iced::widget::scrollable].
    pub fn update(&mut self, direction: Direction) -> f32 {
        self.pointers[self.focused].0.update(direction)
    }

    pub fn jump_to(&mut self, to: usize) -> f32 {
        self.pointers[self.focused].0.jump_to(to)
    }

    pub fn plus(&mut self, to_add: isize) -> f32 {
        self.pointers[self.focused].0.plus(to_add)
    }

    pub fn start(&mut self) -> f32 {
        self.pointers[self.focused].0.start()
    }

    pub fn end(&mut self) -> f32 {
        self.pointers[self.focused].0.end()
    }

    /// Sets the [`TitleCache`][TitleCache] of the indexed [`TitleCache`][TitleCache].
    /// ## Panics
    /// May panic if `number` is out of bounds.
    pub fn set_title_cache(&mut self, title_cache: TitleCache, title_number: usize) {
        self.focused = title_number + 1;

        let pointer = &mut self.pointers[self.focused].0;
        pointer.size = title_cache.size;

        self.data.set_title_cache(title_cache, title_number)
    }

    /// Sets the [`EpisodeCache`][EpisodeCache] of the indexed [`EpisodeCache`][EpisodeCache].
    pub fn set_episodes_cache(&mut self, title_number: usize, episodes_cache: Vec<EpisodeCache>) {
        let title = self.data.get_mut_title(title_number);

        for ep in episodes_cache {
            title.set_episode_cache(ep)
        }
    }

    /// Return the [`MetaCache`] of the focused element.
    pub fn get_metacache(&self) -> Arc<MetaCache> {
        match self.pointers[self.focused] {
            (pointer, FocusedType::Title(_)) => self.data.get_title_cache(pointer.focused),
            (pointer, FocusedType::Episode(_, _)) => match self
                .data
                .get_title(self.focused - 1)
                .get_episode_cache(pointer.focused)
            {
                Some(metacache) => metacache,
                None => Arc::from(MetaCache::empty()),
            },
        }
    }

    /// Returns the [`FocusedType`] of the focused element.
    pub fn get_type(&self) -> FocusedType {
        match self.pointers[self.focused] {
            (pointer, FocusedType::Title(_)) => FocusedType::Title(pointer.focused),
            (pointer, FocusedType::Episode(_, _)) => {
                FocusedType::Episode(self.focused - 1, pointer.focused)
            }
        }
    }

    /// Goes back to the [`FocusedType::Title`] list.
    pub fn back(&mut self) {
        self.focused = 0;
    }

    /// Returns a [`Column`][iced_native::widget::Column] with a [`Button`][iced_native::widget::Button]
    /// for each element in the focused list.
    pub fn view<'a>(&self) -> Element<'a, FrontendMessage> {
        match self.pointers[self.focused] {
            (pointer, FocusedType::Title(_)) => {
                pointer.view(&self.data.titles_names, |_, _| theme::Text::Default)
            }
            (pointer, FocusedType::Episode(_, _)) => {
                let title = self.data.get_title(self.focused - 1);
                let names = title.episodes_names.as_ref().unwrap();

                pointer.view(names, |focused, id| {
                    let watched = title.get_episode(id).unwrap().watched;

                    match id == focused {
                        true if watched => theme::Text::WatchedFocus,
                        false if watched => theme::Text::Watched,
                        true | false => theme::Text::Default,
                    }
                })
            }
        }
    }
}
