use std::ops::{Mul, AddAssign};

#[derive(Copy, Clone, Default, Debug)]
pub struct RectF32 {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl RectF32 {

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
#[derive(Copy, Clone, Default, Debug)]
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
