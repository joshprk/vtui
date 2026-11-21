use vtui::element::{Component, Ui};
use vtui_macros::component;

#[component]
fn App(c: Component) {
    c.compose(|ui| {
        ui.element(Button { label: "Click me".into() });
        ui.element(Div {
            children: |ui| {
                ui.element(Button { label: "Save".into() });
                ui.element(Button { label: "Cancel".into() });
            }
        });
    });
}

#[component]
fn Button(c: Component, label: String) {
    c.render(|| {});

    c.compose(|_| {})
}

#[component]
fn Div<F>(c: Component, children: F)
where
    F: FnOnce(&mut Ui) + Clone,
{
    c.compose(children)
}

fn main() {
    vtui::launch(App);
}
