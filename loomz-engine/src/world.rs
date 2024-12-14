use std::{slice, u32};
use std::sync::Arc;
use fnv::FnvHashMap;
use loomz_engine_core::{LoomzEngineCore, VulkanContext, Texture, alloc::{VertexAlloc, StorageAlloc}, descriptors::*, pipelines::*};
use loomz_shared::api::{LoomzApi, WorldComponent, WorldComponentUpdate, WorldAnimation, WorldAnimationUpdate};
use loomz_shared::{assets::{LoomzAssetsBundle, TextureId}, CommonError, CommonErrorType};
use loomz_shared::{backend_init_err, assets_err, chain_err};

const WORLD_VERT_SRC: &[u8] = include_bytes!("../../assets/shaders/world.vert.spv");
const WORLD_FRAG_SRC: &[u8] = include_bytes!("../../assets/shaders/world.frag.spv");

const PUSH_STAGE_FLAGS: vk::ShaderStageFlags = vk::ShaderStageFlags::VERTEX;
const PUSH_SIZE: u32 = size_of::<WorldPushConstant>() as u32;

const GLOBAL_LAYOUT_INDEX: usize = 0;
const SPRITES_BUFFER_DATA_BINDING: usize = 0;

const BATCH_LAYOUT_INDEX: usize = 1;
const TEXTURE_BINDING: usize = 0;

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct WorldPushConstant {
    pub screen_width: f32,
    pub screen_height: f32,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct WorldVertex {
    pub pos: [f32; 2],
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct SpriteData {
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub uv_offset: [f32; 2],
    pub uv_size: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct WorldBatch {
    pub dst_set: vk::DescriptorSet,
    pub instances_count: u32,
    pub instances_offset: u32,
}

#[derive(Copy, Clone)]
struct WorldObject {
    image_view: vk::ImageView,
    sprite_index: u32,
    component: WorldComponent,
}

struct WorldModuleObjects {
    animations: Vec<WorldAnimation>,
    instances: Vec<WorldObject>,
}

struct WorldModuleDescriptors {
    alloc: DescriptorsAllocator,
    updates: DescriptorWriteBuffer,
    texture_params: DescriptorWriteImageParams,
}

struct WorldData {
    textures: FnvHashMap<TextureId, Texture>,
    descriptors: WorldModuleDescriptors,
    sprites: StorageAlloc<SpriteData>,
    objects: WorldModuleObjects,
}

#[repr(C)]
pub(crate) struct WorldModule {
    assets: Arc<LoomzAssetsBundle>,
    data: Box<WorldData>,

    pipeline: GraphicsPipeline,
    vertex: VertexAlloc<WorldVertex>,

    sprites_data_set: vk::DescriptorSet,

    push_constants: [WorldPushConstant; 1],
    batches: Vec<WorldBatch>,

    update_batches: bool,
}

impl WorldModule {

    pub fn init(core: &mut LoomzEngineCore, api: &LoomzApi) -> Result<Box<Self>, CommonError> {
        let objects = WorldModuleObjects {
            animations: Vec::with_capacity(16),
            instances: Vec::with_capacity(16),
        };

        let descriptors = WorldModuleDescriptors {
            alloc: DescriptorsAllocator::default(),
            updates: DescriptorWriteBuffer::default(),
            texture_params: DescriptorWriteImageParams::default(),
        };

        let data = WorldData {
            textures: FnvHashMap::default(),
            descriptors,
            sprites: StorageAlloc::default(),
            objects,
        };

        let mut world = WorldModule {
            assets: api.assets(),
            data: Box::new(data),

            pipeline: GraphicsPipeline::new(),
            vertex: VertexAlloc::default(),

            
            sprites_data_set: vk::DescriptorSet::null(),
           
            push_constants: [WorldPushConstant::default(); 1],
            batches: Vec::with_capacity(16),
            update_batches: false,
        };

        world.setup_pipeline(core)?;
        world.setup_descriptors(core)?;
        world.setup_vertex_buffers(core)?;
        world.setup_sprites_buffers(core)?;

        Ok(Box::new(world))
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.data.descriptors.alloc.destroy(core);

        for texture in self.data.textures.values() {
            core.destroy_texture(*texture);
        }

        self.pipeline.destroy(&core.ctx);
        self.vertex.free(core);
        self.data.sprites.free(core);
    }

    pub fn pipeline(&mut self) -> &mut GraphicsPipeline {
        &mut self.pipeline
    }

    pub fn set_output(&mut self, core: &LoomzEngineCore) {
        let extent = core.info.swapchain_extent;
        self.push_constants[0] = WorldPushConstant {
            screen_width: extent.width as f32,
            screen_height: extent.height as f32,
        };
    }

    pub fn rebuild(&mut self, core: &LoomzEngineCore) {
        let swapchain_extent = core.info.swapchain_extent;
        let push = &mut self.push_constants[0];
        push.screen_width = swapchain_extent.width as f32;
        push.screen_height = swapchain_extent.height as f32;
    }

    //
    // Rendering
    //
    
    #[inline(always)]
    fn push_values(&self) -> &[u8] {
        unsafe { self.push_constants.align_to::<u8>().1 }
    }

    pub fn render(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        let device = &ctx.device;
        let layout = self.pipeline.pipeline_layout();

        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline.handle());
        device.cmd_bind_index_buffer(cmd, self.vertex.buffer, self.vertex.index_offset(), vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, slice::from_ref(&self.vertex.buffer), &self.vertex.vertex_offset());
        device.cmd_push_constants(cmd, layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, self.push_values());
        device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, layout, GLOBAL_LAYOUT_INDEX as u32, slice::from_ref(&self.sprites_data_set), &[]);

        for batch in self.batches.iter() {
            device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, layout, BATCH_LAYOUT_INDEX as u32, slice::from_ref(&batch.dst_set), &[]);
            device.cmd_draw_indexed(cmd, 6, batch.instances_count, 0, 0, batch.instances_offset);
        }
    }

    //
    // Updates
    //
    
    fn update_world_component(&mut self, core: &mut LoomzEngineCore, update: WorldComponentUpdate, index: usize) -> Result<(), CommonError> {
        let component = update.component;
        let instance = &self.data.objects.instances[index];
        let old_texture_id = instance.component.texture_id;
        let mut image_view = instance.image_view;
        if old_texture_id != component.texture_id {
            image_view = self.fetch_texture_view(core, update.component.texture_id)?;
        }

        self.data.objects.instances[index] = WorldObject {
            component,
            image_view,
            sprite_index: u32::MAX,
        };
        
        Ok(())
    }

    fn create_world_component(&mut self, core: &mut LoomzEngineCore, update: WorldComponentUpdate) -> Result<(), CommonError> {
        let component = update.component;
        let image_view = self.fetch_texture_view(core, component.texture_id)?;
        let obj = WorldObject {
            component,
            image_view,
            sprite_index: u32::MAX,
        };

        let instances = &mut self.data.objects.instances;
        let new_id = instances.len() as u32;
        instances.push(obj);
        update.uid.bind(new_id);

        Ok(())
    }

    fn create_world_animation(&mut self, update: WorldAnimationUpdate) {
        let animations = &mut self.data.objects.animations;
        let new_id = animations.len() as u32;
        animations.push(update.animation);
        update.uid.bind(new_id);
    }

    fn api_update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        if let Some(animations) = api.world().animations() {
            for update in animations {
                match update.uid.bound_value() {
                    Some(index) => { },
                    None => {
                        self.create_world_animation(update);
                    }
                }
            }
        }
        
        if let Some(components) = api.world().components() {
            for update in components {
                match update.uid.bound_value() {
                    Some(index) => self.update_world_component(core, update, index)?,
                    None => {
                        self.create_world_component(core, update)?;
                        self.update_batches = true;
                    }
                }
                
            }
        }

        Ok(())
    }

    fn batches_update(&mut self, core: &mut LoomzEngineCore) {
        if !self.update_batches {
            return;
        }

        self.clear_batches();

        if !self.data.objects.instances.is_empty() {
            self.build_batches(core);
        }

        self.update_batches = false;
    }

    fn animation_update(&mut self, core: &mut LoomzEngineCore) {

    }

    pub fn update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        self.api_update(api, core)?;
        self.animation_update(core);
        self.batches_update(core);
        Ok(())
    }

    //
    // Batching
    //

    fn clear_batches(&mut self) {
        self.data.descriptors.alloc.clear_sets(BATCH_LAYOUT_INDEX);
        self.batches.clear();
    }
   
    fn build_batches(&mut self, core: &mut LoomzEngineCore) {
        let mut batcher = WorldBatcher::new(self);
        batcher.build();
        self.data.descriptors.updates.submit(core);
    }

    //
    // Data
    //
   
    fn fetch_texture_view(&mut self, core: &mut LoomzEngineCore, id: TextureId) -> Result<vk::ImageView, CommonError> {
        if let Some(texture) = self.data.textures.get(&id) {
            return Ok(texture.view);
        }

        let texture_asset = self.assets.texture(id)
            .ok_or_else(|| assets_err!("Unkown asset with ID {id:?}") )?;

        let texture = core.create_texture_from_asset(&texture_asset)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendGeneric, "Failed to create image from asset") )?;

        self.data.textures.insert(id, texture);

        Ok(texture.view)
    }
    
    //
    // Setup
    //

    fn pipeline_descriptor_bindings_batch() -> &'static [PipelineLayoutSetBinding; 1] {
        &[
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                descriptor_count: 1,
            },
        ]
    }

    fn pipeline_descriptor_bindings_global() -> &'static [PipelineLayoutSetBinding; 1] {
        &[
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                descriptor_count: 1,
            },
        ]
    }

    fn setup_pipeline(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let ctx = &core.ctx;

        // Descriptor set layouts
        let bindings_global = Self::pipeline_descriptor_bindings_global();
        let layout_global = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, bindings_global)
            .map_err(|err| backend_init_err!("Failed to create global descriptor set layout: {}", err) )?;

        let bindings_batch = Self::pipeline_descriptor_bindings_batch();
        let layout_batch = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, bindings_batch)
            .map_err(|err| backend_init_err!("Failed to create batch descriptor set layout: {}", err) )?;

        let layouts = &[layout_global, layout_batch];

        // Pipeline layout
        let constant_range = vk::PushConstantRange {
            offset: 0,
            size: ::std::mem::size_of::<WorldPushConstant>() as u32,
            stage_flags: vk::ShaderStageFlags::VERTEX,
        };
        let pipeline_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: layouts.len() as u32,
            p_set_layouts: layouts.as_ptr(),
            push_constant_range_count: 1,
            p_push_constant_ranges: &constant_range,
            ..Default::default()
        };
        let pipeline_layout = ctx.device.create_pipeline_layout(&pipeline_create_info)
            .map_err(|err| backend_init_err!("Failed to create pipeline layout: {}", err) )?;
        

        // Shader source
        let modules = GraphicsShaderModules::new(ctx, WORLD_VERT_SRC, WORLD_FRAG_SRC)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to compute world pipeline shader modules") )?;

        // Pipeline
        let vertex_fields = [
            PipelineVertexFormat {
                location: 0,
                offset: 0,
                format: vk::Format::R32G32_SFLOAT,
            },
        ];

        let pipeline = &mut self.pipeline;
        pipeline.set_shader_modules(modules);
        pipeline.set_vertex_format::<WorldVertex>(&vertex_fields);
        pipeline.set_pipeline_layout(pipeline_layout);
        pipeline.set_descriptor_set_layout(GLOBAL_LAYOUT_INDEX as usize, layout_global);
        pipeline.set_descriptor_set_layout(BATCH_LAYOUT_INDEX as usize, layout_batch);
        pipeline.set_depth_testing(false);
        pipeline.rasterization(&vk::PipelineRasterizationStateCreateInfo {
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            ..Default::default()
        });
        pipeline.blending(
            &vk::PipelineColorBlendAttachmentState {
                blend_enable: 1,
                src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                src_alpha_blend_factor: vk::BlendFactor::ZERO,
                dst_alpha_blend_factor: vk::BlendFactor::ONE,
                ..Default::default()
            },
            &vk::PipelineColorBlendStateCreateInfo {
                attachment_count: 1,
                ..Default::default()
            }
        );

        let info = &core.info;
        pipeline.set_sample_count(info.sample_count);
        pipeline.set_color_attachment_format(info.color_format);
        pipeline.set_depth_attachment_format(info.depth_format);

        Ok(())
    }

    fn setup_descriptors(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        use loomz_engine_core::descriptors::DescriptorsAllocation;
        
        let allocations = [
            DescriptorsAllocation {
                layout: self.pipeline.descriptor_set_layout(GLOBAL_LAYOUT_INDEX),
                bindings: Self::pipeline_descriptor_bindings_global(),
                count: 1,
            },
            DescriptorsAllocation {
                layout: self.pipeline.descriptor_set_layout(BATCH_LAYOUT_INDEX),
                bindings: Self::pipeline_descriptor_bindings_batch(),
                count: 10,
            },
        ];

        self.data.descriptors.alloc = DescriptorsAllocator::new(core, &allocations)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to prellocate descriptor sets") )?;

        self.data.descriptors.texture_params = DescriptorWriteImageParams {
            sampler: core.resources.linear_sampler,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            dst_binding: TEXTURE_BINDING as u32,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };

        Ok(())
    }

    fn setup_vertex_buffers(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let vertex_capacity = 4;
        let index_capacity = 6;
        self.vertex = VertexAlloc::new(core, index_capacity, vertex_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create vertex alloc: {err}") )?;

        let indices = [0, 1, 2, 2, 3, 1];
        let vertex = [
            WorldVertex { pos: [0.0,   0.0] },
            WorldVertex { pos: [1.0,   0.0] },
            WorldVertex { pos: [0.0,   1.0] },
            WorldVertex { pos: [1.0,   1.0] },
        ];

        self.vertex.set_data(core, &indices, &vertex);

        Ok(())
    }

    fn setup_sprites_buffers(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let descriptors = &mut self.data.descriptors;

        let sprites_capacity = 100;
        self.data.sprites = StorageAlloc::new(core, sprites_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create storage alloc: {err}") )?;

        self.sprites_data_set = descriptors.alloc.next_set(GLOBAL_LAYOUT_INDEX);
        descriptors.updates.write_storage_buffer(self.sprites_data_set, &self.data.sprites, SPRITES_BUFFER_DATA_BINDING as u32);
        descriptors.updates.submit(core);

        Ok(())
    }

}

struct WorldBatcher<'a> {
    current_view: vk::ImageView,
    batches: &'a mut Vec<WorldBatch>,
    data: &'a mut WorldData,
    instance_index: usize,
    batch_index: usize,
}

impl<'a> WorldBatcher<'a> {

    fn new(world: &'a mut WorldModule) -> Self {
        let mut batcher = WorldBatcher {
            current_view: vk::ImageView::null(),
            batches: &mut world.batches,
            data: &mut world.data,
            instance_index: 0,
            batch_index: 0,
        };

        batcher.first_batch();

        batcher
    }

    fn build(&mut self) {
        let max_instance = self.data.objects.instances.len();
        while self.instance_index != max_instance {
            let instance = &mut self.data.objects.instances[self.instance_index];
            instance.sprite_index = self.instance_index as u32;

            let image_view = instance.image_view;            
            if self.current_view == image_view {
                self.batches[self.batch_index].instances_count += 1;
            } else {
                self.next_batch(image_view)
            }

            self.instance_index += 1;
        }
    }

    fn first_batch(&mut self) {
        let instance = &mut self.data.objects.instances[0];
        instance.sprite_index = self.instance_index as u32;

        let dst_set = self.data.descriptors.alloc.next_set(BATCH_LAYOUT_INDEX);
        self.data.descriptors.updates.write_simple_image(dst_set, instance.image_view, &self.data.descriptors.texture_params);
        
        let component = instance.component;
        self.data.sprites.write_data(self.instance_index, SpriteData {
            offset: component.position.splat(),
            size: component.size.splat(),
            uv_offset: component.uv.offset(),
            uv_size: component.uv.size(),
        });

        self.batches.push(WorldBatch {
            dst_set,
            instances_count: 1,
            instances_offset: 0,
        });

        self.current_view = instance.image_view;
        self.instance_index += 1;
    }

    fn next_batch(&mut self, image_view: vk::ImageView) {
        let dst_set = self.data.descriptors.alloc.next_set(BATCH_LAYOUT_INDEX);
        self.data.descriptors.updates.write_simple_image(dst_set, image_view, &self.data.descriptors.texture_params);

        self.batches.push(WorldBatch {
            dst_set,
            instances_count: 1,
            instances_offset: self.instance_index as u32,
        });

        self.current_view = image_view;
        self.batch_index += 1;
    }

}
