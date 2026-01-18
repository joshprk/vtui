use crate::canvas::LogicalRect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    #[default]
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Measure {
    Exact(i32),
}

pub(crate) fn compute_split(
    axis: Axis,
    area: LogicalRect,
    measures: &[Measure],
) -> Vec<LogicalRect> {
    let mut rects = Vec::with_capacity(measures.len());

    match axis {
        Axis::Horizontal => {
            let mut x = area.x;
            for Measure::Exact(w) in measures {
                rects.push(LogicalRect {
                    x,
                    y: area.y,
                    width: *w,
                    height: area.height,
                });
                x += *w;
            }
        }

        Axis::Vertical => {
            let mut y = area.y;
            for Measure::Exact(h) in measures {
                rects.push(LogicalRect {
                    x: area.x,
                    y,
                    width: area.width,
                    height: *h,
                });
                y += *h;
            }
        }
    }

    rects
}
