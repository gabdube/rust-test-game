use std::ops::{Mul, AddAssign};

#[derive(Copy, Clone, Default, Debug)]
pub struct RectF32 {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl RectF32 {

    pub fn from_position_and_size(pos: PositionF32, size: SizeF32) -> Self {
        RectF32 {
            left: pos.x,
            top: pos.y,
            right: pos.x + size.width,
            bottom: pos.y + size.height
        }
    }

    pub fn from_size(size: SizeF32) -> Self {
        RectF32 {
            left: 0.0,
            top: 0.0,
            right: size.width,
            bottom: size.height,
        }
    }

    #[inline]
    pub const fn splat(&self) -> [f32; 4] {
        [self.left, self.top, self.right, self.bottom]
    }

    #[inline]
    pub const fn offset(&self) -> [f32; 2] {
        [self.left, self.top]
    }

    #[inline]
    pub const fn width(&self) -> f32 {
        self.right - self.left
    }

    #[inline]
    pub const fn height(&self) -> f32 {
        self.bottom - self.top
    }

    #[inline]
    pub const fn size(&self) -> [f32; 2] {
        [self.right - self.left, self.bottom - self.top]
    }

    #[inline]
    pub const fn translate_into(&self, x: f32, y: f32) -> Self {
        RectF32 {
            left: self.left + x,
            top: self.top + y,
            right: self.right + x,
            bottom: self.bottom + y,
        }
    }

    #[inline]
    pub const fn is_point_inside(&self, point: PositionF32) -> bool {
        let [x, y] = point.splat();
        x >= self.left && y >= self.top && x < self.right && y < self.bottom
    }

}

#[derive(Copy, Clone, Default, Debug)]
pub struct RgbaU8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaU8 {
    pub const fn rgb(r: u8, g: u8, b: u8) -> RgbaU8 {
        RgbaU8 { r, g, b, a: 255 }
    }

    pub const fn splat(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Into<u32> for RgbaU8 {
    fn into(self) -> u32 {
        (self.r as u32) + ((self.g as u32) << 8) + ((self.b as u32) << 16) + ((self.a as u32) << 24)
    }
}

impl From<u32> for RgbaU8 {
    fn from(value: u32) -> Self {
        RgbaU8 {
            r: ((value & 0x000000FF) >> 0) as u8,
            g: ((value & 0x0000FF00) >> 8) as u8,
            b: ((value & 0x00FF0000) >> 16) as u8,
            a: ((value & 0xFF000000) >> 24) as u8,
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct PositionF32 {
    pub x: f32,
    pub y: f32,
}

impl PositionF32 {
    #[inline(always)]
    pub const fn splat(&self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl PositionF32 {
    pub const fn out_of_range(&self, target: Self, fuzz: f32) -> bool {
        self.x < (target.x - fuzz) || self.x > (target.x + fuzz) ||
        self.y < (target.y - fuzz) || self.y > (target.y + fuzz)
    }
}

impl AddAssign<PositionF64> for PositionF32 {
    fn add_assign(&mut self, rhs: PositionF64) {
        self.x += rhs.x as f32;
        self.y += rhs.y as f32;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct PositionF64 {
    pub x: f64,
    pub y: f64,
}

impl PositionF64 {
    pub fn as_f32(&self) -> PositionF32 {
        PositionF32 {
            x: self.x as f32,
            y: self.y as f32
        }
    }
}

impl Mul<f64> for PositionF64 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        PositionF64 {
            x: self.x * rhs,
            y: self.y * rhs
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct SizeF32 {
    pub width: f32,
    pub height: f32,
}

impl SizeF32 {
    #[inline(always)]
    pub const fn splat(&self) -> [f32; 2] {
        [self.width, self.height]
    }
}

pub const fn rect(left: f32, top: f32, right: f32, bottom: f32) -> RectF32 {
    RectF32 { left, top, right, bottom }
}

pub const fn size(width: f32, height: f32) -> SizeF32 {
    SizeF32 { width, height }
}

pub const fn rgb(r: u8, g: u8, b: u8) -> RgbaU8 {
    RgbaU8 { r, g, b, a: 255 }
}
