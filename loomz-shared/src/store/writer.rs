use super::StoreAndLoad;

pub struct SaveFileWriterBase {
    pub data_offset: u32,
    pub data: Vec<u32>
}

impl SaveFileWriterBase {
    pub fn new(capacity: usize) -> Self {
        SaveFileWriterBase {
            data_offset: 0,
            data: vec![0; capacity]
        }
    }

    pub fn write_str(&mut self, value: &str) {
        let padding = 4 - (value.len() % 4);
        let length = value.len();
        let padded_length = length + padding;

        let u32_count = padded_length / 4;
        self.try_realloc(u32_count + 2);
        
        self.write_u32_inner(length as u32);
        self.write_u32_inner(padded_length as u32);

        unsafe { 
            ::std::ptr::copy_nonoverlapping::<u8>(
                value.as_ptr(),
                self.data.as_ptr().offset(self.data_offset as isize) as *mut u8,
                length as usize
            );
        }

        self.data_offset += u32_count as u32;
    }

    pub fn write_slice<T: Copy>(&mut self, values: &[T]) {
        let (x, aligned, y) = unsafe { values.align_to::<u32>() };
        if !x.is_empty() || !y.is_empty() {
            panic!("slice must be aligned to 4 bytes");
        }

        let u32_count = aligned.len();
        self.try_realloc(u32_count + 1);

        self.write_u32_inner(values.len() as u32);

        for &value in aligned {
            self.write_u32_inner(value);
        }
    }

    pub fn write_bool_slice(&mut self, values: &[bool]) {
        let u32_count = values.len() + 1;
        self.try_realloc(u32_count + 1);
        self.write_u32_inner(values.len() as u32);
        for &value in values {
            self.write_u32_inner(value as u32);
        }
    }

    pub fn write_u32(&mut self, data: u32) {
        self.try_realloc(1);
        self.write_u32_inner(data);
    }

    pub fn write_f32(&mut self, data: f32) {
        self.try_realloc(1);
        self.write_u32_inner(data.to_bits());
    }

    pub fn write<T: Copy>(&mut self, data: &T) {
        let data_array = ::std::slice::from_ref(data);
        let (x, aligned, y) = unsafe { data_array.align_to::<u32>() };
        if !x.is_empty() || !y.is_empty() {
            panic!("Data must be aligned to 4 bytes");
        }

        let u32_count = aligned.len();
        self.try_realloc(u32_count);

        for &value in aligned {
            self.write_u32_inner(value);
        }
    }

    pub fn write_into_u32<T: Into<u32>>(&mut self, data: T) {
        self.write_u32(data.into());
    }

    pub fn store<T: StoreAndLoad>(&mut self, data: &T) {
        data.store(self);
    }

    #[inline(always)]
    fn write_u32_inner(&mut self, value: u32) {
        self.data[self.data_offset as usize] = value;
        self.data_offset += 1;
    }

    fn try_realloc(&mut self, data_count: usize) {
        let data_offset = self.data_offset as usize;
        if data_offset + data_count >= self.data.len() {
            self.data.reserve_exact(2000 + data_count);
            unsafe { self.data.set_len(self.data.capacity()); }
        }
    }
}
