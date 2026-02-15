use core::{any::Any, cell::RefCell, panic::Location};

use crate::arena::Node;

pub type Factory<P = ()> = fn(Component, &P) -> Node;

pub trait Props: Clone + PartialEq + 'static {}

impl Props for () {}

trait AnyProps: Any {
    fn eq(&self, other: &dyn AnyProps) -> bool;
}

impl<P: Props> AnyProps for P {
    fn eq(&self, other: &dyn AnyProps) -> bool {
        match (other as &dyn Any).downcast_ref::<P>() {
            Some(other) => self == other,
            None => false,
        }
    }
}

#[derive(Default)]
pub struct Component {
    node: RefCell<Node>,
}

impl Component {
    pub fn compose<F>(mut self, composer: F) -> Node
    where
        F: Fn(&mut Ui) + 'static,
    {
        let composer = Box::new(composer);
        self.node.get_mut().composer = composer;
        self.node.into_inner()
    }
}

#[derive(Default)]
pub struct Ui {
    descriptors: Vec<Descriptor>,
}

impl Ui {
    #[track_caller]
    pub fn child<P: Props>(&mut self, factory: Factory<P>, props: P) -> &mut Descriptor {
        let descriptor = Descriptor::new(factory, props);
        self.descriptors.push(descriptor);
        self.descriptors.last_mut().unwrap()
    }

    pub(crate) fn descriptors(&self) -> &[Descriptor] {
        &self.descriptors
    }
}

pub struct Descriptor {
    id: Identity,
    build: Box<dyn Fn(&dyn AnyProps) -> Node>,
    props: Box<dyn AnyProps>,
}

impl Descriptor {
    pub fn id(&mut self, key: u32) -> &mut Self {
        self.id.key = Some(key);
        self
    }

    #[track_caller]
    pub(crate) fn new<P: Props>(factory: Factory<P>, props: P) -> Self {
        let id = Identity {
            key: None,
            location: Location::caller(),
        };

        let build = Box::new(move |props: &dyn AnyProps| {
            let props = (props as &dyn Any).downcast_ref::<P>().unwrap();
            factory(Component::default(), props)
        });

        let props = Box::new(props);

        Self {
            id,
            build,
            props,
        }
    }

    pub(crate) fn build(&self) -> Node {
        let props = self.props.as_ref();
        (self.build)(props)
    }

    pub(crate) fn identity(&self) -> Identity {
        self.id
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Identity {
    key: Option<u32>,
    location: &'static Location<'static>,
}

impl Identity {
    #[track_caller]
    pub fn unkeyed() -> Self {
        Self {
            key: None,
            location: Location::caller(),
        }
    }

    #[track_caller]
    pub fn keyed(key: u32) -> Self {
        Self {
            key: None,
            location: Location::caller(),
        }
    }
}
