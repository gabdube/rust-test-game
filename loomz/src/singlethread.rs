use loomz_shared::{CommonError, LoomzApi, SizeF32};
use loomz_engine::LoomzEngine;
use winit::window::Window;
use crate::LoomzClient;

/**
    The single threaded container works by calling synchronously the updates function 
    on the engine and the client, then calling engine render on redraw.
    
    Window resized are using a callback from the windowing module
*/
pub struct LoomzApplication {
    api: LoomzApi,
    window: Option<Box<Window>>,
    client: LoomzClient,
    engine: LoomzEngine,
    last_error: Option<CommonError>,
    initial_window_size: SizeF32
}

impl LoomzApplication {

    pub fn init() -> Result<Self, CommonError> {
        let initial_window_size = SizeF32 { width: 1200.0, height: 900.0 };
        let api = LoomzApi::init(initial_window_size)?;
        let client = LoomzClient::init(&api)?;
        let engine = LoomzEngine::init(&api)?;

        let app = LoomzApplication {
            api,
            window: None,
            client,
            engine,
            last_error: None,
            initial_window_size
        };

        Ok(app)
    }

    pub fn api(&self) -> &LoomzApi {
        &self.api
    }

    pub fn exit(self) {
        self.engine.destroy();
    }

    pub fn initial_window_size(&self) -> SizeF32 {
        self.initial_window_size
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
        Ok(())
    }

    pub fn redraw(&mut self) -> Result<(), CommonError> {
        self.engine.update()?;
        self.engine.render()?;
        self.window().request_redraw();
        Ok(())
    }

    pub fn resized(&mut self) -> Result<(), CommonError> {
        self.engine.resize_output()
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

    pub fn try_window(&self) -> Option<&Box<Window>> {
        self.window.as_ref()
    }

}

