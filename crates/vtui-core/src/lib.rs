use std::{any::TypeId, collections::HashMap};

use ratatui::{buffer::Buffer, layout::Rect};

use crate::{
    events::Event,
    runtime::{Node, Scope},
};

pub mod events;
pub mod runtime;
pub mod source;

type DrawHandler = Box<dyn FnMut(DrawContext)>;
// TODO: listener dispatch performs downcast per listener invocation
// since listeners are already bucketed by TypeId, can be removed by storing Vec<Box<dyn FnMut(&E,
// &Scope>> behind a single erased wrapper
type Listener = Box<dyn FnMut(&dyn Event, &Scope)>;

/// A builder which declares the properties of a component.
///
/// Components are consumed into a [`Runtime`] object which performs the declared behavior at
/// runtime.
#[derive(Default)]
pub struct Component {
    draw: Option<DrawHandler>,
    listeners: HashMap<TypeId, Vec<Listener>>,
}

impl Component {
    /// Registers a listener for a specific [`Event`].
    pub fn listen<E: Event>(&mut self, mut listener: impl FnMut(UpdateContext<E>) + 'static) {
        let type_id = TypeId::of::<E>();
        let wrapped = Box::new(move |event: &dyn Event, scope: &Scope| {
            let event = event.as_any().downcast_ref::<E>().expect("TypeId mismatch");
            listener(UpdateContext { event, scope });
        });

        self.listeners.entry(type_id).or_default().push(wrapped);
    }

    /// Registers a draw handler that specifies how this component is rendered.
    pub fn draw(&mut self, listener: impl FnMut(DrawContext) + 'static) {
        self.draw = Some(Box::new(listener));
    }

    /// Builds the [`Component`] into a [`Runtime`], which can be used at runtime to perform the
    /// declared behavior of this [`Component`].
    pub fn build(self) -> Node {
        Node::from(self)
    }
}

/// A context container given to all component draw handlers.
///
/// This currently only contains the basic [`Rect`] and [`Buffer`] objects, but exists to support
/// forward compatibility for new features.
pub struct DrawContext<'a> {
    pub rect: Rect,
    pub buf: &'a mut Buffer,
}

/// A context container given to all component update listeners.
///
/// It provides access to the triggering [`Event`] and its associated [`Scope`].
pub struct UpdateContext<'a, E> {
    pub event: &'a E,
    pub scope: &'a Scope,
}
