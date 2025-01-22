use parking_lot::Mutex;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use loomz_shared::CommonError;

struct InnerMutexData {
    last_error: Option<CommonError>,
}

struct InnerData {
    data: Mutex<InnerMutexData>,
    exit: AtomicBool,
}

/// Synchronization between the container app "loomz" and the client/engine threads
pub struct LoomzMultithreadedShared {
    inner: Arc<InnerData>
}

impl LoomzMultithreadedShared {

    pub fn new() -> Self {
        let inner = InnerData {
            data: Mutex::new(InnerMutexData {
                last_error: None,
            }),
            exit: AtomicBool::new(false)
        };

        LoomzMultithreadedShared {
            inner: Arc::new(inner)
        }
    }

    pub fn exit(&self) {
        self.inner.exit.store(true, Ordering::SeqCst)
    }

    pub fn running(&self) -> bool {
        self.inner.exit.load(Ordering::SeqCst) == false
    }

    pub fn last_error(&self) -> Result<(), CommonError> {
        let mut data = self.inner.data.lock();
        match data.last_error.take() {
            Some(err) => Err(err),
            None => Ok(())
        }
    }

    pub fn set_last_error(&self, error: CommonError) {
        let mut data = self.inner.data.lock();
        if data.last_error.is_none() {
            data.last_error = Some(error);
        }
    }

}

impl Clone for LoomzMultithreadedShared {
    fn clone(&self) -> Self {
        LoomzMultithreadedShared {
            inner: self.inner.clone(),
        }
    }
}
