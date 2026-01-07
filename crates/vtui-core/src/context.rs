use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

pub struct DrawContext<'a> {
    buf: &'a mut Buffer,
    rect: Rect,
}

impl<'a> DrawContext<'a> {
    pub fn new(rect: Rect, buf: &'a mut Buffer) -> Self {
        Self { rect, buf }
    }
}

impl DrawContext<'_> {
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.buf
    }

    pub fn set_string(&mut self, x: u16, y: u16, string: String, style: Style) {
        self.buf
            .set_string(x + self.rect.x, y + self.rect.y, string, style)
    }

    pub fn render_widget(&mut self, widget: impl Widget) {
        widget.render(self.rect, self.buf)
    }
}

pub struct UpdateContext<'a, E> {
    pub event: &'a E,
}
