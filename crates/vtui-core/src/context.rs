use std::ops::Deref;

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::events::Event;

pub struct Canvas<'a> {
    buf: &'a mut Buffer,
    rect: Rect,
    offset_x: u16,
    offset_y: u16,
}

impl<'a> Canvas<'a> {
    pub fn new(rect: Rect, buf: &'a mut Buffer) -> Self {
        Self {
            rect,
            buf,
            offset_x: 0,
            offset_y: 0,
        }
    }

    pub fn set_offset(&mut self, offset_x: u16, offset_y: u16) {
        self.offset_x = offset_x;
        self.offset_y = offset_y;
    }

    pub fn offset(&self) -> (u16, u16) {
        (self.offset_x, self.offset_y)
    }

    fn is_visible(&self, x: u16, y: u16) -> bool {
        x >= self.rect.x
            && x < self.rect.x.saturating_add(self.rect.width)
            && y >= self.rect.y
            && y < self.rect.y.saturating_add(self.rect.height)
    }
}

impl Canvas<'_> {
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.buf
    }

    pub fn text(&mut self, x: u16, y: u16, text: impl Into<String>, style: Style) {
        let abs_x = x.saturating_add(self.offset_x).saturating_add(self.rect.x);
        let abs_y = y.saturating_add(self.offset_y).saturating_add(self.rect.y);

        if !self.is_visible(abs_x, abs_y) {
            return;
        }

        self.buf.set_string(abs_x, abs_y, text.into(), style)
    }

    pub fn render_widget(&mut self, mut rect: Rect, widget: impl Widget) {
        rect.x = rect.x.saturating_add(self.offset_x);
        rect.y = rect.y.saturating_add(self.offset_y);

        let rect = rect.intersection(self.buf.area);

        if rect.area() > 0 {
            widget.render(rect, self.buf);
        }
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
