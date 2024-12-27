mod data;
pub use data::Gui;

use crate::assets::{MsdfFontId, msdf_font::ComputedGlyph};
use super::{Id, MessageQueueEx};

pub struct GuiTextTag;
pub type GuiTextId = Id<GuiTextTag>;

pub enum GuiTextUpdate {
    Font(MsdfFontId),
    Glyphs(&'static [ComputedGlyph]),
}

pub struct GuiApi {
    text: MessageQueueEx<GuiTextId, GuiTextUpdate>
}

impl GuiApi {
    pub fn init() -> Self {
        GuiApi {
            text: MessageQueueEx::with_capacity(16, 1024),
        }
    }

    pub fn update_text_font(&self, id: &GuiTextId, font: MsdfFontId) {
        self.text.push(id, GuiTextUpdate::Font(font));
    }

    pub fn update_text_glyphs(&self, id: &GuiTextId, glyphs: &[ComputedGlyph]) {        
        self.text.push_with_data(id, glyphs, |glyphs| GuiTextUpdate::Glyphs(glyphs) );
    }

    pub fn text_updates<'a>(&'a self) -> Option<impl Iterator<Item = (GuiTextId, GuiTextUpdate)> + 'a> {
        self.text.read_values()
    }
}
