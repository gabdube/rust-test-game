mod helpers;
mod context;
mod setup;

mod prepare;
mod record;
mod submit;

use loomz_shared::CommonError;
use context::VulkanContext;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

/// Regroup common engine information in one single place
pub struct VulkanEngineInfo {
    pub graphics_queue_info: vk::wrapper::QueueInfo,

    pub swapchain_extent: vk::Extent2D,
    pub swapchain_format: vk::Format,
    pub swapchain_image_count: u32,

    pub sample_count: vk::SampleCountFlags,
    pub color_format: vk::Format,
    pub depth_format: vk::Format,
}

/// Holder for the shared gpu resources used in the engine
pub struct VulkanGlobalResources {
    pub command_pool: vk::CommandPool,
    pub drawing_command_buffers: [vk::CommandBuffer; 1],
    pub surface: vk::SurfaceKHR,
    pub attachments: helpers::RenderAttachments,
}

/// Data used when recording GPU commands
pub struct VulkanRecordingInfo {
    pub drawing_command_buffer: vk::CommandBuffer,
    pub output_image: vk::Image,
    pub extent: vk::Extent2D,
    pub color_attachment: vk::RenderingAttachmentInfo,
    pub depth_attachment: vk::RenderingAttachmentInfo,
}

pub struct VulkanSubmitInfo {
    pub render_commands_submit: vk::CommandBufferSubmitInfo,
    pub render_semaphore_wait: [vk::SemaphoreSubmitInfo; 1],
    pub render_semaphore_signal: [vk::SemaphoreSubmitInfo; 2],
    pub submit_infos: [vk::SubmitInfo2; 1],
    pub graphics_queue: vk::Queue,
}

pub struct VulkanOutputInfo {
    pub queue: vk::Queue,
    pub swapchain: vk::SwapchainKHR,
    pub output_attachment_ready: vk::Semaphore,
    pub output_present_ready: vk::Semaphore,
    pub drawings_sync: helpers::TimelineSemaphore,
    pub acquired_image_index: u32,
    pub rebuild: bool,
}

pub struct LoomzEngine {
    ctx: Box<VulkanContext>,
    info: Box<VulkanEngineInfo>,
    resources: Box<VulkanGlobalResources>,
    recording: Box<VulkanRecordingInfo>,
    submit: Box<VulkanSubmitInfo>,
    output: Box<VulkanOutputInfo>,
}

impl LoomzEngine {

    pub fn init() -> Result<Self, CommonError> {
        let mut setup = setup::VulkanEngineSetup::build()?;
        let engine = LoomzEngine {
            ctx: setup.ctx(),
            info: setup.info(),
            resources: setup.resources(),
            recording: setup.recording(),
            submit: setup.submit(),
            output: setup.output(),
        };

        Ok(engine)

    }

    pub fn destroy(self) {
        let mut ctx = self.ctx;
        ctx.device.device_wait_idle().unwrap();

        ctx.extensions.swapchain.destroy_swapchain(self.output.swapchain);
        ctx.device.destroy_semaphore(self.output.output_attachment_ready);
        ctx.device.destroy_semaphore(self.output.output_present_ready);
        ctx.device.destroy_semaphore(self.output.drawings_sync.handle);

        ctx.device.destroy_command_pool(self.resources.command_pool);
        ctx.extensions.surface.destroy_surface(self.resources.surface);
        self.resources.attachments.free(&ctx.device);

        ctx.device.destroy();
        ctx.instance.destroy();
    }

    pub fn update(&mut self) {

    }

    pub fn render(&mut self) -> Result<(), CommonError> {
        use prepare::AcquireReturn;

        match prepare::acquire_frame(self)? {
            AcquireReturn::Invalid => {},
            AcquireReturn::Rebuild => {
                self.ctx.device.device_wait_idle().unwrap();
                setup::setup_target::rebuild_target(self)?;
            },
            AcquireReturn::Render => {
                record::record_commands(self)?;
                submit::submit(self)?;
            }
        }
        
        Ok(())
    }

    pub fn set_output(&mut self, display: RawDisplayHandle, window: RawWindowHandle) -> Result<(), CommonError> {
        let params = crate::setup::setup_target::SetupTargetParams {
            display,
            window,
        };

        crate::setup::setup_target::setup_target(self, &params)?;

        Ok(())
    }

}
