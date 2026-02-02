use ratatui::style::Style;
use vtui::{events::*, prelude::*};

#[allow(non_snake_case)]
fn App(c: Component, p: ()) -> Node {
    let mut clicks = c.state(0);

    c.draw(move |canvas| {
        let text = format!("Clicks: {}", clicks.read());
        canvas.text(0, 0, text, Style::default());
    });

    c.listen::<MouseDown>(move |event| {
        if event.button == MouseButton::Left {
            clicks.set(|n| n + 1);
        }
    });

    c.listen::<KeyPress>(|event| {
        if event.key == KeyCode::Char('q') {
            event.request_shutdown();
        }
    });

    c.compose(|node| {
        node.child(Measure::Exact(10), App, ());
        node.child(Measure::Exact(10), App, ());
        node.set_flow(Flow::Vertical);
    })
}

fn main() {
    vtui::launch(App)
}
