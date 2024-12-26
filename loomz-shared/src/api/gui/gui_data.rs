use fnv::FnvHashMap;
use crate::{LoomzApi, MsdfFontId};
use super::{GuiBuilder, GuiComponentType};

pub(super) struct GuiFontStyle {
    pub font: MsdfFontId,
    pub font_size: f32,
}

pub struct Gui {
    pub(super) api: Option<LoomzApi>,
    pub(super) data: Vec<GuiComponentType>,
    pub(super) font_styles: FnvHashMap<String, GuiFontStyle>,
}

impl Gui {
    pub fn build<F: FnOnce(&mut GuiBuilder)>(api: &LoomzApi, cb: F) -> Self {
        let mut gui = Gui { api: Some(api.clone()), ..Default::default() };
        let mut builder = GuiBuilder::new(api, &mut gui);
        cb(&mut builder);
        gui.sync_data();
        gui
    }

    fn sync_data(&mut self) {
        let api = self.api();
        for data in self.data.iter() {
            match data {
                GuiComponentType::Text(text) => {
                    api.gui().update_text_glyphs(&text.id, &text.glyphs);
                }
            }
        }
    }

    fn api(&self) -> &LoomzApi {
        match self.api.as_ref() {
            Some(api) => api,
            None => unreachable!("Api will always be Some")
        }
    }
}

impl Default for Gui {
    fn default() -> Self {
        Gui {
            api: None,
            data: Vec::with_capacity(16),
            font_styles: FnvHashMap::default(),
        }
    }
}
