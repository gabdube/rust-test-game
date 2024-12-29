use crate::assets::MsdfFontId;
use crate::assets::msdf_font::ComputedGlyph;
use crate::base_types::SizeF32;
use super::{GuiLayoutView, GuiSprite, GuiSpriteType};

pub struct GuiComponentText {
    pub glyphs: Vec<ComputedGlyph>,
    pub font: MsdfFontId,
}

impl GuiComponentText {
    pub fn size(&self) -> SizeF32 {
        let mut size = SizeF32 { 
            width: 0.0,
            height: 0.0
        };

        for glyph in self.glyphs.iter() {
            size.height = f32::max(size.height, glyph.position.height());
        }

        if let Some(glyph) = self.glyphs.last() {
            size.width = glyph.position.right;
        }

        size
    }

    fn generate_sprites(&self, view: &GuiLayoutView, sprites: &mut Vec<GuiSprite>) {
        let font = self.font;
        let [x, y] = view.position.splat();
        for glyph in self.glyphs.iter() {
            sprites.push(GuiSprite {
                ty: GuiSpriteType::Font(font),
                position: glyph.position.translate_into(x, y),
                texcoord: glyph.texcoord
            });
        }
    }
}

pub enum GuiComponentType {
    Text(GuiComponentText),
}

impl GuiComponentType {

    pub fn generate_sprites(&self, view: &GuiLayoutView, sprites: &mut Vec<GuiSprite>) {
        match self {
            GuiComponentType::Text(text) => text.generate_sprites(view, sprites),
        }
    }

}
