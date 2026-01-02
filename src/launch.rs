use vtui_core::component::Component;

type FactoryFn = fn(&mut Component);

#[derive(Default)]
pub struct LaunchBuilder {}

impl LaunchBuilder {
    pub fn launch(self, factory: FactoryFn) {}
}

pub fn launch(app: FactoryFn) {
    LaunchBuilder::default().launch(app)
}
