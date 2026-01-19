pub use vtui_core::canvas;
pub use vtui_core::events;
pub use vtui_core::input;
pub use vtui_core::layout;
pub use vtui_core::runtime;
pub use vtui_core::transport;

pub use crate::launch::launch;

pub mod drivers;

mod launch;

pub mod prelude {
    pub use vtui_core::{
        canvas::LogicalRect,
        component::{Component, Node, Props},
        driver::Driver,
        layout::{Flow, Measure},
        state::State,
    };
    pub use vtui_macros::{component, vtui};
}

#[cfg(not(any(feature = "crossterm")))]
compile_error!("vtui requires a terminal driver. Enable one of: crossterm");
