mod sync_data;
use sync_data::LoomzMultithreadedShared;

use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use loomz_shared::{CommonError, LoomzApi, SizeF32, system_err};
use loomz_engine::LoomzEngine;
use winit::window::Window;
use crate::LoomzClient;

// Game container is running sync until the window is created
pub struct LoomzApplicationSetup {
    api: LoomzApi,
    window: Option<Box<Window>>,
    client: LoomzClient,
    engine: LoomzEngine,
    last_error: Option<CommonError>,
    initial_window_size: SizeF32,
}

pub struct LoomzApplicationRuntime {
    api: LoomzApi,
    window: Option<Box<Window>>,
    last_error: Option<CommonError>,
    shared: LoomzMultithreadedShared,
    client_thread: JoinHandle<()>,
    engine_thread: JoinHandle<()>,
}

/**
    The multithreaded engine
*/
pub enum LoomzApplication {
    Setup(Option<Box<LoomzApplicationSetup>>),
    Runtime(LoomzApplicationRuntime)
}

impl LoomzApplication {

    pub fn init() -> Result<Self, CommonError> {
        let initial_window_size = SizeF32 { width: 1200.0, height: 900.0 };
        let api = LoomzApi::init(initial_window_size)?;
        let client = LoomzClient::init(&api)?;
        let engine = LoomzEngine::init(&api)?;

        let setup = LoomzApplicationSetup {
            api,
            window: None,
            client,
            engine,
            last_error: None,
            initial_window_size
        };

        Ok(LoomzApplication::Setup(Some(Box::new(setup))))
    }

    fn finalize_application_setup(&mut self) -> Result<(), CommonError> {
        let setup = match self {
            Self::Setup(setup) => setup.take().expect("Setup will always be Some"),
            Self::Runtime(_) => panic!("Application state cannot be runtime"),
        };

        let shared = LoomzMultithreadedShared::new();

        let client = setup.client;
        let client_shared = shared.clone();
        let client_thread = ::std::thread::Builder::new()
            .name("client".into())
            .spawn(move || client_loop(client, client_shared) )
            .map_err(|err| system_err!("Failed to start client thread: {err}") )?;

        let engine = setup.engine;
        let engine_shared = shared.clone();
        let engine_thread = ::std::thread::Builder::new()
            .name("engine".into())
            .spawn(move || engine_loop(engine, engine_shared) )
            .map_err(|err| system_err!("Failed to start engine thread: {err}") )?;

        *self = LoomzApplication::Runtime(LoomzApplicationRuntime {
            api: setup.api,
            window: setup.window,
            last_error: None,
            shared,
            client_thread,
            engine_thread,
        });

        Ok(())
    }

    pub fn initial_window_size(&self) -> SizeF32 {
        match self {
            LoomzApplication::Setup(Some(setup)) => {
                setup.initial_window_size
            },
            LoomzApplication::Runtime(run) => {
                SizeF32::default()
            },
            _ => unreachable!(),
        }
    }

    pub fn set_window(&mut self, window: Window) -> Result<(), CommonError> {
        use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

        let setup = match self {
            Self::Setup(Some(setup)) => setup,
            Self::Runtime(_) => panic!("set_window should only be called once at startup"),
            _ => unreachable!(),
        };

        let (display_handle, window_handle) = window.display_handle()
            .and_then(|display| window.window_handle().map(|window| (display, window) ) )
            .map(|(display, window)| (display.as_raw(), window.as_raw()) )
            .map_err(|err| loomz_shared::system_err!("Failed to get system handles: {}", err) )?;

        let size = window.inner_size();
        setup.engine.set_output(display_handle, window_handle, [size.width, size.height])?;
        window.set_visible(true);
        setup.window = Some(Box::new(window));
        
        self.finalize_application_setup()?;

        Ok(())
    }

    pub fn exit(self) {
        match self {
            LoomzApplication::Setup(Some(setup)) => {
                setup.engine.destroy();
            },
            LoomzApplication::Runtime(run) => {
                run.shared.exit();
                run.client_thread.join().unwrap();
                run.engine_thread.join().unwrap();
            },
            _ => unreachable!(),
        }
    }

    pub fn update(&mut self) -> Result<(), CommonError> {
        // Nothing to do here. Last error from threads is collected in `redraw`
        Ok(())
    }

    pub fn redraw(&mut self) -> Result<(), CommonError> {
        match self {
            Self::Runtime(run) => {
                run.shared.last_error()
            },
            _ => Ok(())
        }
    }

    pub fn resized(&mut self, _width: u32, _height: u32) -> Result<(), CommonError> {
        // TODO. Engine thread can resize itself, but we should send a message just to be sure
        Ok(())
    }

    pub fn api(&self) -> &LoomzApi {
        match self {
            LoomzApplication::Setup(Some(setup)) => &setup.api,
            LoomzApplication::Runtime(run) => &run.api,
            _ => unreachable!(),
        }
    }

    pub fn last_error(&mut self) -> Option<CommonError> {
        match self {
            LoomzApplication::Setup(Some(setup)) => {
                setup.last_error.take()
            },
            LoomzApplication::Runtime(run) => {
                run.last_error.take()
            },
            _ => unreachable!(),
        }
    }

    pub fn set_last_error(&mut self, error: CommonError) {
        match self {
            LoomzApplication::Setup(Some(setup)) => {
                setup.last_error = Some(error);
            },
            LoomzApplication::Runtime(run) => {
                run.last_error = Some(error);
            },
            _ => unreachable!(),
        }
    }

    pub fn window(&self) -> &Window {
        let window = match self {
            LoomzApplication::Setup(Some(setup)) => setup.window.as_ref(),
            LoomzApplication::Runtime(run) => run.window.as_ref(),
            _ => unreachable!(),
        };

        match window {
            Some(window) => window,
            None => unreachable!("Window will always be some at runtime")
        }
    }

}

fn client_loop(client: LoomzClient, shared: LoomzMultithreadedShared) {
    let mut client = client;
    let shared = shared;

    while shared.running() {
        if let Err(error) = client.update() {
            shared.set_last_error(error);
            break;
        }

        // Without sleeping, the client thread will flood the engine thread
        // There is a better way to do this (todo)
        sleep(Duration::from_millis(4));
    }
}

fn engine_loop(engine: LoomzEngine, shared: LoomzMultithreadedShared) { 
    let mut engine = engine;
    let shared = shared;

    while shared.running() {
        let result = engine.update()
            .and_then(|_| engine.render() );

        if let Err(error) = result {
            shared.set_last_error(error);
            break;
        }
    }
}
