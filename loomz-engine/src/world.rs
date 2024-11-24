use std::slice;
use std::sync::Arc;
use fnv::FnvHashMap;
use loomz_engine_core::{LoomzEngineCore, VulkanContext, Texture, alloc::VertexAlloc, descriptors::*, pipelines::*};
use loomz_shared::{CommonError, CommonErrorType, assets::{LoomzAssetsBundle, TextureId}, api::{WorldEngineApi, WorldComponent}};
use loomz_shared::{backend_init_err, assets_err, chain_err};

const WORLD_VERT_SRC: &[u8] = include_bytes!("../../assets/shaders/world.vert.spv");
const WORLD_FRAG_SRC: &[u8] = include_bytes!("../../assets/shaders/world.frag.spv");

const PUSH_STAGE_FLAGS: vk::ShaderStageFlags = vk::ShaderStageFlags::VERTEX;
const PUSH_SIZE: u32 = size_of::<WorldPushConstant>() as u32;

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
    pub uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WorldBatch {
    pub set: vk::DescriptorSet,
    pub index_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
}

struct WorldObject {
    component: WorldComponent,
    image_view: vk::ImageView,
    vertex_offset: u32,
}

struct WorldModuleObjects {
    instances: Vec<WorldObject>,
    index: Vec<u32>,
    vertex: Vec<WorldVertex>,
}

struct WorldModuleDescriptors {
    alloc: DescriptorsAllocator,
    write: DescriptorWriteData,
    texture_params: DescriptorWriteImageParams,
}

struct WorldData {
    assets: Arc<LoomzAssetsBundle>,
    textures: FnvHashMap<TextureId, Texture>,
    descriptors: WorldModuleDescriptors,
    objects: WorldModuleObjects,
}

pub(crate) struct WorldModule {
    api: WorldEngineApi,

    pipeline: GraphicsPipeline,
    vertex: VertexAlloc<WorldVertex>,
    data: Box<WorldData>,

    push_constants: [WorldPushConstant; 1],
    batches: Vec<WorldBatch>,
}

impl WorldModule {

    pub fn init(core: &mut LoomzEngineCore, assets: &Arc<LoomzAssetsBundle>, api: WorldEngineApi) -> Result<Box<Self>, CommonError> {
        let objects = WorldModuleObjects {
            instances: Vec::with_capacity(16),
            index: Vec::with_capacity(3000),
            vertex: Vec::with_capacity(2000)
        };

        let descriptors = WorldModuleDescriptors {
            alloc: DescriptorsAllocator::default(),
            write: DescriptorWriteData::default(),
            texture_params: DescriptorWriteImageParams::default(),
        };

        let data = WorldData {
            assets: Arc::clone(assets),
            textures: FnvHashMap::default(),
            objects,
            descriptors,
        };

        let mut world = WorldModule {
            api,
            pipeline: GraphicsPipeline::new(),
            vertex: VertexAlloc::default(),
            data: Box::new(data),
            push_constants: [WorldPushConstant::default(); 1],
            batches: Vec::with_capacity(16),
        };

        world.setup_pipeline(core)?;
        world.setup_buffers(core)?;
        world.setup_descriptors(core)?;

        Ok(Box::new(world))
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.data.descriptors.alloc.destroy(core);

        for texture in self.data.textures.values() {
            core.destroy_texture(*texture);
        }

        self.pipeline.destroy(&core.ctx);
        self.vertex.free(core);
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

    pub fn update(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let mut update_batches = false;

        while let Some(component) = self.api.recv_component() {
            self.update_component(core, component)?;
            update_batches = true;
        }

        if update_batches {
            self.update_batches();
            self.upload_batch_data(core);
            self.write_descriptor_sets(core);
        }

        Ok(())
    }

    pub fn render(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        let device = &ctx.device;

        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline.handle());
        device.cmd_bind_index_buffer(cmd, self.vertex.buffer, self.vertex.index_offset(), vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, slice::from_ref(&self.vertex.buffer), &self.vertex.vertex_offset());

        let layout = self.pipeline.pipeline_layout();
        device.cmd_push_constants(cmd, layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, self.push_values());

        for batch in self.batches.iter() {
            device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, layout, 0, slice::from_ref(&batch.set), &[]);
            device.cmd_draw_indexed(cmd, batch.index_count, 1, batch.first_index, batch.vertex_offset, 0);
        }
    }

    fn push_values(&self) -> &[u8] {
        unsafe { self.push_constants.align_to::<u8>().1 }
    }

    //
    // Updates
    //

    fn update_component(&mut self, core: &mut LoomzEngineCore, component: WorldComponent) -> Result<(), CommonError> {
        let texture = self.fetch_texture(core, component.texture_id)?;
        let image_view = texture.view;
        let obj = WorldObject {
            component,
            image_view,
            vertex_offset: 0,
        };
        
        self.data.objects.instances.push(obj);
        
        Ok(())
    }

    fn update_batches(&mut self) {
        if self.clear_batches() {
            return;
        }

        let objects = &mut self.data.objects;
        unsafe {
            objects.index.set_len(objects.instances.len() * 6);
            objects.vertex.set_len(objects.instances.len() * 4);
        }

        let mut index_offset = 0;
        let mut vertex_offset = 0;
        let mut last_view = objects.instances[0].image_view;
        let mut next_batch = WorldBatch::default();

        for instance in objects.instances.iter_mut() {
            if instance.image_view != last_view {
                Self::generate_batch(&mut self.batches, &mut self.data.descriptors, &mut next_batch, instance.image_view);
                last_view = instance.image_view;
            }

            Self::generate_indices(index_offset, vertex_offset, &mut objects.index);
            Self::generate_vertex(vertex_offset, &mut objects.vertex, instance.component);
            instance.vertex_offset = vertex_offset;

            index_offset += 6;
            vertex_offset += 4;
            next_batch.index_count += 6;
        }

        if next_batch.index_count > 0 {
            Self::generate_batch(&mut self.batches, &mut self.data.descriptors, &mut next_batch, last_view);
        }
    }

    fn generate_batch(
        batches: &mut Vec<WorldBatch>,
        descriptors: &mut WorldModuleDescriptors,
        batch: &mut WorldBatch,
        image_view: vk::ImageView
    ) {
        let next_set = descriptors.alloc.next_set(0);

        let mut new_batch = *batch;
        new_batch.set = next_set;
        batches.push(new_batch);

        descriptors.write.write_simple_image(next_set, image_view, &descriptors.texture_params);

        *batch = WorldBatch::default();
    }

    fn clear_batches(&mut self) -> bool {
        self.data.objects.index.clear();
        self.data.objects.vertex.clear();
        self.data.descriptors.alloc.clear_sets(0);
        self.data.descriptors.write.clear();
        self.batches.clear();
        self.data.objects.instances.len() == 0
    }

    fn generate_indices(index_offset: u32, vertex_offset: u32, indices: &mut [u32]) {
        let i = index_offset as usize;
        let v = vertex_offset;
        indices[i+0] = v;
        indices[i+1] = v+2;
        indices[i+2] = v+1;
        indices[i+3] = v+2;
        indices[i+4] = v+3;
        indices[i+5] = v+1;
    }
    
    fn generate_vertex(vertex_offset: u32, vertex: &mut [WorldVertex], component: WorldComponent) {
        let v = vertex_offset as usize;
        let [x, y] = component.position.splat();
        let [w, h] = component.size.splat();
        vertex[v+0] = WorldVertex { pos: [x,   y],   uv: [0.0, 0.0] };
        vertex[v+1] = WorldVertex { pos: [x+w, y],   uv: [1.0, 0.0] };
        vertex[v+2] = WorldVertex { pos: [x,   y+h], uv: [0.0, 1.0] };
        vertex[v+3] = WorldVertex { pos: [x+w, y+h], uv: [1.0, 1.0] };
    }

    //
    // Upload
    //

    fn upload_batch_data(&self, core: &mut LoomzEngineCore) {
        self.vertex.set_data(core, &self.data.objects.index, &self.data.objects.vertex);
    }

    fn fetch_texture(&mut self, core: &mut LoomzEngineCore, id: TextureId) -> Result<Texture, CommonError> {
        if let Some(texture) = self.data.textures.get(&id) {
            return Ok(*texture);
        }

        let texture_asset = self.data.assets.texture(id)
            .ok_or_else(|| assets_err!("Unkown asset with ID {id:?}") )?;

        let texture = core.create_texture_from_asset(&texture_asset)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendGeneric, "Failed to create image from asset") )?;

        self.data.textures.insert(id, texture);

        Ok(texture)
    }

    fn write_descriptor_sets(&mut self, core: &mut LoomzEngineCore) {
        let write_data = self.data.descriptors.write.write_pointers();
        core.ctx.device.update_descriptor_sets(write_data, &[]);
    }

    //
    // Setup
    //

    fn pipeline_descriptor_bindings() -> &'static [PipelineLayoutSetBinding; 1] {
        &[
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                descriptor_count: 1,
            },
        ]
    }

    fn setup_pipeline(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let ctx = &core.ctx;

        // Descriptor set layouts
        let bindings_batch = Self::pipeline_descriptor_bindings();
        let layout_batch = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, bindings_batch)
            .map_err(|err| backend_init_err!("Failed to create global descriptor set layout: {}", err) )?;

        // Pipeline layout
        let constant_range = vk::PushConstantRange {
            offset: 0,
            size: ::std::mem::size_of::<WorldPushConstant>() as u32,
            stage_flags: vk::ShaderStageFlags::VERTEX,
        };
        let pipeline_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 1,
            p_set_layouts: &layout_batch,
            push_constant_range_count: 1,
            p_push_constant_ranges: &constant_range,
            ..Default::default()
        };
        let pipeline_layout = ctx.device.create_pipeline_layout(&pipeline_create_info)
            .map_err(|err| backend_init_err!("Failed to create pipeline layout: {}", err) )?;
        

        // Shader source
        let modules = GraphicsShaderModules::new(ctx, WORLD_VERT_SRC, WORLD_FRAG_SRC)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to compute debug pipeline shader modules") )?;

        // Pipeline
        let vertex_fields = [
            PipelineVertexFormat {
                location: 0,
                offset: 0,
                format: vk::Format::R32G32_SFLOAT,
            },
            PipelineVertexFormat {
                location: 1,
                offset: 8,
                format: vk::Format::R32G32_SFLOAT,
            },
        ];

        let pipeline = &mut self.pipeline;
        pipeline.set_shader_modules(modules);
        pipeline.set_vertex_format::<WorldVertex>(&vertex_fields);
        pipeline.set_pipeline_layout(pipeline_layout);
        pipeline.set_descriptor_set_layout(0, layout_batch);
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

    fn setup_buffers(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let vertex_capacity = 2000;
        let index_capacity = 3000;
        self.vertex = VertexAlloc::new(core, vertex_capacity, index_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create vertex alloc: {err}") )?;
        Ok(())
    }

    fn setup_descriptors(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        use loomz_engine_core::descriptors::DescriptorsAllocation;
        
        let allocations = [
            DescriptorsAllocation {
                layout: self.pipeline.descriptor_set_layout(0),
                bindings: Self::pipeline_descriptor_bindings(),
                count: 10,
            },
        ];

        self.data.descriptors.alloc = DescriptorsAllocator::new(core, &allocations)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to prellocate descriptor sets: {err}") )?;

        self.data.descriptors.texture_params = DescriptorWriteImageParams {
            sampler: core.resources.linear_sampler,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            dst_binding: 0,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };

        Ok(())
    }

}

impl Default for WorldBatch {
    fn default() -> Self {
        WorldBatch {
            set: vk::DescriptorSet::null(),
            first_index: 0,
            index_count: 0,
            vertex_offset: 0
        }
    }
}