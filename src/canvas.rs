use ratatui::buffer::Buffer;

use crate::{arena::FrameData, context::Context};

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
