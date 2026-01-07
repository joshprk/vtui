use ratatui::style::Style;
use std::{cell::RefCell, rc::Rc};
use vtui::{
    events::{KeyRelease, MouseDown},
    input::{KeyCode, MouseButton},
    prelude::*,
};

#[component]
fn App(c: &mut Component) {
    let counter = Rc::new(RefCell::new(0));
    let set_counter = counter.clone();

    c.draw(move |canvas| {
        let text = format!("Counter: {}", counter.borrow());
        canvas.text(0, 0, text, Style::default());
    });

    c.listen::<MouseDown>(move |event| {
        if event.button != MouseButton::Left {
            *set_counter.borrow_mut() += 1;
        }
    });

    c.listen::<KeyRelease>(|event| {
        if let KeyCode::Char('q') = event.key {
            event.request_shutdown();
        }
    });
}

fn main() {
    vtui::launch(App);
}
