use crate::{
    arena::{ArenaNode, NodeId},
    component::NodeAttributes,
    context::Context,
    layout::LogicalRect,
    selection::{TextOptions, layout_text, normalize_text},
};
use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

/// A drawing surface scoped to a rectangular region of the terminal buffer.
///
/// `Canvas` cannot be constructed directly. It is given to renderers via
/// [Component::draw](crate::component::Component).
pub struct Canvas<'a> {
    buf: &'a mut Buffer,
    context: &'a Context,
    current_node: NodeId,
    attributes: &'a NodeAttributes,
    rect: LogicalRect,
    effective_clip: LogicalRect,
    selection_root: NodeId,
    selectable: bool,
    text_emission: u32,
}

impl<'a> Canvas<'a> {
    /// Creates a new canvas with the given region.
    pub(crate) fn new(
        buf: &'a mut Buffer,
        context: &'a Context,
        current_node: NodeId,
        node: &'a ArenaNode,
    ) -> Self {
        let rect = node.area();
        let attributes = node.attributes();

        Self {
            buf,
            context,
            current_node,
            attributes,
            rect,
            effective_clip: node.effective_clip(),
            selection_root: node.selection_root(),
            selectable: node.selectable(),
            text_emission: 0,
        }
    }
}

impl Canvas<'_> {
    /// Returns the underlying buffer.
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.context
            .occlude_all_selection(LogicalRect::from(self.buf.area));
        self.buf
    }

    /// Returns the relative rectangular region of this canvas.
    pub fn area(&self) -> LogicalRect {
        LogicalRect::origin(self.rect.width, self.rect.height)
    }

    /// Determines if this component is focused.
    pub fn is_focused(&self) -> bool {
        self.context.focused() == Some(self.current_node)
    }

    /// Draws text content at a given position.
    ///
    /// This function is panic-free and text is automatically clipped.
    pub fn text<T, S>(&mut self, x: i32, y: i32, text: T, style: S)
    where
        T: AsRef<str>,
        S: Into<Style>,
    {
        self.text_with(x, y, text, style, TextOptions::default());
    }

    /// Draws semantic text with explicit selection behavior.
    pub fn text_with<T, S>(&mut self, x: i32, y: i32, text: T, style: S, options: TextOptions)
    where
        T: AsRef<str>,
        S: Into<Style>,
    {
        self.set_stringn(x, y, text, usize::MAX, style, options);
    }

    /// Draws a [ratatui] widget at the given region.
    ///
    /// This function is panic-free and text is automatically clipped.
    pub fn widget(&mut self, rect: impl Into<LogicalRect>, widget: impl Widget) {
        self.render_widget(rect.into(), widget);
    }

    /// Returns a copy of the currently selected semantic text.
    pub fn selected_text(&self) -> Option<String> {
        self.context.selected_text()
    }

    /// An internal helper which draws text content at a given position.
    fn set_stringn<T, S>(
        &mut self,
        x: i32,
        y: i32,
        text: T,
        max_width: usize,
        style: S,
        mut options: TextOptions,
    ) where
        T: AsRef<str>,
        S: Into<Style>,
    {
        options.selectable &= self.selectable;
        let normalized = normalize_text(text.as_ref());
        let token = if options.selectable {
            let token = self.context.begin_selectable_text(
                self.selection_root,
                self.current_node,
                self.text_emission,
                normalized.clone(),
                options,
            );
            self.text_emission = self.text_emission.saturating_add(1);
            token
        } else {
            None
        };

        let placements = layout_text(
            &normalized,
            self.get_buf_column(x),
            self.get_buf_row(y),
            self.effective_clip,
            max_width,
        );
        let style = style.into();
        for placement in placements {
            self.paint_grapheme(&placement, style);
            if let Some(token) = token {
                self.context.record_selectable_grapheme(token, &placement);
            }
        }
    }

    fn paint_grapheme(
        &mut self,
        placement: &crate::selection::GraphemePlacement<'_>,
        style: Style,
    ) {
        let row = placement.screen.y as u16;
        let start = placement.screen.x as u16;
        let end = placement.screen.right() as u16;
        self.buf[(start, row)]
            .set_symbol(placement.symbol)
            .set_style(style);
        for column in start + 1..end {
            self.buf[(column, row)].reset();
        }
    }

    /// An internal helper that draws a `ratatui` widget at a given region.
    fn render_widget(&mut self, rect: LogicalRect, widget: impl Widget) {
        let (offset_x, offset_y) = self.attributes.offset;
        let rect = rect.with_offset(offset_x - self.rect.x, offset_y - self.rect.y);

        if !rect.intersects(self.rect) {
            return;
        }

        // Widgets remain bounded by their requested component viewport and all ancestor clips.
        let clip = rect
            .intersection(self.rect)
            .intersection(self.effective_clip);

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

        self.context.occlude_selection(final_clip);
    }

    /// Converts a x-coordinate local to this canvas to the global buffer space.
    fn get_buf_column(&self, x: i32) -> i32 {
        let (offset_x, _) = self.attributes.offset;
        self.rect.x + x - offset_x
    }

    /// Converts a y-coordinate local to this canvas to the global buffer spcae.
    fn get_buf_row(&self, y: i32) -> i32 {
        let (_, offset_y) = self.attributes.offset;
        self.rect.y + y - offset_y
    }
}
