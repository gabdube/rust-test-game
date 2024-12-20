mod store;

mod animations;
use animations::Animations;

use std::time::Instant;
use loomz_shared::{_2d::Position, base_types::_2d::pos};
use loomz_shared::api::{WorldActorId, WorldActor};
use loomz_shared::{chain_err, CommonError, CommonErrorType, LoomzApi};

#[derive(Default)]
pub struct Player {
    id: WorldActorId,
    position: Position<f32>,
}

#[derive(Copy, Clone)]
pub enum GameState {
    Uninitialized,
    Gameplay,
}

struct ClientTiming {
    last: Instant,
    delta_ms: f64,
}

pub struct LoomzClient {
    api: LoomzApi,
    timing: ClientTiming,
    animations: Animations,

    state: GameState,

    player: Player,
    target_position: Position<f32>,
}

impl LoomzClient {

    pub fn init(api: &LoomzApi) -> Result<Self, CommonError> {
        let timing = ClientTiming {
            last: Instant::now(),
            delta_ms: 0.0,
        };
        
        let client = LoomzClient {
            api: api.clone(),
            timing,

            state: GameState::Uninitialized,
            animations: Animations::default(),

            player: Player::default(),
            target_position: Position::default(),
        };

        client.animations.load(api)?;

        Ok(client)
    }

    pub fn init_from_data(api: &LoomzApi, bytes: &Box<[u8]>) -> Result<Self, CommonError> {
        let mut reader = crate::store::SaveFileReader::new(&bytes)
            .map_err(|err| chain_err!(err, CommonErrorType::SaveLoad, "Failed to initialize client from stored session") )?;

        let mut client = Self::init(api)?;
        client.state = reader.read_from_u32();
        client.target_position = reader.read();
        client.player = reader.load();

        Ok(client)
    }

    pub fn export(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_into_u32(self.state);
        writer.write(&self.target_position);
        writer.store(&self.player);
    }

    pub fn update(&mut self) -> Result<(), CommonError> {
        self.update_timing();

        match self.state {
            GameState::Uninitialized => self.uninitialized()?,
            GameState::Gameplay => self.gameplay()?,
        }

        Ok(())
    }

    fn update_timing(&mut self) {
        let elapsed = self.timing.last.elapsed();
        self.timing.last = Instant::now();
        self.timing.delta_ms = elapsed.as_secs_f64();
    }

    fn uninitialized(&mut self) -> Result<(), CommonError> {
        self.init_player()?;
        self.state = GameState::Gameplay;
        Ok(())
    }

    fn gameplay(&mut self) -> Result<(), CommonError> {
        if let Some(new_input) = self.api.read_inputs() {
            if let Some(cursor_position) = new_input.cursor_position() {
                self.on_cursor_moved(cursor_position);
            }
        }

        let world = self.api.world();
        let position = self.player.position;
        let target = self.target_position;

        if position.out_of_range(target, 2.0) {
            let speed = 200.0 * self.timing.delta_ms;
            let angle = f32::atan2(target.y - position.y, target.x - position.x) as f64;
            self.player.position += pos(speed * f64::cos(angle), speed * f64::sin(angle));
            world.update_actor(&self.player.id, WorldActor::Position(self.player.position));
        }

        Ok(())
    }

    fn on_cursor_moved(&mut self, position: Position<f64>) {
        self.target_position = position.as_f32();
    }

    fn init_player(&mut self) -> Result<(), CommonError> {
        let start_position = pos(100.0, 100.0);
        let player = Player {
            id: WorldActorId::new(),
            position: start_position,
        };

        self.api.world().create_actor(
            &player.id,
            player.position,
            &self.animations.pawn.walk,
        );

        self.player = player;
        self.target_position = start_position;

        Ok(())
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

impl loomz_shared::store::StoreAndLoad for Player {
    fn load(reader: &mut loomz_shared::store::SaveFileReaderBase) -> Self {
        Player {
            id: reader.load(),
            position: reader.read(),
        }
    }

    fn store(&self, writer: &mut loomz_shared::store::SaveFileWriterBase) {
        writer.store(&self.id);
        writer.write(&self.position);
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