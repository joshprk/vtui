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

    pub fn render_widget(&mut self, rect: Rect, widget: impl Widget) {
        let temp_rect = Rect::new(0, 0, rect.width, rect.height);
        let mut temp_buf = Buffer::empty(temp_rect);

        widget.render(temp_rect, &mut temp_buf);

        let area = self.buf.area;

        let src_stride = rect.width as usize;
        let dst_stride = area.width as usize;

        for y in 0..rect.height {
            let src_row = y as usize * src_stride;

            let dst_y = rect.y + self.offset_y + y;

            if dst_y >= area.height {
                break;
            }

            let dst_x = rect.x + self.offset_x;

            if dst_x >= area.width {
                break;
            }

            let dst_row = dst_y as usize * dst_stride + dst_x as usize;

            let len = (rect.width as usize).min(area.width as usize - dst_x as usize);

            let src = &temp_buf.content[src_row..src_row + len];
            let dst = &mut self.buf.content[dst_row..dst_row + len];

            dst.clone_from_slice(src);
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
