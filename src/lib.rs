extern crate alloc;

pub use crate::launch::{LaunchBuilder, launch};

pub mod events;

pub mod prelude {
    pub use crate::{
        component::{Component, Node},
        input::{KeyCode, MouseButton},
        layout::{LogicalRect, Measure},
    };
}

pub(crate) mod arena;
pub(crate) mod canvas;
pub(crate) mod component;
pub(crate) mod context;
pub(crate) mod drivers;
pub(crate) mod errors;
pub(crate) mod input;
pub(crate) mod layout;
pub(crate) mod listeners;
pub(crate) mod runtime;
pub(crate) mod transport;

mod launch;
