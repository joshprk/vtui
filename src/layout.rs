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

pub(crate) fn compute_split<I>(flow: Flow, area: LogicalRect, measures: I) -> Vec<LogicalRect>
where
    I: IntoIterator<Item = Measure>,
{
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

fn split_measures<I>(start: i32, viewport: i32, measures: I) -> impl Iterator<Item = Variable>
where
    I: IntoIterator<Item = Measure>,
{
    let mut cursor = start;

    measures.into_iter().map(move |measure| {
        let size = match measure {
            Measure::Exact(size) => size,
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
