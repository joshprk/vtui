use ratatui::layout::Rect;

/// A rectangular area in the logical space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl From<Rect> for LogicalRect {
    fn from(value: Rect) -> Self {
        Self::new(
            value.x as i32,
            value.y as i32,
            value.width as i32,
            value.height as i32,
        )
    }
}

impl LogicalRect {
    /// Creates a new rectangle with the given space.
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns a rectangle with zeroed-out values.
    pub fn zeroed() -> Self {
        LogicalRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    /// Returns a rectangle positioned at `(0, 0)`.
    pub fn origin(width: i32, height: i32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    /// Returns the intersection rectangle of this rectangle and another.
    pub fn intersection(self, other: Self) -> Self {
        if !self.intersects(other) {
            return Self {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            };
        }

        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = self.right().min(other.right());
        let y2 = self.bottom().min(other.bottom());

        if x2 <= x1 || y2 <= y1 {
            LogicalRect {
                x: x1,
                y: y1,
                width: 0,
                height: 0,
            }
        } else {
            LogicalRect {
                x: x1,
                y: y1,
                width: x2 - x1,
                height: y2 - y1,
            }
        }
    }

    /// Determines if this rectangle intersects another.
    pub fn intersects(self, other: Self) -> bool {
        self.y < other.y + other.height
            && self.y + self.height > other.y
            && self.x < other.x + other.width
            && self.x + self.width > other.x
    }

    /// Returns the same [`LogicalRect`] with an offset.
    pub fn with_offset(mut self, offset_x: i32, offset_y: i32) -> Self {
        self.x -= offset_x;
        self.y -= offset_y;
        self
    }

    /// The area of the rectangle.
    pub const fn area(self) -> i64 {
        (self.width as i64) * (self.height as i64)
    }

    /// The x-value of the rectangle's left edge.
    pub const fn left(self) -> i32 {
        self.x
    }

    /// The x-value of the rectangle's right edge.
    pub const fn right(self) -> i32 {
        self.x.saturating_add(self.width)
    }

    /// The y-value of the rectangle's top edge.
    pub const fn top(self) -> i32 {
        self.y
    }

    /// The y-value of the rectangle's bottom edge.
    pub const fn bottom(self) -> i32 {
        self.y.saturating_add(self.height)
    }

    /// Returns a new rectangle with the given inset applied.
    ///
    /// Negative insets expand the rectangle. Positive insets shrink it, with
    /// x/y clamped to not exceed the original bounds when the rect collapses.
    pub fn inset(self, inset: Inset) -> Self {
        let left = inset.left();
        let top = inset.top();
        let right = inset.right();
        let bottom = inset.bottom();

        let w0 = self.width.max(0);
        let h0 = self.height.max(0);

        let dx = if left >= 0 { left.min(w0) } else { left };
        let dy = if top >= 0 { top.min(h0) } else { top };

        let x = self.x.saturating_add(dx);
        let y = self.y.saturating_add(dy);
        let width = self.width.saturating_sub(left).saturating_sub(right).max(0);
        let height = self
            .height
            .saturating_sub(top)
            .saturating_sub(bottom)
            .max(0);

        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Spacing applied to a rectangle's edges for margin or padding.
///
/// Fields are ordered as `(top, right, bottom, left)`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Inset(i32, i32, i32, i32);

impl Inset {
    pub fn all(size: i32) -> Self {
        Self(size, size, size, size)
    }

    pub fn top(self) -> i32 {
        self.0
    }

    pub fn right(self) -> i32 {
        self.1
    }

    pub fn bottom(self) -> i32 {
        self.2
    }

    pub fn left(self) -> i32 {
        self.3
    }
}

/// A layout mode that changes how a parent node interprets [`Measure`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Placement {
    /// Places measures with exact, deterministic sizes.
    #[default]
    Overflow,

    /// Constrains measures to the parent's render region.
    Fit,
}

/// A layout quantity describing how much space a node occupies along its parent's primary axis.
#[derive(Debug, Clone, Copy)]
pub enum Measure {
    /// Occupies an exact number of cells.
    Exact(i32),

    /// Occupies a fraction of the viewport size on the primary axis.
    ///
    /// A viewport unit is defined as the space allocated to the parent along its primary axis.
    Percent(f64),

    /// Positioned locally within the parent without participating in layout flow.
    ///
    /// Defined as a tuple of `(x, y, width, height)`.
    Fixed(i32, i32, i32, i32),
}

impl Default for Measure {
    fn default() -> Self {
        Measure::Percent(1.0)
    }
}

/// The primary layout axis of a node.
///
/// A node's children are arranged sequentially along this axis, with each child consuming space
/// according to its [`Measure`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Flow {
    /// Children are laid out from top to bottom.
    #[default]
    Vertical,

    /// Children are laid out from left to right.
    Horizontal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Variable {
    start: i32,
    size: i32,
}

pub fn compute_split<I>(
    flow: Flow,
    placement: Placement,
    area: LogicalRect,
    measures: I,
) -> Vec<LogicalRect>
where
    I: IntoIterator<Item = Measure>,
{
    let measures: Vec<Measure> = measures.into_iter().collect();

    // Separate Fixed measures from flow measures, tracking original indices
    let mut fixed_rects: Vec<(usize, LogicalRect)> = Vec::new();
    let mut flow_measures: Vec<(usize, Measure)> = Vec::new();

    for (i, measure) in measures.into_iter().enumerate() {
        match measure {
            Measure::Fixed(x, y, width, height) => {
                fixed_rects.push((
                    i,
                    LogicalRect {
                        x: area.x + x,
                        y: area.y + y,
                        width,
                        height,
                    },
                ));
            }
            m => flow_measures.push((i, m)),
        }
    }

    // Compute layout for flow measures
    let flow_only = flow_measures.iter().map(|(_, m)| *m);
    let splits: Vec<LogicalRect> = match flow {
        Flow::Horizontal => split_measures(placement, area.x, area.width, flow_only)
            .into_iter()
            .map(|v| LogicalRect {
                x: v.start,
                y: area.y,
                width: v.size,
                height: area.height,
            })
            .collect(),
        Flow::Vertical => split_measures(placement, area.y, area.height, flow_only)
            .into_iter()
            .map(|v| LogicalRect {
                x: area.x,
                y: v.start,
                width: area.width,
                height: v.size,
            })
            .collect(),
    };

    // Merge back into original order
    let total = flow_measures.len() + fixed_rects.len();
    let mut result: Vec<LogicalRect> = vec![LogicalRect::zeroed(); total];

    for ((orig_idx, _), rect) in flow_measures.into_iter().zip(splits) {
        result[orig_idx] = rect;
    }
    for (orig_idx, rect) in fixed_rects {
        result[orig_idx] = rect;
    }

    result
}

fn split_measures<I>(placement: Placement, start: i32, viewport: i32, measures: I) -> Vec<Variable>
where
    I: IntoIterator<Item = Measure>,
{
    match placement {
        Placement::Overflow => split_overflow(start, viewport, measures),
        Placement::Fit => split_fit(start, viewport, measures),
    }
}

fn split_overflow<I>(start: i32, viewport: i32, measures: I) -> Vec<Variable>
where
    I: IntoIterator<Item = Measure>,
{
    measures
        .into_iter()
        .scan(start, |cursor, measure| {
            let size = match measure {
                Measure::Exact(n) => n.max(0),
                Measure::Percent(p) => (viewport as f64 * p.max(0.0)).round() as i32,
                Measure::Fixed(..) => unreachable!("fixed measures are handled in compute_split"),
            };
            let v = Variable {
                start: *cursor,
                size,
            };
            *cursor += size;
            Some(v)
        })
        .collect()
}

fn split_fit<I>(start: i32, viewport: i32, measures: I) -> Vec<Variable>
where
    I: IntoIterator<Item = Measure>,
{
    let measures: Vec<Measure> = measures.into_iter().collect();

    // Pass 1: sum Exact sizes and Percent weights
    let mut sum_exact: i32 = 0;
    let mut total_weight: f64 = 0.0;

    for measure in &measures {
        match *measure {
            Measure::Exact(n) => sum_exact += n.max(0),
            Measure::Percent(w) => total_weight += w.max(0.0),
            Measure::Fixed(..) => unreachable!("fixed measures are handled in compute_split"),
        }
    }

    let remaining = (viewport - sum_exact).max(0);

    // Pass 2: compute sizes using largest-remainder for percent measures
    let mut sizes: Vec<i32> = Vec::with_capacity(measures.len());
    let mut percent_indices: Vec<(usize, f64)> = Vec::new();
    let mut floor_sum: i32 = 0;

    for (i, measure) in measures.iter().enumerate() {
        match *measure {
            Measure::Exact(n) => sizes.push(n.max(0)),
            Measure::Percent(w) => {
                let w = w.max(0.0);
                if total_weight == 0.0 {
                    sizes.push(0);
                } else {
                    let raw = (remaining as f64) * w / total_weight;
                    let floored = raw.floor() as i32;
                    let frac = raw - (floored as f64);
                    sizes.push(floored);
                    floor_sum += floored;
                    percent_indices.push((i, frac));
                }
            }
            Measure::Fixed(..) => unreachable!("fixed measures are handled in compute_split"),
        }
    }

    // Distribute leftover cells to items with largest fractional parts
    let leftover = remaining - floor_sum;
    if leftover > 0 {
        percent_indices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (idx, _) in percent_indices.into_iter().take(leftover as usize) {
            sizes[idx] += 1;
        }
    }

    // Convert to Variables
    sizes
        .into_iter()
        .scan(start, |cursor, size| {
            let v = Variable {
                start: *cursor,
                size,
            };
            *cursor += size;
            Some(v)
        })
        .collect()
}
