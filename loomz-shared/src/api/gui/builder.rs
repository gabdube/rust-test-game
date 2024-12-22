use crate::LoomzApi;

pub struct GuiBuilder {
    api: LoomzApi
}

impl GuiBuilder {

    pub fn new(api: &LoomzApi) -> Self {
        GuiBuilder {
            api: api.clone(),
        }
    }

    pub fn font_style(&mut self, style_key: &str, font_key: &str, font_size: f32) {

    }

    pub fn font(&mut self, style_key: &str) {

    }

    pub fn text(&mut self, value: &str) {

    }

}
