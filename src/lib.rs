pub use vtui_core::events;
pub use vtui_core::runtime;

pub use crate::launch::launch;

mod launch;

pub mod prelude {
    pub use vtui_core::{component::Component, driver::Driver};
    pub use vtui_macros::component;
}

#[cfg(not(any(feature = "crossterm")))]
compile_error!("vtui requires a terminal driver. Enable one of: crossterm");
