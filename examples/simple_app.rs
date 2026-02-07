use ratatui::widgets::Paragraph;
use vtui::{events::*, prelude::*};

#[component]
fn App(c: Component) -> Node {
    c.listen::<KeyPress>(|event| {
        if event.key == KeyCode::Char('q') {
            event.request_shutdown();
        }
    });

    c.compose(|node| {
        node.child(Counter, ());
        node.child(Counter, ());
    })
}

#[component]
fn Counter(c: Component) -> Node {
    let mut clicks = c.state(0);

    c.draw(move |canvas| {
        let paragraph = Paragraph::new(format!("Clicks: {}", clicks.read()))
            .centered();

        canvas.widget(canvas.area(), paragraph);
    });

    c.listen::<MouseDown>(move |event| {
        if event.is_mouse_hit() {
            clicks.set(|c| *c += 1);
        }
    });

    c.compose(|_| {})
}

fn main() {
    vtui::launch(App)
}
