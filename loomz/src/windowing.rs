use loomz_shared::inputs::InputBuffer;
use loomz_shared::{system_err, CommonError};

use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, EventLoop, ControlFlow};
use winit::event::{WindowEvent, ElementState, MouseButton};
use winit::window::{Window, WindowId};
use super::LoomzApplication;

impl<'a> ApplicationHandler for LoomzApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = match create_window(event_loop) {
            Ok(window) => window,
            Err(e) => {
                self.last_error = Some(e);
                event_loop.exit();
                return;
            }
        };

        if let Err(e) = self.set_window(window) {
            self.last_error = Some(e);
            event_loop.exit();
            return;
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.update() {
                    self.last_error = Some(e);
                    event_loop.exit();
                }

                if let Err(e) = self.redraw() {
                    self.last_error = Some(e);
                    event_loop.exit();
                }
            },
            WindowEvent::Resized(size) => {
                if let Err(e) = self.resized(size.width, size.height) {
                    self.last_error = Some(e);
                    event_loop.exit();
                }

                self.api.write_inputs().update_screen_size(size.width as f32, size.height as f32);
            },
            WindowEvent::CursorMoved { device_id: _, position } => {
                self.api.write_inputs().update_cursor_position(position.x, position.y);
            },
            WindowEvent::MouseInput { device_id: _, state, button } => {
                let mut inputs = self.api.write_inputs();
                parse_mouse_input(&mut inputs, state, button);
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            _  => {},
        }
    }

}

fn create_window(event_loop: &ActiveEventLoop) -> Result<Window, CommonError> {
    let window_attr = Window::default_attributes()
        .with_title("Loomz App")
        .with_inner_size(winit::dpi::PhysicalSize::new(1200, 900))
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

pub fn run(app: &mut LoomzApplication) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(app).unwrap();
}
