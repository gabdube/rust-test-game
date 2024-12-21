use std::sync::{Arc, atomic::{AtomicU32, AtomicUsize, Ordering}};
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

pub struct MessageQueue<ID, T> {
    length: AtomicUsize,
    buffer: Mutex<Box<[Option<(ID, T)>]>>,
}

impl<ID: Clone, T> MessageQueue<ID, T> {

    pub fn with_capacity(cap: usize) -> Self {
        let mut buffer = Vec::with_capacity(cap);
        for _ in 0..cap {
            buffer.push(None);
        }

        MessageQueue {
            length: AtomicUsize::new(0),
            buffer: Mutex::new(buffer.into_boxed_slice()),
        }
    }

    pub fn push(&self, id: &ID, data: T) {
        let mut buffer = self.buffer.lock();
        let index = self.length.fetch_add(1, Ordering::SeqCst);
        if index >= buffer.len() {
            println!("ERROR: Not enough capacity to hold more than {} messages. Increase the message queue capacity", buffer.len());
        } else {
            buffer[index] = Some((id.clone(), data));
        }
    }

    pub fn read_values<'a>(&'a self) -> Option<impl Iterator<Item = (ID, T)> + 'a> {
        match self.length.load(Ordering::SeqCst) {
            0 => None,
            _ => {
                let mut index = 0;
                let mut guard = self.buffer.lock();
                self.length.store(0, Ordering::SeqCst);
                Some(::std::iter::from_fn(move || {
                    if index < guard.len() {
                        index += 1;
                        guard[index - 1].take()
                    } else {
                        None
                    }
                }))
            }
        }
    }

}
