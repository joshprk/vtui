<div id="doc-begin" align="center">
  <h1 id="header">
    <pre>[vtui]</pre>
  </h1>
  <p>A simple terminal UI framework.</p>
  <a href="https://crates.io/crates/vtui">
    <img alt="Crates.io Version" src="https://img.shields.io/crates/v/vtui"></img>
  </a>
  </br></br>
  <p align="center">
    <a href="#about">About</a> · <a href="#usage">Usage</a></br>
    <a href="https://docs.rs/vtui/latest/vtui/">Documentation</a></br>
  </p>
</div>

## About

vtui is the framework for sophisticated full-stack terminal applications.

It provides basic building blocks that often come up as boilerplate, from a high-performance runtime
to complex UI features like scrolling.

## Usage

```rs
use std::{rc::Rc, cell::RefCell};

use ratatui::style::Style;
use vtui::{events::*, input::*, prelude::*};

#[component]
fn App(c: &mut Component) -> Inner {
    let clicks = Rc::new(RefCell::new(0));
    let set_clicks = clicks.clone();

    c.draw(move |canvas| {
        canvas.text(0, 0, format!("Clicks: {}", clicks.borrow()), Style::default());
    });

    c.listen::<MouseDown>(move |e| if e.button == MouseButton::Left {
        *set_clicks.borrow_mut() += 1
    });

    c.listen::<KeyPress>(|e| if e.key == KeyCode::Char('q') {
        e.request_shutdown();
    });

    Inner::default()
}

fn main() {
    vtui::launch(App);
}
```
