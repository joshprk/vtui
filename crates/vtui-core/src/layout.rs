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
                let unit_width = rect_width / num_children as f64;
                (0..num_children)
                    .map(|i| {
                        let mut rect = rect;
                        rect.width = (unit_width * (i + 1) as f64).round() as i32;
                        rect.x = (unit_width * i as f64).round() as i32;
                        rect
                    })
                    .collect()
            }
            Axis::Vertical => {
                let unit_height = rect_height / num_children as f64;
                (0..num_children)
                    .map(|i| {
                        let mut rect = rect;
                        rect.height = (unit_height * (i + 1) as f64).round() as i32;
                        rect.y = (unit_height * i as f64).round() as i32;
                        rect
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
