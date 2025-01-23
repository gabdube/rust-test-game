use bitflags::bitflags;
use parking_lot::{Mutex, MutexGuard};

use std::sync::Arc;
use crate::base_types::{PositionF64, SizeF32};

pub mod keys {
    use fnv::FnvHashMap;
    use super::MutexGuard;

    pub(super) type KeysCollection = FnvHashMap<u32, SingleKeyState>;

    pub const ESC: u32 = 114;

    #[derive(Copy, Clone, PartialEq)]
    pub enum SingleKeyState {
        Released,
        Pressed,
        Clicked
    }

    pub struct KeyState<'a> {
        pub(super) guard: MutexGuard<'a, super::InnerInputBuffer>,
    }

    impl<'a> KeyState<'a> {
        pub fn clicked(&self, key_code: u32) -> bool {
            self.guard.keys.get(&key_code)
                .map(|key| *key == SingleKeyState::Clicked )
                .unwrap_or(false)
        }
    }

}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct InputUpdateFlags: u32 {
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
}


struct InnerInputBuffer {
    pub update_flags: InputUpdateFlags,
    pub cursor_position_old: PositionF64,
    pub cursor_position: PositionF64,
    pub mouse_buttons_old: MouseButtonState,
    pub mouse_buttons: MouseButtonState,
    pub screen_size: SizeF32,
    pub keys: keys::KeysCollection,
}

impl InnerInputBuffer {

    fn new() -> Self {
        InnerInputBuffer {
            update_flags: InputUpdateFlags::empty(),
            cursor_position_old: PositionF64 { x: 0.0, y: 0.0 },
            cursor_position: PositionF64 { x: 0.0, y: 0.0 },
            mouse_buttons_old: MouseButtonState::empty(),
            mouse_buttons: MouseButtonState::empty(),
            screen_size: SizeF32 { width: 0.0, height: 0.0 },
            keys: keys::KeysCollection::default()
        }
    }

    fn cursor_position(&self) -> Option<PositionF64> {
        match self.update_flags.contains(InputUpdateFlags::MOUSE_MOVE) {
            true => Some(self.cursor_position),
            false => None
        }
    }

    fn update_cursor_position(&mut self, x: f64, y: f64) {
        self.update_flags |= InputUpdateFlags::MOUSE_MOVE;
        self.cursor_position_old = self.cursor_position;
        self.cursor_position = PositionF64 { x, y };
    }

    fn mouse_buttons(&self) -> Option<MouseButtonState> {
        match self.update_flags.contains(InputUpdateFlags::MOUSE_BTN) {
            true =>  Some(self.mouse_buttons),
            false => None
        }
    }

    fn update_mouse_button(&mut self, btns: MouseButtonState) {
        self.update_flags |= InputUpdateFlags::MOUSE_BTN;
        self.mouse_buttons_old = self.mouse_buttons;
        self.mouse_buttons = btns;
    }

    fn screen_size(&self) -> Option<SizeF32> {
        match self.update_flags.contains(InputUpdateFlags::SCREEN_RESIZED) {
            true => Some(self.screen_size),
            false => None
        }
    }

    fn screen_size_value(&self) -> SizeF32 {
        self.screen_size
    }

    pub fn set_key(&mut self, key_code: u32, pressed: bool) {
        let state = match pressed {
            true => keys::SingleKeyState::Pressed,
            false => keys::SingleKeyState::Clicked,
        };

        self.keys.insert(key_code, state);
        self.update_flags |= InputUpdateFlags::UPDATED_KEYSTATE;
    }

    fn update_screen_size(&mut self, width: f32, height: f32) {
        self.update_flags |= InputUpdateFlags::SCREEN_RESIZED;
        self.screen_size = SizeF32 { width, height };
    }

}

#[derive(Clone)]
pub struct InputBuffer {
    inner: Arc<Mutex<InnerInputBuffer>>,
}

impl InputBuffer {

    pub fn new() -> Self {
        InputBuffer {
            inner: Arc::new(Mutex::new(InnerInputBuffer::new()))
        }
    }

    pub fn cursor_position(&self) -> Option<PositionF64> {
        self.inputs().cursor_position()
    }

    pub fn update_cursor_position(&self, x: f64, y: f64) {
        self.inputs().update_cursor_position(x, y);
    }
    
    pub fn update_mouse_button(&self, btns: MouseButtonState) {
        self.inputs().update_mouse_button(btns)
    }

    pub fn mouse_buttons_value(&self) -> MouseButtonState {
        self.inputs().mouse_buttons
    }

    pub fn mouse_buttons(&self) -> Option<MouseButtonState> {
        self.inputs().mouse_buttons()
    }

    pub fn screen_size(&self) -> Option<SizeF32> {
        self.inputs().screen_size()
    }

    pub fn screen_size_value(&self) -> SizeF32 {
        self.inputs().screen_size_value()
    }

    pub fn update_screen_size(&self, width: f32, height: f32) {
        self.inputs().update_screen_size(width, height);
    }

    pub fn clear_update_flags(&self) {
        let mut inputs = self.inputs();

        // Toggle clicked keys to released state
        if inputs.update_flags.intersects(InputUpdateFlags::UPDATED_KEYSTATE) {
            for v in inputs.keys.values_mut() {
                if *v == keys::SingleKeyState::Clicked {
                    *v = keys::SingleKeyState::Released;
                }
            }
        }

        inputs.update_flags = InputUpdateFlags::empty();
    }

    pub fn keystate<'a>(&'a self) -> Option<keys::KeyState<'a>> {
        let guard = self.inner.lock();
        if guard.update_flags.contains(InputUpdateFlags::UPDATED_KEYSTATE) {
            Some(keys::KeyState { guard })
        } else {
            None
        }
    }

    pub fn set_key(&self, key_code: u32, pressed: bool) {
        self.inputs().set_key(key_code, pressed);
    }

    fn inputs<'a>(&'a self) -> MutexGuard<'a, InnerInputBuffer> {
        self.inner.lock()
    }
}
