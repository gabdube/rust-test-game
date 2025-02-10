use loomz_engine_core::{LoomzEngineCore, alloc::{VertexAlloc, StorageAlloc}, descriptors::*, pipelines::*};
use loomz_engine_core::VulkanContext;
use loomz_shared::api::TERRAIN_CHUNK_SIZE;
use loomz_shared::{CommonError, CommonErrorType, LoomzApi, TerrainType};
use loomz_shared::{backend_init_err, assets_err, chain_err};
use super::{WorldPushConstant, WorldVertex, WorldDebugVertex, data::TerrainSpriteData};


impl super::WorldModule {

    pub(super) fn setup_pipelines(&mut self, core: &mut LoomzEngineCore, api: &LoomzApi) -> Result<(), CommonError> {
        Self::setup_descriptor_set_layouts(&core.ctx, &mut self.resources.pipelines)?;
        Self::setup_pipeline_layouts(&core.ctx, &mut self.resources.pipelines)?;
        Self::setup_terrain_pipeline(core, api, &mut self.resources.pipelines.terrain)?;
        Self::setup_actors_pipeline(core, api, &mut self.resources.pipelines.actors)?;
        Self::setup_debug_pipeline(core, api, &mut self.resources.pipelines.debug)?;
        Ok(())
    }

    fn setup_descriptor_set_layouts(ctx: &VulkanContext, pipelines: &mut super::WorldPipelines) -> Result<(), CommonError> {
        // Terrain
        let bindings_global = [
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                stage_flags: vk::ShaderStageFlags::VERTEX,
            },
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
            },
        ];

        pipelines.terrain_global_layout = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, &bindings_global)
            .map_err(|err| backend_init_err!("Failed to create terrain global descriptor set layout: {}", err) )?;

        // Actors
        let bindings_global = [
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
            },
        ];

        pipelines.actor_batch_layout = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, &bindings_global)
            .map_err(|err| backend_init_err!("Failed to create actors batch descriptor set layout: {}", err) )?;

        let bindings_global = [
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                stage_flags: vk::ShaderStageFlags::VERTEX,
            },
        ];

        pipelines.actor_global_layout =  PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, &bindings_global)
            .map_err(|err| backend_init_err!("Failed to create actors global descriptor set layout: {}", err) )?;

        Ok(())
    }

    fn setup_pipeline_layouts(ctx: &VulkanContext, pipelines: &mut super::WorldPipelines) -> Result<(), CommonError> {
        // Terrain
        let layouts = [pipelines.terrain_global_layout,];

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

        pipelines.terrain.layout = ctx.device.create_pipeline_layout(&pipeline_create_info)
            .map_err(|err| backend_init_err!("Failed to build the terrain pipeline layout: {}", err) )?;
        
        // Actors
        let layouts = [pipelines.actor_global_layout, pipelines.actor_batch_layout];

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

        pipelines.actors.layout = ctx.device.create_pipeline_layout(&pipeline_create_info)
            .map_err(|err| backend_init_err!("Failed to build the actor pipeline layout: {}", err) )?;

        // Debug
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

        pipelines.debug.layout = ctx.device.create_pipeline_layout(&pipeline_create_info)
            .map_err(|err| backend_init_err!("Failed to create world debug pipeline layout: {}", err) )?;
    
        Ok(())
    }

    fn setup_terrain_pipeline(core: &mut LoomzEngineCore, api: &LoomzApi, world_pipeline: &mut super::WorldPipeline) -> Result<(), CommonError> {
        let ctx = &core.ctx;
        let pipeline = &mut world_pipeline.pipeline;

        // Shader source
        world_pipeline.id = api.assets_ref().shader_id_by_name("world_terrain")
            .ok_or_else(|| backend_init_err!("Failed to find world terrain shader") )?;

        let shader = api.assets_ref().shader(world_pipeline.id).unwrap();
        let modules = GraphicsShaderModules::new(ctx, &shader.vert, &shader.frag)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to build the world terrain pipeline shader modules") )?;

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
        pipeline.set_pipeline_layout(world_pipeline.layout);
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

    fn setup_actors_pipeline(core: &mut LoomzEngineCore, api: &LoomzApi, world_pipeline: &mut super::WorldPipeline) -> Result<(), CommonError> {
        let ctx = &core.ctx;
        let pipeline = &mut world_pipeline.pipeline;

        // Shader source
        world_pipeline.id = api.assets_ref().shader_id_by_name("world_actors")
            .ok_or_else(|| backend_init_err!("Failed to find world actors shader") )?;

        let shader = api.assets_ref().shader(world_pipeline.id).unwrap();
        let modules = GraphicsShaderModules::new(ctx, &shader.vert, &shader.frag)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to build world actors pipeline shader modules") )?;

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
        pipeline.set_pipeline_layout(world_pipeline.layout);
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

    fn setup_debug_pipeline(core: &mut LoomzEngineCore, api: &LoomzApi, world_pipeline: &mut super::WorldPipeline) -> Result<(), CommonError> {
        let ctx = &core.ctx;
        let pipeline = &mut world_pipeline.pipeline;

        // Shader source
        world_pipeline.id = api.assets_ref().shader_id_by_name("world_debug")
            .ok_or_else(|| backend_init_err!("Failed to find world debug shader") )?;

        let shader = api.assets_ref().shader(world_pipeline.id).unwrap();
        let modules = GraphicsShaderModules::new(ctx, &shader.vert, &shader.frag)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to build world debug pipeline shader modules") )?;

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
        pipeline.set_pipeline_layout(world_pipeline.layout);
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
        let pipelines = &mut self.resources.pipelines;
        let allocations = [
            DescriptorsAllocation {
                layout: pipelines.terrain_global_layout,
                binding_types: &[vk::DescriptorType::STORAGE_BUFFER, vk::DescriptorType::COMBINED_IMAGE_SAMPLER],
                count: 1,
            },
            DescriptorsAllocation {
                layout: pipelines.actor_global_layout,
                binding_types: &[vk::DescriptorType::STORAGE_BUFFER],
                count: 1,
            },
            DescriptorsAllocation {
                layout: pipelines.actor_batch_layout,
                binding_types: &[vk::DescriptorType::COMBINED_IMAGE_SAMPLER],
                count: 10,
            },
        ];

        self.resources.default_sampler = core.resources.linear_sampler;
        self.resources.descriptors = DescriptorsAllocator::new(core, &allocations)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to prellocate descriptor sets") )?;

        Ok(())
    }

    pub(super) fn setup_buffers(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        self.setup_vertex_buffer(core)?;
        self.setup_terrain_sprites_buffer(core)?;
        self.setup_actor_sprites_buffers(core)?;
        self.setup_debug_vertex_buffer(core)?;
        Ok(())
    }

    fn setup_vertex_buffer(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
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

    fn setup_terrain_sprites_buffer(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let sprites_capacity = TERRAIN_CHUNK_SIZE * TERRAIN_CHUNK_SIZE * 8;
        self.data.terrain_sprites = StorageAlloc::new(core, sprites_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create terrain sprites storage alloc: {err}") )?;

        Ok(())
    }

    fn setup_actor_sprites_buffers(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        use super::ACTOR_GLOBAL_LAYOUT_ID;

        let sprites_capacity = 100;
        self.data.actors_sprites = StorageAlloc::new(core, sprites_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create sprites storage alloc: {err}") )?;

        self.render.actors.sprites_set = self.resources.descriptors.get_set::<ACTOR_GLOBAL_LAYOUT_ID>()
            .ok_or_else(|| backend_init_err!("Failed to fetch actors global descriptor set") )?;

        core.descriptors.write_buffer(
            self.render.actors.sprites_set,
            self.data.actors_sprites.handle(),
            0,
            self.data.actors_sprites.bytes_range(),
            super::ACTOR_SPRITE_BUFFER_BINDING_INDEX,
            vk::DescriptorType::STORAGE_BUFFER
        );

        Ok(())
    }

    fn setup_debug_vertex_buffer(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let vertex_capacity = 1000;
        let index_capacity = 1500;
        
        self.resources.debug_vertex = VertexAlloc::new(core, index_capacity, vertex_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create debug vertex alloc: {err}") )?;

        Ok(())
    }

    pub(super) fn setup_terrain_tilemap(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        self.load_terrain_tilemap()?;
        self.load_terrain_texture(core)?;
        self.setup_terrain_descriptor_set(core)?;
        Ok(())
    }

    fn load_terrain_tilemap(&mut self) -> Result<(), CommonError> {
        let tilemap = &mut self.data.terrain_tilemap;

        let terrain_json_source = self.resources.assets.json_by_name("terrain_sprites")
            .ok_or_else(|| assets_err!("Failed to find terrain json asset") )?;

        let terrain_json: serde_json::Value = serde_json::from_str(&terrain_json_source)
            .map_err(|err| assets_err!("Failed to parse json: {err:?}") )?;

        let obj = terrain_json.as_object()
            .ok_or_else(|| assets_err!("Failed to parse json. terrain_json is not an object") )?;

        for &key in TerrainType::names() {
            let tile_info = obj.get(key)
                .and_then(|v| v.as_object() )
                .ok_or_else(|| assets_err!("Failed to find tilemap key \"{key}\"") )?;

            let uv_x = tile_info.get("uv_x")
                .and_then(|v| v.as_f64() )
                .unwrap_or(0.0) as f32;

            let uv_y = tile_info.get("uv_y")
                .and_then(|v| v.as_f64() )
                .unwrap_or(0.0) as f32;

            tilemap.push(TerrainSpriteData {
                uv_offset: [uv_x, uv_y]
            });
        }

        Ok(())
    }

    fn load_terrain_texture(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let texture_id = self.resources.assets.texture_id_by_name("terrain")
            .ok_or_else(|| assets_err!("Terrain texture asset was not found") )?;

        let texture_asset = self.resources.assets.texture(texture_id)
            .ok_or_else(|| unreachable!("Presence validated by call to texture_id_by_name") )?;

        let texture = core.create_texture_from_asset(&texture_asset)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendGeneric, "Failed to create image from asset") )?;

        self.resources.terrain_texture = Some(texture);

        Ok(())
    }

    fn setup_terrain_descriptor_set(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        use super::TERRAIN_GLOBAL_LAYOUT_ID;

        let image_view = self.resources.terrain_texture
            .map(|t| t.view )
            .ok_or_else(|| unreachable!("Texture presence validated in load_terrain_texture") )?;

        let descriptor_set = self.resources.descriptors.get_set::<TERRAIN_GLOBAL_LAYOUT_ID>()
            .ok_or_else(|| backend_init_err!("Failed to fetch terrain global descriptor set") )?;

        core.descriptors.write_buffer(
            descriptor_set,
            self.data.terrain_sprites.handle(),
            0,
            self.data.terrain_sprites.bytes_range(),
            super::TERRAIN_SPRITE_BUFFER_BINDING_INDEX,
            vk::DescriptorType::STORAGE_BUFFER
        );

        core.descriptors.write_image(
            descriptor_set,
            image_view,
            self.resources.default_sampler,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            super::TERRAIN_SAMPLER_BINDING_INDEX,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        );

        self.render.terrain.terrain_set = descriptor_set;

        Ok(())
    }

    pub(super) fn setup_render_data(&mut self) {
        let res = &self.resources;

        let render = &mut self.render.terrain;
        render.pipeline_layout = res.pipelines.terrain.layout;
        render.vertex_buffer = [res.vertex.buffer];
        render.index_offset = res.vertex.index_offset();
        render.vertex_offset = res.vertex.vertex_offset();

        let render = &mut self.render.actors;
        render.pipeline_layout = res.pipelines.actors.layout;
        render.vertex_buffer = [res.vertex.buffer];
        render.index_offset = res.vertex.index_offset();
        render.vertex_offset = res.vertex.vertex_offset();

        let render = &mut self.render.debug;
        render.pipeline_layout = res.pipelines.debug.layout;
        render.vertex_buffer = [res.debug_vertex.buffer];
        render.index_offset = res.debug_vertex.index_offset();
        render.vertex_offset = res.debug_vertex.vertex_offset();
    }

    pub(super) fn setup_default_data(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        use loomz_shared::api::WorldAnimation;
        use super::{ACTOR_BATCH_LAYOUT_ID, ACTOR_SAMPLER_BINDING_INDEX};

        // Loads and creates a "default" values for the world actors
        // We use "pawn_blue" as a default texture because it will be loaded in during gameplay anyway
        let texture_id = self.resources.assets.texture_id_by_name("pawn_blue")
            .ok_or_else(|| assets_err!("Failed to load pawn_blue asset") )?;

        let texture_asset = self.resources.assets.texture(texture_id)
            .unwrap_or_else(|| unreachable!("Texture presence already validated") );

        let texture = core.create_texture_from_asset(&texture_asset)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendGeneric, "Failed to create image from asset") )?;

        let descriptor_set = self.resources.descriptors.get_set::<ACTOR_BATCH_LAYOUT_ID>()
            .ok_or_else(|| backend_init_err!("No more descriptor set in actor batch layout pool") )?;

        let sampler = self.resources.default_sampler;

        core.descriptors.write_image(
            descriptor_set,
            texture.view,
            sampler,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ACTOR_SAMPLER_BINDING_INDEX,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        );

        self.data.default_actor = Some(Box::new(super::data::WorldActorData {
            descriptor_set,
            animation: WorldAnimation {
                texture_id,
                padding: 0.0,
                x: 0.0,
                y: 0.0,
                sprite_width: 0.0,
                sprite_height: 0.0,
                last_frame: u8::MAX
            },
            position: Default::default(),
            current_frame: 0,
            flipped: false,
        }));

        self.resources.textures.insert(texture_id, super::WorldTexture {
            texture,
            descriptor_set,
        });

        Ok(())
    }

    pub(super) fn reload_shaders(
        &mut self,
        api: &LoomzApi,
        core: &mut LoomzEngineCore,
        shader_id: loomz_shared::ShaderId
    ) -> Result<(), CommonError> {
        let terrain_id = self.resources.pipelines.terrain.id;
        let actors_id = self.resources.pipelines.actors.id;
        let debug_id = self.resources.pipelines.debug.id;

        let mut new_pipeline = 
            if shader_id == terrain_id     { self.resources.pipelines.terrain.pipeline.clone() }
            else if shader_id == actors_id { self.resources.pipelines.actors.pipeline.clone() }
            else if shader_id == debug_id  { self.resources.pipelines.debug.pipeline.clone() }
            else { return Ok(()); };
        
        let shader = api.assets_ref().shader(shader_id)
            .ok_or_else(|| assets_err!("Failed to find shader by ID") )?;

        let modules = GraphicsShaderModules::new(&core.ctx, &shader.vert, &shader.frag)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to reload shader module") )?;

        new_pipeline.set_shader_modules(modules);

        let pipeline_info = [new_pipeline.create_info()];
        let mut pipeline_handle = [vk::Pipeline::null()];
        core.ctx.device.create_graphics_pipelines(vk::PipelineCache::default(), &pipeline_info, &mut pipeline_handle)
            .map_err(|err| backend_init_err!("Failed to recompile world pipelines: {:?}", err) )?;

        new_pipeline.set_handle(pipeline_handle[0]);

        if shader_id == terrain_id { 
            ::std::mem::swap(&mut self.resources.pipelines.terrain.pipeline, &mut new_pipeline);
            self.render.terrain.pipeline_handle = pipeline_handle[0];
        }
        else if shader_id == actors_id { 
            ::std::mem::swap(&mut self.resources.pipelines.actors.pipeline, &mut new_pipeline);
            self.render.actors.pipeline_handle = pipeline_handle[0];
        }
        else if shader_id == debug_id  { 
            ::std::mem::swap(&mut self.resources.pipelines.debug.pipeline, &mut new_pipeline);
            self.render.debug.pipeline_handle = pipeline_handle[0];
        }

        new_pipeline.destroy(&core.ctx);

        Ok(())
    }


}
