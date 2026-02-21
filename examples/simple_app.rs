use std::time::Duration;

use ratatui::widgets::Paragraph;
use vtui::{LaunchBuilder, events::*, prelude::*};

#[component]
fn App(c: Component) -> Node {
    c.listen::<KeyPress>(|event| {
        if event.key == KeyCode::Char('q') {
            event.request_shutdown();
        }
    });

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
    c.draw(|canvas| {
        let paragraph = Paragraph::new("Hello world!");
        canvas.widget(paragraph, canvas.rect());
    });

    c.compose(|_| {})
}

fn main() -> vtui::Result {
    LaunchBuilder::new()
        .frametime(Duration::from_millis(16))
        .launch(App)
}
