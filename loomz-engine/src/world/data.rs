use std::time::Instant;
use loomz_shared::api::{WorldActorId, WorldAnimationId, WorldAnimation, WorldActorUpdate, WorldTerrainChunk, TerrainChunk};
use loomz_shared::{CommonError, PositionF32, RectF32, backend_err};
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

#[derive(Copy, Clone, Default)]
pub(super) struct WorldActorData {
    pub image_view: vk::ImageView,
    pub animation: Option<WorldAnimation>,
    pub position: PositionF32,
    pub current_frame: u8,
    pub flipped: bool,
}


pub(super) struct WorldTerrainSprite {

}

pub(super) struct WorldTerrainChunkData {
    pub position: PositionF32,
    pub cells: Box<TerrainChunk<TerrainSpriteData>>,
}

/// World data to be rendered on screen
pub(super) struct WorldData {
    pub world_view: RectF32,

    pub last_animation_tick: Instant,
    pub animations: Vec<WorldAnimationWithId>,

    pub actors_ids: Vec<u32>,
    pub actors_data: Vec<WorldActorData>,
    pub actors_sprites: StorageAlloc<ActorSpriteData>,

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

    fn create_actor(&mut self, id: u32) -> usize {
        let index = self.data.actors_ids.len();
        self.data.actors_ids.push(id);
        self.data.actors_data.push(WorldActorData::default());
        index
    }

    fn update_world_actor_animation(&mut self, core: &mut LoomzEngineCore, actor_index: usize, animation: WorldAnimation) -> Result<(), CommonError> {
        // Insert or create new animation
        let actor = &mut self.data.actors_data[actor_index];
        match actor.animation.as_mut() {
            Some(old_animation) => {
                if animation.texture_id != old_animation.texture_id {
                    actor.image_view = Self::fetch_texture_view(core, &mut self.resources, animation.texture_id)?;
                    self.flags |= super::WorldFlags::UPDATE_ACTORS;
                }

                *old_animation = animation;
            },
            None => {
                actor.animation = Some(animation);
                actor.image_view = Self::fetch_texture_view(core, &mut self.resources, animation.texture_id)?;
                self.flags |= super::WorldFlags::UPDATE_ACTORS;
            }
        }

        Ok(())

    }

    fn destroy_actor(&mut self, index: usize) {
        self.data.actors_ids.swap_remove(index);
        self.data.actors_data.swap_remove(index);
        self.flags |= super::WorldFlags::UPDATE_ACTORS;
    }
    
    fn write_world_actor_sprite(&mut self, index: usize) {
        if index < self.data.actors_data.len() {
            let actor = &self.data.actors_data[index];
            let sprite = actor.sprite_data();
            self.data.actors_sprites.write_data(index, sprite);
        }
    }

    pub(super) fn update_world_actor(&mut self, core: &mut LoomzEngineCore, id: WorldActorId, update: WorldActorUpdate) -> Result<(), CommonError> {
        let id = id.value();
        let index = self.data.actors_ids.iter().position(|&id2| id2 == id )
            .unwrap_or_else(|| self.create_actor(id) );
        
        match update {
            WorldActorUpdate::Position(position) => {
                self.data.actors_data[index].position = position;
            },
            WorldActorUpdate::Flip(flipped) => {
                self.data.actors_data[index].flipped = flipped;
            },
            WorldActorUpdate::Animation(animation_id) => {
                let animation_id = animation_id.value();
                let animation = self.data.animations.iter().find(|a| a.id == animation_id )
                    .map(|a| a.animation )
                    .ok_or_else(|| backend_err!("Failed to find an animation with ID {animation_id}") )?;

                self.update_world_actor_animation(core, index, animation)?;
            },
            WorldActorUpdate::Destroy => {
                self.destroy_actor(index);
            }
        }

        self.write_world_actor_sprite(index);

        Ok(())
    }

    //
    // Terrain
    //

    pub(super) fn update_terrain_from_batch(&mut self, core: &mut LoomzEngineCore, chunk: &WorldTerrainChunk) {
        println!("TODO update_terrain_from_batch");
    }

    pub(super) fn copy_terrain_cells(&mut self, core: &mut LoomzEngineCore) {
        println!("TODO copy_terrain_cells");
    }

}

impl WorldActorData {
    pub(super) fn sprite_data(&self) -> ActorSpriteData {
        let mut sprite = ActorSpriteData::default();
        if let Some(animation) = self.animation {
            let i = self.current_frame as f32;
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

            actors_ids: Vec::with_capacity(16),
            actors_data: Vec::with_capacity(16),
            actors_sprites: StorageAlloc::default(),

            terrain_chunks: Vec::with_capacity(16),
            terrain_sprites: StorageAlloc::default(),
        }
    }
}
