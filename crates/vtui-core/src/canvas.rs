use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl From<Rect> for LogicalRect {
    fn from(value: Rect) -> Self {
        Self {
            x: value.x as i32,
            y: value.y as i32,
            width: value.width as i32,
            height: value.height as i32,
        }
    }
}

impl LogicalRect {
    pub fn new(x: i32, y: i32, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width: width as i32,
            height: height as i32,
        }
    }

    pub fn intersection(self, other: Self) -> Self {
        if !self.intersects(other) {
            return Self {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            };
        }

        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = self.right().min(other.right());
        let y2 = self.bottom().min(other.bottom());

        if x2 <= x1 || y2 <= y1 {
            LogicalRect {
                x: x1,
                y: y1,
                width: 0,
                height: 0,
            }
        } else {
            LogicalRect {
                x: x1,
                y: y1,
                width: x2 - x1,
                height: y2 - y1,
            }
        }
    }

    #[inline(always)]
    pub fn intersects(self, other: Self) -> bool {
        self.y < other.y + other.height
            && self.y + self.height > other.y
            && self.x < other.x + other.width
            && self.x + self.width > other.x
    }

    pub fn with_offset(mut self, offset_x: i32, offset_y: i32) -> Self {
        self.x -= offset_x;
        self.y -= offset_y;
        self
    }

    pub const fn area(self) -> i64 {
        (self.width as i64) * (self.height as i64)
    }

    pub const fn left(self) -> i32 {
        self.x
    }

    pub const fn right(self) -> i32 {
        self.x.saturating_add(self.width)
    }

    pub const fn top(self) -> i32 {
        self.y
    }

    pub const fn bottom(self) -> i32 {
        self.y.saturating_add(self.height)
    }
}

pub struct Canvas<'a> {
    buf: &'a mut Buffer,
    rect: LogicalRect,
    offset_x: i32,
    offset_y: i32,
}

impl<'a> Canvas<'a> {
    pub fn new(rect: LogicalRect, buf: &'a mut Buffer) -> Self {
        Self {
            rect,
            buf,
            offset_x: 0,
            offset_y: 0,
        }
    }
}

impl Canvas<'_> {
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

    fn get_buf_column(&self, x: i32) -> i32 {
        self.rect.x + x - self.offset_x
    }

    fn get_buf_row(&self, y: i32) -> i32 {
        self.rect.y + y - self.offset_y
    }

    fn clipped(&self) -> bool {
        false
    }
}

impl Canvas<'_> {
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.buf
    }

    pub fn set_offset(&mut self, offset_x: i32, offset_y: i32) {
        self.offset_x = offset_x;
        self.offset_y = offset_y;
    }

    pub fn text<T, S>(&mut self, x: i32, y: i32, text: T, style: S)
    where
        T: AsRef<str>,
        S: Into<Style>,
    {
        self.set_stringn(x, y, text, usize::MAX, style)
    }

    pub fn render_widget(&mut self, rect: LogicalRect, widget: impl Widget) {
        let rect = rect.with_offset(
            self.offset_x - self.rect.x,
            self.offset_y - self.rect.y,
        );

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
}
