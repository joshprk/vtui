//! vtui is the framework for sophisticated full-stack terminal applications.
//!
//! # High-level features
//!
//! - Build applications using a familiar component-based API
//! - First-class focus management and mouse interaction systems
//! - Predictable Elm-inspired UI architecture
//! - Render to any terminal using [ratatui] widgets
//!
//! # Example
//!
//! ```rust,no_run
//! use ratatui::style::Style;
//! use vtui::{events::*, prelude::*};
//!
//! #[component]
//! fn App(c: Component) {
//!     c.draw(|canvas| {
//!         canvas.text(0, 0, "Hello world!", Style::default());
//!     });
//!
//!     c.listen::<KeyPress>(|event| {
//!         if event.key == KeyCode::Char('q') {
//!             event.request_shutdown();
//!         }
//!     });
//! }
//!
//! fn main() {
//!     vtui::launch(App);
//! }
//! ```
//!
//! # Architecture
//!
//! vtui is simple and predictable. Its core runtime loop has three phases:
//!
//! 1. **Draw** - vtui builds the layout and renders the interface
//! 2. **Update** - components handle events, bubbling up from children to parents
//! 3. **Commit** - the runtime interprets actions enqueued during update
//!
//! The component tree is traversed in a deterministic manner that never surprises the developer.
//!
//! Furthermore, it is highly extensible through its broadcast-only event system. The developer
//! never needs to worry about hidden filtering behavior, as filtering is the responsibility of the
//! listening component. vtui offers batteries-included systems such as a focus system to help the
//! developer route events to components.

extern crate alloc;

pub use crate::{
    canvas::Canvas,
    component::{Ui, UiNode},
    context::EventContext,
    launch::{LaunchBuilder, launch},
};

/// Event types emitted by the framework.
pub mod events;

/// Common imports for building components.
pub mod prelude {
    pub use crate::{
        component::{Component, Factory, Node, Props},
        input::{
            KeyCode, MediaKeyCode, ModifierKeyCode, ModifierKeyDirection, MouseButton,
            MouseScrollDirection,
        },
        layout::{Flow, Inset, LogicalRect, Measure, Placement},
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
