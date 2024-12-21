use crate::{CommonError, assets_err};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AtlasInfo {
    pub size: f32,
    pub width: f32,
    pub height: f32,
    pub line_height: f32,
    pub ascender: f32,
    pub descender: f32,
    pub glyph_count: u32,
    pub glyph_max: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct AtlasGlyph {
    pub unicode: u32,
    pub advance: f32,
    pub atlas_bound: [f32; 4],
    pub plane_bound: [f32; 4],
}

pub struct MsdfFontData {
    pub info: AtlasInfo,
    pub glyphs: Vec<AtlasGlyph>,
}

impl MsdfFontData {

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CommonError> {
        let (x, _, y) = unsafe { bytes.align_to::<u32>() };
        if x.len() != 0 || y.len() != 0 {
            return Err(assets_err!("Failed to parse font atlas data. Data must be aligned to 4 bytes"));
        }

        let info = unsafe { *(bytes.as_ptr() as *const AtlasInfo) };
        
        let glyph_ptr = unsafe { bytes.as_ptr().add(size_of::<AtlasInfo>()) as *const AtlasGlyph };
        let mut glyphs = vec![Default::default(); info.glyph_max as usize];
        for i in 0..(info.glyph_count as usize) {
            let glyph: AtlasGlyph = unsafe { glyph_ptr.add(i).read() };
            glyphs[glyph.unicode as usize] = glyph;
        }

        let data = MsdfFontData {
            info,
            glyphs,
        };

        Ok(data)
    }

}

