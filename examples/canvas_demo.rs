use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};
use std::{cell::RefCell, rc::Rc};
use vtui::{events::KeyPress, input::KeyCode, prelude::*};
use vtui_core::canvas::LogicalRect;

#[component]
fn CanvasDemo(c: &mut Component) -> Inner {
    let offset_x = Rc::new(RefCell::new(0));
    let offset_y = Rc::new(RefCell::new(0));
    let offset_x_read = offset_x.clone();
    let offset_y_read = offset_y.clone();

    c.draw(move |canvas| {
        let ox = *offset_x_read.borrow();
        let oy = *offset_y_read.borrow();

        canvas.set_offset(ox, oy);

        let title = format!("Canvas testing (x={}, y={})", ox, oy);
        let block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::all());
        let content = Paragraph::new("This widget content is protected from overflow panics")
            .block(block)
            .wrap(Wrap::default());
        let content_rect = LogicalRect::new(0, 2, 30, 4);

        canvas.text(0, 0, title, Style::default().light_cyan().bold());
        canvas.render_widget(content_rect, content);

        for i in 10..1_000_000 {
            canvas.text(1, i, "Hi there", Style::default());
        }
    });

    c.listen::<KeyPress>(move |event| match event.key {
        KeyCode::Up => {
            let mut oy = offset_y.borrow_mut();
            *oy = oy.saturating_sub(1);
        }
        KeyCode::Down => {
            let mut oy = offset_y.borrow_mut();
            *oy = oy.saturating_add(1);
        }
        KeyCode::Left => {
            let mut ox = offset_x.borrow_mut();
            *ox = ox.saturating_sub(1);
        }
        KeyCode::Right => {
            let mut ox = offset_x.borrow_mut();
            *ox = ox.saturating_add(1);
        }
        KeyCode::Char('q') => std::process::exit(0),
        _ => {}
    });

    Inner::default()
}

fn main() {
    vtui::launch(CanvasDemo);
}
