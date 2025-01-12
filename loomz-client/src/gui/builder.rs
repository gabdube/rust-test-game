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

#[derive(Clone, Copy)]
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
    pub layouts_stack: Vec<GuiLayout>,
    pub default_font: Option<MsdfFontId>,
    pub default_font_size: f32,
}

pub struct GuiBuilder<'a> {
    api: &'a LoomzApi,
    gui: &'a mut Gui,
    layout_item: Option<GuiLayoutItem>,
    frame_style: Option<GuiFrameStyle>,
    layout_index: u32,
    item_index: u32,
    items_level: u32,
}

impl<'a> GuiBuilder<'a> {

    pub fn new(api: &'a LoomzApi, gui: &'a mut Gui) -> Self {
        if gui.builder_data.default_font_size < 1.0 {
            gui.builder_data.default_font_size = 32.0;
        }

        if gui.builder_data.default_font.is_none() {
            // Panic expected. There should always be at least a single font in the app assets
            gui.builder_data.default_font = api.assets_ref()
                .default_font_id()
                .expect("No font found in application")
                .into();
        }

        gui.builder_data.layouts_stack.clear();
        gui.builder_data.layouts_stack.push(gui.components.root_layout);

        GuiBuilder {
            api,
            gui,
            layout_item: None,
            frame_style: None,
            layout_index: 0,
            item_index: 0,
            items_level: 0,
        }
    }

    // pub fn frame_style(&mut self, style_key: &'static str, texture_key: &str, region: RectF32, color: RgbaU8) {
    //     let texture = match self.api.assets_ref().texture_id_by_name(texture_key) {
    //         Some(texture) => texture,
    //         None => {
    //             self.gui.builder_data.errors.push(assets_err!("No texture named {:?} in app", texture_key));
    //             return;
    //         }
    //     };

    //     self.gui.builder_data.frame_styles.insert(style_key, GuiFrameStyle {
    //         texture,
    //         region,
    //         color,
    //     });
    // }

    pub fn frame_style(&mut self, texture_key: &str, region: RectF32, color: RgbaU8) {
        let texture = match self.api.assets_ref().texture_id_by_name(texture_key) {
            Some(texture) => texture,
            None => {
                self.gui.builder_data.errors.push(assets_err!("No texture named {:?} in app", texture_key));
                return;
            }
        };

        self.frame_style = Some(GuiFrameStyle {
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

    /// Sets the layout of the root elements in the gui
    pub fn root_layout(&mut self, layout_type: GuiLayoutType) {
        let root_layout = self.gui.components.root_layout;
        // if root_layout.width != 0.0 || root_layout.height != 0.0 {
        //     eprintln!("Trying to overwrite root layout. Ignoring second call");
        //     return;
        // }

        self.gui.components.root_layout.ty = layout_type;
    }

    /// Sets the layout used to position the child items of the next component
    pub fn layout(&mut self, layout_type: GuiLayoutType) {
        // let next_item_index = self.gui.components.layout_items.len() as u32;
        // self.gui.components.layouts.push(GuiLayout {
        //     ty: layout_type,
        //     width: 0.0,
        //     height: 0.0,
        //     first_component: next_item_index,
        //     last_component: u32::MAX, // u32::MAX means the layout does not have any item yet
        // });

        // self.layout_index = (self.gui.components.layouts.len() as u32) - 1;
    }

    /// Sets the layout item of the next component
    pub fn layout_item(&mut self, width: f32, height: f32) {
        self.layout_item = Some(GuiLayoutItem { 
            position: PositionF32::default(),
            size: SizeF32 { width, height }
        });
    }

    /// Adds a simple text component to the gui
    pub fn label(&mut self, text_value: &str, style_key: &str) {
        let style = match self.gui.builder_data.font_styles.get(style_key) {
            Some(s) => s,
            None => {
                self.gui.builder_data.errors.push(assets_err!("No frame style with key {:?} in builder", style_key));
                return;
            }
        };

        let component = build_text_component(self.api, text_value, &style);
        let layout_item = self.next_layout_item(component.size());

        let compo = &mut self.gui.components;
        compo.layout_items.push(layout_item);
        compo.types.push(GuiComponentType::Text(component));

        self.update_layout(layout_item.size);
        self.item_index += 1;
    }

    /// Adds a frame component into the gui. If style_key is empty, use the last value of `frame_style`
    pub fn frame<F: FnOnce(&mut GuiBuilder)>(&mut self, size: SizeF32, callback: F) {
        let style = match self.frame_style {
            Some(style) => style,
            None => {
                self.gui.builder_data.errors.push(assets_err!("No pushed frame style in builder"));
                return;
            }
        };

        let frame = GuiComponentFrame {
            texture: style.texture,
            size,
            texcoord: style.region,
            color: style.color,
        };

        let layout_item = self.next_layout_item(size);

        let compo = &mut self.gui.components;
        compo.layout_items.push(layout_item);
        compo.types.push(GuiComponentType::Frame(frame));

        self.update_layout(size);

        self.layout_index += 1;
        self.item_index += 1;
        self.items_level += 1;

        callback(self);

        self.items_level -= 1;
    }

    fn next_layout_item(&mut self, size: SizeF32) -> GuiLayoutItem {
        match self.layout_item.take() {
            Some(item) => item,
            None => {
                GuiLayoutItem {
                    position: PositionF32::default(),
                    size
                }
            }
        }
    }

    fn update_layout(&mut self, item_size: SizeF32) {
        let current_layout = match self.gui.builder_data.layouts_stack.last_mut() {
            Some(layout) => layout,
            None => unreachable!("There will always be a root layout at index 0")
        };

        current_layout.children_count += 1;

        match current_layout.ty {
            GuiLayoutType::VBox => {
                current_layout.width = f32::max(current_layout.width, item_size.width);
                current_layout.height += item_size.height;
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
