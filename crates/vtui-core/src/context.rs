use std::ops::Deref;

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::events::Event;

pub struct Canvas<'a> {
    buf: &'a mut Buffer,
    rect: Rect,
    offset_x: i32,
    offset_y: i32,
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

    pub fn set_offset(&mut self, offset_x: i32, offset_y: i32) {
        self.offset_x = offset_x;
        self.offset_y = offset_y;
    }

    pub fn offset(&self) -> (i32, i32) {
        (self.offset_x, self.offset_y)
    }
}

impl Canvas<'_> {
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.buf
    }

    pub fn text<T, S>(&mut self, x: u16, y: u16, text: T, style: S)
    where
        T: AsRef<str>,
        S: Into<Style>,
    {
        self.set_stringn(x, y, text, usize::MAX, style);
    }

    pub fn set_stringn<T, S>(
        &mut self,
        x: u16,
        y: u16,
        text: T,
        max_width: usize,
        style: S,
    ) where
        T: AsRef<str>,
        S: Into<Style>,
    {
        let x_i32 = x as i32;
        let y_i32 = y as i32;

        if y_i32 < self.offset_y || y_i32 >= self.offset_y + (self.rect.height as i32) {
            return;
        }

        let visible_left = self.offset_x;
        let visible_right = self.offset_x + (self.rect.width as i32);

        // Text starts after visible area
        if x_i32 >= visible_right {
            return;
        }

        // Collect graphemes (filtering control chars, zero-width)
        let graphemes: Vec<(&str, u16)> = UnicodeSegmentation::graphemes(text.as_ref(), true)
            .filter(|symbol| !symbol.contains(char::is_control))
            .map(|symbol| (symbol, symbol.width() as u16))
            .filter(|(_symbol, width)| *width > 0)
            .collect();

        if graphemes.is_empty() {
            return;
        }

        // Calculate cells to skip on the left (if text starts before visible area)
        let skip_cells = if x_i32 < visible_left {
            (visible_left - x_i32) as u16
        } else {
            0
        };

        // Skip graphemes until we've covered skip_cells width
        let mut accumulated_width: u16 = 0;
        let mut skip_idx = 0;
        for (i, (_symbol, width)) in graphemes.iter().enumerate() {
            if accumulated_width >= skip_cells {
                skip_idx = i;
                break;
            }
            accumulated_width += width;
        }

        // All text was skipped (started before visible area and ends before visible area starts)
        if skip_idx >= graphemes.len() {
            return;
        }

        // Screen position where remaining text will start
        let screen_x = (self.rect.x as i32 + x_i32.max(visible_left) - self.offset_x) as u16;
        let screen_y = (self.rect.y as i32 + y_i32 - self.offset_y) as u16;

        // Available width: from screen_x to end of visible rect, bounded by max_width
        let available_width = (self.rect.x + self.rect.width)
            .saturating_sub(screen_x)
            .min(max_width as u16) as usize;

        // Reconstruct remaining text from skipped graphemes
        let remaining_text: String = graphemes[skip_idx..].iter().map(|(s, _)| *s).collect();

        let style = style.into();
        self.buf.set_stringn(screen_x, screen_y, remaining_text, available_width, style);
    }

    pub fn render_widget(&mut self, rect: Rect, widget: impl Widget) {
        // Compute intersection of widget rect with visible area (in logical space)
        let widget_left = rect.x as i32;
        let widget_right = (rect.x as i32) + (rect.width as i32);
        let widget_top = rect.y as i32;
        let widget_bottom = (rect.y as i32) + (rect.height as i32);

        let visible_left = widget_left.max(self.offset_x);
        let visible_right = widget_right.min(self.offset_x + (self.rect.width as i32));
        let visible_top = widget_top.max(self.offset_y);
        let visible_bottom = widget_bottom.min(self.offset_y + (self.rect.height as i32));

        // Early exit if completely off-screen
        if visible_right <= visible_left || visible_bottom <= visible_top {
            return;
        }

        // Render widget to temp buffer
        let temp_rect = Rect::new(0, 0, rect.width, rect.height);
        let mut temp_buf = Buffer::empty(temp_rect);
        widget.render(temp_rect, &mut temp_buf);

        let area = self.buf.area;

        // Calculate source and destination offsets
        let src_x_start = (visible_left - widget_left) as usize;
        let src_y_start = (visible_top - widget_top) as usize;
        let dst_x_start = (self.rect.x as i32 + visible_left - self.offset_x) as usize;
        let dst_y_start = (self.rect.y as i32 + visible_top - self.offset_y) as usize;

        let copy_width = (visible_right - visible_left) as usize;
        let copy_height = (visible_bottom - visible_top) as usize;

        let src_stride = rect.width as usize;
        let dst_stride = area.width as usize;

        // Safety: src_x_start and copy_width are derived from widget rect intersection
        // which is guaranteed to be within rect.width. dst offsets are within area bounds
        // by construction (visible area calculation respects canvas rect).
        for y in 0..copy_height {
            let src_row_offset = (src_y_start + y) * src_stride + src_x_start;
            let dst_row_offset = (dst_y_start + y) * dst_stride + dst_x_start;

            let src = &temp_buf.content[src_row_offset..src_row_offset + copy_width];
            let dst = &mut self.buf.content[dst_row_offset..dst_row_offset + copy_width];

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
