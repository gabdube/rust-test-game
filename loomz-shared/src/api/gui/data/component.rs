use crate::assets::MsdfFontId;
use crate::assets::msdf_font::ComputedGlyph;
use crate::base_types::SizeF32;
use crate::api::GuiTextId;

pub struct GuiComponentText {
    pub glyphs: Vec<ComputedGlyph>,
    pub id: GuiTextId,
    pub font: MsdfFontId,
}

impl GuiComponentText {
    pub fn size(&self) -> SizeF32 {
        let mut size = SizeF32 { width: 0.0, height: 0.0 };

        for glyph in self.glyphs.iter() {
            let [width, height] = glyph.position.size();
            size.width += width;
            size.height = f32::max(size.height, height);
        }

        size
    }
}

pub enum GuiComponentType {
    Text(GuiComponentText),
}
