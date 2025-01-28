mod setup;
mod batch;
mod terrain;
mod debug;

use fnv::FnvHashMap;
use bitflags::bitflags;
use std::{slice, sync::Arc, time::Instant, u32, usize};
use loomz_shared::api::{LoomzApi, WorldAnimationId, WorldAnimation, WorldActorId, WorldActorUpdate,  WorldUpdate, WorldDebugFlags};
use loomz_shared::assets::{LoomzAssetsBundle, TextureId, ShaderId, AssetId};
use loomz_shared::{CommonError, CommonErrorType, SizeF32, PositionF32, RgbaU8, size};
use loomz_shared::{assets_err, backend_err, chain_err};
use loomz_engine_core::{LoomzEngineCore, VulkanContext, Texture, alloc::{VertexAlloc, StorageAlloc}, descriptors::*, pipelines::*};
use super::pipeline_compiler::PipelineCompiler;

const PUSH_STAGE_FLAGS: vk::ShaderStageFlags = vk::ShaderStageFlags::VERTEX;
const PUSH_SIZE: u32 = size_of::<WorldPushConstant>() as u32;

const LAYOUT_COUNT: usize = 3;

// Index of the descriptor set layout in the allocator
const TERRAIN_GLOBAL_LAYOUT_ID: u32 = 0;
const ACTOR_GLOBAL_LAYOUT_ID: u32 = 1;
const ACTOR_BATCH_LAYOUT_ID: u32 = 2;

// Index of the descriptor set layout in their respective pipeline
const TERRAIN_GLOBAL_LAYOUT_INDEX: u32 = 0;
const ACTOR_GLOBAL_LAYOUT_INDEX: u32 = 0;
const ACTOR_BATCH_LAYOUT_INDEX: u32 = 1;

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
pub struct WorldDebugVertex {
    pub pos: [f32; 2],
    pub color: RgbaU8,
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

#[derive(Copy, Clone)]
pub struct WorldGridParams {
    screen_size: SizeF32,
    cell_size: f32,
}

struct WorldPipeline {
    pipeline: GraphicsPipeline,
    layout: vk::PipelineLayout,
    id: ShaderId,
}

#[derive(Default)]
struct WorldPipelines {
    terrain: WorldPipeline,
    actors: WorldPipeline,
    debug: WorldPipeline,
}

/// Graphics resources that are not accessed often (not every frame)
struct WorldResources {
    assets: Arc<LoomzAssetsBundle>,
    pipelines: WorldPipelines,
    vertex: VertexAlloc<WorldVertex>,
    debug_vertex: VertexAlloc<WorldDebugVertex>,
    textures: FnvHashMap<TextureId, Texture>,
    terrain_global_layout: vk::DescriptorSetLayout,
    actor_global_layout: vk::DescriptorSetLayout,
    actor_batch_layout: vk::DescriptorSetLayout,
    grid_params: WorldGridParams,
}

#[derive(Default)]
struct WorldDescriptors {
    default_sampler: vk::Sampler,
    allocator: DescriptorsAllocator<LAYOUT_COUNT>
}

#[derive(Copy, Clone)]
struct WorldInstance {
    image_view: vk::ImageView,
    texture_id: Option<TextureId>,
    position: PositionF32,
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

/// World data to be rendered on screen
struct WorldData {
    sprites: StorageAlloc<SpriteData>,
    animations: Vec<WorldAnimation>,
    instance_animations: Vec<WorldInstanceAnimation>,
    instances: Vec<WorldInstance>,
}

/// Data used when rendering the world debug info
#[derive(Copy, Clone)]
struct WorldDebugRender {
    pipeline_handle: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    vertex_buffer: [vk::Buffer; 1],
    vertex_offset: [vk::DeviceSize; 1],
    index_offset: vk::DeviceSize,
    index_count: u32,
}

/// Data used when rendering the world terrain
#[derive(Copy, Clone)]
struct WorldTerrainRender {
    pipeline_handle: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    vertex_buffer: [vk::Buffer; 1],
    vertex_offset: [vk::DeviceSize; 1],
    index_offset: vk::DeviceSize,
    terrain_set: vk::DescriptorSet,
}

/// Data used when rendering the world actors
#[derive(Copy, Clone)]
struct WorldActorRender {
    pipeline_handle: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    vertex_buffer: [vk::Buffer; 1],
    vertex_offset: [vk::DeviceSize; 1],
    index_offset: vk::DeviceSize,
    sprites_set: vk::DescriptorSet,
}

struct WorldRender {
    terrain: WorldTerrainRender,
    actors: WorldActorRender,
    debug: WorldDebugRender,
    push_constants: [WorldPushConstant; 1],
}

bitflags! {
    #[derive(Copy, Clone, Default)]
    pub struct WorldFlags: u8 {
        const SHOW_WORLD      = 0b0001;
        const UPDATE_BATCHES  = 0b0010;
    }
}

pub(crate) struct WorldModule {
    resources: Box<WorldResources>,
    descriptors: Box<WorldDescriptors>,
    data: Box<WorldData>,
    render: Box<WorldRender>,
    batches: Vec<WorldBatch>,
    debug: WorldDebugFlags,
    flags: WorldFlags,
}

impl WorldModule {

    pub fn init(core: &mut LoomzEngineCore, api: &LoomzApi) -> Result<Self, CommonError> {
        let resources = WorldResources {
            assets: api.assets(),
            pipelines: WorldPipelines::default(),
            vertex: VertexAlloc::default(),
            debug_vertex: VertexAlloc::default(),
            textures: FnvHashMap::default(),
            terrain_global_layout: vk::DescriptorSetLayout::null(),
            actor_global_layout: vk::DescriptorSetLayout::null(),
            actor_batch_layout: vk::DescriptorSetLayout::null(),
            grid_params: WorldGridParams {
                screen_size: size(0.0, 0.0),
                cell_size: 64.0,
            }
        };

        let data = WorldData {
            sprites: StorageAlloc::default(),
            animations: Vec::with_capacity(16),
            instance_animations: Vec::with_capacity(16),
            instances: Vec::with_capacity(16),
        };

        let mut world = WorldModule {
            data: Box::new(data),
            resources: Box::new(resources),
            render: Box::default(),
            batches: Vec::with_capacity(16),
            descriptors: Box::default(),
            debug: WorldDebugFlags::empty(),
            flags: WorldFlags::SHOW_WORLD,
        };

        world.setup_pipelines(core, api)?;
        world.setup_descriptors(core)?;
        world.setup_buffers(core)?;
        world.setup_terrain_sampler(core)?;
        world.setup_render_data();

        world.setup_test_world(core)?;

        Ok(world)
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.descriptors.allocator.destroy(core);
        self.data.sprites.free(core);

        for texture in self.resources.textures.values() {
            core.destroy_texture(*texture);
        }

        self.resources.pipelines.terrain.destroy(core);
        self.resources.pipelines.actors.destroy(core);
        self.resources.pipelines.debug.destroy(core);

        self.resources.vertex.free(core);
        self.resources.debug_vertex.free(core);

        let device = &core.ctx.device;
        device.destroy_descriptor_set_layout(self.resources.terrain_global_layout);
        device.destroy_descriptor_set_layout(self.resources.actor_global_layout);
        device.destroy_descriptor_set_layout(self.resources.actor_batch_layout);
    }

    pub fn set_output(&mut self, core: &mut LoomzEngineCore) {
        let extent = core.info.swapchain_extent;
        let width = extent.width as f32;
        let height = extent.height as f32;
        let last_size = self.resources.grid_params.screen_size;
        if width == last_size.width && height == last_size.height {
            return;
        }

        self.resources.grid_params.screen_size = size(width, height);

        self.render.push_constants[0] = WorldPushConstant {
            screen_width: width,
            screen_height: height,
        };

        if !self.debug.is_empty() {
            self.build_debug_data(core);
        }
    }

    pub fn rebuild(&mut self, core: &mut LoomzEngineCore) {
        self.set_output(core);
    }

    //
    // Pipeline setup
    //

    pub fn write_pipeline_create_infos(&mut self, compiler: &mut PipelineCompiler) {
        compiler.add_pipeline_info("world_terrain", &mut self.resources.pipelines.terrain.pipeline);
        compiler.add_pipeline_info("world_actors", &mut self.resources.pipelines.actors.pipeline);
        compiler.add_pipeline_info("world_debug", &mut self.resources.pipelines.debug.pipeline);
    }

    pub fn set_pipeline_handle(&mut self, compiler: &PipelineCompiler) {
        let mut handle = compiler.get_pipeline("world_terrain");
        self.resources.pipelines.terrain.pipeline.set_handle(handle);
        self.render.terrain.pipeline_handle = handle;
        
        handle = compiler.get_pipeline("world_actors");
        self.resources.pipelines.actors.pipeline.set_handle(handle);
        self.render.actors.pipeline_handle = handle;

        handle = compiler.get_pipeline("world_debug");
        self.resources.pipelines.debug.pipeline.set_handle(handle);
        self.render.debug.pipeline_handle = handle;
    }

    //
    // Rendering
    //

    #[inline(always)]
    fn push_values(constants: &[WorldPushConstant; 1]) -> &[u8] {
        unsafe { constants.align_to::<u8>().1 }
    }

    pub fn render(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        if !self.flags.contains(WorldFlags::SHOW_WORLD) {
            return;
        }
        
        self.render_terrain(ctx, cmd);
        self.render_actors(ctx, cmd);

        if !self.debug.is_empty() {
            self.render_debug_info(ctx, cmd);
        }
    }

    fn render_terrain(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        const GRAPHICS: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;
        let device = &ctx.device;
        let push = Self::push_values(&self.render.push_constants);
        let render = self.render.terrain;

        device.cmd_bind_pipeline(cmd, GRAPHICS, render.pipeline_handle);
        device.cmd_bind_index_buffer(cmd, render.vertex_buffer[0], render.index_offset, vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, &render.vertex_buffer, &render.vertex_offset);
        device.cmd_push_constants(cmd, render.pipeline_layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, push);
        device.cmd_bind_descriptor_sets(cmd, GRAPHICS, render.pipeline_layout, TERRAIN_GLOBAL_LAYOUT_INDEX, slice::from_ref(&render.terrain_set), &[]);
        device.cmd_draw_indexed(cmd, 6, 1, 0, 0, 0);
    }

    fn render_actors(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        const GRAPHICS: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;
        let device = &ctx.device;
        let push = Self::push_values(&self.render.push_constants);
        let render = self.render.actors;

        device.cmd_bind_pipeline(cmd, GRAPHICS, render.pipeline_handle);
        device.cmd_bind_index_buffer(cmd, render.vertex_buffer[0], render.index_offset, vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, &render.vertex_buffer, &render.vertex_offset);
        device.cmd_push_constants(cmd, render.pipeline_layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, push);
        device.cmd_bind_descriptor_sets(cmd, GRAPHICS, render.pipeline_layout, ACTOR_GLOBAL_LAYOUT_INDEX, slice::from_ref(&render.sprites_set), &[]);

        for batch in self.batches.iter() {
            device.cmd_bind_descriptor_sets(cmd, GRAPHICS, render.pipeline_layout, ACTOR_BATCH_LAYOUT_INDEX, slice::from_ref(&batch.set), &[]);
            device.cmd_draw_indexed(cmd, 6, batch.instances_count, 0, 0, batch.instances_offset);
        }

    }

    fn render_debug_info(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        const GRAPHICS: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;
        let device = &ctx.device;
        let push = Self::push_values(&self.render.push_constants);
        let render = self.render.debug;

        if render.index_count == 0 {
            return;
        }

        device.cmd_bind_pipeline(cmd, GRAPHICS, render.pipeline_handle);
        device.cmd_bind_index_buffer(cmd, render.vertex_buffer[0], render.index_offset, vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, &render.vertex_buffer, &render.vertex_offset);
        device.cmd_push_constants(cmd, render.pipeline_layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, push);
        device.cmd_draw_indexed(cmd, render.index_count, 1, 0, 0, 0);
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
            self.flags |= WorldFlags::UPDATE_BATCHES;
        }

        // Initialize the instance UV
        let instance = &mut self.data.instances[actor_index];
        instance.uv_offset = [animation.x, animation.y];
        instance.uv_size = [animation.sprite_width, animation.sprite_height];

        Ok(())

    }
    
    fn update_world_actor(&mut self, core: &mut LoomzEngineCore, actor_index: usize, update: WorldActorUpdate) -> Result<(), CommonError> {
        match update {
            WorldActorUpdate::Position(position) => {
                self.data.instances[actor_index].position = position;
            },
            WorldActorUpdate::Flip(flipped) => {
                self.data.instances[actor_index].flipped = flipped;
            },
            WorldActorUpdate::Animation(animation_id) => {
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

    fn create_world_actor(&mut self, id: WorldActorId) -> usize {
        let instances = &mut self.data.instances;
        let new_id = instances.len();
        
        instances.push(WorldInstance {
            image_view: vk::ImageView::null(),
            position: PositionF32::default(),
            uv_offset: [0.0, 0.0],
            uv_size: [0.0, 0.0],
            texture_id: None,
            flipped: false,
        });

        id.bind(new_id as u32);

        new_id
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
                let index = match id.bound_value() {
                    Some(index) => index,
                    None => self.create_world_actor(id),
                };

                self.update_world_actor(core, index, actor)?;
            }
        }

        if let Some(messages) = api.world().read_general() {
            for (_, update) in messages {
                match update {
                    WorldUpdate::DebugFlags(flags) => { self.debug = flags; },
                    WorldUpdate::ShowWorld(visible) => { self.flags.set(WorldFlags::SHOW_WORLD, visible); }
                }
            }

            self.toggle_debug(core);
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
    
    pub fn update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        self.api_update(api, core)?;
        self.animation_update();

        if self.flags.contains(WorldFlags::UPDATE_BATCHES) {
            batch::WorldBatcher::build(self)?;
            self.flags ^= WorldFlags::UPDATE_BATCHES;
        }

        Ok(())
    }

    //
    // Data
    //

    pub fn reload_assets(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore, assets: &Vec<AssetId>) -> Result<(), CommonError> {
        for &assets_id in assets.iter() {
            match assets_id {
                AssetId::ShaderId(shader_id) => self.reload_shaders(api, core, shader_id)?,
                _ => {}
            }
        }

        Ok(())
    }
   
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

}

impl WorldDescriptors {
    pub fn write_sprite_buffer(&mut self, sprites: &StorageAlloc<SpriteData>) -> Result<vk::DescriptorSet, CommonError> {
        self.allocator.write_set::<ACTOR_GLOBAL_LAYOUT_ID>(&[
            DescriptorWriteBinding::from_storage_buffer(sprites)
        ])
    }

    pub fn write_batch_texture(&mut self, image_view: vk::ImageView) -> Result<vk::DescriptorSet, CommonError> {
        self.allocator.write_set::<ACTOR_BATCH_LAYOUT_ID>(&[
            DescriptorWriteBinding::from_image_and_sampler(image_view, self.default_sampler, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        ])
    }

    pub fn write_terrain_texture(&mut self, image_view: vk::ImageView) -> Result<vk::DescriptorSet, CommonError> {
        self.allocator.write_set::<TERRAIN_GLOBAL_LAYOUT_ID>(&[
            DescriptorWriteBinding::from_image_and_sampler(image_view, self.default_sampler, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        ])
    }

    pub fn reset_batch_layout(&mut self) {
        self.allocator.reset_layout::<ACTOR_BATCH_LAYOUT_ID >();
    }
}

impl WorldPipeline {
    fn destroy(self, core: &mut LoomzEngineCore) {
        self.pipeline.destroy(&core.ctx);
        core.ctx.device.destroy_pipeline_layout(self.layout);
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

impl Default for WorldRender {
    fn default() -> Self {
        WorldRender {
            terrain: WorldTerrainRender {
                pipeline_handle: vk::Pipeline::null(),
                pipeline_layout: vk::PipelineLayout::null(),
                vertex_buffer: [vk::Buffer::null()],
                vertex_offset: [0],
                index_offset: 0,
                terrain_set: vk::DescriptorSet::null(),
            },
            actors: WorldActorRender {
                pipeline_handle: vk::Pipeline::null(),
                pipeline_layout: vk::PipelineLayout::null(),
                vertex_buffer: [vk::Buffer::null()],
                vertex_offset: [0],
                index_offset: 0,
                sprites_set: vk::DescriptorSet::null(),
            },
            debug: WorldDebugRender {
                pipeline_handle: vk::Pipeline::null(),
                pipeline_layout: vk::PipelineLayout::null(),
                vertex_buffer: [vk::Buffer::null()],
                vertex_offset: [0],
                index_offset: 0,
                index_count: 0,
            },

            push_constants: [WorldPushConstant::default(); 1],
        }
    }
}

impl Default for WorldPipeline {
    fn default() -> Self {
        WorldPipeline {
            pipeline: GraphicsPipeline::new(),
            layout: vk::PipelineLayout::null(),
            id: ShaderId(0)
        }
    }
}
