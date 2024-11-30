use loomz_shared::{backend_init_err, CommonError};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use crate::LoomzEngineCore;

#[allow(dead_code)]
pub struct SetupTargetParams {
    pub display: RawDisplayHandle,
    pub window: RawWindowHandle,
}

pub fn setup_target(engine: &mut LoomzEngineCore, params: &SetupTargetParams) -> Result<(), CommonError> {
    setup_surface(engine, params)?;
    setup_swapchain(engine)?;
    setup_swapchain_images(engine)?;
    setup_attachments(engine)?;
    setup_sync(engine)?;
    Ok(())
}

pub fn rebuild_target(engine: &mut LoomzEngineCore) -> Result<(), CommonError> {
    clean_resources(engine);
    setup_swapchain(engine)?;
    setup_swapchain_images(engine)?;
    setup_attachments(engine)?;
    setup_sync(engine)?;

    engine.output.rebuild = false;

    Ok(())
}


//
// Surface
//

#[cfg(windows)]
fn setup_surface(engine: &mut LoomzEngineCore, params: &SetupTargetParams) -> Result<(), CommonError> {
    let handle = match params.window {
        RawWindowHandle::Win32(handle) => handle,
        h => {
            return Err(backend_init_err!("Bad window handle, expected win32, got {:?}", h));
        }
    };

    let hinstance = match handle.hinstance {
        Some(hinstance) => hinstance.get(),
        None => {
            return Err(backend_init_err!("Failed to get HINSTANCE"));
        }
    };

    let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
        hinstance: hinstance as _,
        hwnd: handle.hwnd.get() as _,
        ..Default::default()
    };

    engine.resources.surface = engine.ctx.extensions.win32_surface.create_win32_surface(&win32_create_info)
        .map_err(|err| backend_init_err!("Failed to create window surface: {err}") )?;

    Ok(())
}

#[cfg(target_os="linux")]
fn setup_surface(engine: &mut LoomzEngineCore, params: &SetupTargetParams) -> Result<(), CommonError> {
    match (params.window, params.display) {
        (RawWindowHandle::Wayland(h1), RawDisplayHandle::Wayland(h2)) => {
            let wayland_create_info = vk::WaylandSurfaceCreateInfoKHR {
                surface: h1.surface.as_ptr(),
                display: h2.display.as_ptr(),
                ..Default::default()
            };

            let ext = &engine.ctx.extensions.linux_surface.wayland_surface;
            engine.resources.surface = ext.create_wayland_surface(&wayland_create_info)
                .map_err(|err| backend_init_err!("Failed to create window surface: {err}") )?;
           
            Ok(())
        },
        _ => panic!("Only Wayland surface is supported for now")
    }
}

//
// Swapchain
//

fn setup_swapchain(engine: &mut LoomzEngineCore) -> Result<(), CommonError> {
    const VSYNC: bool = true;
    
    let ctx = &engine.ctx;
    let surface = engine.resources.surface;
    let surface_ext = &ctx.extensions.surface;
    let device = &ctx.device;

    let output = &mut engine.output;
    let info = &mut engine.info;

    // Check surface support
    let queue_family = info.graphics_queue_info.family_index;
    let support = surface_ext.get_physical_device_surface_support(device.physical_device, queue_family, surface)
        .map_err(|err| backend_init_err!("Failed to query surface support: {err}") )?;
    
    if !support {
        return Err(backend_init_err!("Main engine queue cannot present to surface"));
    }
    
    // Swapchain creation
    let caps = fetch_surface_caps(surface_ext, device.physical_device, surface)?;
    let image_extent = select_swapchain_extent(&caps, info.window_extent);
    let image_count = select_image_count(&caps)?;
    let swapchain_format = select_swapchain_format(surface_ext, device.physical_device, surface)?;
    let present_mode = select_present_mode(surface_ext, device.physical_device, surface, VSYNC)?;
    let transform = select_transform(&caps);
    let image_usage = vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::COLOR_ATTACHMENT;
    let composite_alpha = vk::CompositeAlphaFlagsKHR::OPAQUE;
    let old_swapchain = output.swapchain;

    let create_info = vk::SwapchainCreateInfoKHR {
        surface,
        image_usage,
        image_extent,
        image_format: swapchain_format.format,
        image_color_space: swapchain_format.color_space,
        min_image_count: image_count,
        present_mode,
        pre_transform: transform,
        composite_alpha,
        image_array_layers: 1,
        old_swapchain,
        ..Default::default()
    };

    let swapchain_ext = &ctx.extensions.swapchain;
    output.swapchain = swapchain_ext.create_swapchain(&create_info).map_err(|err| backend_init_err!("Failed to create swapchain: {err}"))?;
    info.swapchain_image_count = image_count;
    info.swapchain_extent = image_extent;
    info.swapchain_format = swapchain_format.format;

    if !old_swapchain.is_null() {
        swapchain_ext.destroy_swapchain(old_swapchain);
    }

    Ok(())
}

fn fetch_surface_caps(surface_ext: &vk::wrapper::Surface, physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Result<vk::SurfaceCapabilitiesKHR, CommonError> {
    surface_ext.get_physical_device_surface_capabilities(physical_device, surface)
        .map_err(|err| backend_init_err!("Failed to list surface capabilities: {err}"))
}

fn select_swapchain_extent(caps: &vk::SurfaceCapabilitiesKHR, requested_extent: vk::Extent2D) -> vk::Extent2D {
    let mut extent = caps.current_extent;

    // On windows, caps.current_extent returns the size of the window
    // On linux / wayland. The values are always u32::MAX

    if extent.width == u32::MAX || extent.height == u32::MAX {
        extent = requested_extent;
    } else if extent.width == 0 || extent.height == 0 {
        // This can happen when a window is minimized on Windows or if the window size is set to 0,0 and invisible.
        // At startup, this cannot happen
        // At runtime, this must be caught before this function
        panic!("Invalid window size: {:?}", extent);
    }
    extent
}

fn select_image_count(caps: &vk::SurfaceCapabilitiesKHR) -> Result<u32, CommonError> {
    let mut image_count = 2;

    if caps.min_image_count > image_count {
        image_count = caps.min_image_count;
    }

    if caps.max_image_count != 0 && caps.max_image_count < image_count || caps.min_image_count > image_count {
        return Err(backend_init_err!(
            "Could not match requested swapchain image format count {image_count:?}. Min: {:?}. Max: {:?}",
            caps.min_image_count, caps.max_image_count
        ));
    }

    Ok(image_count)
}

fn select_present_mode(surface_ext: &vk::wrapper::Surface, physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR, vsync: bool) -> Result<vk::PresentModeKHR, CommonError> {
    let mut present_modes = Vec::new();
    surface_ext.get_physical_device_surface_present_modes(physical_device, surface, &mut present_modes)
        .map_err(|err| backend_init_err!("Failed to list surface present mode: {err}"))?;

    let immediate_supported = present_modes.iter().any(|&pm| pm == vk::PresentModeKHR::IMMEDIATE);

    let mut present_mode = vk::PresentModeKHR::FIFO;
    if !vsync && immediate_supported {
        present_mode = vk::PresentModeKHR::IMMEDIATE;
    }

    Ok(present_mode)
}

fn select_transform(caps: &vk::SurfaceCapabilitiesKHR) -> vk::SurfaceTransformFlagsKHR {
    let mut transform = caps.current_transform;
    if caps.supported_transforms.contains(vk::SurfaceTransformFlagsKHR::IDENTITY) {
        transform = vk::SurfaceTransformFlagsKHR::IDENTITY;
    }

    transform
}

fn select_swapchain_format(surface_ext: &vk::wrapper::Surface, physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Result<vk::SurfaceFormatKHR, CommonError> {
    const SWAPCHAIN_FMT: &[vk::Format] = &[
        vk::Format::B8G8R8A8_UNORM,
        vk::Format::R8G8B8A8_UNORM,
    ];

    let mut supported_formats: Vec<vk::SurfaceFormatKHR> = Vec::new();
    surface_ext.get_physical_device_surface_formats(physical_device, surface, &mut supported_formats)
        .map_err(|err| backend_init_err!( "Listing surface formats failed: {err}"))?;
       

    for &format in SWAPCHAIN_FMT.iter() {
        for surface_format in supported_formats.iter() {
            if surface_format.format == format {
                return Ok(*surface_format);
            }
        }
    }

    let required: Vec<u32> = SWAPCHAIN_FMT.iter().map(|f| f.0).collect();
    let available: Vec<u32> = supported_formats.iter().map(|f| f.format.0).collect();
    Err(backend_init_err!("Unsupported surface formats: required: {required:?}, available: {available:?}"))
}

fn setup_swapchain_images(engine: &mut LoomzEngineCore) -> Result<(), CommonError> {
    const MAX_IMAGE_COUNT: usize = 16; // This should be enough to hold the images (ex: min_image is 4 on my machine)

    let swapchain_ext = &engine.ctx.extensions.swapchain;
    let image_count = engine.info.swapchain_image_count;
    let output = &mut engine.output;
    let resources = &mut engine.resources;

    let mut swapchain_images = [vk::Image::null(); MAX_IMAGE_COUNT];
    swapchain_ext.get_swapchain_images(output.swapchain, &mut swapchain_images)
        .map_err(|err| backend_init_err!("Failed fetch swapchain images: {err}") )?;

    for i in 0..image_count {
        resources.attachments.output.push(crate::helpers::Attachment { image: swapchain_images[i as usize], view: vk::ImageView::null() });
    }

    Ok(())
}

//
// Attachments
//

fn setup_attachments(engine: &mut LoomzEngineCore) -> Result<(), CommonError> {
    setup_images(engine)?;
    setup_attachments_memory(engine)?;
    setup_view(engine)?;
    Ok(())
}

fn setup_images(engine: &mut LoomzEngineCore) -> Result<(), CommonError> {
    let device = &engine.ctx.device;
    let info = &engine.info;
    let resources = &mut engine.resources;

    let vk::Extent2D { width, height } = info.swapchain_extent;
    let mut create_info = vk::ImageCreateInfo {
        image_type: vk::ImageType::TYPE_2D,
        extent: vk::Extent3D { width, height, depth: 1 },
        ..Default::default()
    };

    let attachments = &mut resources.attachments;
    create_info.format = info.color_format;
    create_info.usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;
    create_info.samples = info.sample_count;
    attachments.color.image = device.create_image(&create_info)
        .map_err(|err| backend_init_err!("Failed to create color attachment image: {err}"))?;

    create_info.format = info.depth_format;
    create_info.usage = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
    create_info.samples = info.sample_count;
    attachments.depth.image = device.create_image(&create_info)
        .map_err(|err| backend_init_err!("Failed to create depth attachment image: {err}"))?;

    Ok(())
}

fn setup_attachments_memory(engine: &mut LoomzEngineCore) -> Result<(), CommonError> {
    let attachments = &mut engine.resources.attachments;

    let instance = &engine.ctx.instance.instance;
    let device = &engine.ctx.device;
    let memory_type_index = crate::helpers::fetch_attachments_memory_index(instance, device.physical_device);

    let mut allocation_size = 0;

    let images_count = 2;
    let mut image_memory_bind_offset = [0; 2];
    let images = [
        attachments.color.image,
        attachments.depth.image,
    ];

    for i in 0..images_count {
        let req = device.get_image_memory_requirements(images[i]);

        let align_offset = crate::helpers::align_device(allocation_size, req.alignment);
        image_memory_bind_offset[i] = align_offset;

        allocation_size = align_offset;
        allocation_size += req.size;
    }

    let alloc_info = vk::MemoryAllocateInfo {
        allocation_size,
        memory_type_index,
        ..Default::default()
    };

    let memory = device.allocate_memory(&alloc_info)
        .map_err(|err| backend_init_err!("Failed to allocate attachments memory: {err}"))?;

    for i in 0..images_count {
        device.bind_image_memory(images[i], memory, image_memory_bind_offset[i])
            .map_err(|err| backend_init_err!("Failed to bind attachment image memory: {err}"))?;
    }

    attachments.memory = memory;

    Ok(())
}

fn setup_view(engine: &mut LoomzEngineCore) -> Result<(), CommonError> {
    let device = &engine.ctx.device;
    let info = &engine.info;
    let attachments = &mut engine.resources.attachments;
    
    let mut create_info = vk::ImageViewCreateInfo {
        view_type: vk::ImageViewType::TYPE_2D,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags(0),
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1
        },
        ..Default::default()
    };

    create_info.image = attachments.color.image;
    create_info.format = info.color_format;
    create_info.subresource_range.aspect_mask = vk::ImageAspectFlags::COLOR;
    attachments.color.view = device.create_image_view(&create_info)
        .map_err(|err| backend_init_err!("Failed to create color attachment image view: {err}"))?;

    create_info.image = attachments.depth.image;
    create_info.format = info.depth_format;
    create_info.subresource_range.aspect_mask = vk::ImageAspectFlags::DEPTH;
    attachments.depth.view = device.create_image_view(&create_info)
        .map_err(|err| backend_init_err!("Failed to create color attachment image view: {err}"))?;

    for i in 0..attachments.output.len() {
        create_info.image = attachments.output[i].image;
        create_info.format = info.color_format;
        create_info.subresource_range.aspect_mask = vk::ImageAspectFlags::COLOR;
        attachments.output[i].view = device.create_image_view(&create_info)
            .map_err(|err| backend_init_err!("Failed to create color attachment image view: {err}"))?;
    }

    Ok(())
}


//
// Sync
//

fn setup_sync(engine: &mut LoomzEngineCore) -> Result<(), CommonError> {
    let device = &engine.ctx.device;
    let output = &mut engine.output;

    let create_semaphore = || {
        let sm_create_info = vk::SemaphoreCreateInfo::default();
        device.create_semaphore(&sm_create_info)
            .map_err(|err| backend_init_err!("Failed to create semaphore: {err}"))
    };

    output.output_attachment_ready = create_semaphore()?;
    output.output_present_ready = create_semaphore()?;

    Ok(())
}

//
// Rebuild
//

fn clean_resources(engine: &mut LoomzEngineCore) {
    let ctx = &engine.ctx;
    let output = &engine.output;
    let resource = &mut engine.resources;

    resource.attachments.free(&ctx.device);
    resource.attachments.output.clear();

    ctx.device.destroy_semaphore(output.output_attachment_ready);
    ctx.device.destroy_semaphore(output.output_present_ready);
}
