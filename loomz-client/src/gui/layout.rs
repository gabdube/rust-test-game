mod compute;
pub use compute::compute;

use loomz_shared::base_types::{PositionF32, SizeF32};

#[derive(Copy, Clone)]
pub(super) enum GuiLayoutType {
    VBox
}

#[derive(Copy, Clone)]
pub(super) struct GuiLayout {
    pub ty: GuiLayoutType,
    pub height: f32,
    pub width: f32,
    pub first_component: u32,
    pub component_count: u32,
}

#[derive(Copy, Clone, Default, Debug)]
pub(super) struct GuiLayoutView {
    pub position: PositionF32,
    pub size: SizeF32,
}
