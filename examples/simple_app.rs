use std::{cell::RefCell, rc::Rc};

use ratatui::style::Style;
use vtui::prelude::{Component, component};
use vtui_core::Tick;

#[component]
fn App(c: &mut Component) {
    let text = Rc::new(RefCell::new(String::from("Hello world")));
    let text_2 = text.clone();

    c.draw(move |ctx| {
        let text = text.clone();
        ctx.buf
            .set_string(1, 1, text.borrow().as_str(), Style::default());
    });

    c.listen::<Tick>(move |evt| {
        *text_2.borrow_mut() = "Goodbye world".into();
    });
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    vtui::launch(App).await
}
