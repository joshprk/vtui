use ratatui::style::Style;
use vtui::prelude::*;

#[component]
fn App(c: &mut Component) {
    c.draw(move |ctx| {
        let style = Style::default();
        let text = "Hello world";
        ctx.buf.set_string(1, 1, text, style);
    });
}

fn main() {
    vtui::launch(App)
}
