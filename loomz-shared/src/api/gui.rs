use crate::base_types::RectF32;
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
}

pub struct GuiApi {
    gui: MessageQueueEx<GuiId, &'static [GuiSprite]>
}

impl GuiApi {
    pub fn init() -> Self {
        GuiApi {
            gui: MessageQueueEx::with_capacity(8, 5120),
        }
    }

    pub fn update_gui(&self, id: &GuiId, sprites: &[GuiSprite]) {
        if sprites.len() > 0 {
            self.gui.push_with_data(id, sprites, |sprites| sprites );
        }
    }

    pub fn gui_updates<'a>(&'a self) -> Option<impl Iterator<Item = (GuiId, &'static [GuiSprite])> + 'a> {
        self.gui.read_values()
    }
}
