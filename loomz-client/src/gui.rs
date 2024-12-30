mod component;
use component::GuiComponentType;

mod layout;
use layout::{GuiLayout, GuiLayoutView};

mod builder;
use builder::{GuiBuilder, GuiBuilderData};

use loomz_shared::base_types::RectF32;
use loomz_shared::api::{LoomzApi, GuiId, GuiSprite, GuiSpriteType};
use loomz_shared::{CommonError, client_err};

struct GuiComponents {
    base_view: RectF32,
    layouts: Vec<GuiLayout>,
    views: Vec<GuiLayoutView>,
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
        self.components.base_view = *view;
        self.clear();
        
        let mut builder = GuiBuilder::new(api, self);
        cb(&mut builder);
        
        if self.builder_data.errors.len() > 0 {
            let mut error_base = client_err!("Failed to build Gui");
            for error in self.builder_data.errors.drain(..) {
                error_base.merge(error);
            }

            return Err(error_base);
        }

        layout::compute(self);
        self.generate_sprites();

        Ok(())
    }

    pub fn resize(&mut self, view: &RectF32) {
        self.components.base_view = *view;
        layout::compute(self);
        self.generate_sprites();
    }

    pub fn sync_with_engine(&self, api: &LoomzApi) {
        api.gui().update_gui(&self.id, &self.components.sprites);
    }

    fn generate_sprites(&mut self) {
        let sprites = &mut self.components.sprites;
        sprites.clear();

        let component_count = self.components.views.len();
        for i in 0..component_count {
            let view = &self.components.views[i];
            let component_type = &self.components.types[i];
            component_type.generate_sprites(view, sprites);
        }
    }

    fn clear(&mut self) {
        let c = &mut self.components;
        c.views.clear();
        c.types.clear();
        c.sprites.clear();
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
                views: Vec::with_capacity(16),
                types: Vec::with_capacity(16),
                sprites: Vec::with_capacity(64),
            }),
        }
    }
}
