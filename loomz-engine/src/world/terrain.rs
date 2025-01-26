use loomz_shared::CommonError;
use loomz_engine_core::LoomzEngineCore;
use super::WorldModule;

impl WorldModule {

    pub(super) fn setup_test_world(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        Ok(())
    }

}
