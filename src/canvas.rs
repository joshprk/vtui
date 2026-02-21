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

    pub fn rect(&self) -> Region {
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
    if region.width <= 0 || region.height <= 0 {
        return;
    }

    let w = region.width.clamp(0, u16::MAX as i32) as u16;
    let h = region.height.clamp(0, u16::MAX as i32) as u16;

    if w == 0 || h == 0 {
        return;
    }

    let rendered_region = Region::new(region.x, region.y, w as i32, h as i32);
    let visible = rendered_region.intersection(Region::from(buf.area));

    if visible.width <= 0 || visible.height <= 0 {
        return;
    }

    let temp_area = Rect::new(0, 0, w, h);
    let mut temp_buf = Buffer::empty(temp_area);
    widget.render(temp_area, &mut temp_buf);

    let dst_x = visible.x as usize;
    let dst_y = visible.y as usize;
    let src_x = visible.x.saturating_sub(rendered_region.x).max(0) as usize;
    let src_y = visible.y.saturating_sub(rendered_region.y).max(0) as usize;
    let copy_w = visible.width.max(0) as usize;
    let copy_h = visible.height.max(0) as usize;

    let buf_w = buf.area.width as usize;
    let tmp_w = w as usize;

    for row in 0..copy_h {
        let src_row = src_y.saturating_add(row);
        let dst_row = dst_y.saturating_add(row);
        let src_base = src_row.saturating_mul(tmp_w).saturating_add(src_x);
        let dst_base = dst_row.saturating_mul(buf_w).saturating_add(dst_x);

        for col in 0..copy_w {
            let src_idx = src_base.saturating_add(col);
            let dst_idx = dst_base.saturating_add(col);

            let Some(src_cell) = temp_buf.content.get(src_idx) else {
                continue;
            };

            let Some(dst_cell) = buf.content.get_mut(dst_idx) else {
                continue;
            };

            *dst_cell = src_cell.clone();
        }
    }
}
