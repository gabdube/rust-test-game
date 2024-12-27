use std::sync::Arc;
use bitflags::bitflags;
use parking_lot::{Mutex, MutexGuard};
use crate::base_types::_2d::{Position, Size, pos, size};

bitflags! {
    #[derive(Copy, Clone)]
    pub struct InputUpdateFlags: u32 {
        const SCREEN_RESIZED = 0b00000001;
        const MOUSE_MOVE = 0b00000010;
        const MOUSE_BTN = 0b00000100;
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct MouseButtonState: u32 {
        const LEFT = 0b0001;
        const RIGHT = 0b0001;
    }
}


struct InnerInputBuffer {
    pub update_flags: InputUpdateFlags,
    pub cursor_position_old: Position<f64>,
    pub cursor_position: Position<f64>,
    pub mouse_buttons_old: MouseButtonState,
    pub mouse_buttons: MouseButtonState,
    pub screen_size: Size<f32>,
}

impl InnerInputBuffer {

    fn new() -> Self {
        InnerInputBuffer {
            update_flags: InputUpdateFlags::empty(),
            cursor_position_old: pos(0.0, 0.0),
            cursor_position: pos(0.0, 0.0),
            mouse_buttons_old: MouseButtonState::empty(),
            mouse_buttons: MouseButtonState::empty(),
            screen_size: size(0.0, 0.0),
        }
    }

    fn cursor_position(&mut self) -> Option<Position<f64>> {
        match self.update_flags.contains(InputUpdateFlags::MOUSE_MOVE) {
            true => {
                self.update_flags.remove(InputUpdateFlags::MOUSE_MOVE);
                Some(self.cursor_position)
            },
            false => {
                None
            }
        }
    }

    fn update_cursor_position(&mut self, x: f64, y: f64) {
        self.update_flags |= InputUpdateFlags::MOUSE_MOVE;
        self.cursor_position_old = self.cursor_position;
        self.cursor_position = pos(x, y);
    }

    fn update_mouse_button(&mut self, btns: MouseButtonState) {
        self.update_flags |= InputUpdateFlags::MOUSE_BTN;
        self.mouse_buttons_old = self.mouse_buttons;
        self.mouse_buttons = btns;
    }

    fn screen_size(&mut self) -> Option<Size<f32>> {
        match self.update_flags.contains(InputUpdateFlags::SCREEN_RESIZED) {
            true => {
                self.update_flags.remove(InputUpdateFlags::SCREEN_RESIZED);
                Some(self.screen_size)
            },
            false => {
                None
            }
        }
    }

    fn screen_size_value(&self) -> Size<f32> {
        self.screen_size
    }

    fn update_screen_size(&mut self, width: f32, height: f32) {
        self.update_flags |= InputUpdateFlags::SCREEN_RESIZED;
        self.screen_size = size(width, height);
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

    pub fn cursor_position(&self) -> Option<Position<f64>> {
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

    pub fn screen_size(&self) -> Option<Size<f32>> {
        self.inputs().screen_size()
    }

    pub fn screen_size_value(&self) -> Size<f32> {
        self.inputs().screen_size_value()
    }

    pub fn update_screen_size(&self, width: f32, height: f32) {
        self.inputs().update_screen_size(width, height);
    }

    fn inputs<'a>(&'a self) -> MutexGuard<'a, InnerInputBuffer> {
        self.inner.lock()
    }
}
