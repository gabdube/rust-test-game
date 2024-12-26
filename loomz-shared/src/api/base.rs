use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
use std::marker::PhantomData;
use parking_lot::Mutex;
use crate::store::{StoreAndLoad, SaveFileReaderBase, SaveFileWriterBase};

/// ID that ties data between the client and the engine
pub struct Id<T> {
    value: Arc<AtomicU32>,
    _t: PhantomData<T>
}

impl<T> Id<T> {
    pub fn new() -> Self {
        Id {
            value: Arc::new(AtomicU32::new(u32::MAX)),
            _t: PhantomData,
        }
    }

    #[inline]
    pub fn bind(&self, val: u32) {
        self.value.store(val, Ordering::SeqCst);
    }

    #[inline]
    pub fn is_unbound(&self) -> bool {
        self.value.load(Ordering::SeqCst) == u32::MAX
    }

    #[inline]
    pub fn value(&self) -> u32 {
        self.value.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn bound_value(&self) -> Option<usize> {
        let value = self.value.load(Ordering::SeqCst) as usize;
        match value == (u32::MAX as usize) {
            true => None,
            false => Some(value)
        }
    }
}

impl<T> StoreAndLoad for Id<T> {
    fn load(reader: &mut SaveFileReaderBase) -> Self {
        Id {
            value: Arc::new(AtomicU32::new(reader.read_u32())),
            _t: PhantomData,
        }
    }

    fn store(&self, writer: &mut SaveFileWriterBase) {
        writer.write_u32(self.value.load(Ordering::Relaxed));
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id {
            value: self.value.clone(),
            _t: PhantomData
        }
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

    pub fn push(&self, id: &ID, data: T) {
        let mut inner = self.inner.lock();
        let next_index = inner.length;
        let max_index = inner.buffer.len();
        if next_index >= max_index {
            println!("ERROR: Not enough capacity to hold more than {} messages. Increase the message queue capacity", max_index);
        } else {
            inner.length += 1;
            inner.buffer[next_index] = Some((id.clone(), data));
        }
    }

    pub fn read_values<'a>(&'a self) -> Option<impl Iterator<Item = (ID, T)> + 'a> {
        let mut inner = self.inner.lock();
        match inner.length {
            0 => None,
            _ => {
                inner.length = 0;

                let mut index = 0;
                Some(::std::iter::from_fn(move || {
                    let next_value = match inner.buffer.get_mut(index) {
                        Some(op) => op.take(),
                        None => None,
                    };
                    index += 1;
                    next_value
                }))
            }
        }
    }

}
