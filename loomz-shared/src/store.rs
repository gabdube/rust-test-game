mod reader;
mod writer;

pub use reader::*;
pub use writer::*;

const ALIGN: usize = size_of::<u32>();

pub trait StoreAndLoad {
    fn store(&self, writer: &mut SaveFileWriterBase);
    fn load(reader: &mut SaveFileReaderBase) -> Self;
}
