use ratatui::style::Style;
use vtui::{
    events::{KeyPress, MouseDown},
    input::{KeyCode, MouseButton},
    prelude::*,
};

#[component]
fn App(c: &mut Component) -> Inner {
    let mut counter = c.state(0);

    c.draw(move |canvas| {
        let text = format!("Counter: {}", counter.read());
        canvas.text(0, 0, text, Style::default());
    });

    c.listen::<MouseDown>(move |event| {
        if event.button == MouseButton::Left {
            *counter.write() += 1;
        }
    });

    c.listen::<KeyPress>(|event| {
        if let KeyCode::Char('q') = event.key {
            event.request_shutdown();
        }
    });

    Inner::default()
}

fn main() {
    vtui::launch(App);
}
