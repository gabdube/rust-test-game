use bitflags::bitflags;
use crate::{RectF32, PositionF32, PositionU32, SizeU32, rect};
use crate::assets::TextureId;
use super::base::{Id, MessageQueue, MessageQueueEx};

/// Number of cells in a chunk row
pub const TERRAIN_CHUNK_STRIDE: usize = 16;
pub const TERRAIN_CHUNK_SIZE: usize = TERRAIN_CHUNK_STRIDE * TERRAIN_CHUNK_STRIDE;

/// The size of a cell in px on screen
pub const TERRAIN_CELL_SIZE_PX: usize = 64;

pub type TerrainChunk<T> = [[T; TERRAIN_CHUNK_STRIDE]; TERRAIN_CHUNK_STRIDE];


pub struct WorldAnimationTag;
pub type WorldAnimationId = Id<WorldAnimationTag>;

pub struct WorldActorTag;
pub type WorldActorId = Id<WorldActorTag>;

bitflags! {
    #[derive(Copy, Clone, Default)]
    pub struct WorldDebugFlags: u8 {
        const SHOW_MAIN_GRID       = 0b0001;
        const SHOW_SUB_GRID        = 0b0010;
        const SHOW_MAIN_GRID_TYPES = 0b0100;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TerrainType {
    #[default]
    Grass,
    Sand,
    Water,
    Max,
}

impl TerrainType {
    pub const fn names() -> &'static [&'static str] {
        &[
            "grass",
            "sand",
            "water",
        ]
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct WorldTerrainChunk {
    pub position: PositionU32,
    pub view: RectF32,
    pub cells: TerrainChunk<TerrainType>,
}

impl WorldTerrainChunk {
    pub fn new(chunk_x: usize, chunk_y: usize) -> Self {
        let stride_px = (TERRAIN_CHUNK_STRIDE as f32) * (TERRAIN_CELL_SIZE_PX as f32);
        let [x, y] = [(chunk_x as f32) * stride_px, (chunk_y as f32) * stride_px];
        WorldTerrainChunk {
            position: PositionU32 { x: chunk_x as u32, y: chunk_y as u32 },
            view: rect(x, y, x+stride_px, y+stride_px),
            cells: Default::default()
        }
    }
}

#[derive(Copy, Clone)]
pub struct WorldAnimation {
    pub texture_id: TextureId,
    pub padding: f32,
    pub x: f32,
    pub y: f32,
    pub sprite_width: f32,
    pub sprite_height: f32,
    pub last_frame: u8,
}

pub enum WorldActorUpdate {
    Position(PositionF32),
    Animation(WorldAnimationId),
    Flip(bool),
    Destroy,
}

pub enum WorldUpdate {
    DebugFlags(WorldDebugFlags),
    ShowWorld(bool),
    WorldView(RectF32),
    WorldSize(SizeU32),
    WorldTerrain(&'static [WorldTerrainChunk])
}

pub struct WorldApi {
    pub animations: MessageQueue<WorldAnimationId, WorldAnimation>,
    pub actors: MessageQueue<WorldActorId, WorldActorUpdate>,
    pub general: MessageQueueEx<(), WorldUpdate>,
}

impl WorldApi {

    pub fn init() -> Self {
        WorldApi {
            animations: MessageQueue::with_capacity(16),
            actors: MessageQueue::with_capacity(16),
            general: MessageQueueEx::with_capacity(16, 5012)
        }
    }

    pub fn create_animation(&self, id: &WorldAnimationId, animation_data: WorldAnimation) {
        self.animations.push(id, animation_data);
    }

    pub fn read_animations<'a>(&'a self) -> Option<impl Iterator<Item = (WorldAnimationId, WorldAnimation)> + 'a> {
        self.animations.read_values()
    }

    pub fn create_actor(&self, id: &WorldActorId, position: PositionF32, animation_id: &WorldAnimationId) {
        self.actors.push(id, WorldActorUpdate::Position(position));
        self.actors.push(id, WorldActorUpdate::Animation(animation_id.clone()));
    }

    pub fn destroy_actor(&self, id: &WorldActorId) {
        self.actors.push(id, WorldActorUpdate::Destroy);
    }

    pub fn update_actor_position(&self, id: &WorldActorId, position: PositionF32) {
        self.actors.push(id, WorldActorUpdate::Position(position));
    }

    pub fn update_actor_animation(&self, id: &WorldActorId, anim: &WorldAnimationId) {
        self.actors.push(id, WorldActorUpdate::Animation(anim.clone()));
    }
    
    pub fn flip_actor(&self, id: &WorldActorId, flip: bool) {
        self.actors.push(id, WorldActorUpdate::Flip(flip));
    }

    pub fn read_actors<'a>(&'a self) -> Option<impl Iterator<Item = (WorldActorId, WorldActorUpdate)> + 'a> {
        self.actors.read_values()
    }

    pub fn toggle_debug(&self, debug: WorldDebugFlags) {
        self.general.push(&(), WorldUpdate::DebugFlags(debug));
    }

    pub fn toggle_world(&self, visible: bool) {
        self.general.push(&(), WorldUpdate::ShowWorld(visible));
    }

    /// Sets the world view (in pixels)
    pub fn set_world_view(&self, view: RectF32) {
        self.general.push(&(), WorldUpdate::WorldView(view))
    }

    /// Sets the world size (in cells)
    pub fn set_world_size(&self, size: SizeU32) {
        self.general.push(&(), WorldUpdate::WorldSize(size));
    }

    pub fn update_terrain(&self, chunk: &WorldTerrainChunk) {
        self.general.push_with_data(&(), ::std::slice::from_ref(chunk), |chunk| WorldUpdate::WorldTerrain(chunk) );
    }

    pub fn read_general<'a>(&'a self) -> Option<impl Iterator<Item=((), WorldUpdate)> + 'a> {
        self.general.read_values()
    }
}
