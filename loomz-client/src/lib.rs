use loomz_shared::{LoomzApi, LoomzClientApi};
use loomz_shared::{RgbaU8, _2d::{Position, pos}};

pub struct Player {
    pub pos: Position,
    pub color: RgbaU8,
    pub visible: bool,
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
        let api = api.client_api();
        let player = Player {
            pos: pos(0.0, 0.0),
            color: RgbaU8::rgb(255, 0, 0),
            visible: false,
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

        self.api.world.update_component(WorldComponent { position: self.player.pos, color: self.player.color });

        self.api.world.update_component(WorldComponent { position: pos(128.0, 128.0), color: RgbaU8::rgb(255, 255, 255) });

        self.player.visible = true;
        self.state = GameState::Gameplay;
    }

    fn gameplay(&mut self) {

    }

}
