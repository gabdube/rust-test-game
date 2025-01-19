mod style;
use style::{GuiStyleBuilder, GuiStyleMap, GuiComponentStyle};
pub use style::GuiStyleState;

mod component;
use component::{GuiComponentText, GuiComponentType};

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
struct GuiComponentState {
    hovered_index: u32,
    selected_index: u32,
}


#[derive(Default, Copy, Clone)]
pub struct GuiUpdates {
    pub cursor_position: Option<PositionF32>,
    pub view: Option<RectF32>,
}

struct GuiBuilderData {
    errors: Vec<crate::CommonError>,
    layouts_stack: Vec<(usize, GuiLayout)>,
    font_styles: GuiStyleMap,
    frame_styles: GuiStyleMap,
    root_layout_type: GuiLayoutType,
}

struct GuiInnerState {
    base_view: RectF32,
    state: GuiComponentState,
    styles: Vec<GuiComponentStyle>,
    layouts: Vec<GuiLayout>,
    layout_items: Vec<GuiLayoutItem>,
    types: Vec<GuiComponentType>,
    sprites: Vec<GuiSprite>,
}

pub struct Gui {
    id: GuiId,
    builder_data: Box<GuiBuilderData>,
    inner_state: Box<GuiInnerState>,
}

impl Gui {

    pub fn build_style<F: FnOnce(&mut GuiStyleBuilder)>(&mut self, api: &LoomzApi, cb: F) -> Result<(), CommonError> {
        let mut builder = GuiStyleBuilder::new(api, self);
        cb(&mut builder);

        if self.builder_data.errors.len() > 0 {
            let mut error_base = client_err!("Failed to build Gui style");
            for error in self.builder_data.errors.drain(..) {
                error_base.merge(error);
            }

            return Err(error_base);
        }

        Ok(())
    }

    pub fn build<F: FnOnce(&mut GuiBuilder)>(&mut self, api: &LoomzApi, view: &RectF32, cb: F) -> Result<(), CommonError> {
        let mut builder = GuiBuilder::new(api, view, self);
        cb(&mut builder);
        
        if self.builder_data.errors.len() > 0 {
            let mut error_base = client_err!("Failed to build Gui components");
            for error in self.builder_data.errors.drain(..) {
                error_base.merge(error);
            }

            return Err(error_base);
        }

        self.inner_state.layouts[0] = self.get_root_layout();

        layout::compute(self);

        self.sync_with_engine(api);

        Ok(())
    }

    pub fn update(&mut self, api: &LoomzApi, updates: &GuiUpdates) {
        let mut need_sync = false;

        if let Some(view) = updates.view {
            self.resize(&view);
            need_sync = true;
        }

        if let Some(cursor_position) = updates.cursor_position {
            self.update_cursor_position(cursor_position, &mut need_sync);
        }

        if need_sync {
            self.sync_with_engine(api);
        }
    }

    fn resize(&mut self, view: &RectF32) {
        self.inner_state.base_view = *view;
        layout::compute(self);
    }

    fn on_hovered_changed(&mut self, new_position: u32, last_position: u32) {
        let styles = &self.inner_state.styles;
        let types_count = self.inner_state.types.len() as u32;

        if last_position != u32::MAX {
            assert!(last_position < types_count, "last_position is not in scope");
            self.inner_state.types[last_position as usize].update_style(styles, GuiStyleState::Base);
        }
        
        if new_position != u32::MAX {
            assert!(new_position < types_count, "new_position is not in scope");
            self.inner_state.types[new_position as usize].update_style(styles, GuiStyleState::Hovered);
        }
    }

    fn update_cursor_position(&mut self, position: PositionF32, need_sync: &mut bool) {
        let inner_state = &mut self.inner_state;

        let last_position = inner_state.state.hovered_index;
        let mut new_position = u32::MAX;

        let mut index = 0;
        let max_components = inner_state.layout_items.len();
        while index < max_components {
            let item = inner_state.layout_items[index];
            let view = RectF32::from_position_and_size(item.position, item.size);
            if view.is_point_inside(position)  {
                new_position = index as u32;
            }

            index += 1;
        }

        if last_position != new_position {
            inner_state.state.hovered_index = new_position;
            self.on_hovered_changed(new_position, last_position);
            *need_sync = true;
        }
    }

    fn generate_sprites(&mut self) {
        let inner = &mut self.inner_state;
        let sprites = &mut inner.sprites;
        sprites.clear();

        let component_count = inner.layout_items.len();
        for i in 0..component_count {
            let view = &inner.layout_items[i];
            let component_type = &inner.types[i];
            component_type.generate_sprites(view, sprites);
        }
    }

    fn sync_with_engine(&mut self, api: &LoomzApi) {
        self.generate_sprites();
        api.gui().update_gui(&self.id, &self.inner_state.sprites);
    }

    fn get_root_layout(&self) -> GuiLayout {
        match self.builder_data.layouts_stack.get(0).copied() {
            Some((_, root)) => root,
            None => unreachable!("Root layout will always be present")
        }
    }

    //
    // Load / Store
    //

    fn store_components_data(&self, writer: &mut SaveFileWriterBase) {
        let inner = &self.inner_state;
        writer.write_u32(inner.types.len() as u32);
        for component_type in inner.types.iter() {
            match component_type {
                GuiComponentType::Frame(frame) => {
                    writer.write_u32(0);
                    writer.write(frame);
                },
                GuiComponentType::Text(text) => {
                    writer.write_u32(1);
                    writer.write(&text.font);
                    writer.write_into_u32(text.color);
                    writer.write_u32(text.style_index);
                    writer.write_slice(&text.glyphs);
                }
            }
        }
    }

    fn load_components_data(reader: &mut SaveFileReaderBase, inner_state: &mut GuiInnerState) {
        let component_types_count = reader.read_u32() as usize;
        inner_state.types = Vec::with_capacity(component_types_count);
        for _ in 0..component_types_count {
            let enum_identifier = reader.read_u32();
            match enum_identifier {
                0 => {
                    inner_state.types.push(GuiComponentType::Frame(reader.read()));
                },
                1 => {
                    let font = reader.read();
                    let color = reader.read_from_u32();
                    let style_index = reader.read_u32();
                    let glyphs = reader.read_slice().to_vec();
                    inner_state.types.push(GuiComponentType::Text(GuiComponentText {
                        font,
                        color,
                        glyphs,
                        style_index
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
        let inner = &self.inner_state;

        writer.store(&self.id);
        writer.write(&inner.base_view);
        writer.write(&inner.state);
        writer.write_slice(&inner.layouts);
        writer.write_slice(&inner.layout_items);
        self.store_components_data(writer);
    }

    fn load(reader: &mut SaveFileReaderBase) -> Self {
        let id = reader.load();
        let builder_data = Box::default();

        let mut inner_state = GuiInnerState {
            base_view: RectF32::default(),
            state: GuiComponentState::default(),
            layouts: Vec::new(),
            styles: Vec::new(),
            layout_items: Vec::new(),
            types: Vec::new(),
            sprites: Vec::with_capacity(64),
        };

        inner_state.base_view = reader.read();
        inner_state.state = reader.read();
        inner_state.layouts = reader.read_slice().to_vec();
        inner_state.layout_items = reader.read_slice().to_vec();
        Self::load_components_data(reader, &mut inner_state);

        Gui {
            id,
            builder_data,
            inner_state: Box::new(inner_state),
        }
    }

}

impl Default for Gui {
    fn default() -> Self {
        Gui {
            id: GuiId::default(),
            builder_data: Box::default(),
            inner_state: Box::new(GuiInnerState {
                base_view: RectF32::default(),
                state: GuiComponentState::default(),
                layouts: Vec::with_capacity(8),
                styles: Vec::with_capacity(16),
                layout_items: Vec::with_capacity(16),
                types: Vec::with_capacity(16),
                sprites: Vec::with_capacity(64),
            }),
        }
    }
}

impl Default for GuiBuilderData {

    fn default() -> Self {
        GuiBuilderData {
            errors: Vec::with_capacity(0),
            font_styles: GuiStyleMap::default(),
            frame_styles: GuiStyleMap::default(),
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
