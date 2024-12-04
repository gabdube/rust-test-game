#![allow(dead_code)]

#[derive(Debug, Default, Copy, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Offset {
    pub x: u32,
    pub y: u32
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Rect {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl Rect {
    pub fn width(&self) -> u32 {
        self.right - self.left
    }

    pub fn height(&self) -> u32 {
        self.bottom - self.top
    }
}

pub const fn size(width: u32, height: u32) -> Size {
    Size { width, height }
}

pub const fn offset(x: u32, y: u32) -> Offset {
    Offset { x, y }
}

pub const fn rect(left: u32, top: u32, right: u32, bottom: u32) -> Rect {
    Rect { left, top, right, bottom }
}

