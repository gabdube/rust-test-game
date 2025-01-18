mod store;
mod gui;

mod animations;
use animations::{Animations, PawnAnimationType};


use std::time::Instant;
use loomz_shared::base_types::{PositionF32, PositionF64, rect, rgb};
use loomz_shared::api::WorldActorId;
use loomz_shared::inputs::InputBuffer;
use loomz_shared::{chain_err, CommonError, CommonErrorType, LoomzApi};


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
    Gameplay,
}

struct ClientTiming {
    last: Instant,
    delta_ms: f64,
}

#[repr(C)]
pub struct LoomzClient {
    api: LoomzApi,
    timing: ClientTiming,
    animations: Box<Animations>,

    player: Player,
    target_position: PositionF32,

    main_menu: gui::Gui,

    state: GameState,
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
            animations: Box::default(),

            player: Player::default(),
            target_position: PositionF32::default(),

            main_menu: gui::Gui::default(),

            state: GameState::Uninitialized,
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
        client.main_menu = reader.load();

        client.init_main_menu()?;

        Ok(client)
    }

    pub fn export(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_into_u32(self.state);
        writer.write(&self.target_position);
        writer.store(&self.player);
        writer.store(&self.main_menu);
    }

    pub fn update(&mut self) -> Result<(), CommonError> {
        self.update_timing();

        match self.state {
            GameState::Uninitialized => self.uninitialized()?,
            GameState::MainMenu => self.main_menu(),
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
        //self.init_player();
        self.init_main_menu()?;
        self.state = GameState::MainMenu;
        Ok(())
    }

    fn main_menu(&mut self) {
        let new_inputs = match self.api.read_inputs() {
            Some(inputs) => inputs,
            None => { return; }
        };

        let mut gui_updates = gui::GuiUpdates::default();

        if let Some(cursor_position) = new_inputs.cursor_position() {
            gui_updates.cursor_position = Some(cursor_position.as_f32());
        }

        if let Some(new_size) = new_inputs.screen_size() {
            gui_updates.view = Some(rect(0.0, 0.0, new_size.width, new_size.height));
        }

        self.main_menu.update(&self.api, &gui_updates);
    }

    fn gameplay(&mut self) -> Result<(), CommonError> {
        if let Some(new_input) = self.api.read_inputs() {
           self.gameplay_inputs(new_input);
        }

        self.gameplay_loop();

        Ok(())
    }

    fn gameplay_inputs(&mut self, new_input: InputBuffer) {
        let mut gui_updates = gui::GuiUpdates::default();

        if let Some(cursor_position) = new_input.cursor_position() {
            self.target_position = cursor_position.as_f32();
        }

        if let Some(new_size) = new_input.screen_size() {
            gui_updates.view = Some(rect(0.0, 0.0, new_size.width, new_size.height));
        }

        self.main_menu.update(&self.api, &gui_updates);
    }

    fn gameplay_loop(&mut self) {
        let world = self.api.world();
        let position = self.player.position;
        let target = self.target_position;

        if position.out_of_range(target, 2.0) {
            let speed = 200.0 * self.timing.delta_ms;
            let angle = f32::atan2(target.y - position.y, target.x - position.x) as f64;
            let speed_x = speed * f64::cos(angle);
            let speed_y = speed * f64::sin(angle);

            self.player.position += PositionF64 { x: speed_x, y: speed_y };
            world.update_actor_position(&self.player.id, self.player.position);

            if self.player.animation != PawnAnimationType::Walk {
                world.update_actor_animation(&self.player.id, &self.animations.warrior.walk);
                self.player.animation = PawnAnimationType::Walk;
            }

            if speed_x < 0.0 && !self.player.flip {
                self.player.flip = true;
                world.flip_actor(&self.player.id, true);
            } else if speed_x > 0.0 && self.player.flip {
                self.player.flip = false;
                world.flip_actor(&self.player.id, false);
            }
        } else {
            if self.player.animation != PawnAnimationType::Idle {
                world.update_actor_animation(&self.player.id, &self.animations.warrior.idle);
                self.player.animation = PawnAnimationType::Idle;
            }
        }
    }

    fn init_player(&mut self) {
        let start_position = PositionF32 { x: 100.0, y: 500.0 };
        let player = Player {
            id: WorldActorId::new(),
            position: start_position,
            animation: PawnAnimationType::Idle,
            flip: false,
        };

        self.api.world().create_actor(
            &player.id,
            player.position,
            &self.animations.warrior.idle,
        );

        self.player = player;
        self.target_position = start_position;
    }

    fn init_main_menu(&mut self) -> Result<(), CommonError> {
        use crate::gui::{GuiLayoutType::VBox, GuiStyleState};

        let screen_size = self.api.inputs().screen_size_value();
        let view = loomz_shared::RectF32{ 
            left: 0.0, right: screen_size.width,
            top: 0.0, bottom: screen_size.height,
        };

        self.main_menu.build_style(&self.api, |style| {
            style.root_layout(VBox);
            style.font("menu_item", GuiStyleState::Base, "bubblegum", 100.0, rgb(71, 43, 26));
            style.frame("main_menu_panel", GuiStyleState::Base, "gui", rect(0.0, 0.0, 2.0, 2.0), rgb(24, 18, 15));
        })?;

        self.main_menu.build(&self.api, &view, |gui| {
            gui.layout(VBox);
            gui.layout_item(400.0, 440.0);
            gui.frame("main_menu_panel", |gui| {
                gui.layout_item(300.0, 100.0);
                gui.label("Start", "menu_item");
                gui.label("Debug", "menu_item");
                gui.label("Exit", "menu_item");
            });
        })?;

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

impl From<u32> for GameState {
    fn from(value: u32) -> Self {
        match value {
            1 => GameState::MainMenu,
            2 => GameState::Gameplay,
            _ => GameState::Uninitialized,
        }
    }
}

impl From<GameState> for u32 {
    fn from(value: GameState) -> Self {
        match value {
            GameState::Uninitialized => 0,
            GameState::MainMenu => 1,
            GameState::Gameplay => 2,
        }
    }
}