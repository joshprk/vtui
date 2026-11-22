use vtui::{events::Tick, prelude::*};

#[component]
fn App(c: Component) {
    c.compose(|ui| {
        ui.element(Button {
            label: "Click me".into(),
        });
    });
}

#[component]
fn Button(c: Component, label: String) {
    c.listen::<Tick>(|evt| {
        evt.request_shutdown();
    });

    c.compose(|_| {})
}

fn main() {
    vtui::launch(App);
}
