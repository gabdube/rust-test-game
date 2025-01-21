mod windowing;

#[cfg(feature="hot-reload")]
mod hot_reload;

#[cfg(feature="multithreading")]
mod multithreading;

use loomz_shared::{system_err, CommonError, LoomzApi};
use loomz_engine::LoomzEngine;
use winit::window::Window;

#[cfg(feature="multithreading")]
use multithreading::LoomzMultithreadedShared;

#[cfg(not(feature="hot-reload"))]
use loomz_client::LoomzClient;

#[cfg(feature="hot-reload")]
use hot_reload::LoomzClient;

#[cfg(not(feature="multithreading"))]
pub struct LoomzApplication {
    api: LoomzApi,
    window: Option<Box<Window>>,
    client: LoomzClient,
    engine: LoomzEngine,
    last_error: Option<CommonError>,
}

#[cfg(not(feature="multithreading"))]
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

}


#[cfg(feature="multithreading")]
pub struct LoomzApplication {
    api: LoomzApi,
    window: Option<Box<Window>>,
    shared: multithreading::LoomzMultithreadedShared,
    client_thread: ::std::thread::JoinHandle<()>,
    engine_thread: ::std::thread::JoinHandle<()>,
    last_error: Option<CommonError>,
}

#[cfg(feature="multithreading")]
impl LoomzApplication {
    pub fn init() -> Result<Self, CommonError> {
        let api = LoomzApi::init()?;

        let shared = LoomzMultithreadedShared::new();

        let client = LoomzClient::init(&api)?;
        let client_shared = shared.clone();
        let client_thread = ::std::thread::Builder::new()
            .name("client".into())
            .spawn(move || Self::client_loop(client, client_shared) )
            .map_err(|err| system_err!("Failed to start client thread: {err}") )?;

        let engine = LoomzEngine::init(&api)?;
        let engine_shared = shared.clone();
        let engine_thread = ::std::thread::Builder::new()
            .name("engine".into())
            .spawn(move|| Self::engine_loop(engine, engine_shared) )
            .map_err(|err| system_err!("Failed to start engine thread: {err}") )?;

        let app = LoomzApplication {
            api,
            window: None,
            shared,
            client_thread,
            engine_thread,
            last_error: None,
        };

        Ok(app)
    }

    fn client_loop(client: LoomzClient, shared: LoomzMultithreadedShared) {
        let mut client = client;

        while shared.waiting_for_window() {
            if shared.exit() { return; }

            ::std::thread::sleep(::std::time::Duration::from_millis(1));
        }

        loop {
            if let Err(error) = client.update() {
                shared.set_last_error(error);
                break;
            }

            if shared.exit() {
                break;
            } else {
                ::std::thread::sleep(::std::time::Duration::from_millis(1));
            }
        }
    }

    fn engine_loop(engine: LoomzEngine, shared: LoomzMultithreadedShared) {
        let mut engine = engine;

        while shared.waiting_for_window() {
            if shared.exit() { return; }

            ::std::thread::sleep(::std::time::Duration::from_millis(1));
        }

        loop {
            if let Err(error) = engine.update() {
                shared.set_last_error(error);
                break;
            }

            if let Err(error) = engine.render() {
                shared.set_last_error(error);
                break;
            }
            
            if shared.exit() {
                break;
            } else {
                ::std::thread::sleep(::std::time::Duration::from_millis(1));
            }
        }
    }

    fn exit(self) {
        self.shared.set_exiting();
        self.client_thread.join().unwrap();
        self.engine_thread.join().unwrap();
    }

    pub fn set_window(&mut self, window: Window) -> Result<(), CommonError> {
        use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

        let (display_handle, window_handle) = window.display_handle()
            .and_then(|display| window.window_handle().map(|window| (display, window) ) )
            .map(|(display, window)| (display.as_raw(), window.as_raw()) )
            .map_err(|err| loomz_shared::system_err!("Failed to get system handles: {}", err) )?;

        let size = window.inner_size();
        //self.engine.set_output(display_handle, window_handle, [size.width, size.height])?;

        window.set_visible(true);

        self.window = Some(Box::new(window));

        Ok(())
    }

    pub fn update(&mut self) -> Result<(), CommonError> {
        // Updates are handled independently by the threads
        self.shared.last_error()
    }

    pub fn redraw(&mut self) -> Result<(), CommonError> {
        // Redraw are handled independenty by the graphics thread 
        self.window().request_redraw();
        self.shared.last_error()
    }

    pub fn resized(&mut self, width: u32, height: u32) -> Result<(), CommonError> {
        Ok(())
    }
}

impl LoomzApplication {
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
