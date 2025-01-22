use crate::base_types::{RectF32, RgbaU8};
use crate::assets::{MsdfFontId, TextureId};
use super::{Id, MessageQueueEx};

pub struct GuiTag;
pub type GuiId = Id<GuiTag>;


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GuiSpriteType {
    Image(TextureId),
    Font(MsdfFontId)
}

#[derive(Copy, Clone, Debug)]
pub struct GuiSprite {
    pub ty: GuiSpriteType,
    pub position: RectF32,
    pub texcoord: RectF32,
    pub color: RgbaU8
}

pub enum GuiApiUpdate {
    ToggleGui(bool),
    UpdateSprites(&'static [GuiSprite]),
}

pub struct GuiApi {
    gui: MessageQueueEx<GuiId, GuiApiUpdate>
}

impl GuiApi {
    pub fn init() -> Self {
        GuiApi {
            gui: MessageQueueEx::with_capacity(8, 5120),
        }
    }

    pub fn toggle_gui(&self, id: &GuiId, visible: bool) {
        self.gui.push(id, GuiApiUpdate::ToggleGui(visible));
    }

    pub fn update_gui(&self, id: &GuiId, sprites: &[GuiSprite]) {
        self.gui.push_with_data(id, sprites, |sprites| GuiApiUpdate::UpdateSprites(sprites) );
    }

    pub fn gui_updates<'a>(&'a self) -> Option<impl Iterator<Item = (GuiId, GuiApiUpdate)> + 'a> {
        self.gui.read_values()
    }
}
