use crate::assets::MsdfFontId;
use crate::assets::msdf_font::ComputedGlyph;
use crate::api::GuiTextId;

pub struct GuiComponentText {
    pub glyphs: Vec<ComputedGlyph>,
    pub id: GuiTextId,
    pub font: MsdfFontId,
}

pub enum GuiComponentType {
    Text(GuiComponentText),
}
