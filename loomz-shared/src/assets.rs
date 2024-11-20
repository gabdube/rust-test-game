use fnv::FnvHashMap;
use std::sync::Arc;
use std::num::NonZeroU32;

const ASSET_METADATA_PATH: &'static str = "./assets/assets.csv";

#[derive(Copy, Clone, Debug)]
pub struct TextureId(pub NonZeroU32);

#[derive(Copy, Clone)]
enum AssetId {
    Texture(TextureId)
}

#[derive(Debug)]
pub struct AssetsTextureMetadata {
    pub path: String,
}

/// Static asset bundle referencing all the assets in the program
pub struct LoomzAssetsBundle {
    assets_by_name: FnvHashMap<String, AssetId>,
    textures: FnvHashMap<NonZeroU32, AssetsTextureMetadata>
}

impl LoomzAssetsBundle {

    pub fn init() -> Arc<Self> {
        Arc::new(Self::load())
    }

    pub fn texture_by_name(&self, name: &str) -> Option<TextureId> {
        match self.assets_by_name.get(name) {
            Some(AssetId::Texture(id)) => Some(*id),
            _ => None,
        }
    }

    fn load() -> Self {
        let mut bundle = LoomzAssetsBundle::default();
        let meta_csv = match ::std::fs::read_to_string(ASSET_METADATA_PATH) {
            Ok(v) => v,
            Err(e) => { panic!("Failed to load assets metadata: {e}"); }
        };

        Self::split_csv(&meta_csv, |args| {
            let id = match Self::parse_asset_id(args[1]) {
                Some(id) => id,
                None => { 
                    err1(args[1]);
                    return;
                }
            };

            match args[0] {
                "TEXTURE" => { bundle.parse_texture(id, args); },
                _ => { err2(args[0]); }
            }
        });

        println!("{:?}", bundle.textures);

        bundle
    }
    
    fn split_csv<CB: FnMut(&[&str])>(csv: &str, mut cb: CB) {
        let mut start = 0;
        let mut end = 0;
        let mut chars_iter = csv.chars();
        let mut args: [&str; 8] = [""; 8];
        while let Some(c) = chars_iter.next() {
            end += 1;
            if c == '\n' {
                let line = &csv[start..end];
                let mut args_count = 0;
                for substr in line.split(';') {
                    args[args_count] = substr;
                    args_count += 1;
                }

                if args_count > 0 {
                    cb(&args[0..(args_count-1)]);
                }

                start = end;
            }
        }
    }

    fn parse_asset_id(id: &str) -> Option<NonZeroU32> {
        let id = id.parse::<u32>().ok()?;
        NonZeroU32::new(id)
    }

    fn parse_texture(&mut self, id: NonZeroU32, args: &[&str]) {
        let name = args[2].to_string();
        self.assets_by_name.insert(name, AssetId::Texture(TextureId(id)));

        let path = format!("./assets/textures/{}", args[3]);
        self.textures.insert(id, AssetsTextureMetadata {
            path
        });
    }

}

impl Default for LoomzAssetsBundle {
    fn default() -> Self {
        LoomzAssetsBundle {
            assets_by_name: FnvHashMap::default(),
            textures: FnvHashMap::default(),
        }
    }
}

#[cold]
#[inline(never)]
fn err1(id: &str) {
    eprintln!("Failed to parse asset ID {}. ID must be a none-zero positive int.", id);
}

#[cold]
#[inline(never)]
fn err2(ty: &str) {
    eprintln!("UNKNOWN asset type {ty:?}");
}
