mod component;
use component::*;
pub use component::GuiComponentTextGlyph;

mod gui_data;
pub use gui_data::Gui;
use gui_data::GuiFontStyle;

mod builder;
pub use builder::GuiBuilder;

use crate::assets::MsdfFontId;
use super::{Id, MessageQueueEx};

pub struct GuiTextTag;
pub type GuiTextId = Id<GuiTextTag>;

pub enum GuiTextUpdate {
    Font(MsdfFontId),
    Glyphs(&'static [GuiComponentTextGlyph]),
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

    pub fn update_text_glyphs(&self, id: &GuiTextId, glyphs: &[GuiComponentTextGlyph]) {        
        self.text.push_with_data(id, glyphs, |glyphs| GuiTextUpdate::Glyphs(glyphs) );
    }

    pub fn text_updates<'a>(&'a self) -> Option<impl Iterator<Item = (GuiTextId, GuiTextUpdate)> + 'a> {
        self.text.read_values()
    }
}
