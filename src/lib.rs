extern crate alloc;

pub mod prelude {
    pub use vtui_macros::component;
}

pub(crate) mod arena;
pub(crate) mod component;
pub(crate) mod listeners;
pub(crate) mod state;
pub(crate) mod transport;
