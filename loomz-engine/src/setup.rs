mod setup_ctx;
mod setup_resources;
pub mod setup_target;

use loomz_shared::CommonError;
use crate::context::VulkanContext;
use super::{VulkanEngineInfo, VulkanGlobalResources, VulkanRecordingInfo, VulkanSubmitInfo, VulkanOutputInfo};

pub(crate) struct VulkanEngineSetup {
    ctx: Option<Box<VulkanContext>>,
    info: Option<Box<VulkanEngineInfo>>,
    resources: Option<Box<VulkanGlobalResources>>,
    recording: Option<Box<VulkanRecordingInfo>>,
    submit: Option<Box<VulkanSubmitInfo>>,
    output: Option<Box<VulkanOutputInfo>>,
}

impl VulkanEngineSetup {

    pub fn build() -> Result<Self, CommonError> {
        let mut setup = VulkanEngineSetup {
            ctx: None,
            info: None,
            resources: None,
            recording: None,
            submit: None,
            output: None,
        };

        setup_ctx::setup(&mut setup)?;
        setup_resources::setup(&mut setup)?;

        Ok(setup)
    }

    pub fn ctx(&mut self) -> Box<VulkanContext> {
        match self.ctx.take() {
            Some(ctx) => ctx,
            None => unreachable!("Context will always be initialized during build function")
        }
    }

    pub fn info(&mut self) -> Box<VulkanEngineInfo> {
        match self.info.take() {
            Some(resources) => resources,
            None => unreachable!("Engine info will always be initialized during build function")
        }
    }

    pub fn resources(&mut self) -> Box<VulkanGlobalResources> {
        match self.resources.take() {
            Some(resources) => resources,
            None => unreachable!("Resources will always be initialized during build function")
        }
    }

    pub fn recording(&mut self) -> Box<VulkanRecordingInfo> {
        match self.recording.take() {
            Some(recording) => recording,
            None => unreachable!("Recording info will always be initialized during build function")
        }
    }

    pub fn submit(&mut self) -> Box<VulkanSubmitInfo> {
        match self.submit.take() {
            Some(submit) => submit,
            None => unreachable!("Submit info will always be initialized during build function")
        }
    }

    pub fn output(&mut self) -> Box<VulkanOutputInfo> {
        match self.output.take() {
            Some(output) => output,
            None => unreachable!("Output info will always be initialized during build function")
        }
    }

}
