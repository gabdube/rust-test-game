mod setup;
mod batch;

use fnv::FnvHashMap;
use std::{slice, sync::Arc};
use loomz_shared::api::{LoomzApi, GuiSprite};
use loomz_shared::assets::{LoomzAssetsBundle, AssetId, MsdfFontId, TextureId, ShaderId};
use loomz_shared::{CommonError, CommonErrorType};
use loomz_shared::{assets_err, backend_err, chain_err};
use loomz_engine_core::{LoomzEngineCore, VulkanContext, Texture, alloc::VertexAlloc, descriptors::*, pipelines::*};
use super::pipeline_compiler::PipelineCompiler;

const LAYOUT_COUNT: usize = 1;
const BATCH_LAYOUT_INDEX: u32 = 0;
const BATCH_TEXTURE_BINDING: u32 = 0;

const PUSH_STAGE_FLAGS: vk::ShaderStageFlags = vk::ShaderStageFlags::VERTEX;
const PUSH_SIZE: u32 = size_of::<GuiPushConstant>() as u32;

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct GuiPushConstant {
    pub screen_width: f32,
    pub screen_height: f32,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct GuiVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [u8; 4],
}

struct GuiTexture {
    texture: Texture,
    descriptor_set: vk::DescriptorSet,
}

/// Static resources used by the gui module
struct GuiResources {
    assets: Arc<LoomzAssetsBundle>,
    text_pipeline: GraphicsPipeline,
    image_pipeline: GraphicsPipeline,
    batch_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    text_pipeline_id: ShaderId,
    image_pipeline_id: ShaderId,

    default_sampler: vk::Sampler,
    textures: FnvHashMap<AssetId, GuiTexture>,
    descriptors_allocator: DescriptorsAllocator<LAYOUT_COUNT>,
}

#[derive(Copy, Clone)]
struct GuiViewSprite {
    descriptor_set: vk::DescriptorSet,
    sprite: GuiSprite
}

struct GuiView {
    sprites: Vec<GuiViewSprite>,
    id: u32,
    visible: bool,
}

/// Generic gui data not yet formatted for rendering 
struct GuiData {
    gui: Vec<GuiView>,
    indices: Vec<u32>,
    vertex: Vec<GuiVertex>,
    vertex_alloc: VertexAlloc<GuiVertex>,
}

#[derive(Copy, Clone, Default)]
pub struct GuiBatch {
    pipeline: vk::Pipeline,
    set: vk::DescriptorSet,
    index_count: u32,
    first_index: u32,
}

/// Data used on rendering
struct GuiRender {
    pipeline_layout: vk::PipelineLayout,
    vertex_buffer: vk::Buffer,
    index_offset: vk::DeviceSize,
    vertex_offset: [vk::DeviceSize; 1],
    push_constants: [GuiPushConstant; 1],
    batches: Vec<GuiBatch>,
}

pub(crate) struct GuiModule {
    resources: Box<GuiResources>,
    data: Box<GuiData>,
    render: Box<GuiRender>,
    update_batches: bool,
}

impl GuiModule {

    pub fn init(core: &mut LoomzEngineCore, api: &LoomzApi) -> Result<Self, CommonError> {
        let resources = GuiResources {
            assets: api.assets(),
            text_pipeline: GraphicsPipeline::new(),
            image_pipeline: GraphicsPipeline::new(),

            batch_layout: vk::DescriptorSetLayout::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            image_pipeline_id: ShaderId(0),
            text_pipeline_id: ShaderId(0),
           
            textures: FnvHashMap::default(),
            descriptors_allocator: DescriptorsAllocator::default(),
            default_sampler: vk::Sampler::null(),
        };

        let data = GuiData {
            gui: Vec::with_capacity(4),
            vertex: Vec::new(),
            indices: Vec::new(),
            vertex_alloc: VertexAlloc::default(),
        };
        
        let render = GuiRender {
            pipeline_layout: vk::PipelineLayout::null(),
            vertex_buffer: vk::Buffer::null(),
            index_offset: 0,
            vertex_offset: [0],
            push_constants: [GuiPushConstant::default(); 1],
            batches: Vec::with_capacity(16),
        };

        let mut gui = GuiModule {
            resources: Box::new(resources),
            data: Box::new(data),
            render: Box::new(render),
            update_batches: false,
        };

        gui.setup_pipelines(api, core)?;
        gui.setup_vertex_buffers(core)?;
        gui.setup_descriptors(core)?;
        gui.setup_render_data();

        Ok(gui)
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.data.vertex_alloc.free(core);

        self.resources.descriptors_allocator.destroy(core);
        self.resources.text_pipeline.destroy(&core.ctx);
        self.resources.image_pipeline.destroy(&core.ctx);

        core.ctx.device.destroy_pipeline_layout(self.resources.pipeline_layout);
        core.ctx.device.destroy_descriptor_set_layout(self.resources.batch_layout);

        for gui_texture in self.resources.textures.values() {
            core.destroy_texture(gui_texture.texture);
        }
    }

    pub fn set_output(&mut self, core: &LoomzEngineCore) {
        let extent = core.info.swapchain_extent;
        self.render.push_constants[0] = GuiPushConstant {
            screen_width: extent.width as f32,
            screen_height: extent.height as f32,
        };
    }

    pub fn rebuild(&mut self, core: &LoomzEngineCore) {
        self.set_output(core);
    }

    //
    // Pipeline setup
    //

    pub fn write_pipeline_create_infos(&mut self, compiler: &mut PipelineCompiler) {
        compiler.add_pipeline_info("gui_component", &mut self.resources.image_pipeline);
        compiler.add_pipeline_info("gui_text", &mut self.resources.text_pipeline);
    }

    pub fn set_pipeline_handle(&mut self, compiler: &PipelineCompiler) {
        let mut handle = compiler.get_pipeline("gui_component");
        self.resources.image_pipeline.set_handle(handle);

        handle = compiler.get_pipeline("gui_text");
        self.resources.text_pipeline.set_handle(handle);
    }

    //
    // Rendering
    //

    pub fn render(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        #[inline(always)]
        fn push_values(constants: &[GuiPushConstant; 1]) -> &[u8] {
            unsafe { constants.align_to::<u8>().1 }
        }

        const GRAPHICS: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;

        let device = &ctx.device;
        let render = &self.render;

        device.cmd_bind_index_buffer(cmd, render.vertex_buffer, render.index_offset, vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, slice::from_ref(&render.vertex_buffer), &render.vertex_offset);
        device.cmd_push_constants(cmd, render.pipeline_layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, push_values(&render.push_constants));

        let mut last_pipeline = vk::Pipeline::null();

        for batch in render.batches.iter() {
            // improvement: Try to use one pipeline for all of the GUI rendering
            if last_pipeline != batch.pipeline {
                device.cmd_bind_pipeline(cmd, GRAPHICS, batch.pipeline);
                last_pipeline = batch.pipeline;
            }
            
            device.cmd_bind_descriptor_sets(cmd, GRAPHICS, render.pipeline_layout, BATCH_LAYOUT_INDEX, slice::from_ref(&batch.set), &[]);
            device.cmd_draw_indexed(cmd, batch.index_count, 1, batch.first_index, 0, 0);
        }
    }

    //
    // Updates
    //

    fn create_gui(&mut self, id: u32) -> usize {
        let index = self.data.gui.len();
        self.data.gui.push(GuiView {
            sprites: Vec::new(),
            id,
            visible: true,
        });

        index
    }

    fn update_gui_sprites<'a>(&mut self, core: &mut LoomzEngineCore, index: usize, sprites: &'a [GuiSprite]) -> Result<(), CommonError> {
        let gui = match self.data.gui.get_mut(index) {
            Some(gui) => gui,
            None => {
                return Err(backend_err!("Tried to fetch gui at index {}, but it does not exits", index));
            } 
        };

        gui.sprites.clear();

        if gui.sprites.capacity() < sprites.len() {
            gui.sprites.reserve(sprites.len());
        }

        for &sprite in sprites.iter() {
            let descriptor_set = match sprite.ty {
                loomz_shared::GuiSpriteType::Image(texture_id) => Self::fetch_texture_descriptor_set(core, &mut self.resources, texture_id)?,
                loomz_shared::GuiSpriteType::Font(font_id) => Self::fetch_font_texture_descriptor_set(core, &mut self.resources, font_id)?
            };

            gui.sprites.push(GuiViewSprite {
                descriptor_set,
                sprite
            }); 
        }

        self.update_batches = true;

        Ok(())
    }

    fn toggle_gui_visibility(&mut self, index: usize, visible: bool) -> Result<(), CommonError> {
        match self.data.gui.get_mut(index) {
            Some(gui) => {
                gui.visible = visible;
                self.update_batches = true;
                Ok(())
            },
            None => {
                Err(backend_err!("Tried to fetch gui at index {}, but it does not exits", index))
            } 
        }
    }

    fn api_update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        use loomz_shared::GuiApiUpdate;
        
        if let Some(updates) = api.gui().gui_updates() {
            for (id, update) in updates {
                let id = id.value();
                let index = self.data.gui.iter().position(|gui| gui.id == id )
                    .unwrap_or_else(|| self.create_gui(id) );

                match update {
                    GuiApiUpdate::ToggleGui(visible) => {
                        self.toggle_gui_visibility(index, visible)?;
                    },
                    GuiApiUpdate::UpdateSprites(sprites) => {
                        self.update_gui_sprites(core, index, sprites)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        self.api_update(api, core)?;

        if self.update_batches {
            batch::build(core, self)?;
            self.update_batches = false;
        }

        Ok(())
    }

    //
    // Data
    //

    pub fn reload_assets(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore, assets: &Vec<AssetId>) -> Result<(), CommonError> {
        for &assets_id in assets.iter() {
            match assets_id {
                AssetId::ShaderId(shader_id) => {
                    // If a gui shader is reloaded, we need to rebuild all the gui batches
                    self.reload_shaders(api, core, shader_id)?;
                    batch::build(core, self)?;
                    self.update_batches = false;
                },
                _ => {}
            }
        }

        Ok(())
    }

    fn fetch_font_texture_descriptor_set(core: &mut LoomzEngineCore, resources: &mut GuiResources, font_id: MsdfFontId) -> Result<vk::DescriptorSet, CommonError> {
        let asset_id = AssetId::MsdfFont(font_id);
        if let Some(texture) = resources.textures.get(&asset_id) {
            return Ok(texture.descriptor_set);
        }

        let texture_asset = resources.assets.font(font_id)
            .ok_or_else(|| assets_err!("Unkown asset with ID {font_id:?}") )?;

        let texture = core.create_texture_from_font_asset(&texture_asset)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendGeneric, "Failed to create image from font asset") )?;

        let descriptor_set = resources.descriptors_allocator.get_set::<BATCH_LAYOUT_INDEX>()
            .ok_or_else(|| backend_err!("No more descriptor set in gui batch layout pool") )?;

        core.descriptors.write_image(
            descriptor_set,
            texture.view,
            resources.default_sampler,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            BATCH_TEXTURE_BINDING,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        );

        resources.textures.insert(asset_id, GuiTexture {
            texture,
            descriptor_set
        });

        Ok(descriptor_set)
    }

    fn fetch_texture_descriptor_set(core: &mut LoomzEngineCore, resources: &mut GuiResources, texture_id: TextureId) -> Result<vk::DescriptorSet, CommonError> {
        let asset_id = AssetId::Texture(texture_id);
        if let Some(texture) = resources.textures.get(&asset_id) {
            return Ok(texture.descriptor_set);
        }
        
        let texture_asset = resources.assets.texture(texture_id)
            .ok_or_else(|| assets_err!("Unkown asset with ID {texture_id:?}") )?;

        let texture = core.create_texture_from_asset(&texture_asset)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendGeneric, "Failed to create image from asset") )?;

        let descriptor_set = resources.descriptors_allocator.get_set::<BATCH_LAYOUT_INDEX>()
            .ok_or_else(|| backend_err!("No more descriptor set in gui batch layout pool") )?;

        core.descriptors.write_image(
            descriptor_set,
            texture.view,
            resources.default_sampler,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            BATCH_TEXTURE_BINDING,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        );

        resources.textures.insert(asset_id, GuiTexture {
            texture,
            descriptor_set
        });

        Ok(descriptor_set)
    }

}

impl PartialEq for GuiViewSprite {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor_set == other.descriptor_set
    }
}

impl Eq for GuiViewSprite { }

impl PartialOrd for GuiViewSprite {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.descriptor_set.partial_cmp(&other.descriptor_set)
    }
}

impl Ord for GuiViewSprite {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(&other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
