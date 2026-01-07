use std::{cell::RefCell, rc::Rc};

use ratatui::style::Style;
use vtui::{events::MouseDown, input::MouseButton, prelude::*};

#[component]
fn App(c: &mut Component) {
    let counter = Rc::new(RefCell::new(0));
    let set_counter = counter.clone();

    c.draw(move |ctx| {
        let style = Style::default();
        let text = format!("Counter: {}", counter.borrow());
        ctx.buffer_mut().set_string(1, 1, text, style);
    });

    c.listen::<MouseDown>(move |ctx| {
        if ctx.event.button != MouseButton::Left {
            return;
        }

        *set_counter.borrow_mut() += 1;
    });
}

fn main() {
    vtui::launch(App);
}
