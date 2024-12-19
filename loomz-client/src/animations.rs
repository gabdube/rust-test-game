use loomz_shared::api::{LoomzApi, WorldAnimationId};
use loomz_shared::base_types::RectF32;
use loomz_shared::{assets_err, CommonError};

const ANIMATION_INTERVAL: f32 = 1.0 / 24.0; // 24fps

#[derive(Default)]
pub struct PawnAnimation {
    pub idle: WorldAnimationId,
    pub walk: WorldAnimationId,
    pub hammer: WorldAnimationId,
    pub axe: WorldAnimationId,
    pub idle_hold: WorldAnimationId,
    pub idle_walk: WorldAnimationId,
}

impl PawnAnimation {
    fn map_id(&self, name: &str) -> Option<&WorldAnimationId> {
        match name {
            "idle" => Some(&self.idle),
            "walk" => Some(&self.walk),
            "hammer" => Some(&self.hammer),
            "axe" => Some(&self.axe),
            "idle_hold" => Some(&self.idle_hold),
            "idle_walk" => Some(&self.idle_walk),
            _ => None,
        }
    }
}


#[derive(Default)]
pub struct Animations {
    pub pawn: PawnAnimation,
}

impl Animations {
    pub fn load(&self, api: &LoomzApi) -> Result<(), CommonError> {
        self.load_animation(api, "pawn_sprites", |name| { self.pawn.map_id(name) })?;
        Ok(())
    }

    fn load_animation<'a, F>(&'a self, api: &LoomzApi, asset_name: &str, id_map: F) -> Result<(), CommonError> 
    where 
        F: Fn(&str) -> Option<&'a WorldAnimationId>
    {
        let assets = api.assets_ref();
        let world = api.world();

        let json_source = assets.json_by_name(asset_name).ok_or_else(|| assets_err!("Failed to find json {asset_name:?}") )?;
        let json = jsonic::parse(json_source).map_err(|err| assets_err!("Failed to parse json: {err:?}") )?;
        
        let texture_asset_name = json["asset"].as_str().unwrap_or("");
        let texture_id = assets.texture_id_by_name(texture_asset_name).ok_or_else(|| assets_err!("Failed to find texture asset {texture_asset_name:?}") )?;
        
        let animations = json["animations"].elements().ok_or_else(|| assets_err!("Missing json key \"animations\"") )?;
        
        for animation in animations {
            let id = match animation["name"].as_str().and_then(|name| id_map(name) ) {
                Some(id) => id,
                None => { continue; }
            };

            let sprite_count: usize = parse(&animation["sprite_count"]);
            if sprite_count == 0 {
                continue;
            }

            let sprite_padding: u32 = parse(&animation["sprite_padding"]);
            let sprite_width: u32 = parse(&animation["sprite_width"]);
            let sprite_height: u32 = parse(&animation["sprite_height"]);

            let mut sprites = Vec::with_capacity(sprite_count);
            let sprites_data = &animation["sprites"];
            for i in 0..sprite_count {
                sprites.push(parse_rect(&sprites_data[i]));
            }

            let animation = loomz_shared::WorldAnimation {
                texture_id,
                last_frame: (sprite_count - 1) as u32,
                interval: ANIMATION_INTERVAL
            };

            world.create_animation(id, animation);
        }

        Ok(())
    }
}

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