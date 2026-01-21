pub use crate::launch::launch;

pub mod canvas;
pub mod component;
pub mod context;
pub mod drivers;
pub mod error;
pub mod events;
pub mod input;
pub mod layout;
pub mod runtime;
pub mod state;
pub mod transport;

pub mod prelude {
    pub use crate::{
        canvas::LogicalRect,
        component::{Component, Node, Props},
        drivers::Driver,
        layout::{Flow, Measure},
        state::State,
    };
    pub use vtui_macros::{component, vtui};
}

pub(crate) mod arena;
pub(crate) mod listeners;

mod launch;

#[cfg(not(any(feature = "crossterm")))]
compile_error!("vtui requires a terminal driver. Enable one of: crossterm");
