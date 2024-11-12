use loomz_shared::{system_err, CommonError};

use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, EventLoop, ControlFlow};
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};
use super::TestApplication;

impl<'a> ApplicationHandler for TestApplication {
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
                self.update();

                let successful = self.redraw();
                if !successful {
                    event_loop.exit();
                }
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
        .with_title("Demo App")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
        .with_visible(false);

    event_loop.create_window(window_attr)
        .map_err(|err| system_err!("Failed to create system window: {}", err) )
}

pub fn run(app: &mut TestApplication) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(app).unwrap();
}
