use std::cell::RefCell;

use crate::{canvas::Canvas, context::EventContext, listeners::Listeners, transport::Event};

pub type BoxedRenderer = Box<dyn Fn(&mut Canvas)>;
pub type Factory<P = ()> = fn(Component, P) -> Node;

#[derive(Default)]
pub struct Component {
    node: RefCell<Node>,
}

impl Component {
    pub fn draw(&self, renderer: impl Fn(&mut Canvas) + 'static) {
        let renderer = Box::new(renderer);
        self.node.borrow_mut().draw_fn = Some(renderer);
    }

    pub fn listen<E>(&self, callback: impl FnMut(&mut EventContext<E>) + 'static)
    where
        E: Event,
    {
        self.node.borrow_mut().listeners.push(callback);
    }
}

#[derive(Default)]
pub struct Node {
    draw_fn: Option<BoxedRenderer>,
    listeners: Listeners,
}

impl From<Component> for Node {
    fn from(component: Component) -> Self {
        component.node.into_inner()
    }
}

impl From<Factory> for Node {
    fn from(factory: Factory) -> Self {
        factory(Component::default(), ())
    }
}

impl Node {
    pub fn listeners(&mut self) -> &Listeners {
        &self.listeners
    }

    pub fn listeners_mut(&mut self) -> &mut Listeners {
        &mut self.listeners
    }

    pub fn render(&self, canvas: &mut Canvas) {
        if let Some(renderer) = &self.draw_fn {
            renderer(canvas);
        }
    }
}
