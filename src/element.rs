use std::{any::TypeId, cell::RefCell};

use ratatui::{Frame, buffer::Buffer, layout::Rect};

use crate::runtime::{Context, Event};

pub trait Element: Clone {
    fn run(self, c: Component);
}

pub enum Pass<'a> {
    Render {
        area: Rect,
        buffer: &'a mut Buffer,
    },
    Update {
        event: &'a dyn Event,
        pending: RefCell<Vec<Box<dyn FnOnce()>>>,
    },
}

impl Pass<'_> {
    pub fn next(&mut self) -> Pass<'_> {
        match self {
            Pass::Render { area, buffer } => Pass::Render {
                area: *area,
                buffer: &mut **buffer,
            },
            Pass::Update { event, .. } => Pass::Update {
                event: *event,
                pending: RefCell::default(),
            },
        }
    }
}

pub struct Component<'a> {
    context: &'a mut Context,
    pass: Pass<'a>,
}

impl<'a> Component<'a> {
    pub fn new_render(context: &'a mut Context, frame: &'a mut Frame) -> Self {
        let pass = Pass::Render {
            area: frame.area(),
            buffer: frame.buffer_mut(),
        };
        Self { context, pass }
    }

    pub fn new_update(context: &'a mut Context, event: &'a dyn Event) -> Self {
        let pass = Pass::Update {
            event,
            pending: RefCell::default(),
        };
        Self { context, pass }
    }
}

impl Component<'_> {
    pub fn compose(self, callback: impl FnOnce(&mut Ui)) {
        let mut ui = Ui(self);
        callback(&mut ui);

        if let Pass::Update { pending, .. } = ui.0.pass {
            for listener_callback in pending.into_inner().into_iter() {
                listener_callback();
            }
        }
    }

    pub fn render(&self, callback: impl Fn()) {
        if let Pass::Render { .. } = self.pass {
            callback();
        }
    }

    pub fn listen<E: Event>(&self, callback: impl FnOnce() + 'static) {
        if let Pass::Update { ref pending, event } = self.pass
            && event.type_id() == TypeId::of::<E>()
        {
            pending.borrow_mut().push(Box::new(callback));
        }
    }
}

pub struct Ui<'a>(Component<'a>);

impl Ui<'_> {
    pub fn element(&mut self, element: impl Element) {
        element.run(Component {
            context: &mut *self.0.context,
            pass: self.0.pass.next(),
        });
    }
}
