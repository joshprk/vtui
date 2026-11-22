use ratatui::layout::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl From<Rect> for LogicalRect {
    fn from(rect: Rect) -> Self {
        Self {
            x: rect.x as i32,
            y: rect.y as i32,
            width: rect.width as i32,
            height: rect.height as i32,
        }
    }
}
