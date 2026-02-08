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

vtui is a framework for modern terminal applications.

It provides basic building blocks that often come up as boilerplate, from a high-performance runtime
to complex UI features like scrolling.

An overview of the features is provided in [this blog post](https://joshprk.me/writing/02-vtui/).

## Usage

```rs
use ratatui::style::Style;
use vtui::{events::*, input::*, prelude::*};

#[component]
fn App(c: Component) -> Node {
    let mut clicks = c.state(0);

    c.draw(move |canvas| {
        let text = format!("Clicks: {}", *clicks.read());
        canvas.text(0, 0, text, Style::default());
    });

    c.listen::<MouseDown>(move |event| {
        if event.button == MouseButton::Left {
            *clicks.write() += 1;
        }
    });

    c.listen::<KeyPress>(|event| {
        if event.key == KeyCode::Char('q') {
            event.request_shutdown();
        }
    });

    c.compose(|_| {})
}

fn main() {
    vtui::launch(App);
}
```
