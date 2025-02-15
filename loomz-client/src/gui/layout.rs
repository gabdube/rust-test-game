mod compute;
pub use compute::compute;

use loomz_shared::base_types::{PositionF32, SizeF32};

#[derive(Copy, Clone)]
pub enum GuiLayoutPosition {
    TopLeft,
    Center,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GuiLayoutType {
    VBox,
    HBox,
}

#[derive(Copy, Clone)]
pub(super) struct GuiLayout {
    pub ty: GuiLayoutType,
    pub position: GuiLayoutPosition,
    pub children_count: u32,
    pub width: f32,
    pub height: f32,
}

#[derive(Copy, Clone, Default)]
pub(super) struct GuiLayoutItem {
    pub has_layout: bool,
    pub position: PositionF32,
    pub size: SizeF32,
}

impl Default for GuiLayout {
    fn default() -> Self {
        GuiLayout {
            ty: GuiLayoutType::VBox,
            position: GuiLayoutPosition::Center,
            children_count: 0,
            width: 0.0,
            height: 0.0,
        }
    }
}
