use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{arena::ArenaNode, layout::LogicalRect};

/// A drawing surface scoped to a rectangular region of the terminal buffer.
pub struct Canvas<'a> {
    buf: &'a mut Buffer,
    clipped: bool,
    rect: LogicalRect,
    offset_x: i32,
    offset_y: i32,
}

impl<'a> Canvas<'a> {
    /// Creates a new canvas with the given region.
    pub(crate) fn new(node: &ArenaNode, buf: &'a mut Buffer) -> Self {
        let rect = node.area();
        let attributes = node.attributes();
        let clipped = attributes.clipped;
        let (offset_x, offset_y) = attributes.offset;

        Self {
            buf,
            clipped,
            rect,
            offset_x,
            offset_y,
        }
    }
}

impl Canvas<'_> {
    /// Returns the underlying buffer.
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.buf
    }

    /// Returns the underlying rectangular region.
    pub fn rect(&self) -> LogicalRect {
        self.rect
    }

    /// Returns a [`LogicalRect`] with the same width and height, but with an origin of `(0, 0)`.
    pub fn area(&self) -> LogicalRect {
        LogicalRect::new(0, 0, self.rect.width, self.rect.height)
    }

    /// Draws text content at a given position.
    ///
    /// This function is panic-free and text is automatically clipped.
    pub fn text<T, S>(&mut self, x: i32, y: i32, text: T, style: S)
    where
        T: AsRef<str>,
        S: Into<Style>,
    {
        self.set_stringn(x, y, text, usize::MAX, style);
    }

    /// Draws a `ratatui` widget at the given region.
    ///
    /// This function is panic-free and text is automatically clipped.
    pub fn widget(&mut self, rect: impl Into<LogicalRect>, widget: impl Widget) {
        self.render_widget(rect.into(), widget);
    }

    /// An internal helper which draws text content at a given position.
    fn set_stringn<T, S>(&mut self, x: i32, y: i32, text: T, max_width: usize, style: S)
    where
        T: AsRef<str>,
        S: Into<Style>,
    {
        let buffer_area = {
            let inner = LogicalRect::from(self.buf.area);

            if self.clipped() {
                inner.intersection(self.rect)
            } else {
                inner
            }
        };

        let buf_x = self.get_buf_column(x);
        let buf_y = self.get_buf_row(y);
        let max_width = max_width.try_into().unwrap_or(i32::MAX);

        if buf_y < buffer_area.top() || buf_y >= buffer_area.bottom() {
            return;
        }

        let start = buf_x.max(buffer_area.left());

        if start >= buffer_area.right() {
            return;
        }

        let mut remaining = (buffer_area.right() - start).clamp(0, max_width) as u16;
        let mut skip_width = (start - buf_x).max(0) as u16;
        let mut cursor = start as u16;
        let row = buf_y as u16;

        let style = style.into();

        for g in UnicodeSegmentation::graphemes(text.as_ref(), true) {
            if g.contains(char::is_control) {
                continue;
            }

            let width = g.width() as u16;

            if width == 0 || skip_width > 0 {
                skip_width = skip_width.saturating_sub(width);
                continue;
            }

            if remaining < width {
                break;
            }

            self.buf[(cursor, row)].set_symbol(g).set_style(style);

            let end = cursor + width;
            cursor += 1;

            while cursor < end {
                self.buf[(cursor, row)].reset();
                cursor += 1;
            }

            remaining -= width;
        }
    }

    /// An internal helper that draws a `ratatui` widget at a given region.
    fn render_widget(&mut self, rect: LogicalRect, widget: impl Widget) {
        let rect = rect.with_offset(self.offset_x - self.rect.x, self.offset_y - self.rect.y);

        if !rect.intersects(self.rect) {
            return;
        }

        // Clip to canvas viewport
        let clip = rect.intersection(self.rect);

        // Clip to buffer bounds - this ensures non-negative coordinates
        let buffer_bounds = LogicalRect::from(self.buf.area);
        let final_clip = clip.intersection(buffer_bounds);

        // Early exit if nothing visible in buffer
        if final_clip.width <= 0 || final_clip.height <= 0 {
            return;
        }

        // Now final_clip.{x,y} are guaranteed >= buffer_bounds.{x,y}
        // For typical case where buffer starts at (0,0), they're guaranteed >= 0

        // Create temporary buffer for widget rendering
        let temp_rect = Rect {
            x: 0,
            y: 0,
            width: rect.width.min(u16::MAX as i32) as u16,
            height: rect.height.min(u16::MAX as i32) as u16,
        };

        let mut temp_buf = Buffer::empty(temp_rect);
        widget.render(temp_rect, &mut temp_buf);

        // Calculate source offset in temp buffer
        // This tells us which part of the rendered widget to copy
        let src_x0 = (final_clip.x - rect.x) as usize;
        let src_y0 = (final_clip.y - rect.y) as usize;

        // Calculate destination offset in canvas buffer
        // Subtract buffer area offset to get array index
        let dst_x0 = (final_clip.x - self.buf.area.x as i32) as usize;
        let dst_y0 = (final_clip.y - self.buf.area.y as i32) as usize;

        let src_stride = rect.width as usize;
        let dst_stride = self.buf.area.width as usize;
        let row_len = final_clip.width as usize;

        // Copy visible portion from temp buffer to canvas buffer
        for row in 0..final_clip.height as usize {
            let src_row = (src_y0 + row) * src_stride + src_x0;
            let dst_row = (dst_y0 + row) * dst_stride + dst_x0;

            let src = &temp_buf.content[src_row..src_row + row_len];
            let dst = &mut self.buf.content[dst_row..dst_row + row_len];
            dst.clone_from_slice(src);
        }
    }

    /// Converts a x-coordinate local to this canvas to the global buffer space.
    fn get_buf_column(&self, x: i32) -> i32 {
        self.rect.x + x - self.offset_x
    }

    /// Converts a y-coordinate local to this canvas to the global buffer spcae.
    fn get_buf_row(&self, y: i32) -> i32 {
        self.rect.y + y - self.offset_y
    }

    /// Determines if content should be drawn even if outside of the canvas region.
    fn clipped(&self) -> bool {
        self.clipped
    }
}
