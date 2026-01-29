use ratatui::layout::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl From<Rect> for LogicalRect {
    fn from(value: Rect) -> Self {
        Self::new(value.x as i32, value.y as i32, value.width, value.height)
    }
}

impl LogicalRect {
    pub fn new(x: i32, y: i32, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width: width as i32,
            height: height as i32,
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
