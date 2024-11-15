use std::slice;
use loomz_engine_core::{pipelines::GraphicsPipeline, alloc::VertexAlloc, descriptors::DescriptorsAlloc, LoomzEngineCore, VulkanContext};
use loomz_shared::{backend_init_err, chain_err, CommonError, CommonErrorType};

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
    pub color: [u8; 4],
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct WorldBatch {
    pub index_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
}

pub(crate) struct WorldModule {
    pipeline: GraphicsPipeline,
    vertex: VertexAlloc<WorldVertex>,
    descriptors: DescriptorsAlloc,
    push_constants: [WorldPushConstant; 1],
    batches: Vec<WorldBatch>
}

impl WorldModule {

    pub fn init(core: &mut LoomzEngineCore) -> Result<Self, CommonError> {
        let mut world = WorldModule {
            pipeline: GraphicsPipeline::new(),
            vertex: VertexAlloc::default(),
            descriptors: DescriptorsAlloc::default(),
            push_constants: [WorldPushConstant::default(); 1],
            batches: Vec::with_capacity(16)
        };

        world.setup_pipeline(core)?;
        world.setup_buffers(core)?;

        Ok(world)
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.pipeline.destroy(&core.ctx);
        self.vertex.free(core);
    }

    pub fn pipeline(&mut self) -> &mut GraphicsPipeline {
        &mut self.pipeline
    }

    pub fn update(&mut self, core: &mut LoomzEngineCore) {
        
    }

    pub fn render(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        let device = &ctx.device;

        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline.handle());
        device.cmd_bind_index_buffer(cmd, self.vertex.buffer, self.vertex.index_offset(), vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, slice::from_ref(&self.vertex.buffer), &self.vertex.vertex_offset());

        let layout = self.pipeline.pipeline_layout();
        device.cmd_push_constants(cmd, layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, self.push_values());

        for batch in self.batches.iter() {
            device.cmd_draw_indexed(cmd, batch.index_count, 1, batch.first_index, batch.vertex_offset, 0);
        }
    }

    fn push_values(&self) -> &[u8] {
        unsafe { self.push_constants.align_to::<u8>().1 }
    }

    //
    // Setup
    //

    fn setup_pipeline(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        use loomz_engine_core::pipelines::*;

        let ctx = &core.ctx;

        // Descriptor set layouts
        let bindings_global = &[];
        let layout_global = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, &bindings_global)
            .map_err(|err| backend_init_err!("Failed to create global descriptor set layout: {}", err) )?;

        // Pipeline layout
        let constant_range = vk::PushConstantRange {
            offset: 0,
            size: ::std::mem::size_of::<WorldPushConstant>() as u32,
            stage_flags: vk::ShaderStageFlags::VERTEX,
        };
        let pipeline_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 1,
            p_set_layouts: &layout_global,
            push_constant_range_count: 1,
            p_push_constant_ranges: &constant_range,
            ..Default::default()
        };
        let pipeline_layout = ctx.device.create_pipeline_layout(&pipeline_create_info)
            .map_err(|err| backend_init_err!("Failed to create pipeline layout: {}", err) )?;
        

        // Shader source
        let modules = GraphicsShaderModules::new(ctx, WORLD_VERT_SRC, WORLD_FRAG_SRC)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to compute debug pipeline shader modules: {}", err) )?;

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
            PipelineVertexFormat {
                location: 2,
                offset: 16,
                format: vk::Format::R8G8B8A8_UNORM,
            },
        ];

        let pipeline = &mut self.pipeline;
        pipeline.set_shader_modules(modules);
        pipeline.set_vertex_format::<WorldVertex>(&vertex_fields);
        pipeline.set_pipeline_layout(pipeline_layout);
        pipeline.set_layout_bindings_global(layout_global);
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
        let index_capacity = vertex_capacity + (vertex_capacity / 3);
        self.vertex = VertexAlloc::new(core, vertex_capacity, index_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create vertex alloc: {err}") )?;
        Ok(())
    }
    

}
