use std::{path::PathBuf, sync::mpsc, sync::atomic::{AtomicBool, Ordering}};
use libloading::Library;
use loomz_shared::{system_err, undefined_err, CommonError, LoomzApi};

type InitSym = unsafe extern fn(&LoomzApi);
type UpdateSym = unsafe extern fn();
type ExportClientSym = unsafe extern fn(&mut usize, &mut Option<*mut u8>);
type ImportClientSym = unsafe extern fn(&LoomzApi, &Box<[u8]>);
type LastErrorSym = unsafe extern fn(&mut Option<CommonError>);

static RELOAD: AtomicBool = AtomicBool::new(false);

struct ClientLibrary {
    handle: Library,
    init: InitSym,
    update: UpdateSym,
    export: ExportClientSym,
    import: ImportClientSym,
    last_err: LastErrorSym,
}

impl ClientLibrary {

    fn init(&self, api: &LoomzApi) -> Result<(), CommonError> {
        unsafe { (self.init)(api) };
        self.get_last_error()
    }

    fn init_from_data(&self, api: &LoomzApi, data: &Box<[u8]>) -> Result<(), CommonError> {
        unsafe { (self.import)(api, data) };
        self.get_last_error()
    }

    fn update(&self) -> Result<(), CommonError> {
        unsafe { (self.update)() };
        self.get_last_error()
    }

    fn unload(self) -> Result<Box<[u8]>, CommonError> {
        let mut session_size = 0;
        let mut session_data = None;

        unsafe { (self.export)(&mut session_size, &mut session_data) };
        if let Err(e) = self.get_last_error() {
            return Err(e);
        }

        // Copy the session bytes into local memory
        // The original memory should be automatically freed with the library
        let session_data_ptr = session_data.expect("Session data ptr should always be some");
        let mut local_data = vec![0u8; session_size];

        unsafe {
            ::std::ptr::copy_nonoverlapping(session_data_ptr, local_data.as_mut_ptr(), session_size);
        }

        drop(self.handle);

        Ok(local_data.into_boxed_slice())
    }

    fn get_last_error(&self) -> Result<(), CommonError> {
        let mut err = None;
        unsafe { (self.last_err)(&mut err) };
        match err {
            Some(e) => Err(e),
            None => Ok(())
        }
    }

}

/// Hot reloaded interface for the client code
pub struct LoomzClient {
    api: LoomzApi,
    client_library: Option<ClientLibrary>,
}

impl LoomzClient {

    pub fn init(api: &LoomzApi) -> Result<Self, CommonError> {
        let client_library = Self::load_library()?;
        client_library.init(api)?;

        Self::setup_watcher()?;

        let client = LoomzClient {
            api: api.clone(),
            client_library: Some(client_library),
        };

        println!("Client initialized with hot reload");

        Ok(client)
    }

    pub fn update(&mut self) -> Result<(), CommonError> {
        self.try_reload()?;

        let lib = self.client_library.as_ref()
            .ok_or_else(|| undefined_err!("Client library should always be Some") )?;

        lib.update()
    }

    fn try_reload(&mut self) -> Result<(), CommonError> {
        if !RELOAD.fetch_and(false, Ordering::SeqCst) {
            return Ok(());
        }

        let data = self.client_library.take()
            .ok_or_else(|| undefined_err!("Client library should always be Some") )?
            .unload()?;

        let lib = Self::load_library()?;
        lib.init_from_data(&self.api, &data)?;

        self.client_library = Some(lib);
        
        println!("CLIENT RELOADED");

        Ok(())
    }

    fn load_library() -> Result<ClientLibrary, CommonError> {
        let lib_path = lib_path()?;
        let handle = unsafe { libloading::Library::new(lib_path) }
            .map_err(|err| system_err!("Failed to load client library: {err:?}") )?;

        let init: InitSym = unsafe { handle.get(b"init_client\0") }.map(|s| *s)
            .map_err(|err| system_err!("Failed to fetch `init_client` function for client library: {err:?}") )?;

        let update: UpdateSym = unsafe { handle.get(b"update_client\0") }.map(|s| *s)
            .map_err(|err| system_err!("Failed to fetch `update_client` function for client library: {err:?}") )?;

        let export: ExportClientSym = unsafe { handle.get(b"export_client\0") }.map(|s| *s)
            .map_err(|err| system_err!("Failed to fetch `export_client` function for client library: {err:?}") )?;

        let import: ImportClientSym = unsafe { handle.get(b"import_client\0") }.map(|s| *s)
            .map_err(|err| system_err!("Failed to fetch `import_client` function for client library: {err:?}") )?;

        let last_err: LastErrorSym = unsafe { handle.get(b"last_error\0") }.map(|s| *s)
            .map_err(|err| system_err!("Failed to fetch `last_error` function for client library: {err:?}") )?;

        let lib = ClientLibrary {
            handle,
            init,
            update,
            export,
            import,
            last_err,
        };

        Ok(lib)
    }

    fn setup_watcher() -> Result<(), CommonError> {
        use notify::{Event, RecursiveMode, Result, Watcher};
        use std::{thread, time};

        let (sender, receiver) = mpsc::channel::<Result<Event>>();
        let mut watcher = notify::recommended_watcher(sender)
            .map_err(|err| system_err!("Failed to create watcher: {err:?}") )?;

        let mut watch_path = PathBuf::new();
        watch_path.push("target");
        watch_path.push(match cfg!(debug_assertions) {
            true => "debug",
            false => "release",
        });

        thread::spawn(move || {
            let name = lib_name();
            watcher.watch(&watch_path, RecursiveMode::NonRecursive).unwrap();
            'outer: loop {
                let mut reload = false;

                loop {
                    match receiver.recv_timeout(time::Duration::from_millis(100)) {
                        Ok(Ok(event)) => {
                            if is_reload_event(&name, &event) {
                                reload = true;
                            }
                        },
                        Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Disconnected) => { break 'outer; },
                        Err(mpsc::RecvTimeoutError::Timeout) => { break; },
                    }
                }

                if reload {
                    RELOAD.store(true, Ordering::SeqCst);
                }
            }
        });

        Ok(())
    }
}

fn lib_name() -> String {
    use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
    format!("{DLL_PREFIX}loomz_client{DLL_SUFFIX}")
}

fn lib_path() -> Result<PathBuf, CommonError> {
    let shadow_dir = PathBuf::from("./target/shadow");
    if !shadow_dir.exists() {
        ::std::fs::create_dir(&shadow_dir).unwrap();
    }

    let name = lib_name();

    let mut src = match cfg!(debug_assertions) {
        true => PathBuf::from("./target/debug"),
        false => PathBuf::from("./target/debug"),
    };
    src.push(&name);

    let mut dst = shadow_dir.clone();
    dst.push(&name);

    if dst.exists() {
        ::std::fs::remove_file(&dst)
            .map_err(|err| system_err!("Failed to remove old library from destination folder: {err}"))?;
    }

    ::std::fs::copy(&src, &dst)
        .map_err(|err| system_err!("Failed to copy library to temporary folder: {err}"))?;

    Ok(dst)
}

fn is_reload_event(lib_name: &str, event: &notify::Event) -> bool {
    match event.kind {
        notify::EventKind::Modify(_) => {},
        notify::EventKind::Create(_) => {},
        _ => { return false; }
    }

    let mut update = false;
    for path in event.paths.iter() {
        let update_name = path.file_name()
            .and_then(|name| name.to_str() )
            .unwrap_or("UNKNOWN");

        update |= update_name == lib_name;
    }

    update
}
