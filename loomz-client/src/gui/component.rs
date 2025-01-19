use loomz_shared::base_types::{SizeF32, RectF32, RgbaU8};
use loomz_shared::assets::{MsdfFontId, TextureId};
use loomz_shared::assets::msdf_font::ComputedGlyph;
use super::{GuiLayoutItem, GuiSprite, GuiSpriteType, GuiComponentStyle, GuiStyleState};

#[derive(Clone, Copy)]
pub(super) struct GuiLabelStyle {
    pub font: MsdfFontId,
    pub font_size: f32,
    pub color: RgbaU8
}

pub struct GuiLabel {
    pub glyphs: Box<[ComputedGlyph]>,
    pub color: RgbaU8,
    pub font: MsdfFontId,
    pub style_index: u32,
}

impl GuiLabel {
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

    fn update_style(&mut self, styles: &Vec<GuiComponentStyle>, new_state: GuiStyleState) {
        let style = match styles.get(self.style_index as usize) {
            Some(GuiComponentStyle::Label(label_style)) => label_style,
            _ => unreachable!("Styles are always valid")
        };

        let style = match new_state {
            GuiStyleState::Base => style.base,
            GuiStyleState::Hovered => style.hovered,
            GuiStyleState::Active => style.active,
        };

        // Note: Font change not supported because recomputing the glyph would be a pain in the ass
        self.color = style.color;
    }
}

#[derive(Clone, Copy)]
pub(super) struct GuiFrameStyle {
    pub texture: TextureId,
    pub region: RectF32,
    pub color: RgbaU8,
}

#[derive(Copy, Clone)]
pub struct GuiFrame {
    pub texture: TextureId,
    pub size: SizeF32,
    pub texcoord: RectF32,
    pub color: RgbaU8,
    pub style_index: u32,
}

impl GuiFrame {
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

    fn update_style(&mut self, styles: &Vec<GuiComponentStyle>, new_state: GuiStyleState) {
        let style = match styles.get(self.style_index as usize) {
            Some(GuiComponentStyle::Frame(frame_style)) => frame_style,
            _ => unreachable!("Styles are always valid")
        };

        let style = match new_state {
            GuiStyleState::Base => style.base,
            GuiStyleState::Hovered => style.hovered,
            GuiStyleState::Active => style.active,
        };

        self.texture = style.texture;
        self.texcoord = style.region;
        self.color = style.color;
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum GuiComponentTag {
    Frame,
    Label
}

pub enum GuiComponentType {
    Frame(GuiFrame),
    Label(GuiLabel),
}

impl GuiComponentType {

    pub fn generate_sprites(&self, item: &GuiLayoutItem, sprites: &mut Vec<GuiSprite>) {
        match self {
            GuiComponentType::Frame(frame) => frame.generate_sprites(item, sprites),
            GuiComponentType::Label(label) => label.generate_sprites(item, sprites),
        }
    }

    pub fn update_style(&mut self, styles: &Vec<GuiComponentStyle>, new_state: GuiStyleState) {
        match self {
            GuiComponentType::Frame(frame) => frame.update_style(styles, new_state),
            GuiComponentType::Label(label) => label.update_style(styles, new_state),
        }
    }

}
