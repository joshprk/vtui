use std::ops::Deref;

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::events::Event;

pub struct Canvas<'a> {
    buf: &'a mut Buffer,
    rect: Rect,
}

impl<'a> Canvas<'a> {
    pub fn new(rect: Rect, buf: &'a mut Buffer) -> Self {
        Self { rect, buf }
    }
}

impl Canvas<'_> {
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.buf
    }

    pub fn text(&mut self, x: u16, y: u16, text: impl Into<String>, style: Style) {
        let x = x + self.rect.x;
        let y = y + self.rect.y;
        let text = text.into();
        self.buf.set_string(x, y, text, style)
    }

    pub fn render_widget(&mut self, widget: impl Widget) {
        widget.render(self.rect, self.buf)
    }
}

pub struct EventContext<'a, E: Event> {
    event: &'a E,
}

impl<'a, E: Event> Deref for EventContext<'a, E> {
    type Target = E;

    fn deref(&self) -> &'a Self::Target {
        self.event
    }
}

impl<'a, E: Event> EventContext<'a, E> {
    pub fn new(event: &'a E) -> Self {
        Self { event }
    }
}
