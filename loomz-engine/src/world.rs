use std::{slice, sync::Arc, time::Instant, u32, usize};
use fnv::FnvHashMap;
use loomz_engine_core::{LoomzEngineCore, VulkanContext, Texture, alloc::{VertexAlloc, StorageAlloc}, descriptors::*, pipelines::*};
use loomz_shared::api::{LoomzApi, WorldAnimationId, WorldAnimation, WorldActorId, WorldActor};
use loomz_shared::{assets::{LoomzAssetsBundle, TextureId}, _2d::Position, CommonError, CommonErrorType};
use loomz_shared::{backend_init_err, assets_err, backend_err, chain_err};
use super::pipeline_compiler::PipelineCompiler;

const WORLD_VERT_SRC: &[u8] = include_bytes!("../../assets/shaders/world.vert.spv");
const WORLD_FRAG_SRC: &[u8] = include_bytes!("../../assets/shaders/world.frag.spv");

const PUSH_STAGE_FLAGS: vk::ShaderStageFlags = vk::ShaderStageFlags::VERTEX;
const PUSH_SIZE: u32 = size_of::<WorldPushConstant>() as u32;

const GLOBAL_LAYOUT_INDEX: u32 = 0;
const SPRITES_BUFFER_DATA_BINDING: usize = 0;

const BATCH_LAYOUT_INDEX: u32 = 1;
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
    pub set: vk::DescriptorSet,
    pub instances_count: u32,
    pub instances_offset: u32,
}

struct WorldModuleDescriptors {
    alloc: DescriptorsAllocator,
    updates: DescriptorWriteBuffer,
    texture_params: DescriptorWriteImageParams,
}

/// Graphics resources that are not accessed often
struct WorldResources {
    assets: Arc<LoomzAssetsBundle>,
    descriptors: WorldModuleDescriptors,
    pipeline: GraphicsPipeline,
    vertex: VertexAlloc<WorldVertex>,
    textures: FnvHashMap<TextureId, Texture>,
}

#[derive(Copy, Clone)]
struct WorldInstance {
    image_view: vk::ImageView,
    texture_id: Option<TextureId>,
    position: Position<f32>,
    uv_offset: [f32; 2],
    uv_size: [f32; 2],
    flipped: bool,
}
 
struct WorldInstanceAnimation {
    tick: Instant,
    instance_index: usize,
    animation: WorldAnimation,
    current_frame: u32,
}

/// Data used on rendering
#[derive(Copy, Clone)]
struct WorldRender {
    pipeline_handle: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    vertex_buffer: vk::Buffer,
    sprites: vk::DescriptorSet,
    index_offset: vk::DeviceSize,
    vertex_offset: [vk::DeviceSize; 1],
    push_constants: [WorldPushConstant; 1],
}

/// World data to be rendered on screen
struct WorldData {
    sprites: StorageAlloc<SpriteData>,
    animations: Vec<WorldAnimation>,
    instance_animations: Vec<WorldInstanceAnimation>,
    instances: Vec<WorldInstance>,
}

pub(crate) struct WorldModule {
    resources: Box<WorldResources>,
    data: Box<WorldData>,
    render: Box<WorldRender>,
    batches: Vec<WorldBatch>,
    update_batches: bool,
}

impl WorldModule {

    pub fn init(core: &mut LoomzEngineCore, api: &LoomzApi) -> Result<Self, CommonError> {
        let descriptors = WorldModuleDescriptors {
            alloc: DescriptorsAllocator::default(),
            updates: DescriptorWriteBuffer::default(),
            texture_params: DescriptorWriteImageParams::default(),
        };

        let resources = WorldResources {
            assets: api.assets(),
            descriptors,
            pipeline: GraphicsPipeline::new(),
            vertex: VertexAlloc::default(),
            textures: FnvHashMap::default(),
        };

        let data = WorldData {
            sprites: StorageAlloc::default(),
            animations: Vec::with_capacity(16),
            instance_animations: Vec::with_capacity(16),
            instances: Vec::with_capacity(16),
        };

        let render = WorldRender {
            pipeline_handle: vk::Pipeline::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            vertex_buffer: vk::Buffer::null(),
            sprites: vk::DescriptorSet::null(),
            index_offset: 0,
            vertex_offset: [0],
            push_constants: [WorldPushConstant::default(); 1],
        };

        let mut world = WorldModule {
            data: Box::new(data),
            resources: Box::new(resources),
            render: Box::new(render),
            batches: Vec::with_capacity(16),
            update_batches: false,
        };

        world.setup_pipeline(core)?;
        world.setup_descriptors(core)?;
        world.setup_vertex_buffers(core)?;
        world.setup_sprites_buffers(core)?;
        world.setup_render_data(core);

        Ok(world)
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.resources.descriptors.alloc.destroy(core);
        self.resources.pipeline.destroy(&core.ctx);
        self.resources.vertex.free(core);
        self.data.sprites.free(core);

        for texture in self.resources.textures.values() {
            core.destroy_texture(*texture);
        }
    }

    pub fn set_output(&mut self, core: &LoomzEngineCore) {
        let extent = core.info.swapchain_extent;
        self.render.push_constants[0] = WorldPushConstant {
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
        compiler.add_pipeline_info("world", &mut self.resources.pipeline);
    }

    pub fn set_pipeline_handle(&mut self, compiler: &PipelineCompiler) {
        let handle = compiler.get_pipeline("world");
        self.resources.pipeline.set_handle(handle);
        self.render.pipeline_handle = handle;
    }

    //
    // Rendering
    //

    pub fn render(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        #[inline(always)]
        fn push_values(constants: &[WorldPushConstant; 1]) -> &[u8] {
            unsafe { constants.align_to::<u8>().1 }
        }
        
        const GRAPHICS: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;
        let device = &ctx.device;
        let render = *self.render;

        device.cmd_bind_pipeline(cmd, GRAPHICS, render.pipeline_handle);
        device.cmd_bind_index_buffer(cmd, render.vertex_buffer, render.index_offset, vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, slice::from_ref(&render.vertex_buffer), &render.vertex_offset);
        device.cmd_push_constants(cmd, render.pipeline_layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, push_values(&render.push_constants));
        device.cmd_bind_descriptor_sets(cmd, GRAPHICS, render.pipeline_layout, GLOBAL_LAYOUT_INDEX, slice::from_ref(&render.sprites), &[]);

        for batch in self.batches.iter() {
            device.cmd_bind_descriptor_sets(cmd, GRAPHICS, render.pipeline_layout, BATCH_LAYOUT_INDEX, slice::from_ref(&batch.set), &[]);
            device.cmd_draw_indexed(cmd, 6, batch.instances_count, 0, 0, batch.instances_offset);
        }
    }

    //
    // Updates
    //

    fn update_world_actor_animation(&mut self, core: &mut LoomzEngineCore, actor_index: usize, animation: WorldAnimation) -> Result<(), CommonError> {
        let instance_animation = WorldInstanceAnimation {
            tick: Instant::now(),
            instance_index: actor_index,
            animation,
            current_frame: 0,
        };

        // Insert or create new animation
        let instance_animation_index = self.data.instance_animations
            .iter().position(|anim| anim.instance_index == actor_index )
            .unwrap_or(usize::MAX);

        if instance_animation_index == usize::MAX {
            self.data.instance_animations.push(instance_animation);
        } else {
            self.data.instance_animations[instance_animation_index] = instance_animation;
        }

        // Update instance image
        let new_texture_id = animation.texture_id;
        let old_texture_id = self.data.instances[actor_index].texture_id;
        if Some(new_texture_id) != old_texture_id {
            let new_image_view = self.fetch_texture_view(core, animation.texture_id)?;
            self.data.instances[actor_index].image_view = new_image_view;
            self.data.instances[actor_index].texture_id = Some(new_texture_id);
            self.update_batches = true;
        }

        // Initialize the instance UV
        let instance = &mut self.data.instances[actor_index];
        instance.uv_offset = [animation.x, animation.y];
        instance.uv_size = [animation.sprite_width, animation.sprite_height];

        Ok(())

    }
    
    fn update_world_actor(&mut self, core: &mut LoomzEngineCore, actor_index: usize, param: WorldActor) -> Result<(), CommonError> {
        match param {
            WorldActor::Position(position) => {
                self.data.instances[actor_index].position = position;
            },
            WorldActor::Flip(flipped) => {
                self.data.instances[actor_index].flipped = flipped;
            },
            WorldActor::Animation(animation_id) => {
                let animation_index = animation_id.bound_value();
                let animation = animation_index.and_then(|index| self.data.animations.get(index).copied() );
                match animation {
                    Some(animation) => { self.update_world_actor_animation(core, actor_index, animation)?; },
                    None => { return Err(backend_err!("Failed to find an animation with ID {animation_index:?}")); }
                }
            }
        }

        self.data.sprites.write_data(actor_index, SpriteData::from(self.data.instances[actor_index]));

        Ok(())
    }

    fn create_world_actor(&mut self, id: WorldActorId) -> Result<usize, CommonError> {
        let instances = &mut self.data.instances;
        let new_id = instances.len();
        
        instances.push(WorldInstance {
            image_view: vk::ImageView::null(),
            position: Position::default(),
            uv_offset: [0.0, 0.0],
            uv_size: [0.0, 0.0],
            texture_id: None,
            flipped: false,
        });

        id.bind(new_id as u32);

        Ok(new_id)
    }

    fn create_world_animation(&mut self, id: WorldAnimationId, animation: WorldAnimation) {
        let animations = &mut self.data.animations;
        let new_id = animations.len() as u32;
        animations.push(animation);
        id.bind(new_id);
    }

    fn api_update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        if let Some(animations) = api.world().read_animations() {
            for (id, animation) in animations {
                match id.bound_value() {
                    Some(index) => panic!("Animations cannot be updated. Tried to update animation ID {index}"),
                    None => self.create_world_animation(id, animation)
                }
            }
        }

        if let Some(actors) = api.world().read_actors() {
            for (id, actor) in actors {
                match id.bound_value() {
                    Some(index) => self.update_world_actor(core, index, actor)?,
                    None => {
                        let index = self.create_world_actor(id)?;
                        self.update_world_actor(core, index, actor)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn animation_update(&mut self) {
        let now = Instant::now();
        for instance_animation in self.data.instance_animations.iter_mut() {
            let elapsed = now.duration_since(instance_animation.tick).as_secs_f32();
            if elapsed < instance_animation.animation.interval {
                continue;
            }

            let i = instance_animation.current_frame as f32;
            let animation = instance_animation.animation;
            let mut instance = self.data.instances[instance_animation.instance_index];

            let uv_x = animation.x + (animation.sprite_width * i) + (animation.padding * i);
            instance.uv_offset[0] = uv_x;

            self.data.instances[instance_animation.instance_index] = instance;
            self.data.sprites.write_data(instance_animation.instance_index, SpriteData::from(instance));

            instance_animation.tick = now;
            
            if instance_animation.current_frame == instance_animation.animation.last_frame {
                instance_animation.current_frame = 0;
            } else {
                instance_animation.current_frame += 1;
            }
        }
    }

    fn batches_update(&mut self, core: &mut LoomzEngineCore) {
        self.resources.descriptors.alloc.clear_sets(BATCH_LAYOUT_INDEX);
        self.batches.clear();

        WorldBatcher::build(self);

        self.resources.descriptors.updates.submit(core);
        self.update_batches = false;
    }
    
    pub fn update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        self.api_update(api, core)?;
        self.animation_update();

        if self.update_batches {
            self.batches_update(core);
        }

        Ok(())
    }

    //
    // Data
    //
   
    fn fetch_texture_view(&mut self, core: &mut LoomzEngineCore, id: TextureId) -> Result<vk::ImageView, CommonError> {
        if let Some(texture) = self.resources.textures.get(&id) {
            return Ok(texture.view);
        }

        let texture_asset = self.resources.assets.texture(id)
            .ok_or_else(|| assets_err!("Unkown asset with ID {id:?}") )?;

        let texture = core.create_texture_from_asset(&texture_asset)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendGeneric, "Failed to create image from asset") )?;

        self.resources.textures.insert(id, texture);

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

        let layouts = [layout_global, layout_batch];

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

        let pipeline = &mut self.resources.pipeline;
        pipeline.set_shader_modules(modules);
        pipeline.set_vertex_format::<WorldVertex>(&vertex_fields);
        pipeline.set_pipeline_layout(pipeline_layout, false);
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
                layout: self.resources.pipeline.descriptor_set_layout(GLOBAL_LAYOUT_INDEX),
                bindings: Self::pipeline_descriptor_bindings_global(),
                count: 1,
            },
            DescriptorsAllocation {
                layout: self.resources.pipeline.descriptor_set_layout(BATCH_LAYOUT_INDEX),
                bindings: Self::pipeline_descriptor_bindings_batch(),
                count: 10,
            },
        ];

        self.resources.descriptors.alloc = DescriptorsAllocator::new(core, &allocations)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to prellocate descriptor sets") )?;

        self.resources.descriptors.texture_params = DescriptorWriteImageParams {
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

    fn setup_sprites_buffers(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let sprites_capacity = 100;
        self.data.sprites = StorageAlloc::new(core, sprites_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create storage alloc: {err}") )?;

        Ok(())
    }

    fn setup_render_data(&mut self, core: &mut LoomzEngineCore) {
        let render = &mut self.render;
        let descriptors = &mut self.resources.descriptors;

        render.pipeline_layout = self.resources.pipeline.pipeline_layout();
        render.vertex_buffer = self.resources.vertex.buffer;
        render.index_offset = self.resources.vertex.index_offset();
        render.vertex_offset = self.resources.vertex.vertex_offset();

        render.sprites = descriptors.alloc.next_set(GLOBAL_LAYOUT_INDEX);
        descriptors.updates.write_storage_buffer(render.sprites, &self.data.sprites, SPRITES_BUFFER_DATA_BINDING as u32);
        descriptors.updates.submit(core);
    }

}

struct WorldBatcher<'a> {
    current_view: vk::ImageView,
    batches: &'a mut Vec<WorldBatch>,
    instances: &'a mut [WorldInstance],
    descriptors: &'a mut WorldModuleDescriptors,
    instance_index: usize,
    batch_index: usize,
}

impl<'a> WorldBatcher<'a> {

    fn build(world: &'a mut WorldModule) {
        let mut batcher = WorldBatcher {
            current_view: vk::ImageView::null(),
            batches: &mut world.batches,
            instances: &mut world.data.instances,
            descriptors: &mut world.resources.descriptors,
            instance_index: 0,
            batch_index: 0,
        };

        batcher.first_batch();
        batcher.remaining_batches();
    }

    fn first_batch(&mut self) {
        let mut found = false;
        let max_instance = self.instances.len();

        while !found && self.instance_index != max_instance {
            let instance = self.instances[self.instance_index];
            if instance.image_view.is_null() {
                // Sprite is not renderable
                self.instance_index += 1;
                continue;
            }

            let set = self.descriptors.alloc.next_set(BATCH_LAYOUT_INDEX);
            self.descriptors.updates.write_simple_image(set, instance.image_view, &self.descriptors.texture_params);
    
            self.batches.push(WorldBatch {
                set,
                instances_count: 1,
                instances_offset: 0,
            });
    
            self.current_view = instance.image_view;
            self.instance_index += 1;
            found = true;
        }
    }

    fn remaining_batches(&mut self) {
        let max_instance = self.instances.len();
        while self.instance_index != max_instance {
            let instance = self.instances[self.instance_index];
            if instance.image_view.is_null() {
                // Sprite is not renderable
                self.instance_index += 1;
                continue;
            }

            let image_view = instance.image_view;
            if self.current_view == image_view {
                self.batches[self.batch_index].instances_count += 1;
            } else {
                self.next_batch(image_view)
            }

            self.instance_index += 1;
        }
    }

    fn next_batch(&mut self, image_view: vk::ImageView) {
        let set = self.descriptors.alloc.next_set(BATCH_LAYOUT_INDEX);
        self.descriptors.updates.write_simple_image(set, image_view, &self.descriptors.texture_params);

        self.batches.push(WorldBatch {
            set,
            instances_count: 1,
            instances_offset: self.instance_index as u32,
        });

        self.current_view = image_view;
        self.batch_index += 1;
    }

}

impl From<WorldInstance> for SpriteData {
    fn from(instance: WorldInstance) -> Self {
        let size = instance.uv_size;

        let mut offset = instance.position.splat();
        offset[0] -= size[0] * 0.5;
        offset[1] -= size[1] * 0.5;

        let mut uv_offset = instance.uv_offset;
        let mut uv_size = instance.uv_size;
        if instance.flipped {
            uv_offset[0] += size[0];
            uv_size[0] *= -1.0;
        }

        SpriteData {
            offset,
            size,
            uv_offset,
            uv_size,
        }
    }
}
