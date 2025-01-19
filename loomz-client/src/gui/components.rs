mod label;
pub(crate) use label::*;

mod frame;
pub(crate) use frame::*;

use super::{GuiLayoutItem, GuiSprite, GuiSpriteType, GuiComponentStyle, GuiStyleState};

#[repr(u8)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum GuiComponentTag {
    Frame,
    Label
}

pub(crate) enum GuiComponentType {
    Frame(GuiFrame),
    Label(GuiLabel),
}

impl GuiComponentType {

    pub fn generate_sprites(&self, item: &GuiLayoutItem, sprites: &mut Vec<GuiSprite>) {
        match self {
            GuiComponentType::Frame(frame) => frame.generate_sprites(item, sprites),
            GuiComponentType::Label(label) => label.generate_sprites(item, sprites),
        }
    }

    pub fn update_style(&mut self, styles: &Vec<GuiComponentStyle>, new_state: GuiStyleState) {
        match self {
            GuiComponentType::Frame(frame) => frame.update_style(styles, new_state),
            GuiComponentType::Label(label) => label.update_style(styles, new_state),
        }
    }

}
