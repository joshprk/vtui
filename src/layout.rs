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

/// Computes the layout of an area across one or more measures.
///
/// It is possible for a layout to overflow its area.
pub fn compute_split<I>(flow: Flow, area: LogicalRect, measures: I) -> Vec<LogicalRect>
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

/// Computes the split of an one-dimensional line across one or more measures.
fn split_measures<I>(start: i32, viewport: i32, measures: I) -> impl Iterator<Item = Variable>
where
    I: IntoIterator<Item = Measure>,
{
    let mut cursor = start;

    measures.into_iter().map(move |measure| {
        let size = match measure {
            Measure::Exact(size) => size,
            Measure::Percent(percent) => (viewport as f64 * percent).round() as i32,
        };

        let v = Variable {
            start: cursor,
            size,
        };
        cursor += size;
        v
    })
}
