
use loomz_shared::base_types::{SizeF32, RectF32, RgbaU8};
use loomz_shared::assets::TextureId;
use super::{GuiLayoutItem, GuiSprite, GuiSpriteType, GuiComponentStyle, GuiStyleState};

#[derive(Clone, Copy)]
pub(crate) struct GuiFrameStyle {
    pub texture: TextureId,
    pub region: RectF32,
    pub color: RgbaU8,
}

#[derive(Copy, Clone)]
pub(crate) struct GuiFrame {
    pub texture: TextureId,
    pub size: SizeF32,
    pub texcoord: RectF32,
    pub color: RgbaU8,
}

impl GuiFrame {
    pub fn generate_sprites(&self, item: &GuiLayoutItem, sprites: &mut Vec<GuiSprite>) {
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

    pub fn update_style(&mut self, style: &GuiComponentStyle, new_state: GuiStyleState) {
        let style = match style {
            GuiComponentStyle::Frame(frame_style) => frame_style,
            _ => unreachable!("Styles are always valid")
        };

        let style = match new_state {
            GuiStyleState::Base => style.base,
            GuiStyleState::Hovered => style.hovered,
            GuiStyleState::Selected => style.selected,
        };

        self.texture = style.texture;
        self.texcoord = style.region;
        self.color = style.color;
    }
}
