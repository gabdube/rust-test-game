use loomz_shared::base_types::{SizeF32, RectF32, RgbaU8};
use loomz_shared::assets::{MsdfFontId, TextureId};
use loomz_shared::assets::msdf_font::ComputedGlyph;
use super::{GuiLayoutItem, GuiSprite, GuiSpriteType};

pub struct GuiComponentText {
    pub glyphs: Vec<ComputedGlyph>,
    pub color: RgbaU8,
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

    fn generate_sprites(&self, item: &GuiLayoutItem, sprites: &mut Vec<GuiSprite>) {
        let font = self.font;
        let color = self.color;
        let [mut x, mut y] = item.position.splat();

        let size = self.size();
        x += (item.size.width - size.width) / 2.0;
        y += (item.size.height - size.height) / 2.0;

        for glyph in self.glyphs.iter() {
            sprites.push(GuiSprite {
                ty: GuiSpriteType::Font(font),
                position: glyph.position.translate_into(x, y),
                texcoord: glyph.texcoord,
                color,
            });
        }
    }
}

#[derive(Copy, Clone)]
pub struct GuiComponentFrame {
    pub texture: TextureId,
    pub size: SizeF32,
    pub texcoord: RectF32,
    pub color: RgbaU8,
}

impl GuiComponentFrame {
    fn generate_sprites(&self, item: &GuiLayoutItem, sprites: &mut Vec<GuiSprite>) {
        sprites.push(GuiSprite {
            ty: GuiSpriteType::Image(self.texture),
            position: RectF32 {
                left: item.position.x, right: item.position.x + self.size.width,
                top: item.position.y, bottom: item.position.y + self.size.height,
            },
            texcoord: self.texcoord,
            color: self.color,
        });
    }
}

pub enum GuiComponentType {
    Frame(GuiComponentFrame),
    Text(GuiComponentText),
}

impl GuiComponentType {

    pub fn generate_sprites(&self, item: &GuiLayoutItem, sprites: &mut Vec<GuiSprite>) {
        match self {
            GuiComponentType::Frame(frame) => frame.generate_sprites(item, sprites),
            GuiComponentType::Text(text) => text.generate_sprites(item, sprites),
        }
    }

}
