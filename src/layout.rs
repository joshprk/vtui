#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl LogicalRect {
    pub fn zeroed() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn left(self) -> i32 {
        self.x
    }

    pub fn right(self) -> i32 {
        self.x + self.width
    }

    pub fn top(self) -> i32 {
        self.y
    }

    pub fn bottom(self) -> i32 {
        self.y + self.height
    }
}
