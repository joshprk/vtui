use crate::{arena::ArenaNode, canvas::LogicalRect};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Layout {
    axis: Axis,
}

impl Layout {
    pub(crate) fn split<'a, I>(self, rect: LogicalRect, children: I) -> Vec<LogicalRect>
    where
        I: ExactSizeIterator<Item = &'a ArenaNode>,
    {
        let rect_width = rect.width as f64;
        let rect_height = rect.height as f64;
        let num_children = children.len();

        match self.axis {
            Axis::Horizontal => {
                let unit = rect_width / num_children as f64;
                (0..num_children)
                    .map(|i| {
                        let mut r = rect;
                        let x0 = (unit * i as f64).round() as i32;
                        let x1 = (unit * (i + 1) as f64).round() as i32;
                        r.x = x0;
                        r.width = x1 - x0;
                        r
                    })
                    .collect()
            }
            Axis::Vertical => {
                let unit = rect_height / num_children as f64;
                (0..num_children)
                    .map(|i| {
                        let mut r = rect;
                        let y0 = (unit * i as f64).round() as i32;
                        let y1 = (unit * (i + 1) as f64).round() as i32;
                        r.y = y0;
                        r.height = y1 - y0;
                        r
                    })
                    .collect()
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    #[default]
    Vertical,
}
