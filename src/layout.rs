use std::fmt::Debug;

use ratatui::layout::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Region {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

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

impl From<Rect> for Region {
    fn from(rect: Rect) -> Self {
        Self {
            x: rect.x as i32,
            y: rect.y as i32,
            width: rect.width as i32,
            height: rect.height as i32,
        }
    }
}
