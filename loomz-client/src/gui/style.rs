use fnv::FnvHashMap;
use loomz_shared::{LoomzApi, RgbaU8, RectF32, assets_err};
use loomz_shared::assets::{MsdfFontId, TextureId};
use crate::gui::{Gui, GuiBuilderData, GuiLayoutType};


pub(super) type GuiStyleMap = FnvHashMap<&'static str, u32>;

#[derive(Clone, Copy)]
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

#[derive(Copy, Clone)]
pub enum GuiStyleState {
    Base,
    Hovered,
    Active
}

pub(super) struct GuiComponentStyleBase<T: Copy> {
    pub base: T,
    pub hovered: T,
    pub active: T,
}

pub(super) enum GuiComponentStyle {
    Font(GuiComponentStyleBase<GuiFontStyle>),
    Frame(GuiComponentStyleBase<GuiFrameStyle>)
}

pub struct GuiStyleBuilder<'a> {
    api: &'a LoomzApi,
    builder_data: &'a mut GuiBuilderData,
    styles: &'a mut Vec<GuiComponentStyle>
}

impl<'a> GuiStyleBuilder<'a> {

    pub fn new(api: &'a LoomzApi, gui: &'a mut Gui) -> Self {
        Self::clear_gui_styles(gui);

        GuiStyleBuilder {
            api,
            builder_data: &mut gui.builder_data,
            styles: &mut gui.inner_state.styles,
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

        let font_style_value = GuiFontStyle {
            font,
            font_size,
            color,
        };

        if let Some(index) = self.builder_data.font_styles.get(style_key) {
            let style_index = *index as usize;
            match &mut self.styles[style_index] {
                GuiComponentStyle::Font(font_style) => update_style(state, font_style, font_style_value),
                _ => unreachable!("Style type is enforced by the code")
            };
        } else {
            let style_index = self.styles.len();
            self.builder_data.font_styles.insert(style_key, style_index as u32);
            self.styles.push(GuiComponentStyle::Font(GuiComponentStyleBase {
                base: font_style_value,
                hovered: font_style_value,
                active: font_style_value,
            }))
        }
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

        let frame_style_value = GuiFrameStyle {
            texture,
            region,
            color,
        };

        if let Some(index) = self.builder_data.frame_styles.get(style_key) {
            let style_index = *index as usize;
            match &mut self.styles[style_index] {
                GuiComponentStyle::Frame(frame_style) => update_style(state, frame_style, frame_style_value),
                _ => unreachable!("Style type is enforced by the code")
            };
        } else {
            let style_index = self.styles.len();
            self.builder_data.frame_styles.insert(style_key, style_index as u32);
            self.styles.push(GuiComponentStyle::Frame(GuiComponentStyleBase {
                base: frame_style_value,
                hovered: frame_style_value,
                active: frame_style_value,
            }))
        }
    }

}

fn update_style<T: Copy>(state: GuiStyleState, style: &mut GuiComponentStyleBase<T>, value: T) {
    let style = match state {
        GuiStyleState::Base => &mut style.base,
        GuiStyleState::Hovered => &mut style.hovered,
        GuiStyleState::Active => &mut style.active,
    };

    *style = value;
}