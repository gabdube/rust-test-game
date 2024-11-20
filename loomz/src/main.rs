mod windowing;

use loomz_shared::{LoomzApi, CommonError};
use loomz_client::LoomzClient;
use loomz_engine::LoomzEngine;
use winit::window::Window;

pub struct LoomzApplication {
    window: Option<Box<Window>>,
    client: LoomzClient,
    engine: LoomzEngine,
    last_error: Option<CommonError>,
}

impl LoomzApplication {

    pub fn init() -> Result<Self, CommonError> {
        let mut api = LoomzApi::init();
        let client = LoomzClient::init(&mut api);
        let engine = LoomzEngine::init(&mut api)?;

        let app = LoomzApplication {
            window: None,
            client,
            engine,
            last_error: None,
        };

        Ok(app)
    }

    pub fn exit(self) {
        self.engine.destroy();
    }

    pub fn set_window(&mut self, window: Window) -> Result<(), CommonError> {
        use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

        let (display_handle, window_handle) = window.display_handle()
            .and_then(|display| window.window_handle().map(|window| (display, window) ) )
            .map(|(display, window)| (display.as_raw(), window.as_raw()) )
            .map_err(|err| loomz_shared::system_err!("Failed to get system handles: {}", err) )?;

        let size = window.inner_size();
        self.engine.set_output(display_handle, window_handle, [size.width, size.height])?;

        window.set_visible(true);
        self.window = Some(Box::new(window));

        Ok(())
    }

    pub fn update(&mut self) {
        self.client.update();
        self.engine.update();
    }

    pub fn redraw(&mut self) -> Result<(), CommonError> {
        self.engine.render()?;
        self.window().request_redraw();
        Ok(())
    }

    pub fn resized(&mut self, width: u32, height: u32) -> Result<(), CommonError> {
        self.engine.resize_output(width, height)
    }

    fn window(&self) -> &Window {
        match self.window.as_ref() {
            Some(window) => window,
            None => unreachable!("Window will always be some at runtime")
        }
    }

}

pub fn main() {
    let mut app = match LoomzApplication::init() {
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
