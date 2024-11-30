mod store;

use loomz_shared::base_types::_2d::{pos, size};
use loomz_shared::api::{Uid, WorldComponent};
use loomz_shared::{assets_err, chain_err, CommonError, CommonErrorType, LoomzApi};

pub struct Player {
    pub id: Uid,
    pub component: WorldComponent,
}

#[derive(Copy, Clone)]
pub enum GameState {
    Uninitialized,
    Gameplay,
}

pub struct LoomzClient {
    api: LoomzApi,
    state: GameState,
}

impl LoomzClient {

    pub fn init(api: &LoomzApi) -> Result<Self, CommonError> {
        let client = LoomzClient {
            api: api.clone(),
            state: GameState::Uninitialized,
        };

        Ok(client)
    }

    pub fn init_from_data(api: &LoomzApi, bytes: &Box<[u8]>) -> Result<Self, CommonError> {
        let mut reader = crate::store::SaveFileReader::new(&bytes)
            .map_err(|err| chain_err!(err, CommonErrorType::SaveLoad, "Failed to initialize client from stored session") )?;

        let mut client = Self::init(api)?;
        client.state = reader.read_from_u32();

        Ok(client)
    }

    pub fn export(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_into_u32(self.state);
    }

    pub fn update(&mut self) -> Result<(), CommonError> {
        match self.state {
            GameState::Uninitialized => self.uninitialized(),
            GameState::Gameplay => self.gameplay(),
        }

        Ok(())
    }

    fn uninitialized(&mut self) {
        self.state = GameState::Gameplay;
    }

    fn gameplay(&mut self) {
        // if let Some(mut input) = self.api.new_inputs() {
        //     let world = self.api.world();

        //     if let Some(screen_size) = input.screen_size() {
        //         let extent = self.api.assets_ref().texture(self.player.component.texture_id).unwrap().data.extent();
        //         self.player.component.position.x = (screen_size.width - (extent.width as f32)) / 2.0;
        //         self.player.component.position.y = (screen_size.height - (extent.height as f32)) / 2.0;
        //         world.update_component(&self.player.id, self.player.component);
        //     }
        // }
    }

    // fn init_player(api: &LoomzApi) -> Result<Player, CommonError> {
    //     let assets = api.assets_ref();
    //     let (texture_id, texture) = assets.texture_id_by_name("creatura")
    //         .and_then(|id| assets.texture(id).map(|tex| (id, tex) ) )
    //         .ok_or_else(|| assets_err!("Failed to find texture \"creatura\"") )?;

    //     let texture_extent = texture.data.extent();

    //     let player = Player {
    //         id: Uid::new(),
    //         component: WorldComponent {
    //             position: pos(0.0, 0.0),
    //             size: size(texture_extent.width as f32, texture_extent.height as f32),
    //             texture_id,
    //         }
    //     };

    //     Ok(player)
    // }

}

impl loomz_shared::store::StoreAndLoad for Player {
    fn load(reader: &mut loomz_shared::store::SaveFileReaderBase) -> Self {
        Player {
            id: reader.load(),
            component: reader.read(),
        }
    }

    fn store(&self, writer: &mut loomz_shared::store::SaveFileWriterBase) {
        writer.store(&self.id);
        writer.write(&self.component);
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
        let mut writer = crate::store::SaveFileWriter::new();
        
        let mut client_guard = CLIENT.lock().unwrap();
        match client_guard.as_mut() {
            Some(client) => {
                client.export(&mut writer);
            },
            None => {
                set_last_error(undefined_err!("Client was not initialized"));
                return;
            }
        };

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


impl From<u32> for GameState {
    fn from(value: u32) -> Self {
        match value {
            1 => GameState::Gameplay,
            _ => GameState::Uninitialized,
        }
    }
}

impl From<GameState> for u32 {
    fn from(value: GameState) -> Self {
        match value {
            GameState::Uninitialized => 0,
            GameState::Gameplay => 1,
        }
    }
}