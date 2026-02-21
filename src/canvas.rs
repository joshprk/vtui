use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{arena::FrameData, context::Context, layout::Region};

pub struct Canvas<'a> {
    buffer: &'a mut Buffer,
    context: &'a Context,
    data: FrameData,
}

impl<'a> Canvas<'a> {
    pub(crate) fn new(buffer: &'a mut Buffer, context: &'a Context, data: FrameData) -> Self {
        Self {
            buffer,
            context,
            data,
        }
    }
}

fn render_widget(widget: impl Widget, region: Region, buf: &mut Buffer) {}
