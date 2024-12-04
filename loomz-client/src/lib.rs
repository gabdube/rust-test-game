mod store;

use loomz_shared::base_types::{RectF32, _2d::{pos, size}};
use loomz_shared::api::{Uid, WorldComponent};
use loomz_shared::{assets_err, chain_err, CommonError, CommonErrorType, LoomzApi};

#[derive(Default)]
pub struct PawnTemplate {
    pub idle: Uid,
}

#[derive(Default)]
pub struct Templates {
    pub pawn: PawnTemplate,
}

#[derive(Copy, Clone)]
pub enum GameState {
    Uninitialized,
    Gameplay,
}

pub struct LoomzClient {
    api: LoomzApi,
    state: GameState,
    templates: Templates,
}

impl LoomzClient {

    pub fn init(api: &LoomzApi) -> Result<Self, CommonError> {
        let client = LoomzClient {
            api: api.clone(),
            state: GameState::Uninitialized,
            templates: Templates::default(),
        };

        Self::init_player(api)?;
        client.init_templates()?;

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

    fn init_templates(&self) -> Result<(), CommonError> {
        let assets = self.api.assets_ref();
        let world = self.api.world();

        #[inline]
        fn parse<T: ::std::str::FromStr>(item: &jsonic::json_item::JsonItem) -> T {
            match item.as_str().and_then(|value| value.parse::<T>().ok() ) {
                Some(v) => v,
                _ => panic!("Failed to parse json value")
            }
        }

        fn parse_rect(item: &jsonic::json_item::JsonItem) -> RectF32 {
            RectF32 {
                left: parse(&item[0]),
                top: parse(&item[1]),
                right: parse(&item[2]),
                bottom: parse(&item[3]),
            }
        }

        // Pawn
        {
            let pawn_json_id = assets.json_id_by_name("pawn_sprites").ok_or_else(|| assets_err!("Failed to find json \"pawn_sprites\"") )?;
            let pawn_json_source = assets.json(pawn_json_id).unwrap();
            let pawn_json = jsonic::parse(pawn_json_source).map_err(|err| assets_err!("Failed to parse json: {:?}", err) )?;
            
            let animations = pawn_json["animations"].elements().unwrap();
            for animation in animations {
                let uid = match animation["name"].as_str() {
                    Some("idle") => &self.templates.pawn.idle,
                    _ => { continue; }
                };

                let sprite_count: usize = parse(&animation["sprite_count"]);
                let sprite_padding: u32 = parse(&animation["sprite_padding"]);
                let sprite_width: u32 = parse(&animation["sprite_width"]);
                let sprite_height: u32 = parse(&animation["sprite_height"]);

                let mut sprites = Vec::with_capacity(sprite_count);
                let sprites_data = &animation["sprites"];
                for i in 0..sprite_count {
                    sprites.push(parse_rect(&sprites_data[i]));
                }

                let animation = loomz_shared::WorldAnimation {
                    
                };

                world.create_animation(uid, animation);
            }
        }

        Ok(())
    }

    fn init_player(api: &LoomzApi) -> Result<(), CommonError> {
        let assets = api.assets_ref();
        let texture_id = assets.texture_id_by_name("pawn")
            .ok_or_else(|| assets_err!("Failed to find texture \"pawn\"") )?;

        let uid = Uid::new();
        let component = WorldComponent {
            position: pos(10.0, 10.0),
            size: size(59.0, 59.0),
            uv: RectF32 { left: 1.0, top: 0.0, right: 59.0, bottom: 59.0 },
            texture_id,
        };

        api.world().update_component(&uid, component);

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