#[derive(Copy, Clone, Default, Debug)]
pub struct PosF32 {
    pub x: f32,
    pub y: f32,
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
    pub fn splat(&self) -> [f32; 4] {
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
