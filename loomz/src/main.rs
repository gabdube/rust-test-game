#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;


mod windowing;

#[cfg(feature="hot-reload")]
mod hot_reload;

#[cfg(feature="multithreading")]
mod multithread;

#[cfg(not(feature="multithreading"))]
mod singlethread;

#[cfg(not(feature="hot-reload"))]
use loomz_client::LoomzClient;

#[cfg(feature="hot-reload")]
use hot_reload::LoomzClient;

#[cfg(not(feature="multithreading"))]
use singlethread::LoomzApplication;

#[cfg(feature="multithreading")]
use multithread::LoomzApplication;


pub fn main() {
    let mut app = match LoomzApplication::init() {
        Ok(app) => { app },
        Err(e) => {
            eprintln!("{}", e);
            return
        }
    };

    windowing::run(&mut app);

    if let Some(err) = app.last_error() {
        eprintln!("{}", err);
    }

    app.exit();
}
