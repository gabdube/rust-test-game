mod windowing;

use loomz_shared::{system_err, CommonError};
use winit::window::Window;

pub struct TestApplication {
    window: Option<Window>,
    last_error: Option<CommonError>,
}

impl TestApplication {

    pub fn init() -> Result<Self, CommonError> { 
        let app = TestApplication {
            window: None,
            last_error: None,
        };

        Ok(app)
    }

    pub fn exit(self) {
    }

    pub fn set_window(&mut self, window: Window) -> Result<(), CommonError> {
        window.set_visible(true);
        self.window = Some(window);
        Ok(())
    }

    pub fn update(&mut self) {
    }

    pub fn redraw(&mut self) -> bool {
        self.window().request_redraw();
        true
    }

    fn window(&self) -> &Window {
        match self.window.as_ref() {
            Some(window) => window,
            None => unreachable!("Window will always be some at runtime")
        }
    }

}

pub fn main() {
    let mut app = match TestApplication::init() {
        Ok(app) => { app },
        Err(e) => {
            eprintln!("{}", e);
            return
        }
    };

    windowing::run(&mut app);

    if let Some(err) = app.last_error.take() {
        eprintln!("{}", err);
    }

    app.exit();
}
