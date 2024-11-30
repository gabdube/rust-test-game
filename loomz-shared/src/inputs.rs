use bitflags::bitflags;
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


pub struct InputBuffer {
    pub update_flags: InputUpdateFlags,
    pub cursor_position_old: Position<f64>,
    pub cursor_position: Position<f64>,
    pub mouse_buttons_old: MouseButtonState,
    pub mouse_buttons: MouseButtonState,
    pub screen_size: Size<f32>,
}

impl InputBuffer {

    pub fn new() -> Self {
        InputBuffer {
            update_flags: InputUpdateFlags::empty(),
            cursor_position_old: pos(0.0, 0.0),
            cursor_position: pos(0.0, 0.0),
            mouse_buttons_old: MouseButtonState::empty(),
            mouse_buttons: MouseButtonState::empty(),
            screen_size: size(0.0, 0.0),
        }
    }

    pub fn cursor_position(&mut self) -> Option<Position<f64>> {
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

    pub fn update_cursor_position(&mut self, x: f64, y: f64) {
        self.update_flags |= InputUpdateFlags::MOUSE_MOVE;
        self.cursor_position_old = self.cursor_position;
        self.cursor_position = pos(x, y);
    }

    pub fn update_mouse_button(&mut self, btns: MouseButtonState) {
        self.update_flags |= InputUpdateFlags::MOUSE_BTN;
        self.mouse_buttons_old = self.mouse_buttons;
        self.mouse_buttons = btns;
    }

    pub fn screen_size(&mut self) -> Option<Size<f32>> {
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

    pub fn update_screen_size(&mut self, width: f32, height: f32) {
        self.update_flags |= InputUpdateFlags::SCREEN_RESIZED;
        self.screen_size = size(width, height);

    }

}
