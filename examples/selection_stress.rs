use ratatui::{
    style::{Color, Style},
    widgets::Block,
};
use vtui::{TextJoin, TextKey, TextOptions, events::*, prelude::*};

const DOCUMENT_LINES: i32 = 120;

#[component]
fn App(c: Component) -> Node {
    c.listen::<KeyPress>(|event| {
        if matches!(event.key, KeyCode::Char('q') | KeyCode::Esc) {
            event.request_shutdown();
        }
    });

    c.compose(|ui| {
        ui.child(DocumentViewport, ()).measure(Measure::Exact(20));
        ui.child(SelectionStatus, ()).measure(Measure::Exact(5));
        ui.child(Instructions, ());
    })
}

#[component]
fn DocumentViewport(c: Component) -> Node {
    c.set_clipped(true);
    c.set_selection_root(true);
    c.set_padding(Inset::all(1));

    c.draw(|canvas| {
        canvas.widget(
            canvas.area(),
            Block::bordered()
                .title(" selectable document — drag text, then scroll ")
                .border_style(Style::default().fg(Color::Cyan)),
        );
    });

    c.compose(|ui| {
        ui.child(LongDocument, ())
            .measure(Measure::Exact(DOCUMENT_LINES * 2 + 4));
    })
}

#[component]
fn LongDocument(c: Component) -> Node {
    let mut scroll = c.state(0i32);

    c.draw(move |canvas| {
        for line in 0..DOCUMENT_LINES {
            let indent = if line % 3 == 0 { 6 } else { 1 };
            let text = match line % 5 {
                0 => format!("Line {line:03}: wide 好 · emoji 🦀 · e\u{301} combining"),
                1 => format!("Line {line:03}: semantic spaces are selectable"),
                2 => format!("Line {line:03}: this run is separated by layout whitespace"),
                3 => format!("Line {line:03}: scroll me offscreen without losing selection"),
                _ => format!("Line {line:03}: styled text remains semantic text"),
            };
            let style = if line % 2 == 0 {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Yellow)
            };

            canvas.text_with(
                indent,
                line * 2,
                text,
                style,
                TextOptions {
                    key: Some(TextKey::new(line as u64)),
                    join: TextJoin::Line,
                    selectable: true,
                },
            );
        }

        canvas.text_with(
            1,
            DOCUMENT_LINES * 2,
            "decorative: this sentence must not be selectable",
            Style::default().fg(Color::DarkGray),
            TextOptions {
                selectable: false,
                ..TextOptions::default()
            },
        );
    });

    c.listen::<KeyPress>(move |event| {
        let next = match event.key {
            KeyCode::Up | KeyCode::Char('k') => (*scroll.read() - 1).max(0),
            KeyCode::Down | KeyCode::Char('j') => (*scroll.read() + 1).min(DOCUMENT_LINES * 2),
            KeyCode::PageUp => (*scroll.read() - 12).max(0),
            KeyCode::PageDown => (*scroll.read() + 12).min(DOCUMENT_LINES * 2),
            KeyCode::Home => 0,
            KeyCode::End => DOCUMENT_LINES * 2,
            _ => return,
        };
        scroll.set(|value| *value = next);
        event.set_offset(0, next);
    });

    c.compose(|ui| {
        ui.child(IsolatedSelectionRoot, ())
            .measure(Measure::Fixed(45, 30, 28, 5));
    })
}

#[component]
fn IsolatedSelectionRoot(c: Component) -> Node {
    c.set_selection_root(true);
    c.set_clipped(true);

    c.draw(|canvas| {
        canvas.widget(
            canvas.area(),
            Block::bordered()
                .title(" isolated root ")
                .border_style(Style::default().fg(Color::Magenta)),
        );
        canvas.text(
            2,
            1,
            "Selection cannot cross",
            Style::default().fg(Color::Magenta),
        );
        canvas.text(
            2,
            2,
            "this root boundary.",
            Style::default().fg(Color::Magenta),
        );
    });

    c.compose(|_| {})
}

#[component]
fn SelectionStatus(c: Component) -> Node {
    c.set_selectable(false);
    c.draw(|canvas| {
        let selected = canvas
            .selected_text()
            .filter(|text| !text.is_empty())
            .unwrap_or_else(|| "<nothing selected>".to_owned())
            .replace('\n', " ↵ ");
        let preview: String = selected.chars().take(140).collect();
        canvas.widget(
            canvas.area(),
            Block::bordered().title(" semantic copy preview "),
        );
        canvas.text(2, 1, preview, Style::default().fg(Color::Green));
        canvas.text(
            2,
            2,
            "The preview includes selected offscreen runs, never margin cells.",
            Style::default().fg(Color::DarkGray),
        );
    });
    c.compose(|_| {})
}

#[component]
fn Instructions(c: Component) -> Node {
    c.set_selectable(false);
    c.draw(|canvas| {
        canvas.text(
            0,
            0,
            "Drag with left mouse. ↑/↓, j/k, PgUp/PgDn, Home/End scroll. q/Esc quits.",
            Style::default().fg(Color::DarkGray),
        );
    });
    c.compose(|_| {})
}

fn main() {
    vtui::launch(App);
}
