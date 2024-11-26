use std::mem;
use loomz_shared::{save_err, CommonError};
use super::{SaveFileHeader, MAGIC, ALIGN};

pub(crate) struct SaveFileReader<'a> {
    current_offset: usize,
    data: &'a [u32]
}

impl<'a> SaveFileReader<'a> {
    pub fn new(bytes: &'a Box<[u8]>) -> Result<SaveFileReader<'a>, CommonError> {
        let data = Self::validate_data_integrity(bytes)?;
        let decoder = SaveFileReader {
            current_offset: 0,
            data,
        };

        decoder.validate_header()?;

        Ok(decoder)
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
        let header_ptr = self.data.as_ptr() as *const SaveFileHeader;
        let header = unsafe { ::std::ptr::read(header_ptr) };
        
        if header.magic != MAGIC {
            return Err(save_err!("Decoder header magic does not match"));
        }

        if header.size != self.data.len() as u32 {
            save_err!("Header data size does not match buffer size");
            return Err(save_err!("Decoder header magic does not match"));
        }

        Ok(())
    }

    //
    // Header
    //

    // pub fn begin_root(&mut self) {
    //     let offset: usize = mem::offset_of!(SaveFileHeader, root) / ALIGN;
    //     self.current_offset = self.data[offset] as usize;
    // }

    //
    // Reader
    //

    pub fn read_str(&mut self) -> &str {
        let length = self.read_u32();
        let length_padded = self.read_u32();

        let str = unsafe {
            let str_ptr = self.data.as_ptr().offset(self.current_offset as isize) as *const u8;
            let str_bytes = ::std::slice::from_raw_parts(str_ptr, length as usize);
            ::std::str::from_utf8(str_bytes).unwrap_or("UTF8 DECODING ERROR")
        };

        self.current_offset += (length_padded / 4) as usize;

        str
    }

    pub fn read_slice<T: Copy>(&mut self) -> &[T] {
        assert!(mem::align_of::<T>() >= ALIGN, "Alignment of T must be at least 4 bytes");

        let length = self.read_u32();

        let data = unsafe {
            let data_ptr = self.data.as_ptr().offset(self.current_offset as isize) as *const T;
            ::std::slice::from_raw_parts(data_ptr, length as usize)
        };

        let u32_count = (length as usize) * size_of::<T>();
        self.current_offset += u32_count / ALIGN;

        data
    }

    pub fn read_bool_vec(&mut self) -> Vec<bool> {
        let length = self.read_u32();
        let mut values = Vec::with_capacity(length as usize);

        for _ in 0..length {
            values.push(self.read_u32() == 1);
        }

        values
    }

    pub fn read_u32(&mut self) -> u32 {
        let value = self.data[self.current_offset];
        self.current_offset += 1;
        value
    }

    pub fn read_f32(&mut self) -> f32 {
        let value = self.data[self.current_offset];
        self.current_offset += 1;
        f32::from_bits(value)
    }

    pub fn read<T: Copy>(&mut self) -> T {
        assert!(mem::align_of::<T>() >= ALIGN, "Alignment of T must be at least 4 bytes");
        let u32_count = size_of::<T>() / ALIGN;
        
        let data = unsafe {
            let data_ptr = self.data.as_ptr().offset(self.current_offset as isize) as *const T;
            *data_ptr
        };

        self.current_offset += u32_count;

        data
    }
    
}
