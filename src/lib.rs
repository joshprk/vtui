extern crate alloc;

pub use crate::{
    context::EventContext,
    launch::{LaunchBuilder, launch},
};

pub mod events;

pub mod prelude {
    pub use crate::{
        component::{Component, Factory, Node, Props},
        input::{KeyCode, MouseButton},
        layout::{Flow, LogicalRect, Measure},
        state::State,
    };
    pub use vtui_macros::component;
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
pub(crate) mod state;
pub(crate) mod transport;

mod launch;
