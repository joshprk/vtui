use crate::runtime::{Context, Event};

pub trait Element: Clone {
    fn run(self, c: Component);
}

pub struct Component<'a> {
    context: &'a mut Context,
}

impl<'a> Component<'a> {
    pub fn new(context: &'a mut Context) -> Self {
        Self { context }
    }
}

impl Component<'_> {
    pub fn compose(self, callback: impl FnOnce(&mut Ui)) {
        callback(&mut Ui);
    }

    pub fn render(&self, callback: impl Fn()) {}
    pub fn listen<E: Event>(&self, callback: impl FnOnce()) {}
}

pub struct Ui;

impl Ui {
    pub fn element(&mut self, element: impl Element) {}
}
