mod pipeline_compiler;
mod world;
mod gui;
mod record;

use std::path::PathBuf;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use loomz_engine_core::LoomzEngineCore;
use loomz_shared::{backend_init_err, CommonError, api::LoomzApi};

pub struct LoomzEngine {
    api: LoomzApi,
    core: LoomzEngineCore,
    world: world::WorldModule,
    gui: gui::GuiModule,
    pipeline_cache: vk::PipelineCache,
}

impl LoomzEngine {

    pub fn init(api: &LoomzApi) -> Result<Self, CommonError> {
        let mut core = LoomzEngineCore::init()?;
        let world = world::WorldModule::init(&mut core, api)?;
        let gui = gui::GuiModule::init(&mut core, api)?;
        let pipeline_cache = Self::load_pipeline_cache(&core)?;
        let mut engine = LoomzEngine {
            api: api.clone(),
            core,
            world,
            gui,
            pipeline_cache,
        };

        engine.compile_pipelines()?;

        Ok(engine)
    }

    pub fn destroy(mut self) {
        self.core.ctx.device.device_wait_idle().unwrap();

        self.store_pipeline_cache();

        self.core.ctx.device.destroy_pipeline_cache(self.pipeline_cache);
        self.world.destroy(&mut self.core);
        self.gui.destroy(&mut self.core);
        self.core.destroy();
    }

    pub fn set_output(&mut self, display: RawDisplayHandle, window: RawWindowHandle, window_size: [u32; 2]) -> Result<(), CommonError> {
        self.core.set_output(display, window, window_size)?;
        self.world.set_output(&self.core);
        self.gui.set_output(&self.core);
        Ok(())
    }

    pub fn resize_output(&mut self, width: u32, height: u32) -> Result<(), CommonError> {
        self.core.resize_output(width, height)?;
        self.world.rebuild(&self.core);
        self.gui.rebuild(&self.core);
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), CommonError> {
        self.world.update(&self.api, &mut self.core)?;
        self.gui.update(&self.api, &mut self.core)?;
        Ok(())
    }

    pub fn render(&mut self) -> Result<(), CommonError> {
        use loomz_engine_core::AcquireReturn;

        match self.core.acquire_frame()? {
            AcquireReturn::Invalid => {},
            AcquireReturn::Rebuild => {
                self.world.rebuild(&self.core);
                self.gui.rebuild(&self.core);
            },
            AcquireReturn::Render => {
                record::record_commands(self)?;
                self.core.submit_frame()?;
            }
        }

        Ok(())
    }

    fn compile_pipelines(&mut self) -> Result<(), CommonError> {
        let mut compiler = pipeline_compiler::PipelineCompiler::new();
        self.world.write_pipeline_create_infos(&mut compiler);
        self.gui.write_pipeline_create_infos(&mut compiler);
        compiler.compile_pipelines(self.pipeline_cache, &self.core.ctx)?;
        self.world.set_pipeline_handle(&compiler);
        self.gui.set_pipeline_handle(&compiler);
        Ok(())
    }

    fn load_pipeline_cache(core: &LoomzEngineCore) -> Result<vk::PipelineCache, CommonError> {
        let mut info = vk::PipelineCacheCreateInfo::default();
    
        let mut cache_data: Option<Vec<u8>> = None;

        let cache_path = Self::pipeline_cache_path();
        if let Ok(data) = ::std::fs::read(cache_path) {
            cache_data = Some(data);
        }

        if let Some(data) = cache_data {
            info.initial_data_size = data.len();
            info.p_initial_data = data.as_ptr() as *const _;
        }
        
        core.ctx.device.create_pipeline_cache(&info)
            .map_err(|err| backend_init_err!("Failed to create pipeline cache: {}", err) )
    }

    fn store_pipeline_cache(&self) {
        let data = match self.core.ctx.device.get_pipeline_cache_data(self.pipeline_cache) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Failed to get pipeline cache data: {err:?}");
                return;
            }
        };

        let path = Self::pipeline_cache_path();
        if let Err(err) = ::std::fs::write(path.as_path(), &data) {
            eprintln!("Failed to save pipeline data: {err:?}");
        }
    }

    fn pipeline_cache_path() -> PathBuf {
        let local_readonly = ::std::fs::metadata(".")
            .map(|md| md.permissions().readonly() )
            .unwrap_or(true);

        if local_readonly {
            let mut base = ::std::env::temp_dir();
            base.push("loomz-engine");
            base.push("pipeline.cache");
            base
        } else {
            PathBuf::from("./pipeline.cache")
        }
    }

}
