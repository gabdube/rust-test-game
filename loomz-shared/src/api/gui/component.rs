use crate::assets::MsdfFontId;
use crate::assets::msdf_font::ComputedGlyph;
use crate::base_types::_2d::Position;
use super::GuiTextId;

#[derive(Copy, Clone, Default)]
pub struct GuiComponentTextGlyph {
    pub offset: Position<f32>,
    pub glyph: ComputedGlyph,
}

pub(super) struct GuiComponentText {
    pub glyphs: Vec<GuiComponentTextGlyph>,
    pub id: GuiTextId,
    pub font: MsdfFontId,
}

pub(super) enum GuiComponentType {
    Text(GuiComponentText),
}
