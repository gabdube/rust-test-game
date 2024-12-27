mod component;
use component::GuiComponentType;

mod layout;
use layout::{GuiLayout, GuiLayoutView};

mod builder;
use builder::{GuiBuilder, GuiBuilderData};

use crate::base_types::RectF32;
use crate::LoomzApi;

#[derive(Copy, Clone, Default)]
pub struct GuiNode {
}

struct GuiComponents {
    base_view: RectF32,
    nodes: Vec<GuiNode>,
    layouts: Vec<GuiLayout>,
    views: Vec<GuiLayoutView>,
    types: Vec<GuiComponentType>,
}

pub struct Gui {
    builder_data: Box<GuiBuilderData>,
    components: Box<GuiComponents>,
}

impl Gui {
    pub fn build<F: FnOnce(&mut GuiBuilder)>(&mut self, api: &LoomzApi, view: &RectF32, cb: F) {
        self.components.base_view = *view;
        
        let mut builder = GuiBuilder::new(api, self);
        cb(&mut builder);
        
        self.compute_layout();
        self.sync_data(api);
    }

    fn compute_layout(&mut self) {
        let components = &mut self.components;
        let base = components.base_view;
        for node in self.components.nodes.iter() {

        }
    }

    fn sync_data(&mut self, api: &LoomzApi) {
        for data in self.components.types.iter() {
            match data {
                GuiComponentType::Text(text) => {
                    api.gui().update_text_glyphs(&text.id, &text.glyphs);
                }
            }
        }
    }
}

impl Default for Gui {
    fn default() -> Self {
        Gui {
            builder_data: Box::default(),
            components: Box::new(GuiComponents {
                base_view: RectF32::default(),
                nodes: Vec::with_capacity(16),
                layouts: Vec::with_capacity(16),
                views: Vec::with_capacity(16),
                types: Vec::with_capacity(16),
            }),
        }
    }
}
