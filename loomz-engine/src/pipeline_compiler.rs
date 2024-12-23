use loomz_engine_core::{VulkanContext, pipelines::GraphicsPipeline};
use loomz_shared::{CommonError, backend_init_err};

pub struct PipelineCompiler {
    pipeline_infos: Vec<vk::GraphicsPipelineCreateInfo>,
    pipelines: Vec<vk::Pipeline>,
    map: fnv::FnvHashMap<&'static str, usize>
}

impl PipelineCompiler {

    pub fn new() -> PipelineCompiler {
        PipelineCompiler {
            pipeline_infos: Vec::with_capacity(4),
            pipelines: Vec::new(),
            map: fnv::FnvHashMap::default(),
        }
    }

    /// SAFETY: pipeline lifetime must not outlive PipelineCompiler
    pub fn add_pipeline_info(&mut self, label: &'static str, pipeline: &mut GraphicsPipeline) {
        self.map.insert(label, self.pipeline_infos.len());
        self.pipeline_infos.push(pipeline.create_info());
    }

    pub fn get_pipeline(&self, label: &'static str) -> vk::Pipeline {
        let index = self.map.get(label).copied().expect("label not found");
        self.pipelines[index]
    }

    pub fn compile_pipelines(&mut self, cache: vk::PipelineCache, ctx: &VulkanContext) -> Result<(), CommonError> {
        self.pipelines = vec![vk::Pipeline::null(); self.pipeline_infos.len()];
        ctx.device.create_graphics_pipelines(cache, &self.pipeline_infos, &mut self.pipelines)
            .map_err(|err| backend_init_err!("Failed to compile create pipelines: {:?}", err) )
    }

}
