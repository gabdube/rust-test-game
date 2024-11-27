use loomz_shared::store::{SaveFileReaderBase, SaveFileWriterBase};
use loomz_shared::{save_err, CommonError};

const MAGIC: u32 = 0x6FAA7601;
const ALIGN: usize = size_of::<u32>();

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SaveFileHeader {
    pub magic: u32,
    pub size: u32,
}

impl SaveFileHeader {
    pub fn new() -> Self {
        SaveFileHeader {
            magic: MAGIC,
            size: 0,
        }
    }
}

pub struct SaveFileReader<'a> {
    base: SaveFileReaderBase<'a>
}

impl<'a> SaveFileReader<'a> {

    pub fn new(bytes: &'a Box<[u8]>) -> Result<SaveFileReader<'a>, CommonError> {
        let data = Self::validate_data_integrity(bytes)?;
        let mut reader = SaveFileReader {
            base: SaveFileReaderBase::new(data),
        };

        reader.validate_header()?;
        reader.current_offset = size_of::<SaveFileHeader>() / ALIGN;

        Ok(reader)
    }

    fn validate_data_integrity(bytes: &Box<[u8]>) -> Result<&[u32], CommonError> {
        let byte_slice: &[u8] = bytes.as_ref();
        if byte_slice.len() < size_of::<SaveFileHeader>() {
            return Err(save_err!("Data is smaller than the save file header size"));
        }

        let bytes_ptr = byte_slice.as_ptr() as usize;
        if bytes_ptr % 4 != 0 {
            return Err(save_err!("Data pointer is not aligned to 4 bytes"));
        }

        unsafe { Ok(byte_slice.align_to::<u32>().1) }
    }

    fn validate_header(&self) -> Result<(), CommonError> {
        let header_ptr = self.base.data.as_ptr() as *const SaveFileHeader;
        let header = unsafe { ::std::ptr::read(header_ptr) };
        
        if header.magic != MAGIC {
            return Err(save_err!("Decoder header magic does not match"));
        }

        if header.size != self.base.data.len() as u32 {
            return Err(save_err!("Header data size does not match buffer size"));
        }

        Ok(())
    }

}

pub struct SaveFileWriter {
    base: SaveFileWriterBase
}

impl SaveFileWriter {
    pub fn new() -> Self {
        let mut writer = SaveFileWriter {
            base: SaveFileWriterBase::new(3000)
        };

        writer.write_header();

        writer
    }

    fn write_header(&mut self) {
        self.write(&SaveFileHeader::new());        
    }

    pub fn finalize(mut self) -> Vec<u8> {
        let total_size = self.data_offset;

        let offset: usize = ::std::mem::offset_of!(SaveFileHeader, size) / ALIGN;
        self.data[offset] = total_size;

        let total_size = total_size as usize;
        let total_size_bytes = total_size * ALIGN;
        let mut out_bytes = vec![0u8; total_size_bytes];
        unsafe { ::std::ptr::copy_nonoverlapping::<u32>(self.data.as_ptr(), out_bytes.as_mut_ptr() as *mut u32, total_size); }

        out_bytes
    }
}

impl ::std::ops::Deref for SaveFileWriter {
    type Target = SaveFileWriterBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl ::std::ops::DerefMut for SaveFileWriter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<'a> ::std::ops::Deref for SaveFileReader<'a> {
    type Target = SaveFileReaderBase<'a>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a> ::std::ops::DerefMut for SaveFileReader<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
