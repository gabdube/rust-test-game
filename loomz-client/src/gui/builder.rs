use loomz_shared::base_types::{PositionF32, SizeF32, RectF32};
use loomz_shared::assets::msdf_font::ComputedGlyph;
use loomz_shared::{LoomzApi, assets_err};
use super::{
    components::*,
    layout::*,
    style::*,
    callbacks::{IntoGuiCallback, GuiComponentCallbacksValue},
    Gui, GuiInnerState
};

pub struct GuiBuilder<'a> {
    api: &'a LoomzApi,
    gui: &'a mut GuiInnerState,
    layout_item: GuiLayoutItem,
    next_layout: GuiLayoutType,
    item_index: u32,
}

impl<'a> GuiBuilder<'a> {

    pub fn new(api: &'a LoomzApi, view: &RectF32, gui: &'a mut Gui) -> Self {
        Self::clear_gui_components(view, gui);

        GuiBuilder {
            api,
            gui: &mut gui.inner_state,
            layout_item: GuiLayoutItem::default(),
            next_layout: GuiLayoutType::VBox,
            item_index: 0,
        }
    }

    fn clear_gui_components(view: &RectF32, gui: &mut Gui) {
        let inner = &mut gui.inner_state;
        inner.base_view = *view;
        inner.state = Default::default();
        inner.layouts.clear();
        inner.callbacks.clear();
        inner.layout_items.clear();
        inner.component_base.clear();
        inner.component_data.clear();
        inner.sprites.clear();

        let builder_data = &mut inner.builder_data;
        builder_data.layouts_stack.clear();
        builder_data.layouts_stack.push((0, GuiLayout::default()));
        builder_data.last_callbacks = GuiComponentCallbacksValue::None;

        inner.layouts.push(GuiLayout::default());
    }

    /// Sets the layout used to position the child items of the next component
    pub fn layout(&mut self, layout_type: GuiLayoutType) {
        self.next_layout = layout_type;
    }

    /// Sets the layout item of the next components
    pub fn layout_item(&mut self, width: f32, height: f32) {
        self.layout_item = GuiLayoutItem {
            has_layout: false,
            position: PositionF32::default(),
            size: SizeF32 { width, height }
        };
    }

    pub fn label_callback<ID: IntoGuiCallback>(&mut self, _callback: GuiLabelCallback, callback_id: ID) {
        let click = callback_id.into_u64();
        self.gui.builder_data.last_callbacks = GuiComponentCallbacksValue::Label(GuiLabelCallbackValues { click });
    }

    /// Adds a simple text component to the gui
    pub fn label(&mut self, text_value: &str, style_key: &str) {
        let gui = &mut self.gui;
        let builder_data = &mut gui.builder_data;

        // Layout item
        gui.layout_items.push(self.layout_item);

        // Component base
        let callbacks_index = match builder_data.last_callbacks.take() {
            cb @ GuiComponentCallbacksValue::Label(_) => {
                gui.callbacks.push(cb);
                (gui.callbacks.len() - 1) as u32
            },
            _ => u32::MAX,
        };

        let style_key = (style_key, GuiComponentTag::Label);
        let style_index = match builder_data.styles.get(&style_key) {
            Some(style_index) => *style_index,
            None => {
                builder_data.errors.push(assets_err!("No label style with key {:?} in builder", style_key.0));
                return;
            }
        };

        gui.component_base.push(GuiComponentBase {
            callbacks_index,
            style_index,
        });
        
        // Component data
        let style = match gui.styles.get(style_index as usize) {
            Some(GuiComponentStyle::Label(label_style)) => &label_style.base,
            _ => unreachable!("GuiComponentStyle cannot be something else than Font")
        };
        let label = build_label_component(self.api, text_value, style);
        gui.component_data.push(GuiComponentData::Label(label));

        self.update_layout(self.layout_item.size);
        self.item_index += 1;
    }

    /// Adds a frame component into the gui using the last defined frame style
    pub fn frame<F: FnOnce(&mut GuiBuilder)>(&mut self, style_key: &'static str, callback: F) {
        let gui = &mut self.gui;
        let builder_data = &mut gui.builder_data;

        // Layout item
        let mut item = self.layout_item;
        item.has_layout = true;
        gui.layout_items.push(item);

        // Component base
        let style_key = (style_key, GuiComponentTag::Frame);
        let style_index = match builder_data.styles.get(&style_key) {
            Some(style_index) => *style_index,
            None => {
                builder_data.errors.push(assets_err!("No frame style with key {:?} in builder", style_key.0));
                return;
            }
        };
        gui.component_base.push(GuiComponentBase {
            callbacks_index: u32::MAX,
            style_index,
        });

        // Component data
        let style = match gui.styles.get(style_index as usize) {
            Some(GuiComponentStyle::Frame(frame_style)) => frame_style.base,
            _ => unreachable!("GuiComponentStyle cannot be something else than Frame")
        };
        let frame = GuiFrame {
            texture: style.texture,
            size: item.size,
            texcoord: style.region,
            color: style.color,
        };
        gui.component_data.push(GuiComponentData::Frame(frame));


        self.update_layout(item.size);
        self.push_next_layout();

        self.item_index += 1;

        callback(self);

        self.store_layout();
    }

    fn update_layout(&mut self, item_size: SizeF32) {
        let current_layout = match self.gui.builder_data.layouts_stack.last_mut() {
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
        let next_layout_index = self.gui.layouts.len();
        let new_layout = GuiLayout {
            ty: self.next_layout,
            width: 0.0,
            height: 0.0,
            children_count: 0,
        };

        self.gui.layouts.push(new_layout);
        self.gui.builder_data.layouts_stack.push((next_layout_index, new_layout));
    }

    fn store_layout(&mut self) {
        let (layout_index, new_layout) = match self.gui.builder_data.layouts_stack.pop() {
            Some(value) => value,
            None => unreachable!("There must always be at least 2 layouts (root+new_layout) in the stack when calling this function")
        };

        self.gui.layouts[layout_index] = new_layout;
    }

}

fn build_label_component(
    api: &LoomzApi,
    text_value: &str,
    style: &GuiLabelStyle,
) -> GuiLabel {
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

    GuiLabel {
        glyphs: glyphs.into_boxed_slice(),
        font: style.font,
        color: style.color
    }
}
