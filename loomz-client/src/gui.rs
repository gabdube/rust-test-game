mod component;
use component::{GuiComponentText, GuiComponentType};

mod layout;
use layout::{GuiLayout, GuiLayoutItem};
pub use layout::GuiLayoutType;

mod builder;
use builder::{GuiBuilder, GuiBuilderData};

use loomz_shared::base_types::RectF32;
use loomz_shared::api::{LoomzApi, GuiId, GuiSprite, GuiSpriteType};
use loomz_shared::store::*;
use loomz_shared::{CommonError, client_err};

struct GuiComponents {
    base_view: RectF32,
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
    pub fn build<F: FnOnce(&mut GuiBuilder)>(&mut self, api: &LoomzApi, view: &RectF32, cb: F) -> Result<(), CommonError> {
        self.clear();
        self.components.base_view = *view;
        
        let mut builder = GuiBuilder::new(api, self);
        cb(&mut builder);
        
        if self.builder_data.errors.len() > 0 {
            let mut error_base = client_err!("Failed to build Gui");
            for error in self.builder_data.errors.drain(..) {
                error_base.merge(error);
            }

            return Err(error_base);
        }

        self.components.layouts[0] = self.get_root_layout();

        layout::compute(self);

        Ok(())
    }

    pub fn resize(&mut self, view: &RectF32) {
        self.components.base_view = *view;
        layout::compute(self);
    }

    pub fn sync_with_engine(&mut self, api: &LoomzApi) {
        self.generate_sprites();
        api.gui().update_gui(&self.id, &self.components.sprites);
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

    fn clear(&mut self) {
        let c = &mut self.components;
        c.base_view = RectF32::default();
        c.layouts.clear();
        c.layout_items.clear();
        c.types.clear();
        c.sprites.clear();
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
            layouts: Vec::new(),
            layout_items: Vec::new(),
            types: Vec::new(),
            sprites: Vec::with_capacity(64),
        };

        components.base_view = reader.read();
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
                layouts: Vec::with_capacity(8),
                layout_items: Vec::with_capacity(16),
                types: Vec::with_capacity(16),
                sprites: Vec::with_capacity(64),
            }),
        }
    }
}
