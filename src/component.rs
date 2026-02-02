use core::cell::RefCell;

use crate::{
    canvas::Canvas,
    context::EventContext,
    layout::{Flow, Measure},
    listeners::Listeners,
    state::{State, StateStore},
    transport::Event,
};

pub type BoxedChild = Box<dyn Fn() -> (Measure, Node)>;
pub type BoxedRenderer = Box<dyn Fn(&mut Canvas)>;
pub type Factory<P = ()> = fn(Component, P) -> Node;

/// Types that are used when initializing a [`Node`].
pub trait Props: Clone + 'static {}

impl Props for () {}

/// An UI element with rendering and event listening behavior.
#[derive(Default)]
pub struct Component {
    node: RefCell<Node>,
}

impl Component {
    /// Defines a render function for this component.
    pub fn draw(&self, renderer: impl Fn(&mut Canvas) + 'static) {
        let renderer = Box::new(renderer);
        self.node.borrow_mut().draw_fn = Some(renderer);
    }

    /// Adds a listener for an event.
    ///
    /// Listeners are sequential and single-threaded on the runtime thread. It is important to
    /// never run any blocking tasks to avoid UI jitters.
    pub fn listen<E>(&self, callback: impl FnMut(&mut EventContext<E>) + 'static)
    where
        E: Event,
    {
        self.node.borrow_mut().listeners.push(callback);
    }

    /// Initializes component state.
    ///
    /// State is owned by the component and can be passed down to children.
    pub fn state<T: 'static>(&self, value: T) -> State<T> {
        self.node.borrow_mut().state.insert(value)
    }

    /// Builds the component into a [`Node`].
    pub fn compose<F>(self, builder: F) -> Node
    where
        F: FnOnce(&mut Node),
    {
        let mut node = Node::from(self);
        builder(&mut node);
        node
    }
}

/// A compiled description of an application's UI tree.
#[derive(Default)]
pub struct Node {
    flow: Flow,
    state: StateStore,
    draw_fn: Option<BoxedRenderer>,
    listeners: Listeners,
    children: Vec<BoxedChild>,
}

impl From<Component> for Node {
    fn from(component: Component) -> Self {
        component.node.into_inner()
    }
}

impl Node {
    pub fn set_flow(&mut self, flow: Flow) {
        self.flow = flow;
    }

    pub fn child<P: Props>(&mut self, measure: Measure, factory: Factory<P>, props: P) {
        let child_fn = Box::new(move || (measure, factory(Component::default(), props.clone())));
        self.children.push(child_fn)
    }

    pub(crate) fn children(&self) -> &Vec<BoxedChild> {
        &self.children
    }

    pub(crate) fn listeners_mut(&mut self) -> &mut Listeners {
        &mut self.listeners
    }

    pub(crate) fn renderer(&self) -> Option<&BoxedRenderer> {
        self.draw_fn.as_ref()
    }
}
