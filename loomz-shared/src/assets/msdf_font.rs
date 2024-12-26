use crate::{assets_err, base_types::RectF32, CommonError};

#[repr(C)]
#[derive(Copy, Clone)]
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

#[derive(Copy, Clone, Default, Debug)]
pub struct ComputedGlyph {
    pub position: RectF32,
    pub texcoord: RectF32,
}

pub struct MsdfFontData {
    pub info: AtlasInfo,
    pub glyphs: Vec<AtlasGlyph>,
}

impl MsdfFontData {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CommonError> {
        let (x, _, y) = unsafe { bytes.align_to::<u32>() };
        if x.len() != 0 || y.len() != 0 {
            return Err(assets_err!(
                "Failed to parse font atlas data. Data must be aligned to 4 bytes"
            ));
        }

        let info = unsafe { *(bytes.as_ptr() as *const AtlasInfo) };

        let glyph_ptr = unsafe { bytes.as_ptr().add(size_of::<AtlasInfo>()) as *const AtlasGlyph };
        let mut glyphs = vec![Default::default(); info.glyph_max as usize];
        for i in 0..(info.glyph_count as usize) {
            let glyph: AtlasGlyph = unsafe { glyph_ptr.add(i).read() };
            glyphs[glyph.unicode as usize] = glyph;
        }

        let data = MsdfFontData { info, glyphs };

        Ok(data)
    }

    pub fn compute_glyph(&self, c: &str, scale: f32, glyph: &mut ComputedGlyph) -> f32 {
        // Multi characters glyph not supported
        let chr = match c.len() == 1 {
            true => c.chars().next().unwrap_or('?'),
            false => '?'
        };

        let atlas_height = self.info.height;
        let atlas_glyph = self.glyphs.get(chr as usize).copied().unwrap_or_default();

        let top = self.info.line_height - atlas_glyph.plane_bound[1];
        let bottom = self.info.line_height - atlas_glyph.plane_bound[3];

        glyph.position.left = scale * atlas_glyph.plane_bound[0];
        glyph.position.top = scale * top;
        glyph.position.right = scale * atlas_glyph.plane_bound[2];
        glyph.position.bottom = scale * bottom;

        glyph.texcoord.left = atlas_glyph.atlas_bound[0];
        glyph.texcoord.top = atlas_height - atlas_glyph.atlas_bound[1];
        glyph.texcoord.right = atlas_glyph.atlas_bound[2];
        glyph.texcoord.bottom = atlas_height - atlas_glyph.atlas_bound[3];

        atlas_glyph.advance * scale
    }
}
