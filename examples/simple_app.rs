use vtui::prelude::*;

#[component]
fn App(c: Component) -> Node {
    c.compose(|ui| {
        ui.child(Button, ButtonProps {});
        ui.child(Button, ButtonProps {});
    })
}

#[derive(Clone, PartialEq)]
struct ButtonProps {}

impl Props for ButtonProps {}

#[component]
fn Button(c: Component, p: ButtonProps) -> Node {
    c.draw(|canvas| {});

    c.compose(|_| {})
}

fn main() {}
