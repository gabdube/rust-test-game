use fnv::FnvHashMap;
use parking_lot::{Mutex, MutexGuard};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub mod keys {
    pub const _1: u32 = 6;
    pub const _2: u32 = 7;
    pub const _3: u32 = 8;
    pub const ESC: u32 = 114;
}

type KeyStateCollection = FnvHashMap<u32, SingleKeyState>;

#[derive(Copy, Clone, PartialEq)]
pub enum SingleKeyState {
    Released,
    JustReleased,
    Pressed,
    JustPressed,
}

#[derive(Default)]
struct InnerKeyState {
    collection: Mutex<KeyStateCollection>,
    flags: AtomicBool,
}

pub struct KeyStateGuard<'a> {
    inner: MutexGuard<'a, KeyStateCollection>
}

impl<'a> KeyStateGuard<'a> {
    pub fn just_released(&self, key_code: u32) -> bool {
        self.inner.get(&key_code)
            .map(|key| *key == SingleKeyState::JustReleased )
            .unwrap_or(false)
    }

    pub fn just_pressed(&self, key_code: u32) -> bool {
        self.inner.get(&key_code)
            .map(|key| *key == SingleKeyState::JustPressed )
            .unwrap_or(false)
    }

    pub fn set_key(&mut self, key_code: u32, pressed: bool) {
        self.inner.insert(key_code, match pressed {
            true => SingleKeyState::JustPressed,
            false => SingleKeyState::JustReleased,
        });
    }
}

pub struct SharedKeysState {
    inner: Arc<InnerKeyState>,
}

impl SharedKeysState {

    pub fn new() -> Self {
        SharedKeysState {
            inner: Arc::new(InnerKeyState::default()),
        }
    }

    pub fn read_updates<'a>(&'a self) -> Option<KeyStateGuard<'a>> {
        match self.inner.flags.load(Ordering::Relaxed) {
            true => Some(KeyStateGuard { inner: self.inner.collection.lock() }),
            false => None
        }
    }

    pub fn write<'a>(&'a self) -> KeyStateGuard<'a> {
        let guard = KeyStateGuard { inner: self.inner.collection.lock() };
        self.inner.flags.store(true, Ordering::SeqCst);
        guard
    }

    pub fn clear_update_flags(&self) {
        if self.inner.flags.fetch_and(false, Ordering::Relaxed) {
            let mut collection = self.inner.collection.lock();
            for v in collection.values_mut() {
                let state = *v;
                if state == SingleKeyState::JustReleased {
                    *v = SingleKeyState::Released;
                }
                else if state == SingleKeyState::JustPressed {
                    *v = SingleKeyState::Pressed;
                }
            }
        }
    }

}

impl Clone for SharedKeysState {
    fn clone(&self) -> Self {
        SharedKeysState {
            inner: Arc::clone(&self.inner),
        }
    }
}
