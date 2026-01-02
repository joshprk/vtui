pub use vtui_core::events;
pub use vtui_core::runtime;

pub mod prelude {
    pub use vtui_core::{driver::Driver, component::Component};
    pub use vtui_macros::component;
}

#[cfg(not(any(
    feature = "crossterm",
)))]
compile_error!(
    "vtui requires a terminal driver. Enable one of: crossterm"
);
