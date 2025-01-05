mod helpers;
mod context;
mod setup;

mod prepare;
mod upload;
mod submit;

pub mod pipelines;
pub mod descriptors;
pub mod alloc;
pub mod staging;
mod texture;

pub use context::VulkanContext;
pub use prepare::AcquireReturn;
pub use texture::Texture;

use std::sync::Arc;
use parking_lot::Mutex;
use loomz_shared::CommonError;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use staging::VulkanStaging;

/// Regroup common engine information in one single place
pub struct VulkanEngineInfo {
    pub graphics_queue_info: vk::wrapper::QueueInfo,

    pub window_extent: vk::Extent2D,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain_format: vk::Format,
    pub swapchain_image_count: u32,

    pub sample_count: vk::SampleCountFlags,
    pub color_format: vk::Format,
    pub depth_format: vk::Format,

    pub storage_min_align: u32,
}

/// Holder for the shared gpu resources used in the engine
pub struct VulkanGlobalResources {
    pub linear_sampler: vk::Sampler,
    pub command_pool: vk::CommandPool,
    pub drawing_command_buffers: [vk::CommandBuffer; 1],
    pub upload_command_buffers: [vk::CommandBuffer; 1],
    pub surface: vk::SurfaceKHR,
    pub vertex_alloc: alloc::DeviceMemoryAlloc,
    pub images_alloc: alloc::DeviceMemoryAlloc,
    pub uniforms_alloc: alloc::HostVisibleAlloc,
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
    pub upload_semaphore_signal: [vk::SemaphoreSubmitInfo; 1],
    pub upload_commands_submit: vk::CommandBufferSubmitInfo,
    pub render_commands_submit: vk::CommandBufferSubmitInfo,
    pub render_semaphore_wait: [vk::SemaphoreSubmitInfo; 2],
    pub render_semaphore_signal: [vk::SemaphoreSubmitInfo; 2],
    pub submit_infos: [vk::SubmitInfo2; 2],
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
    pub valid: bool,
}

pub struct VulkanDescriptorSubmit {
    pub images: Box<[vk::DescriptorImageInfo]>,
    pub buffers: Box<[vk::DescriptorBufferInfo]>,
    pub writes: Box<[vk::WriteDescriptorSet]>,
    pub images_count: u32,
    pub buffers_count: u32,
    pub writes_count: u32,
}

impl VulkanDescriptorSubmit {

    pub fn write_buffer(
        &mut self,
        dst_set: vk::DescriptorSet,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        range: vk::DeviceSize,
        dst_binding: u32,
        descriptor_type: vk::DescriptorType
    ) {
        let buffer_index = self.buffers_count as usize;
        let write_index = self.writes_count as usize;

        let buffer = vk::DescriptorBufferInfo {
            buffer,
            offset,
            range,
        };
        
        let write_set = vk::WriteDescriptorSet {
            dst_set,
            dst_binding,
            descriptor_type,
            descriptor_count: 1,
            p_buffer_info: &self.buffers[buffer_index] as *const _,
            ..Default::default()
        };

        self.buffers[buffer_index] = buffer;
        self.writes[write_index] = write_set;

        self.writes_count += 1;
        self.buffers_count += 1;
    }

    pub fn write_image(
        &mut self,
        dst_set: vk::DescriptorSet,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
        image_layout: vk::ImageLayout,
        dst_binding: u32,
        descriptor_type: vk::DescriptorType
    ) {
        let image_index = self.images_count as usize;
        let write_index = self.writes_count as usize;

        let image = vk::DescriptorImageInfo {
            image_view,
            image_layout,
            sampler,
        };
        
        let write_set = vk::WriteDescriptorSet {
            dst_set,
            dst_binding,
            descriptor_type,
            descriptor_count: 1,
            p_image_info: &self.images[image_index] as *const _,
            ..Default::default()
        };

        self.images[image_index] = image;
        self.writes[write_index] = write_set;

        self.writes_count += 1;
        self.images_count += 1;
    }
}

pub struct LoomzEngineCore {
    pub ctx: Box<VulkanContext>,
    pub info: Box<VulkanEngineInfo>,
    pub resources: Box<VulkanGlobalResources>,
    pub recording: Box<VulkanRecordingInfo>,
    pub submit: Box<VulkanSubmitInfo>,
    pub output: Box<VulkanOutputInfo>,
    pub staging: Box<VulkanStaging>,
    pub descriptors: Arc<Mutex<VulkanDescriptorSubmit>>,
}

impl LoomzEngineCore {

    pub fn init() -> Result<Self, CommonError> {
        let mut setup = setup::VulkanEngineSetup::build()?;
        let engine = LoomzEngineCore {
            ctx: setup.ctx(),
            info: setup.info(),
            resources: setup.resources(),
            recording: setup.recording(),
            submit: setup.submit(),
            output: setup.output(),
            staging: setup.staging(),
            descriptors: setup.descriptors(),
        };

        Ok(engine)

    }

    pub fn destroy(self) {
        let mut ctx = self.ctx;

        ctx.extensions.swapchain.destroy_swapchain(self.output.swapchain);
        ctx.device.destroy_semaphore(self.output.output_attachment_ready);
        ctx.device.destroy_semaphore(self.output.output_present_ready);
        ctx.device.destroy_semaphore(self.output.drawings_sync.handle);

        ctx.device.destroy_command_pool(self.resources.command_pool);
        ctx.extensions.surface.destroy_surface(self.resources.surface);
        ctx.device.destroy_sampler(self.resources.linear_sampler);
        self.resources.attachments.free(&ctx.device);
        self.resources.vertex_alloc.free(&ctx.device);
        self.resources.images_alloc.free(&ctx.device);
        self.resources.uniforms_alloc.free(&ctx.device);
        self.staging.destroy(&ctx.device);

        ctx.device.destroy();
        ctx.instance.destroy();
    }

    pub fn acquire_frame(&mut self) -> Result<AcquireReturn, CommonError> {
        use prepare::AcquireReturn;

        if !self.output.valid {
            return Ok(AcquireReturn::Invalid);
        }

        let acquire = prepare::acquire_frame(self)?;

        if let AcquireReturn::Render = acquire {
            upload::upload(self)?;
        }

        if let AcquireReturn::Rebuild = acquire {
            self.ctx.device.device_wait_idle().unwrap();
            setup::setup_target::rebuild_target(self)?;
        }

        self.update_descriptor_sets();

        Ok(acquire)
    }

    pub fn submit_frame(&mut self) -> Result<(), CommonError> {
        submit::submit(self)
    }

    pub fn set_output(&mut self, display: RawDisplayHandle, window: RawWindowHandle, window_size: [u32; 2]) -> Result<(), CommonError> {
        self.output.valid = window_size[0] > 0 && window_size[1] > 0;
        if !self.output.valid {
            return Ok(());
        }

        let params = crate::setup::setup_target::SetupTargetParams {
            display,
            window,
        };

        self.info.window_extent = vk::Extent2D { width: window_size[0], height: window_size[1] };
        crate::setup::setup_target::setup_target(self, &params)?;

        Ok(())
    }

    pub fn resize_output(&mut self, width: u32, height: u32) -> Result<(), CommonError> {
        self.output.valid = width > 0 && height > 0;
        if !self.output.valid {
            return Ok(());
        }
        
        let current_extent = self.info.window_extent;
        if current_extent.width == width && current_extent.height == height {
            return Ok(());
        }

        self.ctx.device.device_wait_idle().unwrap();
        self.info.window_extent = vk::Extent2D { width, height };
        setup::setup_target::rebuild_target(self)?;

        Ok(())
    }

    fn update_descriptor_sets(&mut self) {
        let mut descriptors = self.descriptors.lock();
        if descriptors.writes_count == 0 {
            return;
        }

        let count = descriptors.writes_count as usize;
        self.ctx.device.update_descriptor_sets(&mut descriptors.writes[0..count], &[]);

        descriptors.buffers_count = 0;
        descriptors.images_count = 0;
        descriptors.writes_count = 0;
    }

}
