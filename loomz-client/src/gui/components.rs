mod label;
pub(crate) use label::*;
pub use label::GuiLabelCallback;

mod frame;
pub(crate) use frame::*;

use super::{GuiLayoutItem, GuiSprite, GuiSpriteType, GuiComponentStyle, GuiStyleState};

#[repr(u8)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum GuiComponentTag {
    Frame,
    Label
}

#[derive(Copy, Clone)]
pub(super) struct GuiComponentBase {
    pub callbacks_index: u32,
    pub style_index: u32,
}

pub(super) enum GuiComponentData {
    Frame(GuiFrame),
    Label(GuiLabel),
}

impl GuiComponentData {

    pub fn generate_sprites(&self, item: &GuiLayoutItem, sprites: &mut Vec<GuiSprite>) {
        match self {
            GuiComponentData::Frame(frame) => frame.generate_sprites(item, sprites),
            GuiComponentData::Label(label) => label.generate_sprites(item, sprites),
        }
    }

    pub fn update_style(&mut self, style: &GuiComponentStyle, new_state: GuiStyleState) {
        match self {
            GuiComponentData::Frame(frame) => frame.update_style(style, new_state),
            GuiComponentData::Label(label) => label.update_style(style, new_state),
        }
    }

}
