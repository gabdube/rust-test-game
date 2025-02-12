use loomz_shared::api::{LoomzApi, WorldAnimationId};
use loomz_shared::{assets_err, CommonError};

#[derive(Default, Copy, Clone, PartialEq)]
pub enum PawnAnimationType {
    #[default]
    Idle,
    Walk,
    Hammer,
    Axe,
    IdleHold,
    IdleWalk
}

#[derive(Default, Copy, Clone)]
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

#[derive(Default, Copy, Clone)]
pub struct WarriorAnimation {
    pub idle: WorldAnimationId,
    pub walk: WorldAnimationId,
    pub strike_h1: WorldAnimationId,
    pub strike_h2: WorldAnimationId,
    pub strike_b1: WorldAnimationId,
    pub strike_b2: WorldAnimationId,
    pub strike_t1: WorldAnimationId,
    pub strike_t2: WorldAnimationId,
}

impl WarriorAnimation {
    fn map_id(&self, name: &str) -> Option<&WorldAnimationId> {
        match name {
            "idle" => Some(&self.idle),
            "walk" => Some(&self.walk),
            "strike-horz-1" => Some(&self.strike_h1),
            "strike-horz-2" => Some(&self.strike_h2),
            "strike-bottom-1" => Some(&self.strike_b1),
            "strike-bottom-2" => Some(&self.strike_b2),
            "strike-top-1" => Some(&self.strike_t1),
            "strike-top-2" => Some(&self.strike_t2),
            _ => None,
        }
    }
}


#[derive(Default, Copy, Clone)]
pub struct Animations {
    pub pawn: PawnAnimation,
    pub warrior: WarriorAnimation,
}

impl Animations {
    pub fn load(&self, api: &LoomzApi) -> Result<(), CommonError> {
        self.load_animation(api, "pawn_sprites", |name| { self.pawn.map_id(name) })?;
        self.load_animation(api, "warrior_sprites", |name| { self.warrior.map_id(name) })?;
        Ok(())
    }

    fn load_animation<'a, F>(&'a self, api: &LoomzApi, asset_name: &str, id_map: F) -> Result<(), CommonError> 
    where 
        F: Fn(&str) -> Option<&'a WorldAnimationId>
    {
        let assets = api.assets_ref();
        let world = api.world();

        let json_source = assets.json_by_name(asset_name).ok_or_else(|| assets_err!("Failed to find json {asset_name:?}") )?;
        let json: serde_json::Value = serde_json::from_str(&json_source).map_err(|err| assets_err!("Failed to parse json: {err:?}") )?;

        let texture_asset_name = json["asset"].as_str().unwrap_or("");
        let texture_id = assets.texture_id_by_name(texture_asset_name).ok_or_else(|| assets_err!("Failed to find texture asset {texture_asset_name:?}") )?;

        let animations = json["animations"].as_array().ok_or_else(|| assets_err!("Missing json key \"animations\"") )?;

        for animation in animations {
            let id = match animation["name"].as_str().and_then(|name| id_map(name) ) {
                Some(id) => id,
                None => { continue; }
            };

            let sprite_count: u32 = parse_u32(&animation["count"]);
            if sprite_count == 0 {
                continue;
            }

            let padding: f32 = parse_f32(&animation["padding"]);
            let x: f32 = parse_f32(&animation["x"]);
            let y: f32 = parse_f32(&animation["y"]);
            let sprite_width: f32 = parse_f32(&animation["width"]);
            let sprite_height: f32 = parse_f32(&animation["height"]);

            let animation = loomz_shared::WorldAnimation {
                texture_id,
                padding,
                x, y,
                sprite_width, sprite_height,
                last_frame: (sprite_count - 1) as u8,
            };

            world.create_animation(id, animation);
        }

        Ok(())
    }
}

fn parse_u32(item: &serde_json::Value) -> u32 {
    item.as_u64().map(|v| v as u32).unwrap_or(0)
}

fn parse_f32(item: &serde_json::Value) -> f32 {
    item.as_f64().map(|v| v as f32).unwrap_or(0.0)
}

impl From<u32> for PawnAnimationType {
    fn from(value: u32) -> Self {
        match value {
            1 => PawnAnimationType::Walk,
            2 => PawnAnimationType::Hammer,
            3 => PawnAnimationType::Axe,
            4 => PawnAnimationType::IdleHold,
            5 => PawnAnimationType::IdleWalk,
            _ => PawnAnimationType::Idle,
        }
    }
}

impl From<PawnAnimationType> for u32 {
    fn from(value: PawnAnimationType) -> Self {
        match value {
            PawnAnimationType::Idle => 0,
            PawnAnimationType::Walk => 1,
            PawnAnimationType::Hammer => 2,
            PawnAnimationType::Axe => 3,
            PawnAnimationType::IdleHold => 4,
            PawnAnimationType::IdleWalk => 5
        }
    }
}
