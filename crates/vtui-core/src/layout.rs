use crate::canvas::LogicalRect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Flow {
    Horizontal,
    #[default]
    Vertical,
}

#[derive(Clone, Copy, Debug)]
pub enum Measure {
    Exact(i32),
    Percentage(f64),
}

impl Default for Measure {
    fn default() -> Self {
        Measure::Percentage(1.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Layer(i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Variable {
    start: i32,
    size: i32,
}

pub(crate) fn compute_split(
    flow: Flow,
    area: LogicalRect,
    measures: &[Measure],
) -> Vec<LogicalRect> {
    match flow {
        Flow::Horizontal => split_measures(area.x, area.width, measures)
            .map(|v| LogicalRect {
                x: v.start,
                y: area.y,
                width: v.size,
                height: area.height,
            })
            .collect(),
        Flow::Vertical => split_measures(area.y, area.height, measures)
            .map(|v| LogicalRect {
                x: area.x,
                y: v.start,
                width: area.width,
                height: v.size,
            })
            .collect(),
    }
}

fn split_measures(
    start: i32,
    viewport: i32,
    measures: &[Measure],
) -> impl Iterator<Item = Variable> {
    let mut cursor = start;

    measures.iter().map(move |measure| {
        let size = match measure {
            Measure::Exact(size) => *size,
            Measure::Percentage(percent) => (viewport as f64 * percent).round() as i32,
        };

        let v = Variable {
            start: cursor,
            size,
        };
        cursor += size;
        v
    })
}
