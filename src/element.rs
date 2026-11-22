use std::{any::Any, cell::RefCell, ops::Deref};

use ratatui::{buffer::Buffer, layout::Rect};

use crate::runtime::{Context, Event};

pub trait Element: Clone {
    fn run(self, c: Component);

    fn render(self, context: &Context, buffer: &mut Buffer, rect: Rect) {
        let ui = Ui::Render {
            context,
            buffer,
            rect,
        };
        self.run(Component {
            ui: RefCell::new(ui),
        })
    }

    fn update(self, context: &mut Context, event: &dyn Event) {
        let ui = Ui::Update {
            context,
            event,
            pending: Vec::default(),
        };
        self.run(Component {
            ui: RefCell::new(ui),
        })
    }
}

pub struct Component<'a> {
    ui: RefCell<Ui<'a>>,
}

impl Component<'_> {
    pub fn compose(self, callback: impl FnOnce(&mut Ui)) {
        callback(&mut self.ui.into_inner());
    }

    pub fn render(&self, callback: impl Fn(&mut Canvas) + 'static) {
        if let Ui::Render {
            context,
            buffer,
            rect,
        } = &mut *self.ui.borrow_mut()
        {
            callback(&mut Canvas {
                context: *context,
                buffer,
                rect: *rect,
            });
        }
    }

    pub fn listen<E: Event>(&self, callback: impl FnOnce(&mut EventData<E>) + 'static) {
        if let Ui::Update { event, pending, .. } = &mut *self.ui.borrow_mut()
            && let Some(event) = (*event as &dyn Any).downcast_ref::<E>()
        {
            let callback = Box::new(move |ui: &mut Ui| {
                let Ui::Update { context, .. } = ui else {
                    unreachable!("deferred callback must be an update");
                };
                callback(&mut EventData { context, event });
            });
            pending.push(callback);
        }
    }
}

pub type Deferred<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

pub enum Ui<'a> {
    Render {
        context: &'a Context,
        buffer: &'a mut Buffer,
        rect: Rect,
    },
    Update {
        context: &'a mut Context,
        event: &'a dyn Event,
        pending: Vec<Deferred<'a>>,
    },
}

impl Ui<'_> {
    pub fn element(&mut self, element: impl Element) {
        match self {
            Self::Render {
                context,
                buffer,
                rect,
            } => element.render(context, buffer, *rect),
            Self::Update { context, event, .. } => element.update(context, *event),
        }
    }
}

impl Drop for Ui<'_> {
    fn drop(&mut self) {
        if !std::thread::panicking() && let Ui::Update { pending, .. } = self {
            for listener in std::mem::take(pending) {
                listener(self);
            }
        }
    }
}

pub struct Canvas<'a> {
    context: &'a Context,
    buffer: &'a mut Buffer,
    rect: Rect,
}

impl Canvas<'_> {
    pub fn buffer(&self) -> &Buffer {
        self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.buffer
    }

    pub fn area(&self) -> Rect {
        self.rect
    }
}

pub struct EventData<'a, E: Event> {
    context: &'a mut Context,
    event: &'a E,
}

impl<E: Event> EventData<'_, E> {
    pub fn request_shutdown(&mut self) {
        self.context.shutdown_requested = true;
    }
}

impl<'a, E: Event> Deref for EventData<'a, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.event
    }
}
