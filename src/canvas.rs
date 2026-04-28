use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{arena::FrameData, context::Context, layout::Region};

pub struct Canvas<'a> {
    buffer: &'a mut Buffer,
    context: &'a Context,
    data: FrameData,
}

impl<'a> Canvas<'a> {
    pub fn widget(&mut self, widget: impl Widget, region: Region) {
        render_widget(widget, region, self.buffer);
    }

    pub fn area(&self) -> Region {
        Region::origin(self.data.rect.width, self.data.rect.height)
    }

    pub(crate) fn new(buffer: &'a mut Buffer, context: &'a Context, data: FrameData) -> Self {
        Self {
            buffer,
            context,
            data,
        }
    }
}

fn render_widget(widget: impl Widget, region: Region, buf: &mut Buffer) {
    let buf_region = Region::from(buf.area);

    if !region.intersects(buf_region) {
        return;
    }

    let clip = region.intersection(buf_region);
    if clip.width == 0 || clip.height == 0 {
        return;
    }

    if clip == region
        && let Ok(area) = Rect::try_from(region)
    {
        widget.render(area, buf);
        return;
    }

    let temp_rect = Rect {
        x: 0,
        y: 0,
        width: region.width.clamp(0, u16::MAX as i32) as u16,
        height: region.height.clamp(0, u16::MAX as i32) as u16,
    };

    let mut temp_buf = Buffer::empty(temp_rect);
    widget.render(temp_rect, &mut temp_buf);

    let src_x0 = (clip.x - region.x) as usize;
    let src_y0 = (clip.y - region.y) as usize;
    let dst_x0 = (clip.x - buf.area.x as i32) as usize;
    let dst_y0 = (clip.y - buf.area.y as i32) as usize;

    let src_stride = temp_buf.area.width as usize;
    let dst_stride = buf.area.width as usize;
    let row_len = clip.width as usize;

    for row in 0..clip.height as usize {
        let src_row = (src_y0 + row) * src_stride + src_x0;
        let dst_row = (dst_y0 + row) * dst_stride + dst_x0;

        let src = &temp_buf.content[src_row..src_row + row_len];
        let dst = &mut buf.content[dst_row..dst_row + row_len];
        dst.clone_from_slice(src);
    }
}
