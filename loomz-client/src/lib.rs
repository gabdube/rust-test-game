use loomz_shared::_2d::{pos, size};
use loomz_shared::api::WorldComponent;
use loomz_shared::{assets_err, LoomzApi, LoomzClientApi, CommonError};

pub struct Player {
    pub component: WorldComponent,
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

    pub fn init(api: &mut LoomzApi) -> Result<Self, CommonError> {
        let assets = api.assets();
        let api = api.client_api();

        let texture_id = assets.texture_id_by_name("creatura")
            .ok_or_else(|| assets_err!("Failed to find texture \"creatura\"") )?;

        let texture = assets.texture(texture_id).unwrap();
        let texture_extent = texture.data.extent();

        let player = Player {
            component: WorldComponent {
                position: pos(0.0, 0.0),
                size: size(texture_extent.width as f32, texture_extent.height as f32),
                texture_id,
            }
        };

        let client = LoomzClient {
            api,
            state: GameState::Uninitialized,
            player,
        };

        Ok(client)
    }

    pub fn update(&mut self) {
        match self.state {
            GameState::Uninitialized => self.uninitialized(),
            GameState::Gameplay => self.gameplay(),
        }
    }

    fn uninitialized(&mut self) {
        self.api.world.update_component(self.player.component);
        self.state = GameState::Gameplay;
    }

    fn gameplay(&mut self) {

    }

}
