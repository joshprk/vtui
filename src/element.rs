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

#[test]
fn test_main() {
    use crate::{self as vtui, events::Tick};

    #[derive(Clone)]
    struct App;
    impl Element for App {
        fn run(self, c: Component) {
            let Self {} = self;
            c.listen::<Tick>(|| {});
            c.render(|| {});

            c.compose(|ui| {
                let label = String::from("Button");
                ui.element(Button { label });
            })
        }
    }

    #[derive(Clone)]
    struct Button { label: String }
    impl Element for Button {
        fn run(self, c: Component) {
            let Self { label } = self;
            c.compose(|_| {})
        }
    }

    vtui::launch(App)
}
