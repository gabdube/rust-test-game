pub mod writer;
pub mod reader;

const MAGIC: u32 = 0x6FAA7601;
const ALIGN: usize = size_of::<u32>();

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SaveFileHeader {
    pub magic: u32,
    pub size: u32,
}

impl Default for SaveFileHeader {
    fn default() -> Self {
        SaveFileHeader {
            magic: MAGIC,
            size: 0,
        }
    }
}
