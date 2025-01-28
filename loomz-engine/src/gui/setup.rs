use loomz_engine_core::{LoomzEngineCore, alloc::VertexAlloc, descriptors::*, pipelines::*};
use loomz_shared::{CommonError, CommonErrorType, LoomzApi};
use loomz_shared::{backend_init_err, assets_err, chain_err};
use super::{GuiPushConstant, GuiVertex};

impl super::GuiModule {
    pub(super) fn setup_pipelines(&mut self, core: &mut LoomzEngineCore, api: &LoomzApi) -> Result<(), CommonError> {
        let ctx = &core.ctx;

        // Descriptor set layouts
        let bindings_batch = [
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
            },
        ];
        let layout_batch = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, &bindings_batch)
            .map_err(|err| backend_init_err!("Failed to create batch descriptor set layout: {}", err) )?;

        // Pipeline layout
        let layouts = [layout_batch];
        let constant_range = vk::PushConstantRange {
            offset: 0,
            size: ::std::mem::size_of::<GuiPushConstant>() as u32,
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
        self.resources.image_pipeline_id = api.assets_ref().shader_id_by_name("gui_component")
            .ok_or_else(|| backend_init_err!("Failed to find gui_component shader") )?;

        let shader = api.assets_ref().shader(self.resources.image_pipeline_id).unwrap();
        let component_modules = GraphicsShaderModules::new(ctx, &shader.vert, &shader.frag)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to compute gui component pipeline shader modules") )?;

        self.resources.text_pipeline_id = api.assets_ref().shader_id_by_name("gui_text")
            .ok_or_else(|| backend_init_err!("Failed to find gui_text shader") )?;

        let text_shader = api.assets_ref().shader(self.resources.text_pipeline_id).unwrap();
        let text_modules = GraphicsShaderModules::new(ctx, &text_shader.vert, &text_shader.frag)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to compute gui text pipeline shader modules") )?;

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

        let res = &mut self.resources;
        res.batch_layout = layout_batch;
        res.pipeline_layout = pipeline_layout;
        res.image_pipeline.set_shader_modules(component_modules);
        res.text_pipeline.set_shader_modules(text_modules);
        
        for pipeline in [&mut res.image_pipeline, &mut res.text_pipeline] {
            pipeline.set_vertex_format::<GuiVertex>(&vertex_fields);
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
        }

        Ok(())
    }

    pub(super) fn setup_vertex_buffers(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let vertex_capacity = 500;
        let index_capacity = 1000;
        self.data.vertex_alloc = VertexAlloc::new(core, index_capacity, vertex_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create vertex alloc: {err}") )?;

        Ok(())
    }

    pub(super) fn setup_descriptors(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        use loomz_engine_core::descriptors::DescriptorsAllocation;
        
        let allocations = [
            DescriptorsAllocation {
                layout: self.resources.batch_layout,
                binding_types: &[vk::DescriptorType::COMBINED_IMAGE_SAMPLER],
                count: 8,
            },
        ];

        self.descriptors.default_sampler = core.resources.linear_sampler;
        self.descriptors.allocator = DescriptorsAllocator::new(core, &allocations)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to prellocate descriptor sets") )?;

        Ok(())
    }

    pub(super) fn setup_render_data(&mut self) {
        let render = &mut self.render;

        render.pipeline_layout = self.resources.pipeline_layout;
        render.vertex_buffer = self.data.vertex_alloc.buffer;
        render.index_offset = self.data.vertex_alloc.index_offset();
        render.vertex_offset = self.data.vertex_alloc.vertex_offset();
    }
    
    pub(super) fn reload_shaders(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore, shader_id: loomz_shared::ShaderId) -> Result<(), CommonError> {
        let text_id = self.resources.text_pipeline_id;
        let image_id = self.resources.image_pipeline_id;

        let mut new_pipeline = 
            if shader_id == text_id       { self.resources.text_pipeline.clone() }
            else if shader_id == image_id { self.resources.image_pipeline.clone() }
            else { return Ok(()) };

        let shader = api.assets_ref().shader(shader_id)
            .ok_or_else(|| assets_err!("Failed to find shader by ID") )?;

        let modules = GraphicsShaderModules::new(&core.ctx, &shader.vert, &shader.frag)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to reload shader module") )?;

        new_pipeline.set_shader_modules(modules);

        let pipeline_info = [new_pipeline.create_info()];
        let mut pipeline_handle = [vk::Pipeline::null()];
        core.ctx.device.create_graphics_pipelines(vk::PipelineCache::default(), &pipeline_info, &mut pipeline_handle)
            .map_err(|err| backend_init_err!("Failed to recompile gui pipelines: {:?}", err) )?;

        new_pipeline.set_handle(pipeline_handle[0]);

        if shader_id == text_id { 
            ::std::mem::swap(&mut self.resources.text_pipeline, &mut new_pipeline);
        }
        else if shader_id == image_id {
            ::std::mem::swap(&mut self.resources.image_pipeline, &mut new_pipeline);
        }

        new_pipeline.destroy(&core.ctx);

        Ok(())
    }
    
}
