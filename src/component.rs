use core::cell::RefCell;

use crate::{
    canvas::Canvas, context::EventContext, layout::Measure, listeners::Listeners, transport::Event,
};

pub type BoxedRenderer = Box<dyn Fn(&mut Canvas)>;
pub type Factory<P = ()> = fn(Component, P) -> Node;

pub trait Props: Clone {}

impl Props for () {}

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
    pub(crate) draw_fn: Option<BoxedRenderer>,
    pub(crate) listeners: Listeners,
    pub(crate) children: Vec<(Measure, Node)>,
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
    pub fn new(component: Component) -> Self {
        Node::from(component)
    }

    pub fn child<P: Props>(mut self, measure: Measure, factory: Factory<P>, props: P) -> Self {
        let component = Component::default();
        let node = factory(component, props);
        self.children.push((measure, node));
        self
    }
}
