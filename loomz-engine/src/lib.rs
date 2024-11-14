mod world;
mod record;

use std::path::PathBuf;
use loomz_engine_core::LoomzEngineCore;
use loomz_shared::{backend_init_err, CommonError};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub struct LoomzEngine {
    core: LoomzEngineCore,
    world: world::WorldModule,
    pipeline_cache: vk::PipelineCache,
}

impl LoomzEngine {

    pub fn init() -> Result<Self, CommonError> {
        let mut core = LoomzEngineCore::init()?;
        let world = world::WorldModule::init(&mut core)?;
        let pipeline_cache = Self::load_pipeline_cache(&core)?;
        let engine = LoomzEngine {
            core,
            world,
            pipeline_cache
        };

        Ok(engine)
    }

    pub fn destroy(self) {
        self.core.ctx.device.device_wait_idle().unwrap();

        self.store_pipeline_cache();

        self.core.ctx.device.destroy_pipeline_cache(self.pipeline_cache);
        self.world.destroy(&self.core.ctx);
        self.core.destroy();
    }

    pub fn set_output(&mut self, display: RawDisplayHandle, window: RawWindowHandle) -> Result<(), CommonError> {
        self.core.set_output(display, window)?;
        self.compile_pipelines()?;
        
        Ok(())
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

    fn compile_pipelines(&mut self) -> Result<(), CommonError> {
        // Collect pipelines create info from modules
        const PIPELINE_BUILD_CAPACITY: usize = 4;
        let mut pipeline_infos: Vec<vk::GraphicsPipelineCreateInfo> = Vec::with_capacity(PIPELINE_BUILD_CAPACITY);
        pipeline_infos.push(self.world.pipeline().create_info());

        // Pipeline creation
        let device = &self.core.ctx.device;
        let mut pipelines = vec![vk::Pipeline::null(); pipeline_infos.len()];
        device.create_graphics_pipelines(self.pipeline_cache, &pipeline_infos, &mut pipelines)
            .map_err(|err| backend_init_err!("Failed to compile create pipelines: {:?}", err) )?;

        // Assign pipeline handle
        self.world.pipeline().set_handle(pipelines[0]);

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
