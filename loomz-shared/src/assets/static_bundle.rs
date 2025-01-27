use fnv::FnvHashMap;
use std::sync::Arc;
use crate::{assets_err, CommonError};
use super::{ktx, msdf_font, AssetId, AssetsTextureData, AssetsMsdfFontData, ShaderData, TextureId, ShaderId, JsonId, MsdfFontId};
use super::ASSET_METADATA_PATH;

/// Static asset bundle referencing all the assets in the program
pub struct LoomzAssetsBundle {
    pub(super) assets_by_name: FnvHashMap<String, AssetId>,
    pub(super) textures: Vec<AssetsTextureData>,
    pub(super) json: Vec<String>,
    pub(super) msdf_fonts: Vec<AssetsMsdfFontData>,
    pub(super) shaders: Vec<ShaderData>,
}

impl LoomzAssetsBundle {

    #[allow(dead_code)]
    pub fn load() -> Result<Arc<Self>, CommonError> {
        let bundle = Self::load_base_bundle()?;
        Ok(Arc::new(bundle))
    }

    pub fn texture_id_by_name(&self, name: &str) -> Option<TextureId> {
        match self.assets_by_name.get(name) {
            Some(AssetId::Texture(id)) => Some(*id),
            _ => None,
        }
    }

    pub fn texture<'a>(&'a self, id: TextureId) -> Option<&'a AssetsTextureData> {
        self.textures.get(id.0 as usize)
    }

    pub fn json_id_by_name(&self, name: &str) -> Option<JsonId> {
        match self.assets_by_name.get(name) {
            Some(AssetId::Json(id)) => Some(*id),
            _ => None
        }
    }

    pub fn json_by_name<'a>(&'a self, name: &str) -> Option<&'a String> {
        self.json_id_by_name(name)
            .and_then(|id| self.json.get(id.0 as usize) )
    }

    pub fn font_id_by_name(&self, name: &str) -> Option<MsdfFontId> {
        match self.assets_by_name.get(name) {
            Some(AssetId::MsdfFont(id)) => Some(*id),
            _ => None
        }
    }

    pub fn font<'a>(&'a self, id: MsdfFontId) -> Option<&'a AssetsMsdfFontData> {
        self.msdf_fonts.get(id.0 as usize)
    }

    pub fn default_font_id(&self) -> Option<MsdfFontId> {
        if self.msdf_fonts.len() == 0 {
            None
        } else {
            Some(MsdfFontId(0))
        }
    }

    pub fn shader_id_by_name(&self, name: &str) -> Option<ShaderId> {
        match self.assets_by_name.get(name) {
            Some(AssetId::ShaderId(id)) => Some(*id),
            _ => None
        }
    }

    pub fn shader(&self, id: ShaderId) -> Option<&ShaderData> {
        self.shaders.get(id.0 as usize)
    }

    #[allow(dead_code)]
    pub fn changed_assets(&self) -> Option<Vec<AssetId>> {
        None
    }

    pub(super) fn load_base_bundle() -> Result<Self, CommonError> {
        let mut bundle = LoomzAssetsBundle::default();
        let meta_csv = Self::load_asset_metadata()?;

        let mut error: Option<CommonError> = None;

        Self::split_csv(&meta_csv, |args| {
            if let Err(e1) = bundle.parse_asset(args) {
                if error.is_none() {
                    error = Some(e1);
                } else {
                    let e2 = error.as_mut().unwrap();
                    e2.merge(e1);
                }
            };
        });

        if let Some(err) = error {
            return Err(err);
        }

        Ok(bundle)
    }

    pub(super) fn load_asset_metadata() -> Result<String, CommonError> {
        return ::std::fs::read_to_string(ASSET_METADATA_PATH)
            .map_err(|err| assets_err!("Failed to load assets metadata: {err}") );
    }

    pub(super) fn split_csv<CB: FnMut(&[&str])>(csv: &str, mut cb: CB) {
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

                if args_count > 1 {
                    cb(&args[0..(args_count-1)]);
                }

                start = end;
            }
        }
    }

    fn parse_asset(&mut self, args: &[&str]) -> Result<(), CommonError> {
        match args[0] {
            "TEXTURE" => self.parse_texture(args),
            "JSON" => self.parse_json(args),
            "MSDF_FONT" => self.parse_msdf_font(args),
            "SHADER" => self.parse_shader(args),
            _ => Err(assets_err!("Unknown asset type {:?}", args[0]))
        }
    }

    fn parse_texture(&mut self, args: &[&str]) -> Result<(), CommonError> {
        let path = format!("./assets/textures/{}", args[2]);
        let data = ktx::KtxFile::open(&path)?;

        let name = args[1].to_string();
        let id = TextureId(self.textures.len() as u32);
        self.assets_by_name.insert(name, AssetId::Texture(id));
        self.textures.push(AssetsTextureData {
            data,
        });

        Ok(())
    }

    fn parse_json(&mut self, args: &[&str]) -> Result<(), CommonError> {
        let path = format!("./assets/{}", args[2]);
        let src = ::std::fs::read_to_string(path)
            .map_err(|err|  assets_err!("Failed to open json file {:?}", err) )?;

        let name = args[1].to_string();
        let id = JsonId(self.json.len() as u32);

        self.assets_by_name.insert(name, AssetId::Json(id));
        self.json.push(src);

        Ok(())
    }

    fn parse_msdf_font(&mut self, args: &[&str]) -> Result<(), CommonError> {
        let name = args[1].to_string();
        let id = MsdfFontId(self.msdf_fonts.len() as u32);
        self.assets_by_name.insert(name, AssetId::MsdfFont(id));
        
        let (image_info, image_data) = {
            let image_path = format!("./assets/fonts/{}", args[2]);
            let src = ::std::fs::File::open(&image_path)
                .map_err(|err| assets_err!("Failed to open {:?} {:?}", image_path, err) )?;

            // Maybe we could move the decoding on-use to save memory
            // see `upload_font_image_memory`
            let decoder = png::Decoder::new(src);
            let mut reader = decoder.read_info().unwrap();
            let mut image_data = vec![0; reader.output_buffer_size()];
            let image_info = reader.next_frame(&mut image_data).unwrap();

            (image_info, image_data)
        };

        let font_data = {
            let data_path = format!("./assets/fonts/{}", args[3]);
            let src = ::std::fs::read(&data_path)
                .map_err(|err| assets_err!("Failed to open {:?} {:?}", data_path, err) )?;

            msdf_font::MsdfFontData::from_bytes(&src)?
        };
        
        self.msdf_fonts.push(AssetsMsdfFontData {
            image_info,
            image_data: image_data.into_boxed_slice(),
            font_data,
        });

        Ok(())
    }

    fn parse_shader(&mut self, args: &[&str]) -> Result<(), CommonError> {
        let name = args[1].to_string();
        let id = ShaderId(self.shaders.len() as u32);
        self.assets_by_name.insert(name, AssetId::ShaderId(id));

        let vert_path = format!("./assets/shaders/{}", &args[2]);
        let vert = ::std::fs::read(&vert_path)
            .map_err(|err| assets_err!("Failed to open {:?} {:?}", vert_path, err) )?;
        
        let frag_path = format!("./assets/shaders/{}", &args[3]);
        let frag = ::std::fs::read(&frag_path)
            .map_err(|err| assets_err!("Failed to open {:?} {:?}", frag_path, err) )?;

        self.shaders.push(ShaderData { vert, frag });

        Ok(())
    }


}

impl Default for LoomzAssetsBundle {
    fn default() -> Self {
        LoomzAssetsBundle {
            assets_by_name: FnvHashMap::default(),
            textures: Vec::with_capacity(8),
            json: Vec::with_capacity(8),
            msdf_fonts: Vec::with_capacity(8),
            shaders: Vec::with_capacity(8),
        }
    }
}
