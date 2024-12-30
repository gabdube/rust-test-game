use loomz_shared::base_types::{SizeF32, RectF32};
use loomz_shared::assets::{MsdfFontId, TextureId};
use loomz_shared::assets::msdf_font::ComputedGlyph;
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

pub struct GuiComponentFrame {
    pub texture: TextureId,
    pub size: SizeF32,
    pub texcoord: RectF32
}

impl GuiComponentFrame {
    fn generate_sprites(&self, view: &GuiLayoutView, sprites: &mut Vec<GuiSprite>) {
        sprites.push(GuiSprite {
            ty: GuiSpriteType::Image(self.texture),
            position: RectF32 {
                left: view.position.x, right: view.position.x + self.size.width,
                top: view.position.y, bottom: view.position.y + self.size.height,
            },
            texcoord: self.texcoord
        });
    }
}

pub enum GuiComponentType {
    Frame(GuiComponentFrame),
    Text(GuiComponentText),
}

impl GuiComponentType {

    pub fn generate_sprites(&self, view: &GuiLayoutView, sprites: &mut Vec<GuiSprite>) {
        match self {
            GuiComponentType::Frame(frame) => frame.generate_sprites(view, sprites),
            GuiComponentType::Text(text) => text.generate_sprites(view, sprites),
        }
    }

}
