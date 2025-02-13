use loomz_shared::{system_err, LoomzApi, CommonError};

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
        if self.api().must_exit() {
            event_loop.exit();
        }

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
            },
            WindowEvent::Resized(size) => {
                self.api().inputs_ref().update_screen_size(size.width as f32, size.height as f32);

                if let Err(e) = self.resized() {
                    self.set_last_error(e);
                    event_loop.exit();
                }
            },
            WindowEvent::CursorMoved { device_id: _, position } => {
                self.api().inputs_ref().update_cursor_position(position.x, position.y);
            },
            WindowEvent::MouseInput { device_id: _, state, button } => {
                parse_mouse_input(self.api(), state, button);
            },
            WindowEvent::KeyboardInput { device_id: _, is_synthetic: _, event } => {
                if !event.repeat {
                    parse_keyboard_input(self.api(), &event);
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
    let monitor_size = event_loop.primary_monitor()
        .map(|m| m.size() )
        .unwrap_or_default();
    
    let window_attr = Window::default_attributes()
        .with_title("Loomz App")
        .with_inner_size(winit::dpi::PhysicalSize::new(window_size.width, window_size.height))
        .with_position(winit::dpi::PhysicalPosition::new(monitor_size.width - (window_size.width as u32), 0))
        .with_visible(false);

    event_loop.create_window(window_attr)
        .map_err(|err| system_err!("Failed to create system window: {}", err) )
}

fn parse_mouse_input(api: &LoomzApi, state: ElementState, btn: MouseButton) {
    use loomz_shared::inputs::MouseButtonState;

    let flag = match btn {
        MouseButton::Left => MouseButtonState::LEFT,
        MouseButton::Right => MouseButtonState::RIGHT,
        _ => MouseButtonState::empty(),
    };

    if !flag.is_empty() {
        let inputs = api.inputs_ref();
        if state.is_pressed() {
            inputs.add_mouse_button(flag);
        } else {
            inputs.remove_mouse_button(flag);
        }
    }
}

fn parse_keyboard_input(api: &LoomzApi, key: &winit::event::KeyEvent) {
    let key_code = match key.physical_key {
        winit::keyboard::PhysicalKey::Code(code) => code as u32,
        _ => 0,
    };

    if key_code > 0 {
        api.keys_ref().write().set_key(key_code, key.state.is_pressed());
    }
}

pub fn run(app: &mut LoomzApplication) {
    let event_loop = EventLoop::new().unwrap();

    #[cfg(not(feature="multithreading"))]
    event_loop.set_control_flow(ControlFlow::Poll);

    #[cfg(feature="multithreading")]
    event_loop.set_control_flow(ControlFlow::Wait);

    event_loop.run_app(app).unwrap();
}
