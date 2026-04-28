use std::{cell::Cell, rc::Rc, time::Duration};

use ratatui::widgets::Paragraph;
use vtui::{LaunchBuilder, events::*, prelude::*};

#[component]
fn App(c: Component) -> Node {
    let test = Rc::new(Cell::new(0));

    c.listen::<KeyPress>({
        let test = test.clone();
        move |event| {
            test.update(|x| x + 1);
            if event.key == KeyCode::Char('q') {
                event.request_shutdown();
            }
        }
    });

    c.compose(move |ui| {
        let test = test.clone();

        ui.child(Button, ButtonProps {
            text: "Hello world".into(),
            callback: Callback::from(move || {
                test.update(|x| x + 1);
            }),
        });

        ui.child(Button, ButtonProps {
            text: "This is a button".into(),
            callback: Callback::from(|| {

            }),
        });
    })
}

#[derive(Clone, PartialEq)]
struct ButtonProps {
    text: String,
    callback: Callback,
}

impl Props for ButtonProps {}

#[component]
fn Button(c: Component, p: ButtonProps) -> Node {
    c.draw(|canvas| {
        let paragraph = Paragraph::new("Hello world!");
        canvas.widget(paragraph, canvas.area());
    });

    c.compose(|_| {})
}

fn main() -> vtui::Result {
    LaunchBuilder::new()
        .frametime(Duration::from_millis(16))
        .launch(App)
}
