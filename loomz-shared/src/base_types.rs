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

/// User friendly re-export from base types
pub mod _2d {

    #[repr(C)]
    #[derive(Copy, Clone, Default, Debug)]
    pub struct Position<T> {
        pub x: T,
        pub y: T,
    }

    impl<T: Copy> Position<T> {
        #[inline]
        pub const fn splat(&self) -> [T; 2] {
            [self.x, self.y]
        }
    }

    impl Position<f64> {
        pub fn as_f32(&self) -> Position<f32> {
            Position {
                x: self.x as f32,
                y: self.y as f32
            }
        }
    }

    #[repr(C)]
    #[derive(Copy, Clone, Default, Debug)]
    pub struct Size<T> {
        pub width: T,
        pub height: T,
    }

    impl<T: Copy> Size<T> {
        #[inline]
        pub const fn splat(&self) -> [T; 2] {
            [self.width, self.height]
        }
    }

    pub fn pos<T>(x: T, y: T) -> Position<T> {
        Position { x, y }
    }

    pub fn size<T>(width: T, height: T) -> Size<T> {
        Size { width, height }
    }
}

