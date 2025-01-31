use bitflags::bitflags;
use crate::PositionF32;
use crate::assets::TextureId;
use super::base::{Id, MessageQueue};

pub struct WorldAnimationTag;
pub type WorldAnimationId = Id<WorldAnimationTag>;

pub struct WorldActorTag;
pub type WorldActorId = Id<WorldActorTag>;

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

bitflags! {
    #[derive(Copy, Clone, Default)]
    pub struct WorldDebugFlags: u8 {
        const SHOW_MAIN_GRID       = 0b0001;
        const SHOW_SUB_GRID        = 0b0010;
        const SHOW_MAIN_GRID_TYPES = 0b0100;
    }
}

pub enum WorldUpdate {
    DebugFlags(WorldDebugFlags),
    ShowWorld(bool)
}

pub struct WorldApi {
    pub animations: MessageQueue<WorldAnimationId, WorldAnimation>,
    pub actors: MessageQueue<WorldActorId, WorldActorUpdate>,
    pub general: MessageQueue<(), WorldUpdate>,
}

impl WorldApi {

    pub fn init() -> Self {
        WorldApi {
            animations: MessageQueue::with_capacity(16),
            actors: MessageQueue::with_capacity(16),
            general: MessageQueue::with_capacity(8)
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

    pub fn read_general<'a>(&'a self) -> Option<impl Iterator<Item=((), WorldUpdate)> + 'a> {
        self.general.read_values()
    }
}
