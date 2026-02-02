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
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn zeroed() -> Self {
        LogicalRect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

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

    pub fn intersects(self, other: Self) -> bool {
        self.y < other.y + other.height
            && self.y + self.height > other.y
            && self.x < other.x + other.width
            && self.x + self.width > other.x
    }

    pub fn with_offset(mut self, offset_x: i32, offset_y: i32) -> Self {
        self.x -= offset_x;
        self.y -= offset_y;
        self
    }

    pub const fn area(self) -> i64 {
        (self.width as i64) * (self.height as i64)
    }

    pub const fn left(self) -> i32 {
        self.x
    }

    pub const fn right(self) -> i32 {
        self.x.saturating_add(self.width)
    }

    pub const fn top(self) -> i32 {
        self.y
    }

    pub const fn bottom(self) -> i32 {
        self.y.saturating_add(self.height)
    }
}

/// A layout quantity describing how much space a node occupies along its parent's primary axis.
#[derive(Debug, Clone, Copy)]
pub enum Measure {
    /// Occupies an exact number of cells.
    Exact(i32),

    /// Occupies a fraction of the viewport size on the primary axis.
    ///
    /// A viewport unit is defined as the space allocated to the parent along its primary axis.
    Viewport(f64),
}

impl Default for Measure {
    fn default() -> Self {
        Measure::Viewport(1.0)
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
