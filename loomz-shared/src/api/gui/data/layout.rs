use crate::base_types::{PositionF32, SizeF32};

#[derive(Copy, Clone, Default)]
pub struct GuiLayout {
}

#[derive(Copy, Clone, Default)]
pub struct GuiLayoutView {
    pub position: PositionF32,
    pub size: SizeF32,
}
