use std::sync::atomic::{AtomicU32, Ordering};
use std::marker::PhantomData;
use parking_lot::Mutex;
use crate::store::{StoreAndLoad, SaveFileReaderBase, SaveFileWriterBase};

/// ID that ties data between the client and the engine
pub struct Id<T> {
    value: u32,
    _t: PhantomData<T>
}

impl<T> Id<T> {
    pub fn new() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        Id {
            value: COUNTER.fetch_add(1, Ordering::Relaxed),
            _t: PhantomData,
        }
    }

    #[inline]
    pub fn value(&self) -> u32 {
        self.value
    }
}

impl<T> StoreAndLoad for Id<T> {
    fn load(reader: &mut SaveFileReaderBase) -> Self {
        Id {
            value: reader.read_u32(),
            _t: PhantomData,
        }
    }

    fn store(&self, writer: &mut SaveFileWriterBase) {
        writer.write_u32(self.value);
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Id::new()
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id {
            value: self.value,
            _t: PhantomData
        }
    }
}

impl<T> Copy for Id<T> {
}

impl<T> ::std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID({})", self.value)
    }
}


struct InnerBuffer<ID, T> {
    length: usize,
    buffer: Box<[Option<(ID, T)>]>,
}

// TODO: replace this by a ringbuffer (ringbuf crate?)
pub struct MessageQueue<ID, T> {
    inner: Mutex<InnerBuffer<ID, T>>,
}

impl<ID: Clone, T> MessageQueue<ID, T> {

    pub fn with_capacity(cap: usize) -> Self {
        let mut buffer = Vec::with_capacity(cap);
        for _ in 0..cap {
            buffer.push(None);
        }

        MessageQueue {
            inner: Mutex::new(InnerBuffer {
                length: 0,
                buffer: buffer.into_boxed_slice()
            }),
        }
    }

    pub fn push(&self, id: &ID, message: T) {
        let mut inner = self.inner.lock();
        let next_index = inner.length;
        let max_index = inner.buffer.len();
        if next_index >= max_index {
            println!("ERROR: Not enough capacity to hold more than {} messages. Increase the message queue capacity", max_index);
        } else {
            inner.length += 1;
            inner.buffer[next_index] = Some((id.clone(), message));
        }
    }

    pub fn read_values<'a>(&'a self) -> Option<impl Iterator<Item = (ID, T)> + 'a> {
        let mut inner = self.inner.lock();
        if inner.length == 0 {
            return None;
        }

        inner.length = 0;

        let mut index = 0;
        let it = ::std::iter::from_fn(move || {
            let next_value = match inner.buffer.get_mut(index) {
                Some(op) => op.take(),
                None => None,
            };
            index += 1;
            next_value
        });

        Some(it)
    }

}

struct InnerBufferEx<ID, T> {
    length_message: u32,
    length_data: u32,
    buffer: Box<[Option<(ID, T)>]>,
    data: &'static mut [u8],
}

/// A message queue extended to support data of varying size
pub struct MessageQueueEx<ID, T> {
    inner: Mutex<InnerBufferEx<ID, T>>,
}

impl<ID: Clone, T> MessageQueueEx<ID, T> {

    /// Creates a new extended queue with a capacity of `cap_message` size
    /// and a capacity of `cap_data_bytes` of extra data
    pub fn with_capacity(cap_message: usize, cap_data_bytes: usize) -> Self {
        let mut buffer = Vec::with_capacity(cap_message);
        for _ in 0..cap_message {
            buffer.push(None);
        }

        let data = vec![0u8; cap_data_bytes];

        MessageQueueEx {
            inner: Mutex::new(InnerBufferEx {
                length_message: 0,
                length_data: 0,
                buffer: buffer.into_boxed_slice(),
                data: data.leak(),
            }),
        }
    }

    pub fn push(&self, id: &ID, message: T) {
        let mut inner = self.inner.lock();
        let next_index = inner.length_message as usize;
        let max_index = inner.buffer.len();
        if next_index >= max_index {
            println!("ERROR: Not enough capacity to hold more than {} messages. Increase the message queue capacity", max_index);
        } else {
            inner.length_message += 1;
            inner.buffer[next_index] = Some((id.clone(), message));
        }
    }

    pub fn push_with_data<D: Copy+'static, F0>(&self, id: &ID, src_data: &[D], generate_message: F0) where
        F0: FnOnce(&'static [D]) -> T
    {
        let mut inner = self.inner.lock();

        let next_index = inner.length_message as usize;
        if next_index >= inner.buffer.len() {
            println!("ERROR: Not enough capacity to hold more than {} messages. Increase the message queue capacity", inner.buffer.len());
            return;
        }

        let next_data_offset = align(inner.length_data as usize, align_of::<D>());
        let data_size = src_data.len() * size_of::<D>();
        if next_data_offset + data_size > inner.data.len() {
            println!("ERROR: Not enough capacity to hold more than {} bytes of extra data. Increase the buffer capacity", inner.data.len());
            return;
        }

        // Copy data into the queue buffer then run the callback
        // so that the caller can generate the message.
        // Safety: type `D` can be copied safely
        let message = unsafe {
            let bytes_ptr = inner.data.as_mut_ptr().add(next_data_offset);
            let data_ptr = bytes_ptr as *mut D;
            let data_dst: &'static mut [D] = ::std::slice::from_raw_parts_mut(data_ptr, src_data.len());
            for (i, data) in src_data.iter().enumerate() {
                data_dst[i] = *data;
            }
            
            generate_message(data_dst)
        };

        inner.length_message += 1;
        inner.length_data = (next_data_offset + data_size) as u32;
        inner.buffer[next_index] = Some((id.clone(), message));
    }

    pub fn read_values<'a>(&'a self) -> Option<impl Iterator<Item = (ID, T)> + 'a> {
        let mut inner = self.inner.lock();
        if inner.length_message == 0 {
            return None;
        }

        inner.length_message = 0;
        inner.length_data = 0;

        let mut index = 0;
        let it = ::std::iter::from_fn(move || {
            let next_value = match inner.buffer.get_mut(index) {
                Some(op) => op.take(),
                None => None,
            };
            index += 1;
            next_value
        });

        Some(it)
    }

}

#[inline]
const fn align(addr: usize, align: usize) -> usize {
    let addr = addr as isize;
    let align = align as isize;
    ((addr + (align - 1)) & -align) as usize
}
