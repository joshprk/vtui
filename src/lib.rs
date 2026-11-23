pub use crate::launch::launch;

pub mod events;
pub mod input;

pub mod prelude {
    pub use crate::element::{Component, Element};
    pub use crate::input::{
        KeyCode, MediaKeyCode, ModifierKeyCode, ModifierKeyDirection, MouseButton,
        MouseScrollDirection,
    };
    pub use vtui_macros::component;
}

pub(crate) mod drivers;
pub(crate) mod element;
pub(crate) mod launch;
pub(crate) mod layout;
pub(crate) mod runtime;
