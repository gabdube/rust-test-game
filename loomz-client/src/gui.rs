mod component;
use component::{GuiComponentText, GuiComponentType};

mod layout;
use layout::{GuiLayout, GuiLayoutItem};
pub use layout::GuiLayoutType;

mod style;
use style::{GuiStyleBuilder, GuiFontStyle, GuiFrameStyle};

mod builder;
use builder::GuiBuilder;

use fnv::FnvHashMap;
use loomz_shared::base_types::{RectF32, PositionF32};
use loomz_shared::api::{LoomzApi, GuiId, GuiSprite, GuiSpriteType};
use loomz_shared::store::*;
use loomz_shared::{CommonError, client_err};

#[derive(Copy, Clone)]
pub enum GuiStyleState {
    Base,
    Hovered,
    Active
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
}

struct GuiBuilderData {
    pub errors: Vec<crate::CommonError>,
    pub layouts_stack: Vec<(usize, GuiLayout)>,
    pub font_styles: FnvHashMap<&'static str, GuiFontStyle>,
    pub frame_styles: FnvHashMap<&'static str, GuiFrameStyle>,
    pub root_layout_type: GuiLayoutType,
}

struct GuiComponents {
    base_view: RectF32,
    state: GuiComponentState,
    layouts: Vec<GuiLayout>,
    layout_items: Vec<GuiLayoutItem>,
    types: Vec<GuiComponentType>,
    sprites: Vec<GuiSprite>,
}

pub struct Gui {
    id: GuiId,
    builder_data: Box<GuiBuilderData>,
    components: Box<GuiComponents>,
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

        self.components.layouts[0] = self.get_root_layout();

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
        self.components.base_view = *view;
        layout::compute(self);
    }

    fn on_hovered_changed(&mut self, new_position: u32, last_position: u32) {
        if last_position != u32::MAX {
            self.components.types[last_position as usize].on_mouse_left();
        }
        
        if new_position != u32::MAX {
            self.components.types[new_position as usize].on_mouse_enter();
        }
    }

    fn update_cursor_position(&mut self, position: PositionF32, need_sync: &mut bool) {
        let components = &mut self.components;

        let last_position = components.state.hovered_index;
        let mut new_position = u32::MAX;

        let mut index = 0;
        let max_components = components.layout_items.len();
        while index < max_components {
            let item = components.layout_items[index];
            let view = RectF32::from_position_and_size(item.position, item.size);
            if view.is_point_inside(position)  {
                new_position = index as u32;
            }

            index += 1;
        }

        if last_position != new_position {
            components.state.hovered_index = new_position;
            self.on_hovered_changed(new_position, last_position);
            *need_sync = true;
        }
    }

    fn generate_sprites(&mut self) {
        let sprites = &mut self.components.sprites;
        sprites.clear();

        let component_count = self.components.layout_items.len();
        for i in 0..component_count {
            let view = &self.components.layout_items[i];
            let component_type = &self.components.types[i];
            component_type.generate_sprites(view, sprites);
        }
    }

    fn sync_with_engine(&mut self, api: &LoomzApi) {
        self.generate_sprites();
        api.gui().update_gui(&self.id, &self.components.sprites);
    }

    fn get_root_layout(&self) -> GuiLayout {
        match self.builder_data.layouts_stack.get(0).copied() {
            Some((_, root)) => root,
            None => unreachable!("Root layout will always be present")
        }
    }
}

impl StoreAndLoad for Gui {

    fn store(&self, writer: &mut SaveFileWriterBase) {
        // Note: no need to store builder data or the generated sprites in components
        let components = &self.components;

        writer.store(&self.id);
        writer.write(&components.base_view);
        writer.write(&components.state);
        writer.write_slice(&components.layouts);
        writer.write_slice(&components.layout_items);

        writer.write_u32(components.types.len() as u32);
        for component_type in components.types.iter() {
            match component_type {
                GuiComponentType::Frame(frame) => {
                    writer.write_u32(0);
                    writer.write(frame);
                },
                GuiComponentType::Text(text) => {
                    writer.write_u32(1);
                    writer.write(&text.font);
                    writer.write_into_u32(text.color);
                    writer.write_slice(&text.glyphs);
                }
            }
        }
    }

    fn load(reader: &mut SaveFileReaderBase) -> Self {
        let id = reader.load();
        let builder_data = Box::default();

        let mut components = GuiComponents {
            base_view: RectF32::default(),
            state: GuiComponentState::default(),
            layouts: Vec::new(),
            layout_items: Vec::new(),
            types: Vec::new(),
            sprites: Vec::with_capacity(64),
        };

        components.base_view = reader.read();
        components.state = reader.read();
        components.layouts = reader.read_slice().to_vec();
        components.layout_items = reader.read_slice().to_vec();

        let component_types_count = reader.read_u32() as usize;
        components.types = Vec::with_capacity(component_types_count);
        for _ in 0..component_types_count {
            let enum_identifier = reader.read_u32();
            match enum_identifier {
                0 => {
                    components.types.push(GuiComponentType::Frame(reader.read()));
                },
                1 => {
                    let font = reader.read();
                    let color = reader.read_from_u32();
                    let glyphs = reader.read_slice().to_vec();
                    components.types.push(GuiComponentType::Text(GuiComponentText {
                        font,
                        color,
                        glyphs
                    }));
                },
                i => {
                    panic!("Unknown enum identifier {:?}", i);
                }
            }
        }


        Gui {
            id,
            builder_data,
            components: Box::new(components),
        }
    }

}

impl Default for Gui {
    fn default() -> Self {
        Gui {
            id: GuiId::default(),
            builder_data: Box::default(),
            components: Box::new(GuiComponents {
                base_view: RectF32::default(),
                state: GuiComponentState::default(),
                layouts: Vec::with_capacity(8),
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
            font_styles: FnvHashMap::default(),
            frame_styles: FnvHashMap::default(),
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
