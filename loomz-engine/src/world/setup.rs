use loomz_engine_core::{LoomzEngineCore, alloc::{VertexAlloc, StorageAlloc}, descriptors::*, pipelines::*};
use loomz_shared::{CommonError, CommonErrorType};
use loomz_shared::{backend_init_err, chain_err};
use super::{WorldPushConstant, WorldVertex, WorldDebugVertex};

const WORLD_VERT_SRC: &[u8] = include_bytes!("../../../assets/shaders/world.vert.spv");
const WORLD_FRAG_SRC: &[u8] = include_bytes!("../../../assets/shaders/world.frag.spv");

const WORLD_DEBUG_VERT_SRC: &[u8] = include_bytes!("../../../assets/shaders/debug.vert.spv");
const WORLD_DEBUG_FRAG_SRC: &[u8] = include_bytes!("../../../assets/shaders/debug.frag.spv");


impl super::WorldModule {

    pub(super) fn setup_pipeline(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let ctx = &core.ctx;

        // Descriptor set layouts
        let bindings_global = [
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                stage_flags: vk::ShaderStageFlags::VERTEX,
            },
        ];
        
        let layout_global = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, &bindings_global)
            .map_err(|err| backend_init_err!("Failed to create global descriptor set layout: {}", err) )?;

        let bindings_batch = [
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
            },
        ];
        let layout_batch = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, &bindings_batch)
            .map_err(|err| backend_init_err!("Failed to create batch descriptor set layout: {}", err) )?;

        // Pipeline layout
        let layouts = [layout_global, layout_batch];

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

        self.resources.global_layout = layout_global;
        self.resources.batch_layout = layout_batch;
        self.resources.pipeline_layout = pipeline_layout;

        let pipeline = &mut self.resources.pipeline;
        pipeline.set_shader_modules(modules);
        pipeline.set_vertex_format::<WorldVertex>(&vertex_fields);
        pipeline.set_pipeline_layout(pipeline_layout);
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

    pub(super) fn setup_debug_pipeline(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let ctx = &core.ctx;

        // Pipeline layout
        // The debug pipeline does not use any descriptor sets
        let constant_range = vk::PushConstantRange {
            offset: 0,
            size: ::std::mem::size_of::<WorldPushConstant>() as u32,
            stage_flags: vk::ShaderStageFlags::VERTEX,
        };

        let pipeline_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 0,
            p_set_layouts: ::std::ptr::null(),
            push_constant_range_count: 1,
            p_push_constant_ranges: &constant_range,
            ..Default::default()
        };

        let pipeline_layout = ctx.device.create_pipeline_layout(&pipeline_create_info)
            .map_err(|err| backend_init_err!("Failed to create debug pipeline layout: {}", err) )?;
        

        // Shader source
        let modules = GraphicsShaderModules::new(ctx, WORLD_DEBUG_VERT_SRC, WORLD_DEBUG_FRAG_SRC)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to compute world debug pipeline shader modules") )?;

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
                format: vk::Format::R8G8B8A8_UNORM,
            },
        ];

        self.resources.debug_pipeline_layout = pipeline_layout;

        let pipeline = &mut self.resources.debug_pipeline;
        pipeline.set_shader_modules(modules);
        pipeline.set_vertex_format::<WorldDebugVertex>(&vertex_fields);
        pipeline.set_pipeline_layout(pipeline_layout);
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
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::ONE,
                dst_color_blend_factor: vk::BlendFactor::ZERO,
                src_alpha_blend_factor: vk::BlendFactor::ONE,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
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

    pub(super) fn setup_descriptors(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let allocations = [
            DescriptorsAllocation {
                layout: self.resources.global_layout,
                binding_types: &[vk::DescriptorType::STORAGE_BUFFER],
                count: 1,
            },
            DescriptorsAllocation {
                layout: self.resources.batch_layout,
                binding_types: &[vk::DescriptorType::COMBINED_IMAGE_SAMPLER],
                count: 10,
            },
        ];

        self.descriptors.default_sampler = core.resources.linear_sampler;
        self.descriptors.allocator = DescriptorsAllocator::new(core, &allocations)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to prellocate descriptor sets") )?;

        Ok(())
    }

    pub(super) fn setup_vertex_buffer(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let vertex_capacity = 4;
        let index_capacity = 6;
        self.resources.vertex = VertexAlloc::new(core, index_capacity, vertex_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create vertex alloc: {err}") )?;

        let indices = [0, 1, 2, 2, 3, 1];
        let vertex = [
            WorldVertex { pos: [0.0,   0.0] },
            WorldVertex { pos: [1.0,   0.0] },
            WorldVertex { pos: [0.0,   1.0] },
            WorldVertex { pos: [1.0,   1.0] },
        ];

        self.resources.vertex.set_data(core, &indices, &vertex);

        Ok(())
    }

    pub(super) fn setup_debug_vertex_buffer(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let vertex_capacity = 1000;
        let index_capacity = 1500;
        
        self.resources.debug_vertex = VertexAlloc::new(core, index_capacity, vertex_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create debug vertex alloc: {err}") )?;

        Ok(())
    }

    pub(super) fn setup_sprites_buffers(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {  
        let sprites_capacity = 100;
        self.data.sprites = StorageAlloc::new(core, sprites_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create storage alloc: {err}") )?;

        self.render.sprites = self.descriptors.write_sprite_buffer(&self.data.sprites)?;

        Ok(())
    }

    pub(super) fn setup_render_data(&mut self) {
        let res = &self.resources;

        let render = &mut self.render;
        render.pipeline_layout = res.pipeline_layout;
        render.vertex_buffer = [res.vertex.buffer];
        render.index_offset = res.vertex.index_offset();
        render.vertex_offset = res.vertex.vertex_offset();

        let render = &mut self.debug_render;
        render.pipeline_layout = res.debug_pipeline_layout;
        render.vertex_buffer = [res.debug_vertex.buffer];
        render.index_offset = res.debug_vertex.index_offset();
        render.vertex_offset = res.debug_vertex.vertex_offset();
    }

}
