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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Variable {
    start: i32,
    size: i32,
}

pub(crate) fn compute_split(
    axis: Axis,
    area: LogicalRect,
    measures: &[Measure],
) -> Vec<LogicalRect> {
    match axis {
        Axis::Horizontal => split_measures(area.x, measures)
            .map(|v| LogicalRect {
                x: v.start,
                y: area.y,
                width: v.size,
                height: area.height,
            })
            .collect(),
        Axis::Vertical => split_measures(area.y, measures)
            .map(|v| LogicalRect {
                x: area.x,
                y: v.start,
                width: area.width,
                height: v.size,
            })
            .collect(),
    }
}

fn split_measures(start: i32, measures: &[Measure]) -> impl Iterator<Item = Variable> {
    let mut cursor = start;

    measures.iter().map(move |Measure::Exact(size)| {
        let v = Variable {
            start: cursor,
            size: *size,
        };
        cursor += *size;
        v
    })
}
