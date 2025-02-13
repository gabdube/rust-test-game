mod keys_state;
pub use keys_state::*;

use bitflags::bitflags;
use parking_lot::{Mutex, MutexGuard};
use std::sync::atomic::{AtomicU8, Ordering};

use std::sync::Arc;
use crate::base_types::{PositionF64, SizeF32};

bitflags! {
    #[derive(Copy, Clone)]
    pub struct InputUpdateFlags: u8 {
        const SCREEN_RESIZED   = 0b00000001;
        const MOUSE_MOVE       = 0b00000010;
        const MOUSE_BTN        = 0b00000100;
        const UPDATED_KEYSTATE = 0b00001000;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct MouseButtonState: u32 {
        const LEFT = 0b0001;
        const RIGHT = 0b0010;
    }
}

impl MouseButtonState {
    pub const fn left_button_down(&self) -> bool {
        self.contains(MouseButtonState::LEFT)
    }

    pub const fn right_button_down(&self) -> bool {
        self.contains(MouseButtonState::RIGHT)
    }
}


#[derive(Copy, Clone)]
pub struct InputBuffer {
    pub cursor_position_old: PositionF64,
    pub cursor_position: PositionF64,
    pub mouse_buttons_old: MouseButtonState,
    pub mouse_buttons: MouseButtonState,
    pub screen_size: SizeF32,
}

impl InputBuffer {

    fn new(screen_size: SizeF32) -> Self {
        InputBuffer {
            cursor_position_old: PositionF64 { x: 0.0, y: 0.0 },
            cursor_position: PositionF64 { x: 0.0, y: 0.0 },
            mouse_buttons_old: MouseButtonState::empty(),
            mouse_buttons: MouseButtonState::empty(),
            screen_size,
        }
    }

}

struct InnerInputBuffer {
    buffer: Mutex<InputBuffer>,
    flags: AtomicU8
}

pub struct SharedInputBuffer {
    inner: Arc<InnerInputBuffer>,
}

impl SharedInputBuffer {

    pub(super) fn new(screen_size: SizeF32) -> Self {
        let buffer = Mutex::new(InputBuffer::new(screen_size));
        let flags = AtomicU8::new(0);
        SharedInputBuffer {
            inner: Arc::new(InnerInputBuffer {
                buffer,
                flags,
            })
        }
    }

    pub fn cursor_position(&self) -> Option<PositionF64> {
        match self.flags().contains(InputUpdateFlags::MOUSE_MOVE) {
            true => Some(self.lock().cursor_position),
            false => None
        }
    }

    pub fn cursor_position_delta(&self) -> PositionF64 {
        let inputs = self.lock();
        inputs.cursor_position - inputs.cursor_position_old
    }

    pub fn update_cursor_position(&self, x: f64, y: f64) {
        let mut buffer = self.lock();
        buffer.cursor_position_old = buffer.cursor_position;
        buffer.cursor_position = PositionF64 { x, y };
        self.set_flags(InputUpdateFlags::MOUSE_MOVE);
    }
    
    pub fn mouse_buttons(&self) -> Option<MouseButtonState> {
        match self.flags().contains(InputUpdateFlags::MOUSE_BTN) {
            true => Some(self.lock().mouse_buttons),
            false => None
        }
    }

    pub fn mouse_buttons_value(&self) -> MouseButtonState {
        self.lock().mouse_buttons
    }

    pub fn add_mouse_button(&self, btns: MouseButtonState) {
        let mut buffer = self.lock();
        let new = buffer.mouse_buttons | btns;
        buffer.mouse_buttons_old = buffer.mouse_buttons;
        buffer.mouse_buttons = new;

        self.set_flags(InputUpdateFlags::MOUSE_BTN);
    }

    pub fn remove_mouse_button(&self, btns: MouseButtonState) {
        let mut buffer = self.lock();
        
        let mut new = buffer.mouse_buttons;
        new.remove(btns);

        buffer.mouse_buttons_old = buffer.mouse_buttons;
        buffer.mouse_buttons = new;

        self.set_flags(InputUpdateFlags::MOUSE_BTN);
    }

    pub fn screen_size(&self) -> Option<SizeF32> {
        match self.flags().contains(InputUpdateFlags::SCREEN_RESIZED) {
            true => Some(self.lock().screen_size),
            false => None
        }
    }

    pub fn screen_size_value(&self) -> SizeF32 {
        self.lock().screen_size
    }

    pub fn update_screen_size(&self, width: f32, height: f32) {
        self.lock().screen_size = SizeF32 { width, height };
        self.set_flags(InputUpdateFlags::SCREEN_RESIZED);
    }

    pub fn clear_update_flags(&self) {
        let flags = self.flags();

        if flags.contains(InputUpdateFlags::MOUSE_MOVE) {
            let mut buffer = self.lock();
            buffer.cursor_position_old = buffer.cursor_position;
        }

        self.inner.flags.store(0, Ordering::Relaxed);
    }

    fn flags(&self) -> InputUpdateFlags {
        InputUpdateFlags::from_bits_retain(self.inner.flags.load(Ordering::Relaxed))
    }

    fn set_flags(&self, value: InputUpdateFlags) {
        self.inner.flags.fetch_or(value.bits(), Ordering::Relaxed);
    }

    fn lock<'a>(&'a self) -> MutexGuard<'a, InputBuffer> {
        self.inner.buffer.lock()
    }
}

impl Clone for SharedInputBuffer {
    fn clone(&self) -> Self {
        SharedInputBuffer {
            inner: Arc::clone(&self.inner)
        }
    }
}
