pub mod error;
pub use error::*;

pub mod base_types;
pub use base_types::*;

pub mod assets;
pub use assets::*;

pub mod api;
pub use api::*;

/// User friendly re-export from base types
pub mod _2d {
    use crate::{base_types::PosF32, SizeF32};

    pub type Position = PosF32;
    pub type Size = SizeF32;

    pub fn pos(x: f32, y: f32) -> PosF32 {
        PosF32 { x, y }
    }

    pub fn size(width: f32, height: f32) -> SizeF32 {
        SizeF32 { width, height }
    }
}
