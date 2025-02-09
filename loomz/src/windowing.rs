use loomz_shared::inputs::InputBuffer;
use loomz_shared::{system_err, CommonError};

use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, EventLoop, ControlFlow};
use winit::event::{WindowEvent, ElementState, MouseButton};
use winit::window::{Window, WindowId};
use super::LoomzApplication;

impl<'a> ApplicationHandler for LoomzApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = self.initial_window_size();
        let window = match create_window(event_loop, size) {
            Ok(window) => window,
            Err(e) => {
                self.set_last_error(e);
                event_loop.exit();
                return;
            }
        };

        if let Err(e) = self.set_window(window) {
            self.set_last_error(e);
            event_loop.exit();
            return;
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.update() {
                    self.set_last_error(e);
                    event_loop.exit();
                }

                if let Err(e) = self.redraw() {
                    self.set_last_error(e);
                    event_loop.exit();
                }

                if self.api().must_exit() {
                    event_loop.exit();
                }
            },
            WindowEvent::Resized(size) => {
                if let Err(e) = self.resized(size.width, size.height) {
                    self.set_last_error(e);
                    event_loop.exit();
                }

                self.api().write_inputs().update_screen_size(size.width as f32, size.height as f32);
            },
            WindowEvent::CursorMoved { device_id: _, position } => {
                self.api().write_inputs().update_cursor_position(position.x, position.y);
            },
            WindowEvent::MouseInput { device_id: _, state, button } => {
                let mut inputs = self.api().write_inputs();
                parse_mouse_input(&mut inputs, state, button);
            },
            WindowEvent::KeyboardInput { device_id: _, is_synthetic: _, event } => {
                if !event.repeat {
                    let mut inputs = self.api().write_inputs();
                    parse_keyboard_input(&mut inputs, &event);
                }
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            _  => {},
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.window().set_visible(false);
    }

}

fn create_window(event_loop: &ActiveEventLoop, window_size: loomz_shared::SizeF32) -> Result<Window, CommonError> {
    let window_attr = Window::default_attributes()
        .with_title("Loomz App")
        .with_inner_size(winit::dpi::PhysicalSize::new(window_size.width, window_size.height))
        .with_visible(false);

    event_loop.create_window(window_attr)
        .map_err(|err| system_err!("Failed to create system window: {}", err) )
}

fn parse_mouse_input(inputs: &mut InputBuffer, state: ElementState, btn: MouseButton) {
    use loomz_shared::inputs::MouseButtonState;

    let mut button_state = inputs.mouse_buttons_value();
    let flag = match btn {
        MouseButton::Left => MouseButtonState::LEFT,
        MouseButton::Right => MouseButtonState::RIGHT,
        _ => MouseButtonState::empty(),
    };

    if !flag.is_empty() {
        if state.is_pressed() {
            button_state |= flag;
        } else {
            button_state.remove(flag);
        }

        inputs.update_mouse_button(button_state);
    }
}

fn parse_keyboard_input(inputs: &mut InputBuffer, key: &winit::event::KeyEvent) {
    let key_code = match key.physical_key {
        winit::keyboard::PhysicalKey::Code(code) => code as u32,
        _ => 0,
    };

    // println!("{:?}", key_code);

    inputs.set_key(key_code, key.state.is_pressed());
}

pub fn run(app: &mut LoomzApplication) {
    let event_loop = EventLoop::new().unwrap();

    #[cfg(not(feature="multithreading"))]
    event_loop.set_control_flow(ControlFlow::Poll);

    #[cfg(feature="multithreading")]
    event_loop.set_control_flow(ControlFlow::Wait);

    event_loop.run_app(app).unwrap();
}
