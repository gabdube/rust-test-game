use loomz_shared::{CommonError, LoomzApi};
use loomz_engine::LoomzEngine;
use winit::window::Window;
use crate::LoomzClient;

/**
    The single threaded container works by synchronously

       * calling `update` on the client and the engine
       * calling `render` on the engine
    
    Window resized are using a callback from the windowing module
*/
pub struct LoomzApplication {
    api: LoomzApi,
    window: Option<Box<Window>>,
    client: LoomzClient,
    engine: LoomzEngine,
    last_error: Option<CommonError>,
}

impl LoomzApplication {

    pub fn init() -> Result<Self, CommonError> {
        let api = LoomzApi::init()?;
        let client = LoomzClient::init(&api)?;
        let engine = LoomzEngine::init(&api)?;

        let app = LoomzApplication {
            api,
            window: None,
            client,
            engine,
            last_error: None,
        };

        Ok(app)
    }

    pub fn api(&self) -> &LoomzApi {
        &self.api
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

    pub fn update(&mut self) -> Result<(), CommonError> {
        self.client.update()?;
        self.engine.update()?;
        Ok(())
    }

    pub fn redraw(&mut self) -> Result<(), CommonError> {
        self.engine.render()?;
        self.window().request_redraw();
        Ok(())
    }

    pub fn resized(&mut self, width: u32, height: u32) -> Result<(), CommonError> {
        self.engine.resize_output(width, height)
    }

    pub fn last_error(&mut self) -> Option<CommonError> {
        self.last_error.take()
    }

    pub fn set_last_error(&mut self, error: CommonError) {
        self.last_error = Some(error);
    }

    pub fn window(&self) -> &Window {
        match self.window.as_ref() {
            Some(window) => window,
            None => unreachable!("Window will always be some at runtime")
        }
    }

}

