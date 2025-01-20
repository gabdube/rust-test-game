use loomz_shared::base_types::{SizeF32, RgbaU8};
use loomz_shared::assets::MsdfFontId;
use loomz_shared::assets::msdf_font::ComputedGlyph;
use crate::gui::{GuiLayoutItem, GuiSprite, GuiSpriteType, GuiComponentStyle, GuiStyleState,
  GuiComponentCallbacksValue, RawCallbackValue, GuiInnerEvent};

#[derive(Copy, Clone)]
pub enum GuiLabelCallback {
    Click
}

#[derive(Copy, Clone)]
pub struct GuiLabelCallbackValues {
    pub click: u64,
}

#[derive(Clone, Copy)]
pub struct GuiLabelStyle {
    pub font: MsdfFontId,
    pub font_size: f32,
    pub color: RgbaU8
}

pub struct GuiLabel {
    pub glyphs: Box<[ComputedGlyph]>,
    pub color: RgbaU8,
    pub font: MsdfFontId,
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

    pub fn generate_sprites(&self, item: &GuiLayoutItem, sprites: &mut Vec<GuiSprite>) {
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

    pub fn update_style(&mut self, style: &GuiComponentStyle, new_state: GuiStyleState) {
        let style = match style {
            GuiComponentStyle::Label(label_style) => label_style,
            _ => unreachable!("Styles are always valid")
        };

        let style = match new_state {
            GuiStyleState::Base => style.base,
            GuiStyleState::Hovered => style.hovered,
            GuiStyleState::Selected => style.selected,
        };

        // Note: Font change not supported because recomputing the glyph would be a pain in the ass
        self.color = style.color;
    }

    pub fn on_events(&mut self, callbacks: &GuiComponentCallbacksValue, output: &mut Vec<RawCallbackValue>, event: GuiInnerEvent) {
        let callbacks_value = match callbacks {
            GuiComponentCallbacksValue::Label(values) => values,
            _ => unreachable!("Callback types are enforced by the builder")
        };
        
        match event {
            GuiInnerEvent::Click => {
                output.push(callbacks_value.click);
            }
        }
    }
}

