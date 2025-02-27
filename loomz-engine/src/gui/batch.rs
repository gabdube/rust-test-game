use loomz_shared::CommonError;
use loomz_shared::api::GuiSpriteType;
use loomz_engine_core::{alloc::VertexAlloc, LoomzEngineCore};
use super::{GuiModule, GuiBatch, GuiView, GuiViewSprite, GuiVertex};

struct NextBatch<'a> {
    batches: &'a mut Vec<GuiBatch>,
    indices: &'a mut Vec<u32>,
    vertex: &'a mut Vec<GuiVertex>,
    text_pipeline: vk::Pipeline,
    image_pipeline: vk::Pipeline,
    index_count: isize,
    vertex_count: u32,
}

impl<'a> NextBatch<'a> {
    fn prepare(&mut self) {
        self.batches.clear();
    }
    
    fn build_batch(&mut self, sprite_type: GuiSpriteType, set: vk::DescriptorSet, sprites_count: usize) -> Result<(), CommonError> {
        let pipeline = match sprite_type {
            GuiSpriteType::Font(_) => self.text_pipeline,
            GuiSpriteType::Image(_) => self.image_pipeline,
        };

        self.batches.push(GuiBatch {
            pipeline,
            set,
            first_index: self.index_count as u32,
            index_count: (sprites_count * 6) as u32,
        });

        Ok(())
    }

    fn generate_indices(&mut self, sprites: &[GuiViewSprite]) {
        let sprite_count = sprites.len();
        let index_count = (sprite_count * 6) as isize;

        // Safety, index buffer capacity will be greater than written range
        let mut i = self.index_count;
        let mut v = self.vertex_count;
        assert!(i + index_count < self.indices.len() as isize);

        unsafe {
            for _ in 0..sprite_count {
                let indices = self.indices.as_mut_ptr();
                indices.offset(i+0).write(v+0);
                indices.offset(i+1).write(v+1);
                indices.offset(i+2).write(v+2);
                indices.offset(i+3).write(v+2);
                indices.offset(i+4).write(v+3);
                indices.offset(i+5).write(v+1);

                i += 6;
                v += 4;
            }
        }
    
        self.index_count += index_count;
    }

    fn generate_vertex(&mut self, sprites: &[GuiViewSprite]) {
        let sprite_count = sprites.len();
        let vertex_count = (sprite_count * 4) as isize;

        // Safety, vertex capacity will be greater than written range
        let mut v = self.vertex_count as isize;
        assert!(v + vertex_count < self.vertex.len() as isize);

        unsafe {
            let vertex = self.vertex.as_mut_ptr();            
            for sprite_view in sprites {
                let [x1, y1, x2, y2] = sprite_view.sprite.position.splat();
                let [x3, y3, x4, y4] = sprite_view.sprite.texcoord.splat();
                let color = sprite_view.sprite.color.splat();
                
                vertex.offset(v+0).write(GuiVertex { pos: [x1, y1], uv: [x3, y3], color });
                vertex.offset(v+1).write(GuiVertex { pos: [x2, y1], uv: [x4, y3], color });
                vertex.offset(v+2).write(GuiVertex { pos: [x1, y2], uv: [x3, y4], color });
                vertex.offset(v+3).write(GuiVertex { pos: [x2, y2], uv: [x4, y4], color });
                v += 4;
            }
        }

        self.vertex_count += vertex_count as u32;
    }

    fn upload_vertex(&mut self, core: &mut LoomzEngineCore, alloc: &mut VertexAlloc<GuiVertex>) {
        // index_count may be 0 if a gui has no visible data
        if self.index_count > 0 {
            let i = self.index_count as usize;
            let v = self.vertex_count as usize;
            alloc.set_data(core, &self.indices[0..i], &self.vertex[0..v]);
        }
    }
}

fn find_first_sprite_type(gui: &[GuiView], gui_index_out: &mut usize) -> Option<GuiSpriteType> {
    let mut gui_index = 0;
    let mut current_sprite_type = None;

    while current_sprite_type.is_none() && gui_index < gui.len() {
        let gui = gui.get(gui_index);

        if let Some(gui) = gui {
            if gui.visible {
                current_sprite_type = gui.sprites.first()
                    .map(|sprite| sprite.sprite.ty );
            }
        }

        if current_sprite_type.is_none() {
            gui_index += 1;
        }
    }

    *gui_index_out = gui_index;
    current_sprite_type
}

fn groups<'a>(gui: &'a [GuiView]) -> impl Iterator<Item=(GuiSpriteType, vk::DescriptorSet, &'a [GuiViewSprite])> {
    let mut gui_index = 0;
    let mut sprites_start = 0;
    let mut sprites_stop = 0;
    let mut current_sprite_type = find_first_sprite_type(gui, &mut gui_index);

    ::std::iter::from_fn(move || {
        loop {
            let gui = match gui.get(gui_index) {
                Some(gui) => gui,
                None => { return None }
            };

            if !gui.visible {
                gui_index += 1;
                continue;
            }

            loop {
                if sprites_stop == gui.sprites.len() {
                    break;
                }
            
                let sprite_view = &gui.sprites[sprites_stop];
                if Some(sprite_view.sprite.ty) != current_sprite_type {
                    break;
                }
            
                sprites_stop += 1;
            }

            let set = gui.sprites[sprites_start].descriptor_set;
            let sprite_type = gui.sprites[sprites_start].sprite.ty;
            let sprites = &gui.sprites[sprites_start..sprites_stop];
            let value = (sprite_type, set, sprites);

            current_sprite_type = Some(sprite_type);
            sprites_start = sprites_stop;

            if sprites_start == gui.sprites.len() {
                gui_index += 1;
                sprites_start = 0;
                sprites_stop = 0;
            }

            return Some(value);
        }
    })
}

pub(super) fn build(core: &mut LoomzEngineCore, gui_module: &mut GuiModule) -> Result<(), CommonError> {
    let mut batcher = NextBatch {
        batches: &mut gui_module.render.batches,
        indices: &mut gui_module.data.indices,
        vertex: &mut gui_module.data.vertex,
        text_pipeline: gui_module.resources.text_pipeline.handle(),
        image_pipeline: gui_module.resources.image_pipeline.handle(),
        index_count: 0,
        vertex_count: 0,
    };

    batcher.prepare();

    for (sprite_type, image_view, sprites) in groups(&gui_module.data.gui) {
        batcher.build_batch(sprite_type, image_view, sprites.len())?;
        batcher.generate_indices(sprites);
        batcher.generate_vertex(sprites);
    }

    batcher.upload_vertex(core, &mut gui_module.data.vertex_alloc);

    Ok(())
}
