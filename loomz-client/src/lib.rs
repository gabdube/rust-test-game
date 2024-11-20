use loomz_shared::{LoomzApi, LoomzClientApi, LoomzAssetsBundle, TextureId};
use loomz_shared::{RgbaU8, _2d::{Position, pos}};
use std::sync::Arc;

pub struct Player {
    pub pos: Position,
    pub color: RgbaU8,
    pub texture: TextureId,
}

pub enum GameState {
    Uninitialized,
    Gameplay,
}

pub struct LoomzClient {
    api: LoomzClientApi,
    state: GameState,
    player: Player
}

impl LoomzClient {

    pub fn init(api: &mut LoomzApi) -> Self {
        let assets = api.assets();
        let api = api.client_api();

        let texture = assets.texture_by_name("creatura").unwrap();
        let player = Player {
            pos: pos(0.0, 0.0),
            color: RgbaU8::rgb(255, 0, 0),
            texture,
        };

        LoomzClient {
            api,
            state: GameState::Uninitialized,
            player,
        }
    }

    pub fn update(&mut self) {
        match self.state {
            GameState::Uninitialized => self.uninitialized(),
            GameState::Gameplay => self.gameplay(),
        }
    }

    fn uninitialized(&mut self) {
        use loomz_shared::api::WorldComponent;
        self.api.world.update_component(WorldComponent { position: self.player.pos, color: self.player.color, texture: self.player.texture });
        self.state = GameState::Gameplay;
    }

    fn gameplay(&mut self) {

    }

}
