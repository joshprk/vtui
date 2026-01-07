use ratatui::style::Style;
use std::{cell::RefCell, rc::Rc};
use vtui::{
    events::{KeyPress, MouseDown},
    input::{KeyCode, MouseButton},
    prelude::*,
};

#[derive(Default)]
pub struct Element {}

#[component]
fn App(c: &mut Component) -> Inner {
    let counter = Rc::new(RefCell::new(0));
    let set_counter = counter.clone();

    c.draw(move |canvas| {
        let text = format!("Counter: {}", counter.borrow());
        canvas.text(0, 0, text, Style::default());
    });

    c.listen::<MouseDown>(move |event| {
        if event.button == MouseButton::Left {
            *set_counter.borrow_mut() += 1;
        }
    });

    c.listen::<KeyPress>(|event| {
        if let KeyCode::Char('q') = event.key {
            std::process::exit(0);
        }
    });

    Inner::default()
}

fn main() {
    vtui::launch(App);
}
