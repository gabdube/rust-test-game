use super::{ALIGN, StoreAndLoad};

pub struct SaveFileReaderBase<'a> {
    pub current_offset: usize,
    pub data: &'a [u32]
}

impl<'a> SaveFileReaderBase<'a> {

    pub fn new(data: &'a [u32]) -> Self {
        SaveFileReaderBase {
            current_offset: 0,
            data,
        }
    }
  
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
        assert!(align_of::<T>() >= ALIGN, "Alignment of T must be at least 4 bytes");

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
        assert!(align_of::<T>() >= ALIGN, "Alignment of T must be at least 4 bytes");
        let u32_count = size_of::<T>() / ALIGN;
        
        let data = unsafe {
            let data_ptr = self.data.as_ptr().offset(self.current_offset as isize) as *const T;
            *data_ptr
        };

        self.current_offset += u32_count;

        data
    }
    
    pub fn read_from_u32<T: From<u32>>(&mut self) -> T {
        T::from(self.read_u32())
    }

    pub fn read_bool(&mut self) -> bool {
        self.read_u32() == 1
    }

    pub fn load<T: StoreAndLoad>(&mut self) -> T {
        T::load(self)
    }

    pub fn skip(&mut self, count: usize) {
        self.current_offset += count;
    }

}
