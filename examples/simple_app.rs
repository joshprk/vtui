use ratatui::style::Style;
use vtui::prelude::{component, Component};

#[component]
fn App(c: &mut Component) {
    c.draw(|ctx| {
        ctx.buf.set_string(1, 1, "Hello world!", Style::default());
    });
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    vtui::launch(App).await
}
