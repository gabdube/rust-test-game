mod compute;
pub use compute::compute;

use loomz_shared::base_types::{PositionF32, SizeF32};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GuiLayoutType {
    VBox
}

#[derive(Copy, Clone, Debug)]
pub(super) struct GuiLayout {
    pub ty: GuiLayoutType,
    pub children_count: u32,
    pub width: f32,
    pub height: f32,
}

#[derive(Copy, Clone, Default, Debug)]
pub(super) struct GuiLayoutItem {
    pub has_layout: bool,
    pub position: PositionF32,
    pub size: SizeF32,
}

impl Default for GuiLayout {
    fn default() -> Self {
        GuiLayout {
            ty: GuiLayoutType::VBox,
            children_count: 0,
            width: 0.0,
            height: 0.0,
        }
    }
}
