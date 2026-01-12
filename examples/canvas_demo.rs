use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use vtui::{events::KeyPress, input::KeyCode, prelude::*};

#[component]
fn CanvasDemo(c: &mut Component) -> Inner {
    let mut offset = c.state((0, 0));

    c.draw(move |canvas| {
        let (ox, oy) = *offset.read();

        canvas.set_offset(ox, oy);
        let block = Block::new()
            .border_type(BorderType::Plain)
            .borders(Borders::ALL);

        let p = Paragraph::new("Hello world").block(block);

        for i in 0..1_000_000 {
            let rect = LogicalRect::new(0, 3 * i, 20, 3);
            canvas.render_widget(rect, &p);
        }
    });

    c.listen::<KeyPress>(move |event| {
        let mut offset = offset.write();

        if event.key == KeyCode::Down {
            offset.1 += 1;
        } else if event.key == KeyCode::Up && offset.1 > 0 {
            offset.1 -= 1;
        } else if event.key == KeyCode::Char('q') {
            event.request_shutdown();
        }
    });

    Inner::default()
}

fn main() {
    vtui::launch(CanvasDemo)
}
