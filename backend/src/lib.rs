pub mod api;
pub mod backend;
pub mod config;

use std::{fmt::Debug, path::PathBuf};

pub trait Meta {
    fn thumbnail(&self) -> PathBuf;
    fn description(&self) -> String;
}

impl Debug for dyn Meta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Meta")
    }
}
