mod record;

use loomz_engine_core::LoomzEngineCore;
use loomz_shared::CommonError;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub struct LoomzEngine {
    core: LoomzEngineCore
}

impl LoomzEngine {

    pub fn init() -> Result<Self, CommonError> {
        let core = LoomzEngineCore::init()?;
        let engine = LoomzEngine {
            core
        };

        Ok(engine)
    }

    pub fn destroy(self) {
        self.core.destroy();
    }

    pub fn set_output(&mut self, display: RawDisplayHandle, window: RawWindowHandle) -> Result<(), CommonError> {
        self.core.set_output(display, window)
    }

    pub fn update(&mut self) {

    }

    pub fn render(&mut self) -> Result<(), CommonError> {
        let acquired = self.core.acquire_frame()?;
        if acquired {
            record::record_commands(self)?;
            self.core.submit_frame()?;
        }

        Ok(())
    }

}
