use crate::assets::MsdfFontId;
use crate::LoomzApi;
use super::{Gui, GuiTextId, GuiComponentType, GuiComponentText, GuiComponentTextGlyph};

pub struct GuiBuilder<'a> {
    gui: &'a mut Gui,
    font: Option<MsdfFontId>,
    font_size: Option<f32>,
    default_font: MsdfFontId,
    default_font_size: f32,
}

impl<'a> GuiBuilder<'a> {

    pub fn new(api: &LoomzApi, gui: &'a mut Gui) -> Self {
        let default_font = api.assets_ref()
            .font_id_by_name("roboto")
            .expect("Hardcoded default font was not found");

        GuiBuilder {
            gui,
            font: None,
            font_size: None,
            default_font,
            default_font_size: 32.0,
        }
    }

    pub fn font_style(&mut self, style_key: &str, font_key: &str, font_size: f32) {
        let font = self.api().assets_ref()
            .font_id_by_name(font_key)
            .unwrap_or(self.default_font);

        self.gui.font_styles.insert(style_key.to_string(), super::GuiFontStyle {
            font,
            font_size: font_size,
        });
    }

    pub fn font(&mut self, style_key: &str) {
        match self.gui.font_styles.get(style_key) {
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
        let api = self.api();
        let font = self.font.unwrap_or(self.default_font);
        let font_size = self.font_size.unwrap_or(self.default_font_size);
        let component = build_text_component(api, text_value, font, font_size);
        api.gui().update_text_font(&component.id, component.font);
        self.gui.data.push(GuiComponentType::Text(component));
    }

    fn api(&self) -> &LoomzApi {
        match self.gui.api.as_ref() {
            Some(api) => api,
            None => unreachable!("Api will always be Some in builder")
        }
    }

}

fn build_text_component(
    api: &LoomzApi,
    text_value: &str,
    font: MsdfFontId,
    font_size: f32
) -> GuiComponentText {
    use unicode_segmentation::UnicodeSegmentation;

    let id = GuiTextId::new();
    let font_asset = api.assets_ref().font(font).unwrap();  // Font presence is validated by the builder
    let mut glyphs = Vec::with_capacity(text_value.len());
    let mut text_glyph = GuiComponentTextGlyph::default();

    let mut advance = 0.0;

    for g in text_value.graphemes(true) {
        text_glyph.offset.x = advance;
        advance += font_asset.font_data.compute_glyph(g, font_size, &mut text_glyph.glyph);
        glyphs.push(text_glyph);
    }

    GuiComponentText {
        glyphs,
        id,
        font,
    }
}
