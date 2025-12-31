use std::{cell::RefCell, rc::Rc};

use ratatui::style::Style;
use vtui::prelude::{Component, component};
use vtui_core::Tick;

#[component]
fn App(c: &mut Component) {
    let text = Rc::new(RefCell::new(1));
    let text_2 = text.clone();

    c.draw(move |ctx| {
        let displayed = format!("Counter: {}", text.borrow());
        ctx.buf.set_string(1, 1, displayed, Style::default());
    });

    c.listen::<Tick>(move |ctx| {
        *text_2.borrow_mut() += 1;
    });
}

fn main() -> anyhow::Result<()> {
    vtui::launch(App)
}
