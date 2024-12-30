mod setup;
mod batch;

use fnv::FnvHashMap;
use std::{slice, sync::Arc};
use loomz_shared::api::{LoomzApi, GuiId, GuiSprite};
use loomz_shared::assets::{LoomzAssetsBundle, AssetId, MsdfFontId, TextureId};
use loomz_shared::{CommonError, CommonErrorType};
use loomz_shared::{assets_err, chain_err};
use loomz_engine_core::{LoomzEngineCore, VulkanContext, Texture, alloc::VertexAlloc, descriptors::*, pipelines::*};
use super::pipeline_compiler::PipelineCompiler;

const LAYOUT_COUNT: usize = 1;
const BATCH_LAYOUT_INDEX: u32 = 0;

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
}

struct GuiResources {
    assets: Arc<LoomzAssetsBundle>,
    textures: FnvHashMap<AssetId, Texture>,
    text_pipeline: GraphicsPipeline,
    image_pipeline: GraphicsPipeline,
    batch_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
}

#[derive(Default)]
struct GuiDescriptor {
    default_sampler: vk::Sampler,
    allocator: DescriptorsAllocator<LAYOUT_COUNT>
}

/// Data used on rendering
#[derive(Copy, Clone)]
struct GuiRender {
    pipeline_layout: vk::PipelineLayout,
    vertex_buffer: vk::Buffer,
    index_offset: vk::DeviceSize,
    vertex_offset: [vk::DeviceSize; 1],
    push_constants: [GuiPushConstant; 1],
}

#[derive(Copy, Clone)]
struct GuiViewSprite {
    image_view: vk::ImageView,
    sprite: GuiSprite
}

struct GuiView {
    sprites: Vec<GuiViewSprite>,
}

struct GuiData {
    vertex_alloc: VertexAlloc<GuiVertex>,
    gui: Vec<GuiView>,
    indices: Vec<u32>,
    vertex: Vec<GuiVertex>,
}

#[derive(Copy, Clone, Default)]
pub struct GuiBatch {
    pipeline: vk::Pipeline,
    set: vk::DescriptorSet,
    index_count: u32,
    first_index: u32,
}

pub(crate) struct GuiModule {
    resources: Box<GuiResources>,
    descriptors: Box<GuiDescriptor>,
    data: Box<GuiData>,
    render: Box<GuiRender>,
    batches: Vec<GuiBatch>,
    update_batches: bool,
}

impl GuiModule {

    pub fn init(core: &mut LoomzEngineCore, api: &LoomzApi) -> Result<Self, CommonError> {
        let resources = GuiResources {
            assets: api.assets(),
            batch_layout: vk::DescriptorSetLayout::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            text_pipeline: GraphicsPipeline::new(),
            image_pipeline: GraphicsPipeline::new(),
            textures: FnvHashMap::default(),
        };
        
        let render = GuiRender {
            pipeline_layout: vk::PipelineLayout::null(),
            vertex_buffer: vk::Buffer::null(),
            index_offset: 0,
            vertex_offset: [0],
            push_constants: [GuiPushConstant::default(); 1],
        };

        let data = GuiData {
            vertex_alloc: VertexAlloc::default(),
            gui: Vec::with_capacity(4),
            vertex: vec![GuiVertex::default(); 500],
            indices: vec![0; 1000]
        };
        
        let mut gui = GuiModule {
            resources: Box::new(resources),
            descriptors: Box::default(),
            render: Box::new(render),
            data: Box::new(data),
            batches: Vec::with_capacity(16),
            update_batches: false,
        };

        gui.setup_pipelines(core)?;
        gui.setup_vertex_buffers(core)?;
        gui.setup_descriptors(core)?;
        gui.setup_render_data();

        Ok(gui)
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.descriptors.allocator.destroy(core);
        self.resources.text_pipeline.destroy(&core.ctx);
        self.resources.image_pipeline.destroy(&core.ctx);
        self.data.vertex_alloc.free(core);

        core.ctx.device.destroy_pipeline_layout(self.resources.pipeline_layout);
        core.ctx.device.destroy_descriptor_set_layout(self.resources.batch_layout);

        for texture in self.resources.textures.values() {
            core.destroy_texture(*texture);
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
        let render = *self.render;

        device.cmd_bind_index_buffer(cmd, render.vertex_buffer, render.index_offset, vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, slice::from_ref(&render.vertex_buffer), &render.vertex_offset);
        device.cmd_push_constants(cmd, render.pipeline_layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, push_values(&render.push_constants));

        let mut last_pipeline = vk::Pipeline::null();

        for batch in self.batches.iter() {
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

    fn create_gui(&mut self, id: GuiId) -> usize {
        let guis = &mut self.data.gui;
        let next_id = guis.len();

        guis.push(GuiView {
            sprites: Vec::new(),
        });

        id.bind(next_id as u32);

        next_id
    }

    fn update_gui<'a>(&mut self, core: &mut LoomzEngineCore, index: usize, sprites: &'a [GuiSprite]) -> Result<(), CommonError> {
        let gui = &mut self.data.gui[index];
        gui.sprites.clear();

        if gui.sprites.capacity() < sprites.len() {
            gui.sprites.reserve(sprites.len());
        }

        for &sprite in sprites.iter() {
            let image_view = match sprite.ty {
                loomz_shared::GuiSpriteType::Image(texture_id) => Self::fetch_texture_id(core, &mut self.resources, texture_id)?,
                loomz_shared::GuiSpriteType::Font(font_id) => Self::fetch_font_texture_view(core, &mut self.resources, font_id)?
            };

            gui.sprites.push(GuiViewSprite {
                image_view,
                sprite
            });
        }

        self.update_batches = true;

        Ok(())
    }

    fn api_update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        if let Some(updates) = api.gui().gui_updates() {
            for (id, sprites) in updates {
                let index = id.bound_value()
                    .unwrap_or_else(|| self.create_gui(id) );

                self.update_gui(core, index, sprites)?;
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

    fn fetch_font_texture_view(core: &mut LoomzEngineCore, resources: &mut GuiResources, font_id: MsdfFontId) -> Result<vk::ImageView, CommonError> {
        let asset_id = AssetId::MsdfFont(font_id);
        if let Some(texture) = resources.textures.get(&asset_id) {
            return Ok(texture.view);
        }

        let texture_asset = resources.assets.font(font_id)
            .ok_or_else(|| assets_err!("Unkown asset with ID {font_id:?}") )?;

        let texture = core.create_texture_from_font_asset(&texture_asset)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendGeneric, "Failed to create image from asset") )?;

        resources.textures.insert(asset_id, texture);

        Ok(texture.view)
    }

    fn fetch_texture_id(core: &mut LoomzEngineCore, resources: &mut GuiResources, texture_id: TextureId) -> Result<vk::ImageView, CommonError> {
        let asset_id = AssetId::Texture(texture_id);
        if let Some(texture) = resources.textures.get(&asset_id) {
            return Ok(texture.view);
        }
        
        let texture_asset = resources.assets.texture(texture_id)
            .ok_or_else(|| assets_err!("Unkown asset with ID {texture_id:?}") )?;

        let texture = core.create_texture_from_asset(&texture_asset)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendGeneric, "Failed to create image from asset") )?;

        resources.textures.insert(asset_id, texture);

        Ok(texture.view)
    }

}

impl GuiDescriptor {
    pub fn write_batch_texture(&mut self, image_view: vk::ImageView) -> Result<vk::DescriptorSet, CommonError> {
        self.allocator.write_set::<BATCH_LAYOUT_INDEX>(&[
            DescriptorWriteBinding::from_image_and_sampler(image_view, self.default_sampler, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        ])
    }

    pub fn reset_batch_layout(&mut self) {
        self.allocator.reset_layout::<BATCH_LAYOUT_INDEX>();
    }
}

impl PartialEq for GuiViewSprite {
    fn eq(&self, other: &Self) -> bool {
        self.image_view == other.image_view
    }
}

impl Eq for GuiViewSprite {

}

impl PartialOrd for GuiViewSprite {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.image_view.partial_cmp(&other.image_view)
    }
}

impl Ord for GuiViewSprite {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(&other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
