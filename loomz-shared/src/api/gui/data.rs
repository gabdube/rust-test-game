mod component;
use component::GuiComponentType;

mod layout;
use layout::{GuiLayout, GuiLayoutView};

mod builder;
use builder::{GuiBuilder, GuiBuilderData};

use crate::base_types::RectF32;
use crate::LoomzApi;
use super::{GuiSprite, GuiSpriteType};


struct GuiComponents {
    base_view: RectF32,
    layouts: Vec<GuiLayout>,
    views: Vec<GuiLayoutView>,
    types: Vec<GuiComponentType>,
    sprites: Vec<GuiSprite>,
}

pub struct Gui {
    id: super::GuiId,
    builder_data: Box<GuiBuilderData>,
    components: Box<GuiComponents>,
}

impl Gui {
    pub fn build<F: FnOnce(&mut GuiBuilder)>(&mut self, api: &LoomzApi, view: &RectF32, cb: F) {
        self.components.base_view = *view;
        self.clear();
        
        let mut builder = GuiBuilder::new(api, self);
        cb(&mut builder);
        
        self.compute_layout();
    }

    pub fn id(&self) -> &super::GuiId {
        &self.id
    }

    pub(super) fn sprites(&self) -> &[GuiSprite] {
        &self.components.sprites
    }

    fn compute_layout(&mut self) {
        let components = &mut self.components;

        components.sprites.clear();
        
        let base = components.base_view;
        for (i, view) in components.views.iter_mut().enumerate() {
            view.position.x = (base.width() - view.size.width) / 2.0;
            view.position.y = (base.height() - view.size.height) / 2.0;

            let component_type = &mut components.types[i];
            component_type.generate_sprites(view, &mut components.sprites);
        }
    }

    fn clear(&mut self) {
        let c = &mut self.components;
        c.layouts.clear();
        c.views.clear();
        c.types.clear();
        c.sprites.clear();
    }
}

impl Default for Gui {
    fn default() -> Self {
        Gui {
            id: super::GuiId::default(),
            builder_data: Box::default(),
            components: Box::new(GuiComponents {
                base_view: RectF32::default(),
                layouts: Vec::with_capacity(16),
                views: Vec::with_capacity(16),
                types: Vec::with_capacity(16),
                sprites: Vec::with_capacity(64),
            }),
        }
    }
}
