use crate::_2d::{Position, Size};
use crate::assets::TextureId;
use super::base::{Id, MessageQueue};

pub struct WorldAnimationTag;
pub type WorldAnimationId = Id<WorldAnimationTag>;

pub struct WorldActorTag;
pub type WorldActorId = Id<WorldActorTag>;

#[derive(Copy, Clone)]
pub struct WorldAnimation {
    pub texture_id: TextureId,
    pub last_frame: u32,
    pub interval: f32,
}

pub enum WorldActor {
    Position(Position<f32>),
    Size(Size<f32>),
    Animation(WorldAnimationId),
}

pub struct WorldApi {
    pub animations: MessageQueue<WorldAnimationId, WorldAnimation>,
    pub actors: MessageQueue<WorldActorId, WorldActor>,
}

impl WorldApi {

    pub fn init() -> Self {
        WorldApi {
            animations: MessageQueue::with_capacity(16),
            actors: MessageQueue::with_capacity(16),
        }
    }

    pub fn create_animation(&self, id: &WorldAnimationId, animation_data: WorldAnimation) {
        self.animations.push(id, animation_data);
    }

    pub fn read_animations<'a>(&'a self) -> Option<impl Iterator<Item = (WorldAnimationId, WorldAnimation)> + 'a> {
        self.animations.read_values()
    }

    pub fn create_actor(&self, id: &WorldActorId, position: Position<f32>, size: Size<f32>, animation_id: &WorldAnimationId) {
        self.actors.push(id, WorldActor::Position(position));
        self.actors.push(id, WorldActor::Size(size));
        self.actors.push(id, WorldActor::Animation(animation_id.clone()));
    }

    pub fn update_actor(&self, id: &WorldActorId, param: WorldActor) {
        self.actors.push(id, param);
    }

    pub fn read_actors<'a>(&'a self) -> Option<impl Iterator<Item = (WorldActorId, WorldActor)> + 'a> {
        self.actors.read_values()
    }

}
