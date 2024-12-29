use fnv::FnvHashMap;

use crate::base_types::PositionF32;
use crate::assets::{MsdfFontId, msdf_font::ComputedGlyph};
use crate::LoomzApi;
use super::{Gui, component::*};

pub(super) struct GuiFontStyle {
    pub font: MsdfFontId,
    pub font_size: f32,
}

#[derive(Default)]
pub(super) struct GuiBuilderData {
    pub font_styles: FnvHashMap<String, GuiFontStyle>,
    pub default_font: Option<MsdfFontId>,
    pub default_font_size: f32,
}

pub struct GuiBuilder<'a> {
    api: &'a LoomzApi,
    gui: &'a mut Gui,
    font: Option<MsdfFontId>,
    font_size: Option<f32>,
}

impl<'a> GuiBuilder<'a> {

    pub fn new(api: &'a LoomzApi, gui: &'a mut Gui) -> Self {
        if gui.builder_data.default_font_size < 1.0 {
            gui.builder_data.default_font_size = 32.0;
        }

        if gui.builder_data.default_font.is_none() {
            gui.builder_data.default_font = api.assets_ref()
                .font_id_by_name("roboto")
                .expect("Default font \"roboto\" not found")
                .into();
        }

        GuiBuilder {
            api,
            gui,
            font: None,
            font_size: None,
        }
    }

    pub fn font_style(&mut self, style_key: &str, font_key: &str, font_size: f32) {
        let font = self.api.assets_ref()
            .font_id_by_name(font_key)
            .expect("Font not found in assets");

        self.gui.builder_data.font_styles.insert(style_key.to_string(), GuiFontStyle {
            font,
            font_size: font_size,
        });
    }

    pub fn font(&mut self, style_key: &str) {
        match self.gui.builder_data.font_styles.get(style_key) {
            Some(style) => {
                self.font = Some(style.font);
                self.font_size = Some(style.font_size);
            },
            None => {
                self.font = None;
                self.font_size = None;
            }
        }
    }

    pub fn label(&mut self, text_value: &str) {
        let font = self.get_font();
        let font_size = self.get_font_size();

        let component = build_text_component(self.api, text_value, font, font_size);
        let view = super::GuiLayoutView { position: PositionF32::default(), size: component.size() };

        let compo = &mut self.gui.components;
        compo.layouts.push(super::GuiLayout::default());
        compo.views.push(view);
        compo.types.push(GuiComponentType::Text(component));
    }

    fn get_font(&self) -> MsdfFontId {
        self.font.or_else(|| self.gui.builder_data.default_font ).expect("Default for was not set")
    }

    fn get_font_size(&self) -> f32 {
        self.font_size.unwrap_or_else(|| { self.gui.builder_data.default_font_size } )
    }

}

fn build_text_component(
    api: &LoomzApi,
    text_value: &str,
    font: MsdfFontId,
    font_size: f32
) -> GuiComponentText {
    use unicode_segmentation::UnicodeSegmentation;

    let font_asset = api.assets_ref().font(font).unwrap();  // Font presence is validated by the builder
    let mut glyphs = Vec::with_capacity(text_value.len());

    let mut advance = 0.0;
    let mut glyph = ComputedGlyph::default();

    for g in text_value.graphemes(true) {
        let a = font_asset.font_data.compute_glyph(g, font_size, &mut glyph);
        glyph.position.left += advance;
        glyph.position.right += advance;

        advance += a;
        glyphs.push(glyph);
    }

    GuiComponentText {
        glyphs,
        font,
    }
}
