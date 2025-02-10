use std::time::Instant;
use loomz_shared::api::{
    WorldActorId, WorldAnimationId, WorldAnimation, WorldActorUpdate, WorldTerrainChunk,
    TerrainChunk, TERRAIN_CHUNK_SIZE, TERRAIN_CELL_SIZE_PX
};
use loomz_shared::{CommonError, CommonErrorType, TextureId, PositionF32, SizeU32, RectF32, rect, backend_err, assets_err, chain_err};
use loomz_engine_core::{LoomzEngineCore, alloc::StorageAlloc};

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub(super) struct ActorSpriteData {
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub uv_offset: [f32; 2],
    pub uv_size: [f32; 2],
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub(super) struct TerrainSpriteData {
    pub uv_offset: [f32; 2],
}

pub(super) struct WorldAnimationWithId {
    id: u32,
    animation: WorldAnimation,
}

#[derive(Copy, Clone)]
pub(super) struct WorldActorData {
    pub descriptor_set: vk::DescriptorSet,
    pub animation: WorldAnimation,
    pub position: PositionF32,
    pub current_frame: u8,
    pub flipped: bool,
}

pub(super) struct WorldTerrainChunkData {
    pub view: RectF32,
    pub cells: Box<TerrainChunk<TerrainSpriteData>>,
}

impl WorldTerrainChunkData {
    fn new(x: usize, y: usize) -> Self {
        let stride_px = (TERRAIN_CHUNK_SIZE as f32) * (TERRAIN_CELL_SIZE_PX as f32);
        let [x, y] = [x as f32, y as f32];
        let [x, y] = [x * stride_px, y * stride_px];
        WorldTerrainChunkData {
            view: rect(x, y, x+stride_px, y+stride_px),
            cells: Default::default()
        }
    }
}

/// World data to be rendered on screen
pub(super) struct WorldData {
    pub world_view: RectF32,

    pub last_animation_tick: Instant,
    pub animations: Vec<WorldAnimationWithId>,

    pub default_actor: Option<Box<WorldActorData>>,
    pub actors_ids: Vec<u32>,
    pub actors_data: Vec<WorldActorData>,
    pub actors_sprites: StorageAlloc<ActorSpriteData>,

    pub terrain_tilemap: Vec<TerrainSpriteData>,
    pub terrain_size: SizeU32,
    pub terrain_chunks: Vec<WorldTerrainChunkData>,
    pub terrain_sprites: StorageAlloc<TerrainSpriteData>,
}


impl super::WorldModule {

    //
    // Animations
    //

    pub(super) fn update_world_animation(&mut self, id: WorldAnimationId, animation: WorldAnimation) -> Result<(), CommonError> {
        let id = id.value();
        let found = self.data.animations.iter().any(|animation| animation.id == id );
        if found {
            return Err(backend_err!("Tried to update animation with {id:?}, but updating animations is not allowed"));
        }

        self.data.animations.push(WorldAnimationWithId {
            id,
            animation,
        });
        
        Ok(())
    }

    //
    // Actors
    //

    fn fetch_texture_descriptor_set(
        core: &mut LoomzEngineCore,
        resources: &mut super::WorldResources,
        texture_id: TextureId
    ) -> Result<vk::DescriptorSet, CommonError> {
        use super::{ACTOR_BATCH_LAYOUT_ID, ACTOR_SAMPLER_BINDING_INDEX};

        if let Some(texture) = resources.textures.get(&texture_id) {
            return Ok(texture.descriptor_set);
        }

        let texture_asset = resources.assets.texture(texture_id)
            .ok_or_else(|| assets_err!("Unkown asset with ID {texture_id:?}") )?;

        let texture = core.create_texture_from_asset(&texture_asset)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendGeneric, "Failed to create image from asset") )?;

        let descriptor_set = resources.descriptors.get_set::<ACTOR_BATCH_LAYOUT_ID>()
            .ok_or_else(|| backend_err!("No more descriptor set in actor batch layout pool") )?;

        core.descriptors.write_image(
            descriptor_set,
            texture.view,
            resources.default_sampler,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ACTOR_SAMPLER_BINDING_INDEX,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        );

        resources.textures.insert(texture_id, super::WorldTexture {
            texture,
            descriptor_set,
        });

        Ok(descriptor_set)
    }

    fn create_actor(&mut self, id: u32) -> usize {
        let index = self.data.actors_ids.len();
        let actor = self.data.default_actor.as_ref()
            .unwrap_or_else(|| unreachable!("Default actor must have been created at startup") );

        self.data.actors_ids.push(id);
        self.data.actors_data.push(**actor);

        index
    }

    fn update_world_actor_animation(&mut self, core: &mut LoomzEngineCore, actor_index: usize, animation: WorldAnimation) -> Result<(), CommonError> {
        let actor = &mut self.data.actors_data[actor_index];
        let old_animation = actor.animation;
        if animation.texture_id != old_animation.texture_id {
            actor.descriptor_set = Self::fetch_texture_descriptor_set(core, &mut self.resources, animation.texture_id)?;
            self.flags |= super::WorldFlags::UPDATE_ACTORS;
        }

        actor.animation = animation;

        Ok(())

    }

    fn destroy_actor(&mut self, index: usize) {
        self.data.actors_ids.swap_remove(index);
        self.data.actors_data.swap_remove(index);
        self.flags |= super::WorldFlags::UPDATE_ACTORS;
    }
    
    fn write_world_actor_sprite(&mut self, index: usize) {
        let actor = &self.data.actors_data[index];
        let sprite = actor.sprite_data();
        self.data.actors_sprites.write_data(index, sprite);
    }

    pub(super) fn update_world_actor(&mut self, core: &mut LoomzEngineCore, id: WorldActorId, update: WorldActorUpdate) -> Result<(), CommonError> {
        let id = id.value();
        let index = self.data.actors_ids.iter().position(|&id2| id2 == id )
            .unwrap_or_else(|| self.create_actor(id) );
        
        match update {
            WorldActorUpdate::Position(position) => {
                self.data.actors_data[index].position = position;
                self.write_world_actor_sprite(index);
            },
            WorldActorUpdate::Flip(flipped) => {
                self.data.actors_data[index].flipped = flipped;
                self.write_world_actor_sprite(index);
            },
            WorldActorUpdate::Animation(animation_id) => {
                let animation_id = animation_id.value();
                let animation = self.data.animations.iter().find(|a| a.id == animation_id )
                    .map(|a| a.animation )
                    .ok_or_else(|| backend_err!("Failed to find an animation with ID {animation_id}") )?;

                self.update_world_actor_animation(core, index, animation)?;
                self.write_world_actor_sprite(index);
            },
            WorldActorUpdate::Destroy => {
                self.destroy_actor(index);
            }
        }

        Ok(())
    }

    //
    // Terrain
    //

    pub(super) fn set_world_size(&mut self, size: SizeU32) {
        let data = &mut self.data;
        data.terrain_size = size;
        data.terrain_chunks.clear();

        let batch_x = ((size.width as usize) + (TERRAIN_CHUNK_SIZE-1)) / TERRAIN_CHUNK_SIZE;
        let batch_y = ((size.height as usize) + (TERRAIN_CHUNK_SIZE-1)) / TERRAIN_CHUNK_SIZE;

        for y in 0..batch_y {
            for x in 0..batch_x {
                data.terrain_chunks.push(WorldTerrainChunkData::new(x, y))
            }
        }
    }

    pub(super) fn copy_terrain_batch(&mut self, chunk: &WorldTerrainChunk) -> Result<(), CommonError> {
        let data = &mut self.data;

        let x = chunk.position.x as usize;
        let y = chunk.position.y as usize;
        let chunk_index = (y * TERRAIN_CHUNK_SIZE) + x;

        let chunk_data = data.terrain_chunks.get_mut(chunk_index)
            .ok_or_else(|| backend_err!("Tried to update a chunk outside of the terrain range") )?;

        if chunk_data.view != chunk.view {
            return Err(backend_err!("Mismatch between client chunk and engine chunk"));
        }

        let tiles = &data.terrain_tilemap;

        for row in 0..TERRAIN_CHUNK_SIZE {
            let row_type = &chunk.cells[row];
            let row_data = &mut chunk_data.cells[row];
            for (data, cell_type) in row_data.iter_mut().zip(row_type) {
                *data = tiles.get(*cell_type as usize)
                    .copied()
                    .unwrap_or_default();
            } 
        }

        Ok(())
    }

    pub(super) fn generate_terrain_cells(&mut self, core: &mut LoomzEngineCore) {
        let data = &mut self.data;

        let sprites = &mut data.terrain_sprites;
        let mut sprites_offset = 0;

        let view = data.world_view;
        // for chunk in data.terrain_chunks.iter() {
        //     if !view.intersects(&chunk.view) {
        //         continue;
        //     }

        //     for row in chunk.cells.iter() {
        //         for &cell in row.iter() {
        //             sprites.write_data(sprites_offset, cell);
        //             sprites_offset += 1;
        //         }
        //     }
        // }
    }

}

impl WorldActorData {
    pub(super) fn sprite_data(&self) -> ActorSpriteData {
        let mut sprite = ActorSpriteData::default();
        let i = self.current_frame as f32;
        let animation = self.animation;
        sprite.offset[0] = self.position.x - (animation.sprite_width * 0.5);
        sprite.offset[1] = self.position.y - (animation.sprite_height * 0.5);
        sprite.size[0] = animation.sprite_width;
        sprite.size[1] = animation.sprite_height;
        sprite.uv_offset[0] = animation.x + (animation.sprite_width * i) + (animation.padding * i);
        sprite.uv_offset[1] = animation.y;
        sprite.uv_size[0] = animation.sprite_width;
        sprite.uv_size[1] = animation.sprite_height;

        if self.flipped {
            sprite.uv_offset[0] += sprite.size[0];
            sprite.uv_size[0] *= -1.0;
        }

        sprite
    }
}

impl Default for WorldData {
    fn default() -> Self {
        WorldData {
            world_view: RectF32::default(),
            last_animation_tick: Instant::now(),
            animations: Vec::with_capacity(16),

            default_actor: None,
            actors_ids: Vec::with_capacity(16),
            actors_data: Vec::with_capacity(16),
            actors_sprites: StorageAlloc::default(),

            terrain_tilemap: Vec::with_capacity(4),
            terrain_size: SizeU32::default(),
            terrain_chunks: Vec::with_capacity(16),
            terrain_sprites: StorageAlloc::default(),
        }
    }
}
