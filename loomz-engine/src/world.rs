mod setup;
mod data;
mod batch;
mod debug;

use fnv::FnvHashMap;
use bitflags::bitflags;
use std::{slice, sync::Arc, u32, usize};
use loomz_shared::api::{LoomzApi, WorldUpdate, WorldDebugFlags};
use loomz_shared::assets::{LoomzAssetsBundle, TextureId, ShaderId, AssetId};
use loomz_shared::{CommonError, RectF32, RgbaU8, SizeF32, size};
use loomz_engine_core::{LoomzEngineCore, VulkanContext, Texture, alloc::VertexAlloc, descriptors::*, pipelines::*};
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
const TERRAIN_SPRITE_BUFFER_BINDING_INDEX: u32 = 0;
const TERRAIN_SAMPLER_BINDING_INDEX: u32 = 1;

const ACTOR_GLOBAL_LAYOUT_INDEX: u32 = 0;
const ACTOR_SPRITE_BUFFER_BINDING_INDEX: u32 = 0;

const ACTOR_BATCH_LAYOUT_INDEX: u32 = 1;
const ACTOR_SAMPLER_BINDING_INDEX: u32 = 0;

bitflags! {
    #[derive(Copy, Clone, Default)]
    pub struct WorldFlags: u8 {
        const SHOW_WORLD     = 0b0001;
        const UPDATE_ACTORS  = 0b0010;
        const UPDATE_TERRAIN = 0b0100;
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct WorldPushConstant {
    pub screen_width: f32,
    pub screen_height: f32,
    pub view_offset_x: f32,
    pub view_offset_y: f32,
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
    terrain_global_layout: vk::DescriptorSetLayout,
    actor_global_layout: vk::DescriptorSetLayout,
    actor_batch_layout: vk::DescriptorSetLayout,
}

struct WorldTexture {
    texture: Texture,
    descriptor_set: vk::DescriptorSet,
}

/// Graphics resources that are not accessed often (not every frame)
struct WorldResources {
    assets: Arc<LoomzAssetsBundle>,

    pipelines: WorldPipelines,

    vertex: VertexAlloc<WorldVertex>,
    debug_vertex: VertexAlloc<WorldDebugVertex>,

    descriptors: DescriptorsAllocator<LAYOUT_COUNT>,

    default_sampler: vk::Sampler,
    terrain_texture: Option<Texture>,
    textures: FnvHashMap<TextureId, WorldTexture>,

    grid_params: WorldGridParams,
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
struct WorldActorRender {
    pipeline_handle: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    vertex_buffer: [vk::Buffer; 1],
    vertex_offset: [vk::DeviceSize; 1],
    index_offset: vk::DeviceSize,
    sprites_set: vk::DescriptorSet,
    batches: Vec<WorldBatch>,
}

struct WorldRender {
    terrain: WorldTerrainRender,
    actors: WorldActorRender,
    debug: WorldDebugRender,
    push_constants: [WorldPushConstant; 1],
}

pub(crate) struct WorldModule {
    resources: Box<WorldResources>,
    data: Box<data::WorldData>,
    render: Box<WorldRender>,
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

            descriptors: DescriptorsAllocator::default(),

            default_sampler: vk::Sampler::null(),
            terrain_texture: None,
            textures: FnvHashMap::default(),
         
            grid_params: WorldGridParams {
                screen_size: size(0.0, 0.0),
                cell_size: 64.0,
            }
        };

        let mut world = WorldModule {
            resources: Box::new(resources),
            data: Box::default(),
            render: Box::default(),
            debug: WorldDebugFlags::empty(),
            flags: WorldFlags::SHOW_WORLD,
        };

        world.setup_pipelines(core, api)?;
        world.setup_descriptors(core)?;
        world.setup_buffers(core)?;
        world.setup_terrain_tilemap(core)?;
        world.setup_render_data();
        world.setup_default_data(core)?;

        Ok(world)
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.data.actors_sprites.free(core);
        self.data.terrain_sprites.free(core);

        self.resources.vertex.free(core);
        self.resources.debug_vertex.free(core);
        self.resources.descriptors.destroy(core);

        if let Some(texture) = self.resources.terrain_texture.as_ref() {
            core.destroy_texture(*texture);
        }

        for texture in self.resources.textures.values() {
            core.destroy_texture(texture.texture);
        }

        let ctx = &core.ctx;
        let pipelines = self.resources.pipelines;
        pipelines.terrain.destroy(ctx);
        pipelines.actors.destroy(ctx);
        pipelines.debug.destroy(ctx);
        ctx.device.destroy_descriptor_set_layout(pipelines.terrain_global_layout);
        ctx.device.destroy_descriptor_set_layout(pipelines.actor_global_layout);
        ctx.device.destroy_descriptor_set_layout(pipelines.actor_batch_layout);
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

        self.render.push_constants[0].screen_width = width;
        self.render.push_constants[0].screen_height = height;

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

        // device.cmd_bind_pipeline(cmd, GRAPHICS, render.pipeline_handle);
        // device.cmd_bind_index_buffer(cmd, render.vertex_buffer[0], render.index_offset, vk::IndexType::UINT32);
        // device.cmd_bind_vertex_buffers(cmd, 0, &render.vertex_buffer, &render.vertex_offset);
        // device.cmd_push_constants(cmd, render.pipeline_layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, push);
        // device.cmd_bind_descriptor_sets(cmd, GRAPHICS, render.pipeline_layout, TERRAIN_GLOBAL_LAYOUT_INDEX, slice::from_ref(&render.terrain_set), &[]);
        // device.cmd_draw_indexed(cmd, 6, 1, 0, 0, 0);
    }

    fn render_actors(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        const GRAPHICS: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;
        let device = &ctx.device;
        let push = Self::push_values(&self.render.push_constants);
        let render = &self.render.actors;

        device.cmd_bind_pipeline(cmd, GRAPHICS, render.pipeline_handle);
        device.cmd_bind_index_buffer(cmd, render.vertex_buffer[0], render.index_offset, vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, &render.vertex_buffer, &render.vertex_offset);
        device.cmd_push_constants(cmd, render.pipeline_layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, push);
        device.cmd_bind_descriptor_sets(cmd, GRAPHICS, render.pipeline_layout, ACTOR_GLOBAL_LAYOUT_INDEX, slice::from_ref(&render.sprites_set), &[]);

        for batch in render.batches.iter() {
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

    fn set_view_offset(&mut self, view: RectF32) {
        let push = &mut self.render.push_constants[0];
        push.view_offset_x = view.left;
        push.view_offset_y = view.top;

        self.data.world_view = view;
    }

    fn api_update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        if let Some(animations) = api.world().read_animations() {
            for (id, animation) in animations {
                self.update_world_animation(id, animation)?;
            }
        }

        if let Some(actors) = api.world().read_actors() {
            for (id, actor) in actors {
                self.update_world_actor(core, id, actor)?;
            }
        }

        if let Some(messages) = api.world().read_general() {
            for (_, update) in messages {
                match update {
                    WorldUpdate::ShowWorld(visible) => { self.flags.set(WorldFlags::SHOW_WORLD, visible); },
                    WorldUpdate::WorldView(view) => { self.set_view_offset(view); },
                    WorldUpdate::WorldSize(size) => { 
                        self.set_world_size(size);
                        self.flags |= WorldFlags::UPDATE_TERRAIN;
                    },
                    WorldUpdate::WorldTerrain(chunk) => {
                        self.copy_terrain_batch(&chunk[0])?;
                        self.flags |= WorldFlags::UPDATE_TERRAIN;
                    },
                    WorldUpdate::DebugFlags(flags) => {
                        self.debug = flags;
                        self.toggle_debug(core);
                    },
                }
            }
        }

        Ok(())
    }

    fn animation_update(&mut self) { 
        let sprites = &mut self.data.actors_sprites;
        for (index, actor) in self.data.actors_data.iter_mut().enumerate() {
            actor.current_frame += 1;

            if actor.current_frame > actor.animation.last_frame {
                actor.current_frame = 0;
            }

            sprites.write_data(index, actor.sprite_data());
        }

        self.data.last_animation_tick = ::std::time::Instant::now();
    }
    
    pub fn update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        const ANIMATION_INTERVAL: f32 = 1.0 / 16.0; // 16fps

        self.api_update(api, core)?;

        if self.data.last_animation_tick.elapsed().as_secs_f32() > ANIMATION_INTERVAL {
            self.animation_update();
        }

        if self.flags.contains(WorldFlags::UPDATE_ACTORS) {
            batch::batch_actors(self);
            self.flags.remove(WorldFlags::UPDATE_ACTORS);
        }

        if self.flags.contains(WorldFlags::UPDATE_TERRAIN) {
            self.generate_terrain_cells(core);
            self.flags.remove(WorldFlags::UPDATE_TERRAIN);
        }

        Ok(())
    }

    pub fn reload_assets(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore, assets: &Vec<AssetId>) -> Result<(), CommonError> {
        for &assets_id in assets.iter() {
            match assets_id {
                AssetId::ShaderId(shader_id) => self.reload_shaders(api, core, shader_id)?,
                _ => {}
            }
        }

        Ok(())
    }

}

impl WorldPipeline {
    fn destroy(self, ctx: &VulkanContext) {
        self.pipeline.destroy(ctx);
        ctx.device.destroy_pipeline_layout(self.layout);
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
                batches: Vec::with_capacity(16),
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
