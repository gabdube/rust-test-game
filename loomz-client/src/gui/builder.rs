use fnv::FnvHashMap;

use loomz_shared::base_types::{PositionF32, SizeF32, RectF32, RgbaU8};
use loomz_shared::assets::{MsdfFontId, msdf_font::ComputedGlyph};
use loomz_shared::{assets_err, LoomzApi, TextureId};
use super::{Gui, component::*, layout::*};

pub(super) struct GuiFontStyle {
    pub font: MsdfFontId,
    pub font_size: f32,
    pub color: RgbaU8
}

pub(super) struct GuiFrameStyle {
    pub texture: TextureId,
    pub region: RectF32,
    pub color: RgbaU8,
}

#[derive(Default)]
pub(super) struct GuiBuilderData {
    pub errors: Vec<crate::CommonError>,
    pub font_styles: FnvHashMap<&'static str, GuiFontStyle>,
    pub frame_styles: FnvHashMap<&'static str, GuiFrameStyle>,
    pub default_font: Option<MsdfFontId>,
    pub default_font_size: f32,
}

pub struct GuiBuilder<'a> {
    api: &'a LoomzApi,
    gui: &'a mut Gui,
    layout_index: usize,
}

impl<'a> GuiBuilder<'a> {

    pub fn new(api: &'a LoomzApi, gui: &'a mut Gui) -> Self {
        if gui.builder_data.default_font_size < 1.0 {
            gui.builder_data.default_font_size = 32.0;
        }

        if gui.builder_data.default_font.is_none() {
            gui.builder_data.default_font = api.assets_ref()
                .default_font_id()
                .expect("No font found in application")
                .into();
        }

        GuiBuilder {
            api,
            gui,
            layout_index: 0,
        }
    }

    pub fn frame_style(&mut self, style_key: &'static str, texture_key: &str, region: RectF32, color: RgbaU8) {
        let texture = match self.api.assets_ref().texture_id_by_name(texture_key) {
            Some(texture) => texture,
            None => {
                self.gui.builder_data.errors.push(assets_err!("No texture named {:?} in app", texture_key));
                return;
            }
        };

        self.gui.builder_data.frame_styles.insert(style_key, GuiFrameStyle {
            texture,
            region,
            color,
        });
    }

    pub fn font_style(&mut self, style_key: &'static str, font_key: &str, font_size: f32, color: RgbaU8) {
        let font = match self.api.assets_ref().font_id_by_name(font_key) {
            Some(font) => font,
            None => {
                self.gui.builder_data.errors.push(assets_err!("No font named {:?} in app", font_key));
                return;
            }
        };

        self.gui.builder_data.font_styles.insert(style_key, GuiFontStyle {
            font,
            font_size,
            color,
        });
    }

    pub fn vbox_layout(&mut self, width: f32, height: f32) {
        self.layout_index = self.gui.components.layouts.len();
        self.gui.components.layouts.push(GuiLayout {
            ty: GuiLayoutType::VBox,
            width,
            height,
            first_component: self.gui.components.layout_items.len() as u32,
            component_count: 0,
        });
    }

    pub fn label(&mut self, text_value: &str, style_key: &str) {
        self.update_layout();

        let style = match self.gui.builder_data.font_styles.get(style_key) {
            Some(s) => s,
            None => {
                self.gui.builder_data.errors.push(assets_err!("No frame style with key {:?} in builder", style_key));
                return;
            }
        };

        let component = build_text_component(self.api, text_value, &style);
        let item = super::GuiLayoutItem {
            position: PositionF32::default(),
            size: component.size()
        };

        let compo = &mut self.gui.components;
        compo.layout_items.push(item);
        compo.types.push(GuiComponentType::Text(component));
        
    }

    pub fn frame<F: FnOnce(&mut GuiBuilder)>(&mut self, style_key: &'static str, size: SizeF32, callback: F) {
        self.update_layout();
        
        let style = match self.gui.builder_data.frame_styles.get(style_key) {
            Some(s) => s,
            None => {
                self.gui.builder_data.errors.push(assets_err!("No frame style with key {:?} in builder", style_key));
                return;
            }
        };

        let frame = GuiComponentFrame {
            texture: style.texture,
            size,
            texcoord: style.region,
            color: style.color,
        };

        let item = super::GuiLayoutItem {
            position: PositionF32::default(),
            size
        };

        let compo = &mut self.gui.components;
        compo.layout_items.push(item);
        compo.types.push(GuiComponentType::Frame(frame));

        self.layout_index += 1;

        callback(self);
    }

    fn update_layout(&mut self) {
        match self.gui.components.layouts.get_mut(self.layout_index) {
            Some(layout) => { layout.component_count += 1; },
            None => {
                self.gui.components.layouts.push(GuiLayout {
                    ty: GuiLayoutType::VBox,
                    width: 0.0,
                    height: 0.0,
                    first_component: self.gui.components.layout_items.len() as u32,
                    component_count: 1,
                })
            }
        }
    }
}

fn build_text_component(
    api: &LoomzApi,
    text_value: &str,
    style: &GuiFontStyle,
) -> GuiComponentText {
    use unicode_segmentation::UnicodeSegmentation;

    let font_asset = match api.assets_ref().font(style.font) {
        Some(font) => font,
        None => unreachable!("Font presence is validated by the builder")
    };

    let mut glyphs = Vec::with_capacity(text_value.len());

    let scale = style.font_size;
    let mut advance = 0.0;
    let mut glyph = ComputedGlyph::default();

    for g in text_value.graphemes(true) {
        let a = font_asset.font_data.compute_glyph(g, scale, &mut glyph);
        glyph.position.left += advance;
        glyph.position.right += advance;

        advance += a;
        glyphs.push(glyph);
    }

    GuiComponentText {
        glyphs,
        font: style.font,
        color: style.color,
    }
}
