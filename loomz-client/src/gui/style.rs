use loomz_shared::{LoomzApi, RgbaU8, RectF32, assets_err};
use loomz_shared::assets::{MsdfFontId, TextureId};
use crate::gui::{Gui, GuiBuilderData, GuiLayoutType, GuiStyleState};

pub(super) struct GuiFontStyle {
    pub font: MsdfFontId,
    pub font_size: f32,
    pub color: RgbaU8
}

#[derive(Clone, Copy)]
pub(super) struct GuiFrameStyle {
    pub texture: TextureId,
    pub region: RectF32,
    pub color: RgbaU8,
}


pub struct GuiStyleBuilder<'a> {
    api: &'a LoomzApi,
    builder_data: &'a mut GuiBuilderData,
}

impl<'a> GuiStyleBuilder<'a> {

    pub fn new(api: &'a LoomzApi, gui: &'a mut Gui) -> Self {
        Self::clear_gui_styles(gui);

        GuiStyleBuilder {
            api,
            builder_data: &mut gui.builder_data,
        }
    }

    fn clear_gui_styles(gui: &mut Gui) {
        let data = &mut gui.builder_data;
        data.font_styles.clear();
        data.frame_styles.clear();
        data.root_layout_type = GuiLayoutType::VBox;
    }

    /// Sets the layout of the root elements in the gui
    pub fn root_layout(&mut self, ty: GuiLayoutType) {
        self.builder_data.root_layout_type = ty;
    }

    pub fn font(
        &mut self,
        style_key: &'static str,
        state: GuiStyleState,
        font_key: &str,
        font_size: f32,
        color: RgbaU8
    ) {
        let font = match self.api.assets_ref().font_id_by_name(font_key) {
            Some(font) => font,
            None => {
                self.builder_data.errors.push(assets_err!("No font named {:?} in app", font_key));
                return;
            }
        };

        self.builder_data.font_styles.insert(style_key, GuiFontStyle {
            font,
            font_size,
            color,
        });
    }

    pub fn frame(
        &mut self,
        style_key: &'static str,
        state: GuiStyleState,
        texture_key: &str,
        region: RectF32,
        color: RgbaU8
    ) {
        let texture = match self.api.assets_ref().texture_id_by_name(texture_key) {
            Some(texture) => texture,
            None => {
                self.builder_data.errors.push(assets_err!("No texture named {:?} in app", texture_key));
                return;
            }
        };

        self.builder_data.frame_styles.insert(style_key, GuiFrameStyle {
            texture,
            region,
            color,
        });
    }


}
