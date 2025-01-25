use loomz_shared::{CommonError, assets_err};
use loomz_engine_core::LoomzEngineCore;
use super::WorldModule;

impl WorldModule {

    fn load_terrain_asset(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let id = self.resources.assets.texture_id_by_name("terrain")
            .ok_or_else(|| assets_err!("Failed to find terrain asset") )?;

        self.fetch_texture_view(core, id)?;

        Ok(())
    }

    pub(super) fn setup_test_world(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        self.load_terrain_asset(core)?;

        Ok(())
    }

}
