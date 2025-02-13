mod store;
mod gui;
mod animations;
mod terrain;
mod state;

use animations::{Animations, PawnAnimationType};

use bitflags::bitflags;
use std::time::Instant;
use loomz_shared::base_types::PositionF32;
use loomz_shared::api::{WorldActorId, WorldDebugFlags};
use loomz_shared::{chain_err, CommonError, CommonErrorType, LoomzApi};

bitflags! {
    #[derive(Copy, Clone, Default, Debug)]
    pub struct GameInputFlags: u8 {
        const DRAGGING_VIEW  = 0b0001;
    }
}

#[derive(Default)]
pub struct Player {
    id: WorldActorId,
    position: PositionF32,
    animation: PawnAnimationType,
    flip: bool,
}

#[derive(Copy, Clone)]
pub enum GameState {
    Uninitialized,
    MainMenu,
    Game,
    Editor,
}

struct ClientTiming {
    last: Instant,
    delta_ms: f64,
}

#[derive(Default)]
struct DebugState {
    world: WorldDebugFlags,
}

pub struct LoomzClient {
    api: LoomzApi,
    timing: ClientTiming,
    animations: Box<Animations>,

    gui: gui::Gui,
    terrain: terrain::Terrain,
    
    state: GameState,
    input_flags: GameInputFlags,
    debug_state: DebugState
}

impl LoomzClient {

    fn build_client(api: &LoomzApi) -> Self {
        let timing = ClientTiming {
            last: Instant::now(),
            delta_ms: 0.0,
        };
        
        LoomzClient {
            api: api.clone(),
            timing,
            animations: Box::default(),

            gui: gui::Gui::default(),
            terrain: terrain::Terrain::init(),

            state: GameState::Uninitialized,
            input_flags: GameInputFlags::empty(),
            debug_state: DebugState::default(),
        }
    }

    pub fn init(api: &LoomzApi) -> Result<Self, CommonError> {
        let mut client = Self::build_client(api);
        client.animations.load(api)?;
        client.init_editor()?;
        Ok(client)
    }

    pub fn init_from_data(api: &LoomzApi, bytes: &Box<[u8]>) -> Result<Self, CommonError> {
        let mut reader = crate::store::SaveFileReader::new(&bytes)
            .map_err(|err| chain_err!(err, CommonErrorType::SaveLoad, "Failed to initialize client from stored session") )?;

        let mut client = Self::build_client(api);
        
        client.state = reader.read_from_u32();
        client.input_flags = reader.read_from_u32();
        client.debug_state = reader.load();
        client.animations = Box::new(reader.read());
        client.gui = reader.load();
        client.terrain = reader.load();

        client.init_editor()?;

        Ok(client)
    }

    pub fn export(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_into_u32(self.state);
        writer.write_into_u32(self.input_flags);
        writer.store(&self.debug_state);
        writer.write(&*self.animations);
        writer.store(&self.gui);
        writer.store(&self.terrain);
    }

    pub fn update(&mut self) -> Result<(), CommonError> {
        self.update_timing();

        match self.state {
            GameState::Uninitialized => self.uninitialized()?,
            GameState::MainMenu => self.main_menu()?,
            GameState::Game => self.gameplay()?,
            GameState::Editor => self.editor()?,
        }

        self.update_debug_state();
        self.api.client_update_finished();

        Ok(())
    }

    fn update_timing(&mut self) {
        let elapsed = self.timing.last.elapsed();
        self.timing.last = Instant::now();
        self.timing.delta_ms = elapsed.as_secs_f64();
    }

    fn update_debug_state(&mut self) {
        use loomz_shared::inputs::keys;

        if let Some(keystate) = self.api.keys_ref().read_updates() {
            let mut update_debug = false;

            if keystate.just_pressed(keys::_1) {
                self.debug_state.world.toggle(WorldDebugFlags::SHOW_MAIN_GRID);
                update_debug = true;
            }
            if keystate.just_pressed(keys::_2) {
                self.debug_state.world.toggle(WorldDebugFlags::SHOW_SUB_GRID);
                update_debug = true;
            }
            if keystate.just_pressed(keys::_3) {
                self.debug_state.world.toggle(WorldDebugFlags::SHOW_MAIN_GRID_TYPES);
                update_debug = true;
            }

            if update_debug {
                self.api.world().toggle_debug(self.debug_state.world);
            }
        }
    }

    fn uninitialized(&mut self) -> Result<(), CommonError> {
        self.init_main_menu()?;
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

        println!("Marshalled client size (bytes): {:?}", bytes.len());

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
            animation: reader.read_from_u32(),
            flip: reader.read_bool(),
        }
    }

    fn store(&self, writer: &mut loomz_shared::store::SaveFileWriterBase) {
        writer.store(&self.id);
        writer.write(&self.position);
        writer.write_into_u32(self.animation);
        writer.write_into_u32(self.flip);
    }
}

impl loomz_shared::store::StoreAndLoad for DebugState {
    fn load(reader: &mut loomz_shared::store::SaveFileReaderBase) -> Self {
        let world = WorldDebugFlags::from_bits(reader.read_u32() as u8).unwrap_or_default();
        DebugState {
            world
        }
    }

    fn store(&self, writer: &mut loomz_shared::store::SaveFileWriterBase) {
        writer.write_u32(self.world.bits() as u32);
    }
}

impl From<u32> for GameState {
    fn from(value: u32) -> Self {
        match value {
            1 => GameState::MainMenu,
            2 => GameState::Game,
            3 => GameState::Editor,
            _ => GameState::Uninitialized,
        }
    }
}

impl From<GameState> for u32 {
    fn from(value: GameState) -> Self {
        match value {
            GameState::Uninitialized => 0,
            GameState::MainMenu => 1,
            GameState::Game => 2,
            GameState::Editor => 3,
        }
    }
}

impl Into<u32> for GameInputFlags {
    fn into(self) -> u32 {
        self.bits() as u32
    }
}

impl From<u32> for GameInputFlags {
    fn from(value: u32) -> Self {
        GameInputFlags::from_bits(value as u8).unwrap_or_default()
    }
}
