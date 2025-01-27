pub mod ktx;
pub mod msdf_font;

mod static_bundle;

#[cfg(feature="reload-assets")]
mod dynamic_bundle;

#[cfg(not(feature="reload-assets"))]
pub use static_bundle::*;

#[cfg(feature="reload-assets")]
pub use dynamic_bundle::*;

const ASSET_METADATA_PATH: &'static str = "./assets/assets.csv";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TextureId(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct JsonId(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MsdfFontId(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ShaderId(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AssetId {
    Texture(TextureId),
    Json(JsonId),
    MsdfFont(MsdfFontId),
    ShaderId(ShaderId),
}

#[derive(Clone)]
pub struct AssetsTextureData {
    pub data: ktx::KtxFile,
}

#[derive(Clone)]
pub struct ShaderData {
    pub vert: Vec<u8>,
    pub frag: Vec<u8>
}

pub struct AssetsMsdfFontData {
    pub image_info: png::OutputInfo,
    pub image_data: Box<[u8]>,
    pub font_data: msdf_font::MsdfFontData,
}

impl Clone for AssetsMsdfFontData {
    fn clone(&self) -> Self {
        AssetsMsdfFontData {
            image_info: png::OutputInfo {
                width: self.image_info.width,
                height: self.image_info.height,
                color_type: self.image_info.color_type,
                bit_depth: self.image_info.bit_depth,
                line_size: self.image_info.line_size,
            },
            image_data: self.image_data.clone(),
            font_data: self.font_data.clone(),
        }
    }
}
