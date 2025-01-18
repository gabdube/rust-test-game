use fnv::FnvHashMap;

use loomz_shared::base_types::{PositionF32, SizeF32, RectF32, RgbaU8};
use loomz_shared::assets::{MsdfFontId, msdf_font::ComputedGlyph};
use loomz_shared::{assets_err, LoomzApi, TextureId};
use crate::gui::{Gui, GuiComponents, component::*, layout::*};

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

pub(super) struct GuiBuilderData {
    pub errors: Vec<crate::CommonError>,
    pub font_styles: FnvHashMap<&'static str, GuiFontStyle>,
    pub layouts_stack: Vec<(usize, GuiLayout)>,
    pub default_font: Option<MsdfFontId>,
    pub default_font_size: f32,
}

pub struct GuiBuilder<'a> {
    api: &'a LoomzApi,
    builder_data: &'a mut GuiBuilderData,
    components: &'a mut GuiComponents,
    layout_item: GuiLayoutItem,
    frame_style: Option<GuiFrameStyle>,
    next_layout: GuiLayoutType,
    item_index: u32,
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
        gui.builder_data.layouts_stack.push((0, GuiLayout::default()));
        gui.components.layouts.push(GuiLayout::default());

        GuiBuilder {
            api,
            builder_data: &mut gui.builder_data,
            components: &mut gui.components,
            layout_item: GuiLayoutItem::default(),
            frame_style: None,
            next_layout: GuiLayoutType::VBox,
            item_index: 0,
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
                self.builder_data.errors.push(assets_err!("No texture named {:?} in app", texture_key));
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

    /// Sets the layout of the root elements in the gui
    pub fn root_layout(&mut self, layout_type: GuiLayoutType) {
        let root = match self.builder_data.layouts_stack.get_mut(0) {
            Some((_, root)) => root,
            None => unreachable!("Root layout will always be present")
        };

        root.ty = layout_type;
    }

    /// Sets the layout used to position the child items of the next component
    pub fn layout(&mut self, layout_type: GuiLayoutType) {
        self.next_layout = layout_type;
    }

    /// Sets the layout item of the next component
    pub fn layout_item(&mut self, width: f32, height: f32) {
        self.layout_item = GuiLayoutItem {
            has_layout: false,
            position: PositionF32::default(),
            size: SizeF32 { width, height }
        };
    }

    /// Adds a simple text component to the gui
    pub fn label(&mut self, text_value: &str, style_key: &str) {
        let style = match self.builder_data.font_styles.get(style_key) {
            Some(s) => s,
            None => {
                self.builder_data.errors.push(assets_err!("No frame style with key {:?} in builder", style_key));
                return;
            }
        };

        let component = build_text_component(self.api, text_value, &style);
        let layout_item = self.layout_item;

        let compo = &mut self.components;
        compo.layout_items.push(layout_item);
        compo.types.push(GuiComponentType::Text(component));

        self.update_layout(layout_item.size);
        self.item_index += 1;
    }

    /// Adds a frame component into the gui using the last defined frame style
    pub fn frame<F: FnOnce(&mut GuiBuilder)>(&mut self, callback: F) {
        let style = match self.frame_style {
            Some(style) => style,
            None => {
                self.builder_data.errors.push(assets_err!("No pushed frame style in builder"));
                return;
            }
        };

        let mut item = self.layout_item;
        item.has_layout = true;

        let frame = GuiComponentFrame {
            texture: style.texture,
            size: item.size,
            texcoord: style.region,
            color: style.color,
        };

        let compo = &mut self.components;
        compo.layout_items.push(item);
        compo.types.push(GuiComponentType::Frame(frame));

        self.update_layout(item.size);
        self.push_next_layout();

        self.item_index += 1;

        callback(self);

        self.store_layout();
    }

    fn update_layout(&mut self, item_size: SizeF32) {
        let current_layout = match self.builder_data.layouts_stack.last_mut() {
            Some((_, layout)) => layout,
            None => unreachable!("There will always be a layout")
        };

        current_layout.children_count += 1;

        match current_layout.ty {
            GuiLayoutType::VBox => {
                current_layout.width = f32::max(current_layout.width, item_size.width);
                current_layout.height += item_size.height;
            }
        }
    }

    fn push_next_layout(&mut self) {
        let next_layout_index = self.components.layouts.len();
        let new_layout = GuiLayout {
            ty: self.next_layout,
            width: 0.0,
            height: 0.0,
            children_count: 0,
        };

        self.components.layouts.push(new_layout);
        self.builder_data.layouts_stack.push((next_layout_index, new_layout));
    }

    fn store_layout(&mut self) {
        let (layout_index, new_layout) = match self.builder_data.layouts_stack.pop() {
            Some(value) => value,
            None => unreachable!("There must always be at least 2 layouts (root+new_layout) in the stack when calling this function")
        };

        self.components.layouts[layout_index] = new_layout;
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

impl Default for GuiBuilderData {

    fn default() -> Self {
        GuiBuilderData {
            errors: Vec::with_capacity(0),
            font_styles: FnvHashMap::default(),
            layouts_stack: Vec::with_capacity(4),
            default_font: None,
            default_font_size: 0.0,
        }
    }

}
