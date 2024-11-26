mod store;

use loomz_shared::_2d::{pos, size};
use loomz_shared::api::WorldComponent;
use loomz_shared::{assets_err, chain_err, CommonError, CommonErrorType, LoomzApi};

pub struct Player {
    pub component: WorldComponent,
}

pub enum GameState {
    Uninitialized,
    Gameplay,
}

pub struct LoomzClient {
    api: LoomzApi,
    state: GameState,
    player: Player
}

impl LoomzClient {

    pub fn init(api: &LoomzApi) -> Result<Self, CommonError> {
        let assets = api.assets();
        let (texture_id, texture) = assets.texture_id_by_name("creatura")
            .and_then(|id| assets.texture(id).map(|tex| (id, tex) ) )
            .ok_or_else(|| assets_err!("Failed to find texture \"creatura\"") )?;

        let texture_extent = texture.data.extent();

        let player = Player {
            component: WorldComponent {
                position: pos(0.0, 0.0),
                size: size(texture_extent.width as f32, texture_extent.height as f32),
                texture_id,
            }
        };

        let client = LoomzClient {
            api: api.clone(),
            state: GameState::Uninitialized,
            player,
        };

        Ok(client)
    }

    pub fn init_from_data(api: &LoomzApi, bytes: &Box<[u8]>) -> Result<Self, CommonError> {
        let reader = crate::store::reader::SaveFileReader::new(&bytes)
            .map_err(|err| chain_err!(err, CommonErrorType::SaveLoad, "Failed to initialize client from stored session") )?;

        let mut client = Self::init(api)?;

        client.player.component.position = pos(500.0, 0.0);

        Ok(client)
    }

    pub fn update(&mut self) -> Result<(), CommonError> {
        match self.state {
            GameState::Uninitialized => self.uninitialized(),
            GameState::Gameplay => self.gameplay(),
        }

        Ok(())
    }

    fn uninitialized(&mut self) {
        self.api.world().update_component(self.player.component);
        self.state = GameState::Gameplay;
    }

    fn gameplay(&mut self) {
    }

}


//
// Hot reloading interface
//
#[cfg(feature="hot-reload")]
mod hot {
    use std::sync::Mutex;
    use loomz_shared::{undefined_err, LoomzApi, CommonError};
    use super::LoomzClient;

    static LAST_ERROR: Mutex<Option<CommonError>> = Mutex::new(None);
    static CLIENT: Mutex<Option<LoomzClient>> = Mutex::new(None);
    
    #[no_mangle]
    pub extern fn init_client(api: &LoomzApi) {
        let client = match LoomzClient::init(api) {
            Ok(client) => client,
            Err(err) => {
                set_last_error(err);
                return;
            }
        };

        let mut global_client = CLIENT.lock().unwrap();
        *global_client = Some(client);
    }
    
    #[no_mangle]
    pub extern fn update_client() {
        let mut client_guard = CLIENT.lock().unwrap();
        let client = match client_guard.as_mut() {
            Some(client) => client,
            None => {
                set_last_error(undefined_err!("Client was not initialized"));
                return;
            }
        };

        if let Err(e) = client.update() {
            set_last_error(e);
        }
    }
    
    #[no_mangle]
    pub extern fn export_client(session_size: &mut usize, session_bytes: &mut Option<*mut u8>) {
        let mut client_guard = CLIENT.lock().unwrap();
        let client = match client_guard.as_mut() {
            Some(client) => client,
            None => {
                set_last_error(undefined_err!("Client was not initialized"));
                return;
            }
        };

        let mut writer = crate::store::writer::SaveFileWriter::new();
        let bytes = writer.finalize();

        *session_size = bytes.len();
        *session_bytes = Some(bytes.leak().as_mut_ptr());
    }

    #[no_mangle]
    pub extern fn import_client(api: &LoomzApi, bytes: &Box<[u8]>) {
        let client = match LoomzClient::init_from_data(api, bytes) {
            Ok(client) => client,
            Err(err) => {
                set_last_error(err);
                return;
            }
        };

        let mut global_client = CLIENT.lock().unwrap();
        *global_client = Some(client);
    }

    #[no_mangle]
    pub extern fn last_error(error_out: &mut Option<CommonError>) {
        let mut last_err = LAST_ERROR.lock().unwrap();
        *error_out = last_err.take();
    }
    
    fn set_last_error(err: CommonError) {
        let mut last_err = LAST_ERROR.lock().unwrap();
        *last_err = Some(err);
    }
}
