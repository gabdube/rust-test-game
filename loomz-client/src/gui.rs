mod style;
use style::{GuiStyleMap, GuiComponentStyle};
pub use style::{GuiStyleBuilder, GuiStyleState};

mod callbacks;
use callbacks::{IntoGuiCallback, GuiComponentCallbacksValue, RawCallbackValue};

mod components;
use components::{GuiLabel, GuiComponentBase, GuiComponentData};
pub use components::GuiLabelCallback;

mod layout;
use layout::{GuiLayout, GuiLayoutItem};
pub use layout::GuiLayoutType;

mod builder;
use builder::GuiBuilder;

use loomz_shared::base_types::{RectF32, PositionF32};
use loomz_shared::api::{LoomzApi, GuiId, GuiSprite, GuiSpriteType};
use loomz_shared::store::*;
use loomz_shared::{CommonError, client_err};

#[derive(Copy, Clone)]
enum GuiInnerEvent {
    Click
}

#[derive(Copy, Clone)]
struct GuiComponentState {
    hovered_index: u32,
    selected_index: u32,
}

#[derive(Default, Copy, Clone)]
pub struct GuiUpdates {
    pub cursor_position: Option<PositionF32>,
    pub view: Option<RectF32>,
    pub left_mouse_down: Option<bool>,
}

struct GuiBuilderData {
    errors: Vec<crate::CommonError>,
    layouts_stack: Vec<(usize, GuiLayout)>,
    styles: GuiStyleMap,
    last_callbacks: GuiComponentCallbacksValue,
    root_layout_type: GuiLayoutType,
}

pub struct Gui {
    state: GuiComponentState,
    styles: Vec<GuiComponentStyle>,
    callbacks: Vec<GuiComponentCallbacksValue>,
    callbacks_output: Vec<RawCallbackValue>,

    base_view: RectF32,
    layouts: Vec<GuiLayout>,
    layout_items: Vec<GuiLayoutItem>,
    component_base: Vec<GuiComponentBase>,
    component_data: Vec<GuiComponentData>,

    id: GuiId,
    sprites: Vec<GuiSprite>,

    builder_data: Box<GuiBuilderData>,

    visible: bool,
}

impl Gui {

    pub fn build_style<F: FnOnce(&mut GuiStyleBuilder)>(&mut self, api: &LoomzApi, cb: F) -> Result<(), CommonError> {
        let mut builder = GuiStyleBuilder::new(api, self);
        cb(&mut builder);

        self.check_errors()?;

        Ok(())
    }

    pub fn build<F: FnOnce(&mut GuiBuilder)>(&mut self, api: &LoomzApi, view: &RectF32, cb: F) -> Result<(), CommonError> {
        let mut builder = GuiBuilder::new(api, view, self);
        cb(&mut builder);

        self.check_errors()?;

        self.layouts[0] = self.get_root_layout();

        layout::compute(self);

        self.sync_with_engine(api);

        Ok(())
    }

    pub fn update(&mut self, api: &LoomzApi, updates: &GuiUpdates) {
        let mut need_sync = false;

        if let Some(view) = updates.view {
            self.inner_resize(&view);
            need_sync = true;
        }

        if let Some(cursor_position) = updates.cursor_position {
            self.update_cursor_position(cursor_position, &mut need_sync);
        }

        if let Some(left_button) = updates.left_mouse_down {
            self.update_mouse_button(left_button, &mut need_sync);
        }

        if need_sync {
            self.sync_with_engine(api);
        }
    }

    pub fn resize(&mut self, api: &LoomzApi, view: &RectF32) {
        self.inner_resize(view);
        self.sync_with_engine(api);
    }

    pub fn drain_events<'a, E: IntoGuiCallback>(&'a mut self) -> impl Iterator<Item=E> + 'a {
        let callback_outputs = &mut self.callbacks_output;
        callback_outputs.drain(..)
                .map(|raw| E::from_u64(raw) )
    }

    pub fn toggle(&mut self, api: &LoomzApi, visible: bool) {
        self.visible = visible;
        api.gui().toggle_gui(&self.id, visible);
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    fn sync_with_engine(&mut self, api: &LoomzApi) {
        self.generate_sprites();
        api.gui().update_gui(&self.id, &self.sprites);
    }

    fn inner_resize(&mut self, view: &RectF32) {
        self.base_view = *view;
        layout::compute(self);
    }

    fn on_style_update(&mut self, old_state: GuiComponentState) {
        let styles = &self.styles;
        let base = &self.component_base;
        let data = &mut self.component_data;
        let state = self.state;
        let component_count = data.len() as u32;

        let mut update_style = |index: u32, new_state: GuiStyleState| {
            let component_index = index as usize;
            let style_index = base[component_index].style_index as usize;
            let style = &styles[style_index];
            data[component_index].update_style(style, new_state);
        };

        if old_state.selected_index != state.selected_index {
            if state.selected_index < component_count {
                update_style(state.selected_index, GuiStyleState::Selected);
            } else {
                if old_state.selected_index < component_count {
                    update_style(old_state.selected_index, GuiStyleState::Base);
                }

                if state.hovered_index < component_count {
                    update_style(state.hovered_index, GuiStyleState::Hovered);
                }
            }
        }

        if state.selected_index == u32::MAX && old_state.hovered_index != state.hovered_index {
            if old_state.hovered_index < component_count {
                update_style(old_state.hovered_index, GuiStyleState::Base);
            }

            if state.hovered_index < component_count {
                update_style(state.hovered_index, GuiStyleState::Hovered);
            }
        }

    }

    fn on_events(&mut self, component_index: usize, inner_event: GuiInnerEvent) {
        let base = self.component_base[component_index];
        if base.callbacks_index != u32::MAX {
            let callbacks = self.callbacks[base.callbacks_index as usize];
            let callback_output = &mut self.callbacks_output;
            self.component_data[component_index].on_events(&callbacks, callback_output, inner_event);
        }
    }

    fn update_cursor_position(&mut self, position: PositionF32, need_sync: &mut bool) {
        let old_state = self.state;
        let mut new_hovered_index = u32::MAX;

        let mut index = 0;
        let max_components = self.layout_items.len();
        while index < max_components {
            let item = self.layout_items[index];
            let view = RectF32::from_position_and_size(item.position, item.size);
            if view.is_point_inside(position)  {
                new_hovered_index = index as u32;
            }

            index += 1;
        }

        if old_state.hovered_index != new_hovered_index {
            self.state.hovered_index = new_hovered_index;
            self.on_style_update(old_state);
            *need_sync = true;
        }
    }

    fn update_mouse_button(&mut self, left_button_pressed: bool, need_sync: &mut bool) {
        let old_state = self.state;
        let mut new_selected_index = u32::MAX;
        if left_button_pressed {
            new_selected_index = old_state.hovered_index;
        }

        if old_state.selected_index != new_selected_index {
            self.state.selected_index = new_selected_index;
            self.on_style_update(old_state);
            *need_sync = true;
        }

        if !left_button_pressed && old_state.selected_index != u32::MAX {
            if old_state.selected_index == old_state.hovered_index {
                self.on_events(old_state.selected_index as usize, GuiInnerEvent::Click);
            }
        }
    }

    fn generate_sprites(&mut self) {
        let sprites = &mut self.sprites;
        sprites.clear();

        let component_count = self.layout_items.len();
        for i in 0..component_count {
            let view = &self.layout_items[i];
            let component_type = &self.component_data[i];
            component_type.generate_sprites(view, sprites);
        }
    }

    fn get_root_layout(&self) -> GuiLayout {
        match self.builder_data.layouts_stack.get(0).copied() {
            Some((_, root)) => root,
            None => unreachable!("Root layout will always be present")
        }
    }

    fn check_errors(&mut self) -> Result<(), CommonError> {
        let errors = &mut self.builder_data.errors;
        if errors.len() == 0 {
            return Ok(());
        }

        let mut error_base = client_err!("Failed to build Gui components");
        for error in errors.drain(..) {
            error_base.merge(error);
        }

        return Err(error_base);
    }

    //
    // Load / Store
    //

    fn store_components_data(&self, writer: &mut SaveFileWriterBase) {
        let inner = &self;
        writer.write_u32(inner.component_data.len() as u32);
        for component_type in inner.component_data.iter() {
            match component_type {
                GuiComponentData::Frame(frame) => {
                    writer.write_u32(0);
                    writer.write(frame);
                },
                GuiComponentData::Label(text) => {
                    writer.write_u32(1);
                    writer.write(&text.font);
                    writer.write_into_u32(text.color);
                    writer.write_slice(&text.glyphs);
                }
            }
        }
    }

    fn load_components_data(&mut self, reader: &mut SaveFileReaderBase) {
        let component_types_count = reader.read_u32() as usize;
        self.component_data = Vec::with_capacity(component_types_count);
        for _ in 0..component_types_count {
            let enum_identifier = reader.read_u32();
            match enum_identifier {
                0 => {
                    self.component_data.push(GuiComponentData::Frame(reader.read()));
                },
                1 => {
                    let font = reader.read();
                    let color = reader.read_from_u32();
                    let glyphs = reader.read_slice().to_vec().into_boxed_slice();
                    self.component_data.push(GuiComponentData::Label(GuiLabel {
                        font,
                        color,
                        glyphs,
                    }));
                },
                i => {
                    panic!("Unknown enum identifier {:?}", i);
                }
            }
        }
    } 

}

impl StoreAndLoad for Gui {

    fn store(&self, writer: &mut SaveFileWriterBase) {
        // Note: no need to store builder data or the generated sprites in components
        writer.store(&self.id);
        writer.write(&self.base_view);
        writer.write(&self.state);
        writer.write_slice(&self.layouts);
        writer.write_slice(&self.styles);
        writer.write_slice(&self.callbacks);
        writer.write_slice(&self.layout_items);
        writer.write_slice(&self.component_base);
        self.store_components_data(writer);
    }

    fn load(reader: &mut SaveFileReaderBase) -> Self {
        let mut gui = Gui {
            id: GuiId::default(),
            base_view: RectF32::default(),
            state: GuiComponentState::default(),

            styles: Vec::new(),
            callbacks: Vec::new(),
            callbacks_output: Vec::with_capacity(8),

            layouts: Vec::new(),
            layout_items: Vec::new(),
            component_base: Vec::new(),
            component_data: Vec::new(),

            sprites: Vec::with_capacity(64),
            builder_data: Box::default(),

            visible: true,
        };

        gui.id = reader.load();
        gui.base_view = reader.read();
        gui.state = reader.read();
        gui.layouts = reader.read_slice().to_vec();
        gui.styles = reader.read_slice().to_vec();
        gui.callbacks = reader.read_slice().to_vec();
        gui.layout_items = reader.read_slice().to_vec();
        gui.component_base = reader.read_slice().to_vec();
        gui.load_components_data(reader);

        gui
    }

}

impl Default for Gui {
    fn default() -> Self {
        Gui {
            id: GuiId::default(),
            base_view: RectF32::default(),
            state: GuiComponentState::default(),
            
            styles: Vec::with_capacity(8),
            callbacks: Vec::with_capacity(8),
            callbacks_output: Vec::with_capacity(8),

            layouts: Vec::with_capacity(8),
            layout_items: Vec::with_capacity(16),
            component_base: Vec::with_capacity(16),
            component_data: Vec::with_capacity(16),

            sprites: Vec::with_capacity(64),

            builder_data: Box::default(),

            visible: true,
        }
    }
}

impl Default for GuiBuilderData {

    fn default() -> Self {
        GuiBuilderData {
            errors: Vec::with_capacity(0),
            styles: GuiStyleMap::default(),
            last_callbacks: GuiComponentCallbacksValue::None,
            layouts_stack: Vec::with_capacity(4),
            root_layout_type: GuiLayoutType::VBox,
        }
    }

}

impl Default for GuiComponentState {
    fn default() -> Self {
        GuiComponentState {
            hovered_index: u32::MAX,
            selected_index: u32::MAX,
        }
    }
}
