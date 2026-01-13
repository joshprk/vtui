use std::{cell::RefCell, rc::Rc};

use crate::{
    canvas::Canvas,
    channels::{ChannelReceiver, ChannelSender, create_blocking_channel},
    context::{Context, EventContext},
    events::{Event, Message},
    listeners::{DrawListener, ListenerStore},
    state::{State, StateOwner},
};

pub type FactoryFn = fn(&mut Component) -> Inner;

#[derive(Default)]
pub struct Component {
    draw_listener: Option<DrawListener>,
    listeners: Rc<RefCell<ListenerStore>>,
    state_arena: StateOwner,
    inner: Inner,
}

impl Component {
    fn set_inner(&mut self, inner: Inner) {
        self.inner = inner;
    }
}

impl Component {
    pub(crate) fn with_factory(factory: FactoryFn) -> Self {
        let mut component = Component::default();
        let inner = factory(&mut component);
        component.set_inner(inner);
        component
    }

    pub fn draw(&mut self, listener: impl Fn(&mut Canvas) + 'static) {
        self.draw_listener = Some(Box::new(listener));
    }

    pub fn listen<E: Event>(&mut self, listener: impl FnMut(&mut EventContext<E>) + 'static) {
        self.listeners.borrow_mut().push(Box::new(listener))
    }

    pub fn state<T: 'static>(&self, value: T) -> State<T> {
        self.state_arena.insert(value)
    }

    pub fn channel_blocking<T: Send + 'static>(
        &mut self,
    ) -> (ChannelSender<T>, ChannelReceiver<T>) {
        create_blocking_channel(self.listeners.clone())
    }
}

impl Component {
    pub(crate) fn render(&self, ctx: &mut Canvas) {
        if let Some(ref draw_listener) = self.draw_listener {
            draw_listener(ctx)
        }
    }

    pub(crate) fn update(&mut self, msg: &Message, ctx: &mut Context) {
        if let Some(listeners) = self.listeners.borrow_mut().get_mut(msg) {
            listeners.dispatch(msg, ctx);
        }
    }
}

#[derive(Default)]
pub struct Inner {}
