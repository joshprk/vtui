use ratatui::style::Style;
use vtui::{events::*, prelude::*};

#[component]
fn App(c: Component) -> Node {
    c.listen::<KeyPress>(|event| {
        if event.key == KeyCode::Char('q') {
            event.request_shutdown();
        }
    });

    c.compose(|node| {
        node.child(Measure::Viewport(0.5), Test, ());
        node.child(Measure::Viewport(0.5), Test, ());
        node.set_flow(Flow::Vertical);
    })
}

#[component]
fn Test(c: Component) -> Node {
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

    c.compose(|_| {})
}

fn main() {
    vtui::launch(App)
}
