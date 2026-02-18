use crate::{component::{Component, Factory}, runtime::Runtime, transport::MessageBus};

pub fn launch(app: Factory) {
    let root = app(Component::default(), ());
    let bus = MessageBus::new();
    let runtime = Runtime::new(root);
}
