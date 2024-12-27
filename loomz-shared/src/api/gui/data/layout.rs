use crate::base_types::_2d::{Position, Size};

#[derive(Copy, Clone, Default)]
pub struct GuiLayout {
}

#[derive(Copy, Clone, Default)]
pub struct GuiLayoutView {
    pub position: Position<f32>,
    pub size: Size<f32>,
}
