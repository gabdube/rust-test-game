//! Ktx file reader

use std::{ptr, mem, slice};
use crate::{assets_err, CommonError};

const KTX_ID: [u8; 12] = [0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A];


#[repr(C)]
#[derive(Copy, Clone)]
struct LevelIndex {
    byte_offset: u64,
    byte_length: u64,
    uncompressed_byte_length: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct KtxIndex {
    pub dfd_bytes_offset: u32,
    pub dfd_byte_length: u32,
    pub kvd_byte_offset: u32,
    pub kvd_byte_length: u32,
    pub sgd_byte_offset: u64,
    pub sgd_byte_length: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct KtxHeader {
    pub id: [u8; 12],
    pub format: vk::Format,
    pub type_size: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub layer_count: u32,
    pub face_count: u32,
    pub level_count: u32,
    pub super_compression_scheme: u32,
    pub index: KtxIndex,
}


/// A KTX file raw content
pub struct KtxFile {
    pub(crate) data: Box<[u8]>,
}

impl KtxFile {

    pub fn open(path: &str) -> Result<Self, CommonError> {
        let data = ::std::fs::read(path)
            .map(|data| data.into_boxed_slice() )
            .map_err(|err| assets_err!("Failed to open {path:?}: {err}") )?;

        assert!(data.as_ptr() as usize % 4 == 0, "Data must be aligned to 4 bytes");

        Self::check_header(&data)?;
        
        let ktx = KtxFile {
            data,
        };

        Ok(ktx)
    }

    /// Returns the texture extent
    pub fn extent(&self) -> vk::Extent3D {
        let header = self.header();
        vk::Extent3D {
            width: header.width,
            height: header.height.max(1),
            depth: header.depth.max(1),
        }
    }

    /// Return the format of the texture
    pub fn format(&self) -> vk::Format {
        self.header().format
    }

    pub fn image_type(&self) -> vk::ImageType {
        let header = self.header();
        if header.height == 0 {
            vk::ImageType::TYPE_1D
        } else if header.depth == 0 {
            vk::ImageType::TYPE_2D
        } else {
            vk::ImageType::TYPE_3D
        }
    }

    pub fn view_type(&self) -> vk::ImageViewType {
        let header = self.header();
        let is_array = self.array_layers() > 1;
        let is_cube = self.is_cubemap();

        if header.height == 0 {
            match is_array {
                true => vk::ImageViewType::TYPE_1D_ARRAY,
                false => vk::ImageViewType::TYPE_1D
            }
        } else if header.depth == 0 {
            if is_cube {
                vk::ImageViewType::CUBE
            } else {
                match is_array {
                    true => vk::ImageViewType::TYPE_2D_ARRAY,
                    false => vk::ImageViewType::TYPE_2D
                }
            }
        } else {
            vk::ImageViewType::TYPE_3D
        }
    }

    pub fn mip_levels(&self) -> u32 {
        self.header().level_count.max(1)
    }

    pub fn array_layers(&self) -> u32 {
        self.header().layer_count.max(1)
    }

    pub fn face_count(&self) -> u32 {
        self.header().face_count.max(1)
    }

    pub fn is_cubemap(&self) -> bool {
        self.face_count() == 6
    }

    pub fn subresource_range(&self) -> vk::ImageSubresourceRange {
        let header = self.header();

        let mut layer_count = self.array_layers();
        if self.is_cubemap() {
            layer_count = 6;
        }

        vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_array_layer: 0,
            base_mip_level: 0,
            layer_count: layer_count,
            level_count: header.level_count.max(1),
        }
    }

    /// Return the pixels data at a mipmap level
    /// This assumes the data is not super compressed
    pub fn mimap_level_data<'a>(&'a self, mipmap: u32) -> &'a [u8] {
        let header = self.header();
        let level_count = header.level_count as isize;

        // Textures data starts after KtxHeader
        let level_offset = mem::size_of::<KtxHeader>() as isize;
        let levels_ptr = unsafe { self.data.as_ptr().offset(level_offset) } as *const LevelIndex;
        let level_index: &[LevelIndex] = unsafe { slice::from_raw_parts(levels_ptr, level_count as usize) };

        let mipmap_index = level_index[mipmap as usize];
        let start = mipmap_index.byte_offset as usize;
        let end = start + (mipmap_index.uncompressed_byte_length as usize);

        &self.data[start..end]
    }

    /// Check the KTX file header
    fn check_header(data: &[u8]) -> Result<(), CommonError> {
        if data.len() < mem::size_of::<KtxHeader>() { 
            return Err(assets_err!("KTX file too small"));
        }

        unsafe {
            if ptr::read::<[u8; 12]>(data.as_ptr() as _) != KTX_ID {
                return Err(assets_err!("KTX magic number does not match"));
            }
        }

        Ok(())
    }
    
    /// Return the ktx file header
    #[inline(always)]
    fn header<'a>(&'a self) -> &'a KtxHeader {
        unsafe {
            let header_ptr = self.data.as_ptr() as *const KtxHeader;
            &*header_ptr
        }
    }

}
