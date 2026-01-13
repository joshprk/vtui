use ratatui::style::Style;
use vtui::{events::*, input::*, prelude::*};

#[component]
fn App(c: &mut Component) -> Inner {
    let mut clicks = c.state(0);

    c.draw(move |canvas| {
        let text = format!("Clicks: {}", *clicks.read());
        canvas.text(0, 0, text, Style::default());
    });

    c.listen::<MouseDown>(move |event| {
        if event.button == MouseButton::Left {
            *clicks.write() += 1;
        }
    });

    c.listen::<KeyPress>(|event| {
        if event.key == KeyCode::Char('q') {
            event.request_shutdown();
        }
    });

    vtui! {}
}

fn main() {
    vtui::launch(App);
}
