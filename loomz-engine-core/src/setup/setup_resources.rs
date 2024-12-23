use std::u32;

use loomz_shared::{backend_init_err, CommonError};
use vk::CommandBufferSubmitInfo;
use crate::{context::VulkanContext, helpers, VulkanDescriptorSubmit, VulkanEngineInfo, VulkanGlobalResources, VulkanOutputInfo, VulkanRecordingInfo, VulkanStaging, VulkanSubmitInfo};
use super::VulkanEngineSetup;

pub(crate) fn setup(setup: &mut VulkanEngineSetup) -> Result<(), CommonError> {
    setup.info = Some(setup_info(setup)?);
    setup.resources = Some(init_resources(setup)?);
    setup.recording = Some(init_recording());
    setup.output = Some(init_output(setup)?);
    setup.submit = Some(init_submit(setup));
    setup.staging = Some(init_staging(setup)?);
    setup.descriptors = Some(init_descriptors());

    Ok(())
}

//
// Info
//

pub(crate) fn setup_info(setup: &mut VulkanEngineSetup) -> Result<Box<VulkanEngineInfo>, CommonError> {
    let ctx = setup.ctx.as_ref().unwrap();
    let graphics_queue_info = ctx.device.queues[0];

    // Note: swapchain info in set in `setup_target`
    let sample_count = max_sample(ctx);
    let color_format = color_format(ctx)?;
    let depth_format = depth_format(ctx)?;

    let device_info = ctx.instance.instance.get_physical_device_properties(ctx.device.physical_device);

    Ok(Box::new(VulkanEngineInfo {
        graphics_queue_info,
        window_extent: vk::Extent2D::default(),
        swapchain_extent: vk::Extent2D::default(),
        swapchain_format: vk::Format::UNDEFINED,
        swapchain_image_count: 0,
        color_format,
        depth_format,
        sample_count,
        storage_min_align: device_info.limits.min_storage_buffer_offset_alignment as u32,
    }))
}

fn max_sample(ctx: &VulkanContext) -> vk::SampleCountFlags {
    let samples = [
        vk::SampleCountFlags::TYPE_1,
        vk::SampleCountFlags::TYPE_2,
        vk::SampleCountFlags::TYPE_4,
        vk::SampleCountFlags::TYPE_8,
        vk::SampleCountFlags::TYPE_16,
    ];

    let limits = ctx.instance.instance.get_physical_device_properties(ctx.device.physical_device).limits;
    let supported_samples = limits.framebuffer_color_sample_counts;
    
    let mut max_sample = vk::SampleCountFlags::TYPE_1;
    for &sample in samples.iter().rev() {
        if sample & supported_samples == sample {
            max_sample = sample;
            break;
        }
    }

    max_sample
}

fn color_format(ctx: &VulkanContext) -> Result<vk::Format, CommonError> {
    let color_formats = [
        vk::Format::B8G8R8A8_UNORM,
        vk::Format::R8G8B8A8_UNORM,
    ];

    fn color_optimal(instance: &vk::wrapper::Instance, physical_device: vk::PhysicalDevice, format: vk::Format) -> bool {
        let format_properties = instance.get_physical_device_format_properties(physical_device, format);
        format_properties.optimal_tiling_features.contains(vk::FormatFeatureFlags::COLOR_ATTACHMENT)
    }

    let instance = &ctx.instance.instance;
    let physical_device = ctx.device.physical_device;
    color_formats.iter().find(|&&f| color_optimal(instance, physical_device, f) )
        .ok_or_else(|| backend_init_err!("Not color format supported") )
        .copied()
}

fn depth_format(ctx: &VulkanContext) -> Result<vk::Format, CommonError>  {
    let depth_formats = [vk::Format::D32_SFLOAT, vk::Format::D16_UNORM, vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT, vk::Format::D16_UNORM_S8_UINT];

    fn depth_optimal(instance: &vk::wrapper::Instance, physical_device: vk::PhysicalDevice, format: vk::Format) -> bool {
        let format_properties = instance.get_physical_device_format_properties(physical_device, format);
        format_properties.optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
    }

    let instance = &ctx.instance.instance;
    let physical_device = ctx.device.physical_device;
    depth_formats.iter().find(|&&f| depth_optimal(instance, physical_device, f) )
        .ok_or_else(|| backend_init_err!("Not depth stencil format supported") )
        .copied()
}


//
// Global resources
//

fn init_resources(setup: &mut VulkanEngineSetup) -> Result<Box<VulkanGlobalResources>, CommonError> {
    let mut resources = VulkanGlobalResources {
        command_pool: vk::CommandPool::null(),
        linear_sampler: vk::Sampler::null(),
        upload_command_buffers: [vk::CommandBuffer::null(); 1],
        drawing_command_buffers: [vk::CommandBuffer::null(); 1],
        surface: vk::SurfaceKHR::null(),
        vertex_alloc: crate::alloc::DeviceMemoryAlloc::default(),
        images_alloc: crate::alloc::DeviceMemoryAlloc::default(),
        uniforms_alloc: crate::alloc::HostVisibleAlloc::default(),
        attachments: helpers::RenderAttachments::default(),
    };

    setup_commands(setup, &mut resources)?;
    setup_vertex_memory(setup, &mut resources)?;
    setup_images_memory(setup, &mut resources)?;
    setup_uniforms_memory(setup, &mut resources)?;
    setup_samplers(setup, &mut resources)?;

    Ok(Box::new(resources))
}

fn setup_commands(setup: &mut VulkanEngineSetup, resources: &mut VulkanGlobalResources) -> Result<(), CommonError> {
    let ctx = setup.ctx.as_ref().unwrap();
    let info = setup.info.as_ref().unwrap();
    
    let create_info = vk::CommandPoolCreateInfo {
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT,
        queue_family_index: info                                                                .graphics_queue_info.family_index,
        ..Default::default()
    };

    resources.command_pool = ctx.device.create_command_pool(&create_info)
        .map_err(|err| backend_init_err!("Failed to create main command pool: {err}") )?;

    let mut command_buffers = [vk::CommandBuffer::null(); 2];
    let alloc_info = vk::CommandBufferAllocateInfo {
        level: vk::CommandBufferLevel::PRIMARY,
        command_pool: resources.command_pool,
        command_buffer_count: command_buffers.len() as u32,
        ..Default::default()
    };

    ctx.device.allocate_command_buffers(&alloc_info, &mut command_buffers)
        .map_err(|err| backend_init_err!("Failed to allocate command buffers: {err}") )?;

    resources.upload_command_buffers[0] = command_buffers[0];
    resources.drawing_command_buffers[0] = command_buffers[1];

    Ok(())
}

fn setup_vertex_memory(setup: &mut VulkanEngineSetup, resources: &mut VulkanGlobalResources) -> Result<(), CommonError> {
    use crate::alloc::{DeviceMemoryAlloc, KB};

    let ctx = setup.ctx.as_ref().unwrap();
    let instance = &ctx.instance.instance;
    let flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;
    let device_type_index = crate::helpers::fetch_memory_index(instance, ctx.device.physical_device, flags, flags)
        .ok_or_else(|| backend_init_err!("Failed to find memory type suitable for vertex") )?;
    
    let vertex_size = KB*100;
    let default_alloc_capacity = 16;
    
    resources.vertex_alloc = DeviceMemoryAlloc::new(&ctx.device, vertex_size, default_alloc_capacity, device_type_index)
        .map_err(|err| backend_init_err!("Failed to create vertex memory: {err}") )?;

    Ok(())
}

fn setup_images_memory(setup: &mut VulkanEngineSetup, resources: &mut VulkanGlobalResources) -> Result<(), CommonError> {
    use crate::alloc::{DeviceMemoryAlloc, MB};

    let ctx = setup.ctx.as_ref().unwrap();
    let instance = &ctx.instance.instance;
    let flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;
    let device_type_index = crate::helpers::fetch_memory_index(instance, ctx.device.physical_device, flags, flags)
        .ok_or_else(|| backend_init_err!("Failed to find memory type suitable for images") )?;

    let images_size = 5*MB;
    let default_alloc_capacity = 16;

    resources.images_alloc = DeviceMemoryAlloc::new(&ctx.device, images_size, default_alloc_capacity, device_type_index)
        .map_err(|err| backend_init_err!("Failed to create images memory: {err}") )?;

    Ok(())
}

fn setup_uniforms_memory(setup: &mut VulkanEngineSetup, resources: &mut VulkanGlobalResources) -> Result<(), CommonError> {
    use crate::alloc::{HostVisibleAlloc, KB};

    let ctx = setup.ctx.as_ref().unwrap();
    let instance = &ctx.instance.instance;
    let flags = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let host_type_index = crate::helpers::fetch_memory_index(instance, ctx.device.physical_device, flags, flags)
        .ok_or_else(|| backend_init_err!("Failed to find memory type suitable for uniforms") )?;

    let uniforms_size = KB*10;
    let default_alloc_capacity = 16;

    resources.uniforms_alloc = HostVisibleAlloc::new(&ctx.device, uniforms_size, default_alloc_capacity, host_type_index)
        .map_err(|err| backend_init_err!("Failed to create uniforms memory: {err}") )?;

    Ok(())
}

fn setup_samplers(setup: &mut VulkanEngineSetup, resources: &mut VulkanGlobalResources) -> Result<(), CommonError> {
    let ctx = setup.ctx.as_ref().unwrap();

    let create_info = vk::SamplerCreateInfo {
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        ..Default::default()
    };

    resources.linear_sampler = ctx.device.create_sampler(&create_info)
        .map_err(|err| backend_init_err!("Failed to create sampler: {err}") )?;

    Ok(())
}

//
// Recording
//

fn init_recording() -> Box<VulkanRecordingInfo> {
    let color_attachment = vk::RenderingAttachmentInfo {
        image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        resolve_mode: vk::ResolveModeFlagsBits::AVERAGE,
        resolve_image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        clear_value: vk::ClearValue::from(vk::ClearColorValue::from_f32(0.0, 0.0, 0.0, 0.0)),
        ..Default::default()
    };

    let depth_attachment = vk::RenderingAttachmentInfo {
        image_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        resolve_mode: vk::ResolveModeFlagsBits::NONE,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        clear_value: vk::ClearValue::from(vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 }),
        ..Default::default()
    };

    let recording_info = VulkanRecordingInfo {
        drawing_command_buffer: vk::CommandBuffer::null(),
        output_image: vk::Image::null(),
        extent: vk::Extent2D::default(),
        color_attachment,
        depth_attachment,
    };

    Box::new(recording_info)
}

//
// Output
//

fn init_output(setup: &mut VulkanEngineSetup) -> Result<Box<VulkanOutputInfo>, CommonError> {
    let ctx = setup.ctx.as_ref().unwrap();
    let info = setup.info.as_ref().unwrap();

    let drawings_sync = crate::helpers::TimelineSemaphore::new(&ctx.device)?;
    
    let output = VulkanOutputInfo {
        queue: info.graphics_queue_info.handle,
        acquired_image_index: u32::MAX,
        swapchain: vk::SwapchainKHR::null(),
        output_attachment_ready: vk::Semaphore::null(),
        output_present_ready: vk::Semaphore::null(),
        drawings_sync,
        rebuild: false,
        valid: false,
    };

    Ok(Box::new(output))
}

//
// Submit
//

fn init_submit(setup: &mut VulkanEngineSetup) -> Box<VulkanSubmitInfo> {
    let info = setup.info.as_ref().unwrap();

    let mut submit = Box::new(VulkanSubmitInfo {
        graphics_queue: info.graphics_queue_info.handle,
        upload_semaphore_signal: [vk::SemaphoreSubmitInfo::default(); 1],
        upload_commands_submit:  CommandBufferSubmitInfo::default(),
        render_semaphore_wait: [vk::SemaphoreSubmitInfo::default(); 2],
        render_semaphore_signal: [vk::SemaphoreSubmitInfo::default(); 2],
        render_commands_submit: CommandBufferSubmitInfo::default(),
        submit_infos: [vk::SubmitInfo2::default(); 2],
    });  

    // Submit info #0 is the upload commands
    let upload_submit = &mut submit.submit_infos[0];

    upload_submit.signal_semaphore_info_count = submit.upload_semaphore_signal.len() as u32;
    upload_submit.p_signal_semaphore_infos = submit.upload_semaphore_signal.as_ptr();

    upload_submit.command_buffer_info_count = 1;
    upload_submit.p_command_buffer_infos = &submit.upload_commands_submit;

    // Submit info #1 is the rendering commands
    let render_submit = &mut submit.submit_infos[1];
    
    render_submit.wait_semaphore_info_count = submit.render_semaphore_wait.len() as u32;
    render_submit.p_wait_semaphore_infos = submit.render_semaphore_wait.as_ptr();

    render_submit.signal_semaphore_info_count = submit.render_semaphore_signal.len() as u32;
    render_submit.p_signal_semaphore_infos = submit.render_semaphore_signal.as_ptr();

    render_submit.command_buffer_info_count = 1;
    render_submit.p_command_buffer_infos = &submit.render_commands_submit;

    submit
}

//
// Staging
//

fn init_staging(setup: &mut VulkanEngineSetup) -> Result<Box<VulkanStaging>, CommonError> {
    use crate::alloc::MB;

    let ctx = setup.ctx.as_ref().unwrap();
    let instance = &ctx.instance.instance;
    let device = &ctx.device;

    // Buffer
    let flags = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
 
    let staging_size = 5*MB;
    let buffer_info = vk::BufferCreateInfo {
        size: staging_size as _,
        usage: vk::BufferUsageFlags::TRANSFER_SRC,
        ..Default::default()
    };

    let buffer = device.create_buffer(&buffer_info)
        .map_err(|err| backend_init_err!("Failed to create staging buffer: {err}") )?;

    // Memory
    let buffer_info = device.get_buffer_memory_requirements(buffer);
    let staging_type_index = crate::helpers::fetch_memory_index(instance, ctx.device.physical_device, flags, flags)
        .ok_or_else(|| backend_init_err!("Failed to find memory type suitable for staging") )?;

    let alloc_info = vk::MemoryAllocateInfo {
        allocation_size: buffer_info.size,
        memory_type_index: staging_type_index,
        ..Default::default()
    };
    let memory = device.allocate_memory(&alloc_info)
        .map_err(|err| backend_init_err!("Failed to allocate staging memory: {err}") )?;

    device.bind_buffer_memory(buffer, memory, 0)
        .map_err(|err| backend_init_err!("Failed to bind staging memory: {err}") )?;

    // Mapping
    let mapped_data = device.map_memory(memory, 0, staging_size)
        .map_err(|err| backend_init_err!("Failed to map staging memory: {err}") )?;

    let mut staging: Box<VulkanStaging> = Box::default();
    staging.memory = memory;
    staging.buffer = buffer;
    staging.mapped_data = Some(mapped_data as *mut u8);
    staging.buffer_capacity = staging_size;
    
    Ok(staging)
}

//
// Descriptors
//

fn init_descriptors() -> Box<VulkanDescriptorSubmit> {
    Box::new(VulkanDescriptorSubmit {
        images: Vec::with_capacity(16),
        buffers: Vec::with_capacity(16),
        writes: Vec::with_capacity(16),
        updates: Vec::with_capacity(16),
        images_count: 0,
        buffers_count: 0,
    })
}
