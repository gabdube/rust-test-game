#[derive(Copy, Clone, Default, Debug)]
pub struct PosF32 {
    pub x: f32,
    pub y: f32,
}

impl PosF32 {

    #[inline]
    pub const fn splat(&self) -> [f32; 2] {
        [self.x, self.y]
    }

}

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
