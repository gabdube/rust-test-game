mod setup;
mod batch;

use fnv::FnvHashMap;
use std::{slice, sync::Arc, time::Instant, u32, usize};
use loomz_shared::api::{LoomzApi, WorldAnimationId, WorldAnimation, WorldActorId, WorldActorUpdate};
use loomz_shared::{_2d::Position, CommonError, CommonErrorType, assets::{LoomzAssetsBundle, TextureId}};
use loomz_shared::{assets_err, backend_err, chain_err};
use loomz_engine_core::{LoomzEngineCore, VulkanContext, Texture, alloc::{VertexAlloc, StorageAlloc}, descriptors::*, pipelines::*};
use super::pipeline_compiler::PipelineCompiler;

const PUSH_STAGE_FLAGS: vk::ShaderStageFlags = vk::ShaderStageFlags::VERTEX;
const PUSH_SIZE: u32 = size_of::<WorldPushConstant>() as u32;

const LAYOUT_COUNT: usize = 2;
const GLOBAL_LAYOUT_INDEX: u32 = 0;
const BATCH_LAYOUT_INDEX: u32 = 1;

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

/// Graphics resources that are not accessed often
struct WorldResources {
    assets: Arc<LoomzAssetsBundle>,
    pipeline: GraphicsPipeline,
    vertex: VertexAlloc<WorldVertex>,
    textures: FnvHashMap<TextureId, Texture>,
    global_layout: vk::DescriptorSetLayout,
    batch_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
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

/// World data to be rendered on screen
struct WorldData {
    sprites: StorageAlloc<SpriteData>,
    animations: Vec<WorldAnimation>,
    instance_animations: Vec<WorldInstanceAnimation>,
    instances: Vec<WorldInstance>,
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

pub(crate) struct WorldModule {
    resources: Box<WorldResources>,
    descriptors: Box<WorldDescriptors>,
    data: Box<WorldData>,
    render: Box<WorldRender>,
    batches: Vec<WorldBatch>,
    update_batches: bool,
}

impl WorldModule {

    pub fn init(core: &mut LoomzEngineCore, api: &LoomzApi) -> Result<Self, CommonError> {
        let resources = WorldResources {
            assets: api.assets(),
            pipeline: GraphicsPipeline::new(),
            vertex: VertexAlloc::default(),
            textures: FnvHashMap::default(),
            global_layout: vk::DescriptorSetLayout::null(),
            batch_layout: vk::DescriptorSetLayout::null(),
            pipeline_layout: vk::PipelineLayout::null(),
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
            descriptors: Box::default(),
            update_batches: false,
        };

        world.setup_pipeline(core)?;
        world.setup_descriptors(core)?;
        world.setup_vertex_buffers(core)?;
        world.setup_sprites_buffers(core)?;
        world.setup_render_data();

        Ok(world)
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.descriptors.allocator.destroy(core);
        self.resources.pipeline.destroy(&core.ctx);
        self.resources.vertex.free(core);
        self.data.sprites.free(core);

        for texture in self.resources.textures.values() {
            core.destroy_texture(*texture);
        }

        let device = &core.ctx.device;
        device.destroy_pipeline_layout(self.resources.pipeline_layout);
        device.destroy_descriptor_set_layout(self.resources.global_layout);
        device.destroy_descriptor_set_layout(self.resources.batch_layout);
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
            position: Position::default(),
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

        if self.update_batches {
            batch::WorldBatcher::build(self)?;
            self.update_batches = false;
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

}

impl WorldDescriptors {
    pub fn write_sprite_buffer(&mut self, sprites: &StorageAlloc<SpriteData>) -> Result<vk::DescriptorSet, CommonError> {
        self.allocator.write_set::<GLOBAL_LAYOUT_INDEX>(&[
            DescriptorWriteBinding::from_storage_buffer(sprites)
        ])
    }

    pub fn write_batch_texture(&mut self, image_view: vk::ImageView) -> Result<vk::DescriptorSet, CommonError> {
        self.allocator.write_set::<BATCH_LAYOUT_INDEX>(&[
            DescriptorWriteBinding::from_image_and_sampler(image_view, self.default_sampler, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        ])
    }

    pub fn reset_batch_layout(&mut self) {
        self.allocator.reset_layout::<BATCH_LAYOUT_INDEX>();
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
