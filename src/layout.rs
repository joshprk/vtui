use std::fmt::Debug;

use ratatui::layout::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
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

impl TryFrom<Region> for Rect {
    type Error = Region;

    fn try_from(region: Region) -> Result<Self, Self::Error> {
        let Region {
            x,
            y,
            width,
            height,
        } = region;

        let conv = |v: i32| u16::try_from(v).map_err(|_| region);

        Ok(Rect {
            x: conv(x)?,
            y: conv(y)?,
            width: conv(width)?,
            height: conv(height)?,
        })
    }
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

    pub fn origin(width: i32, height: i32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    pub fn intersects(self, other: Self) -> bool {
        self.y < other.y + other.height
            && self.y + self.height > other.y
            && self.x < other.x + other.width
            && self.x + self.width > other.x
    }

    pub fn intersection(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        Self {
            x,
            y,
            width: right.saturating_sub(x),
            height: bottom.saturating_sub(y),
        }
    }

    pub fn left(self) -> i32 {
        self.x
    }

    pub fn right(self) -> i32 {
        self.x.saturating_add(self.width)
    }

    pub fn top(self) -> i32 {
        self.y
    }

    pub fn bottom(self) -> i32 {
        self.y.saturating_add(self.height)
    }
}
