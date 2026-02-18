extern crate alloc;

pub use crate::launch::launch;

mod launch;

pub mod prelude {
    pub use vtui_macros::component;

    pub use crate::{
        arena::Node,
        component::{Component, Factory, Props},
    };
}

pub(crate) mod arena;
pub(crate) mod canvas;
pub(crate) mod component;
pub(crate) mod context;
pub(crate) mod drivers;
pub(crate) mod errors;
pub(crate) mod handler;
pub(crate) mod layout;
pub(crate) mod listeners;
pub(crate) mod runtime;
pub(crate) mod state;
pub(crate) mod transport;
