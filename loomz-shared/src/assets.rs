pub mod ktx;

use std::sync::Arc;
use std::num::NonZeroU32;
use fnv::FnvHashMap;
use crate::{assets_err, CommonError};

const ASSET_METADATA_PATH: &'static str = "./assets/assets.csv";

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextureId(pub NonZeroU32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct JsonId(pub NonZeroU32);

#[derive(Copy, Clone)]
enum AssetId {
    Texture(TextureId),
    Json(JsonId)
}

pub struct AssetsTextureData {
    pub path: String,
    pub data: ktx::KtxFile,
}

/// Static asset bundle referencing all the assets in the program
pub struct LoomzAssetsBundle {
    assets_by_name: FnvHashMap<String, AssetId>,
    textures: FnvHashMap<TextureId, AssetsTextureData>,
    json: FnvHashMap<JsonId, String>,
}

impl LoomzAssetsBundle {

    pub fn load() -> Result<Arc<Self>, CommonError> {
        let mut bundle = LoomzAssetsBundle::default();
        let meta_csv = match ::std::fs::read_to_string(ASSET_METADATA_PATH) {
            Ok(v) => v,
            Err(e) => { return Err(assets_err!("Failed to load assets metadata: {e}")); }
        };

        let mut error: Option<CommonError> = None;

        Self::split_csv(&meta_csv, |args| {
            let result = Self::parse_asset_id(args[1])
                .and_then(|id| { bundle.parse_asset(id, args) });
    
            if let Err(e1) = result {
                if error.is_none() {
                    error = Some(e1);
                } else {
                    let e2 = error.as_mut().unwrap();
                    e2.merge(e1);
                }
            };
        });

        if let Some(err) = error {
            Err(err)
        } else {
            Ok(Arc::new(bundle))
        }
    }

    pub fn texture_id_by_name(&self, name: &str) -> Option<TextureId> {
        match self.assets_by_name.get(name) {
            Some(AssetId::Texture(id)) => Some(*id),
            _ => None,
        }
    }

    pub fn texture<'a>(&'a self, id: TextureId) -> Option<&'a AssetsTextureData> {
        self.textures.get(&id)
    }

    pub fn json_id_by_name(&self, name: &str) -> Option<JsonId> {
        match self.assets_by_name.get(name) {
            Some(AssetId::Json(id)) => Some(*id),
            _ => None
        }
    }

    pub fn json<'a>(&'a self, id: JsonId) -> Option<&'a String> {
        self.json.get(&id)
    }

    pub fn json_by_name<'a>(&'a self, name: &str) -> Option<&'a String> {
        self.json_id_by_name(name)
            .and_then(|id| self.json.get(&id) )
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

    fn parse_asset_id(id: &str) -> Result<NonZeroU32, CommonError> {
        let id = id.parse::<u32>()
            .map_err(|_| assets_err!("Failed to parse asset ID {:?}. Id must be a positive int.", id) )?;

        NonZeroU32::new(id).ok_or_else(|| assets_err!("Failed to parse asset ID {:?}. Id must be a positive int.", id) )
    }

    fn parse_asset(&mut self, id: NonZeroU32, args: &[&str]) -> Result<(), CommonError> {
        match args[0] {
            "TEXTURE" => self.parse_texture(id, args),
            "JSON" => self.parse_json(id, args),
            _ => Err(assets_err!("Unknown asset type {:?}", args[0]))
        }
    }

    fn parse_texture(&mut self, id: NonZeroU32, args: &[&str]) -> Result<(), CommonError> {
        let path = format!("./assets/textures/{}", args[3]);
        let data = ktx::KtxFile::open(&path)?;

        let name = args[2].to_string();
        self.assets_by_name.insert(name, AssetId::Texture(TextureId(id)));
        self.textures.insert(TextureId(id), AssetsTextureData {
            path,
            data,
        });

        Ok(())
    }

    fn parse_json(&mut self, id: NonZeroU32, args: &[&str]) -> Result<(), CommonError> {
        let path = format!("./assets/{}", args[3]);
        let src = ::std::fs::read_to_string(path)
            .map_err(|err|  assets_err!("Failed to open json file {:?}", err) )?;

        let name = args[2].to_string();
        self.assets_by_name.insert(name, AssetId::Json(JsonId(id)));
        self.json.insert(JsonId(id), src);

        Ok(())
    }

}

impl Default for LoomzAssetsBundle {
    fn default() -> Self {
        LoomzAssetsBundle {
            assets_by_name: FnvHashMap::default(),
            textures: FnvHashMap::default(),
            json: FnvHashMap::default(),
        }
    }
}
