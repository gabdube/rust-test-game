use loomz_engine_core::{LoomzEngineCore, alloc::{VertexAlloc, StorageAlloc}, descriptors::*, pipelines::*};
use loomz_shared::{CommonError, CommonErrorType};
use loomz_shared::{backend_init_err, assets_err, chain_err};
use super::{WorldPushConstant, WorldVertex, WorldDebugVertex};

const WORLD_TERRAIN_VERT_SRC: &[u8] = include_bytes!("../../../assets/shaders/world_terrain.vert.spv");
const WORLD_TERRAIN_FRAG_SRC: &[u8] = include_bytes!("../../../assets/shaders/world_terrain.frag.spv");

const WORLD_ACTORS_VERT_SRC: &[u8] = include_bytes!("../../../assets/shaders/world_actors.vert.spv");
const WORLD_ACTORS_FRAG_SRC: &[u8] = include_bytes!("../../../assets/shaders/world_actors.frag.spv");

const WORLD_DEBUG_VERT_SRC: &[u8] = include_bytes!("../../../assets/shaders/world_debug.vert.spv");
const WORLD_DEBUG_FRAG_SRC: &[u8] = include_bytes!("../../../assets/shaders/world_debug.frag.spv");


impl super::WorldModule {

    pub(super) fn setup_terrain_pipeline(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let ctx = &core.ctx;
        let pipeline = &mut self.resources.pipelines.terrain;
        let pipeline_layout = &mut self.resources.pipelines.terrain_layout;
        let global_layout = &mut self.resources.terrain_global_layout;

        // Descriptor set layouts
        let bindings_global = [
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
            },
        ];
        *global_layout = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, &bindings_global)
            .map_err(|err| backend_init_err!("Failed to create globat descriptor set layout: {}", err) )?;

        // Pipeline layout
        let layouts = [*global_layout,];

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

        *pipeline_layout = ctx.device.create_pipeline_layout(&pipeline_create_info)
            .map_err(|err| backend_init_err!("Failed to create world terrain pipeline layout: {}", err) )?;

        // Shader source
        let modules = GraphicsShaderModules::new(ctx, WORLD_TERRAIN_VERT_SRC, WORLD_TERRAIN_FRAG_SRC)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to compute world terrain pipeline shader modules") )?;

        // Pipeline
        let vertex_fields = [
            PipelineVertexFormat {
                location: 0,
                offset: 0,
                format: vk::Format::R32G32_SFLOAT,
            },
        ];

        pipeline.set_shader_modules(modules);
        pipeline.set_vertex_format::<WorldVertex>(&vertex_fields);
        pipeline.set_pipeline_layout(*pipeline_layout);
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

    pub(super) fn setup_actors_pipeline(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let ctx = &core.ctx;
        let pipeline = &mut self.resources.pipelines.actors;
        let pipeline_layout = &mut self.resources.pipelines.actors_layout;
        let global_layout = &mut self.resources.actor_global_layout;
        let batch_layout = &mut self.resources.actor_batch_layout;

        // Descriptor set layouts
        let bindings_global = [
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                stage_flags: vk::ShaderStageFlags::VERTEX,
            },
        ];
        
        *global_layout = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, &bindings_global)
            .map_err(|err| backend_init_err!("Failed to create global descriptor set layout: {}", err) )?;

        let bindings_batch = [
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
            },
        ];
        *batch_layout = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, &bindings_batch)
            .map_err(|err| backend_init_err!("Failed to create batch descriptor set layout: {}", err) )?;

        // Pipeline layout
        let layouts = [*global_layout, *batch_layout];

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

        *pipeline_layout = ctx.device.create_pipeline_layout(&pipeline_create_info)
            .map_err(|err| backend_init_err!("Failed to create pipeline layout: {}", err) )?;
        

        // Shader source
        let modules = GraphicsShaderModules::new(ctx, WORLD_ACTORS_VERT_SRC, WORLD_ACTORS_FRAG_SRC)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to compute world pipeline shader modules") )?;

        // Pipeline
        let vertex_fields = [
            PipelineVertexFormat {
                location: 0,
                offset: 0,
                format: vk::Format::R32G32_SFLOAT,
            },
        ];

        pipeline.set_shader_modules(modules);
        pipeline.set_vertex_format::<WorldVertex>(&vertex_fields);
        pipeline.set_pipeline_layout(*pipeline_layout);
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
        let pipeline = &mut self.resources.pipelines.debug;
        let pipeline_layout = &mut self.resources.pipelines.debug_layout;

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

        *pipeline_layout = ctx.device.create_pipeline_layout(&pipeline_create_info)
            .map_err(|err| backend_init_err!("Failed to create world debug pipeline layout: {}", err) )?;
        

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

        pipeline.set_shader_modules(modules);
        pipeline.set_vertex_format::<WorldDebugVertex>(&vertex_fields);
        pipeline.set_pipeline_layout(*pipeline_layout);
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
                layout: self.resources.terrain_global_layout,
                binding_types: &[vk::DescriptorType::COMBINED_IMAGE_SAMPLER],
                count: 1,
            },
            DescriptorsAllocation {
                layout: self.resources.actor_global_layout,
                binding_types: &[vk::DescriptorType::STORAGE_BUFFER],
                count: 1,
            },
            DescriptorsAllocation {
                layout: self.resources.actor_batch_layout,
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

        self.render.actors.sprites_set = self.descriptors.write_sprite_buffer(&self.data.sprites)?;

        Ok(())
    }

    pub(super) fn setup_terrain_sampler(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let terrain_id = self.resources.assets.texture_id_by_name("terrain")
            .ok_or_else(|| assets_err!("Failed to find terrain asset") )?;

        let view = self.fetch_texture_view(core, terrain_id)?;

        self.render.terrain.terrain_set = self.descriptors.write_terrain_texture(view)?;

        Ok(())
    }

    pub(super) fn setup_render_data(&mut self) {
        let res = &self.resources;

        let render = &mut self.render.terrain;
        render.pipeline_layout = res.pipelines.terrain_layout;
        render.vertex_buffer = [res.vertex.buffer];
        render.index_offset = res.vertex.index_offset();
        render.vertex_offset = res.vertex.vertex_offset();

        let render = &mut self.render.actors;
        render.pipeline_layout = res.pipelines.actors_layout;
        render.vertex_buffer = [res.vertex.buffer];
        render.index_offset = res.vertex.index_offset();
        render.vertex_offset = res.vertex.vertex_offset();

        let render = &mut self.render.debug;
        render.pipeline_layout = res.pipelines.debug_layout;
        render.vertex_buffer = [res.debug_vertex.buffer];
        render.index_offset = res.debug_vertex.index_offset();
        render.vertex_offset = res.debug_vertex.vertex_offset();
    }

}
