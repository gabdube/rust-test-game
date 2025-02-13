use loomz_shared::{system_err, backend_init_err, CommonError};
use vk::wrapper::{Entry, Instance, Device};

use std::ffi::{CString, CStr};

use crate::context::{VulkanContext, VulkanContextExtensions, VulkanContextInstance};
use super::VulkanEngineSetup;

struct VulkanCtxSetup {
    pub entry: Option<Entry>,
    pub instance: Option<Instance>,
    pub device: Option<Device>,
    pub extensions: Option<VulkanContextExtensions>,
}

pub fn setup(setup: &mut VulkanEngineSetup) -> Result<(), CommonError> {
    let mut ctx_setup = init_ctx_setup();
    setup_entry(&mut ctx_setup)?;
    setup_instance(&mut ctx_setup)?;
    setup_device(&mut ctx_setup)?;
    load_extensions(&mut ctx_setup);
    build_final_context(setup, ctx_setup);
    Ok(())
}

fn init_ctx_setup() -> VulkanCtxSetup {
    VulkanCtxSetup {
        entry: None,
        instance: None,
        device: None,
        extensions: None,
    }
}

fn build_final_context(setup: &mut VulkanEngineSetup, ctx_setup: VulkanCtxSetup) {
    assert!(ctx_setup.entry.is_some(), "Vulkan entry point must have been created");
    assert!(ctx_setup.instance.is_some(), "Vulkan instance must have been created");
    assert!(ctx_setup.device.is_some(), "Vulkan device must have been created");
    assert!(ctx_setup.extensions.is_some(), "Vulkan extensions must have been created");

    let ctx = VulkanContext {
        device: ctx_setup.device.unwrap(),
        extensions: ctx_setup.extensions.unwrap(),
        instance: VulkanContextInstance {
            entry: ctx_setup.entry.unwrap(),
            instance: ctx_setup.instance.unwrap(),
        }
    };

    setup.ctx = Some(Box::new(ctx));
}


//
// Entry
//

fn setup_entry(setup: &mut VulkanCtxSetup) -> Result<(), CommonError> {
    setup.entry = Entry::open()
        .map(Some)
        .map_err(|err| system_err!("Failed to load Vulkan: {err}") )?;

    Ok(())
}

//
// Instance
//

fn setup_instance(setup: &mut VulkanCtxSetup) -> Result<(), CommonError> {
    const API: (u32, u32, u32) = (1, 2, 0);

    let entry = setup.entry.as_ref().unwrap();

    // Version info
    let app_name = b"Nimiety\0";
    let engine_name = b"NimietyEngine\0";
    let api_version = vk::make_api_version(0, API.0, API.1, API.2);
    let engine_version = vk::make_api_version(0, 1, 0, 0);

    let app_info = vk::ApplicationInfo {
        api_version,
        p_application_name: app_name.as_ptr(),
        application_version: engine_version,
        p_engine_name: engine_name.as_ptr(),
        engine_version,
        ..Default::default()
    };

    // Layers & extensions
    let mut layers: Vec<CString> = Vec::with_capacity(1);
    let mut instance_extensions: Vec<CString> = Vec::with_capacity(3);
    setup_layers_and_extensions(entry, &mut layers, &mut instance_extensions)?;
    
    let layers_name_ptr: Vec<*const u8> = layers.iter().map(|c| c.as_bytes_with_nul().as_ptr() ).collect();
    let extension_names_ptr: Vec<*const u8> = instance_extensions.iter().map(|c| c.as_bytes_with_nul().as_ptr() ).collect();

    let instance_flags;

    if cfg!(target_os="macos") {
        instance_flags = vk::InstanceCreateFlags::INSTANCE_CREATE_ENUMERATE_PORTABILITY;
    } else {
        instance_flags = vk::InstanceCreateFlags::default();
    }
    
    // Instance creation
    let create_info = vk::InstanceCreateInfo {
        flags: instance_flags,
        p_application_info: &app_info,
        enabled_layer_count: layers.len() as _,
        pp_enabled_layer_names: layers_name_ptr.as_ptr(),
        enabled_extension_count: instance_extensions.len() as _,
        pp_enabled_extension_names: extension_names_ptr.as_ptr(),
        ..Default::default()
    };

    setup.instance = entry.create_instance(&create_info)
        .map_err(|error| backend_init_err!("Failed to create vulkan instance: {error}") )?
        .into();

    Ok(())
}

#[cfg(target_os = "linux")]
fn linux_surface_extensions(entry: &vk::wrapper::Entry, extensions_in: &mut Vec<&'static [u8]>) -> Result<(), CommonError> {
    let extension_names: &[&[u8]] = &[b"VK_KHR_wayland_surface\0", b"VK_KHR_xcb_surface\0", b"VK_KHR_xlib_surface\0"];

    let available_extensions = entry.enumerate_instance_extension_properties()
        .map_err(|err| backend_init_err!("Failed to list instance extensions: {err}") )?;

    for ext in extension_names.iter() {
        let c_ext = CStr::from_bytes_with_nul(ext).unwrap();
        for ext_property in available_extensions.iter() {
            let c_ext2 = CStr::from_bytes_until_nul(&ext_property.extension_name).unwrap();
            if c_ext == c_ext2 {
                extensions_in.push(ext);
            }
        }
    }

    Ok(())
}

fn setup_layers_and_extensions(entry: &vk::wrapper::Entry, layers: &mut Vec<CString>, extensions: &mut Vec<CString>) -> Result<(), CommonError> {
    let layers_in: &[&[u8]] = &[
        #[cfg(debug_assertions)]
        b"VK_LAYER_KHRONOS_validation\0",
    ];

    let mut extensions_in: Vec<&'static [u8]> = vec![
        b"VK_KHR_surface\0",
    ];

    #[cfg(debug_assertions)]
    extensions_in.push(b"VK_EXT_debug_utils\0");

    #[cfg(windows)]
    extensions_in.push(b"VK_KHR_win32_surface\0");

    #[cfg(target_os = "macos")]
    extensions_in.push(b"VK_KHR_portability_enumeration\0");

    #[cfg(target_os = "macos")]
    extensions_in.push(b"VK_EXT_metal_surface\0");

    #[cfg(target_os = "linux")]
    linux_surface_extensions(entry, &mut extensions_in)?;

    let available_layers = entry.enumerate_instance_layer_properties()
        .map_err(|err| backend_init_err!("Failed to list instance layers: {err}") )?;
    
    let available_extensions = entry.enumerate_instance_extension_properties()
        .map_err(|err| backend_init_err!("Failed to list instance extensions: {err}") )?;

    // println!("Layers:");
    // for layer in available_layers.iter() {
    //     let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr() as *const i8) };
    //     println!("{:?}", name);
    // }

    // println!("Extensions:");
    // for ext in available_extensions.iter() {
    //     let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr() as *const i8) };
    //     println!("{:?}", name);
    // }

    let mut missing_layers = Vec::new();
    let mut missing_extensions = Vec::new();

    'layer: for &layer in layers_in {
        let c_layer = CStr::from_bytes_with_nul(layer).unwrap();
        for layer_property in available_layers.iter() {
            let c_layer2 = CStr::from_bytes_until_nul(&layer_property.layer_name).unwrap();
            if c_layer == c_layer2 {
                layers.push(CString::from(c_layer));
                continue 'layer;
            }
        }
        missing_layers.push(layer);
    }

    'ext: for ext in extensions_in {
        let c_ext = CStr::from_bytes_with_nul(ext).unwrap();
        for ext_property in available_extensions.iter() {
            let c_ext2 = CStr::from_bytes_until_nul(&ext_property.extension_name).unwrap();
            if c_ext == c_ext2 {
                extensions.push(CString::from(c_ext));
                continue 'ext;
            }
        }
        missing_extensions.push(ext);
    }

    if !missing_layers.is_empty() {
        Err(backend_init_err!("Missing vulkan validation layers: {:?}", missing_layers))
    } else if !missing_extensions.is_empty() {
        Err(backend_init_err!("Missing vulkan surface instance extension: {:?}", missing_extensions))
    } else {
        Ok(())
    }
}

//
// Device
//

fn setup_device(setup: &mut VulkanCtxSetup) -> Result<(), CommonError> {
    let instance = setup.instance.as_ref().unwrap();

    let mut all_good = false;
    let mut failure_reasons: Vec<String> = Vec::new();

    let mut physical_device: Option<vk::PhysicalDevice> = None;
    let mut queue_create_info: vk::DeviceQueueCreateInfo = Default::default();

    let mut required_extensions: Vec<&[u8]> = vec![
        b"VK_KHR_swapchain\0",
        b"VK_EXT_descriptor_indexing\0",
        b"VK_KHR_dynamic_rendering\0",
        b"VK_KHR_synchronization2\0",
    ];

    if cfg!(target_os="macos") {
        required_extensions.push(b"VK_KHR_portability_subset\0");
    }

    let device_extension_names: Vec<&CStr> = required_extensions.iter()
        .map(|name| CStr::from_bytes_with_nul(name).unwrap() )
        .collect();

    let physical_devices = instance.enumerate_physical_devices()
        .map_err(|err| backend_init_err!("Failed to enumerate physical devices: {err}") )?;

    // Find a valid physical device
    for (device_index, &pdevice) in physical_devices.iter().enumerate() {

        // Features
        let features = get_device_features(instance, pdevice);
        if features.dynamic_rendering.dynamic_rendering != 1 {
            failure_reasons.push(format!("Physical device {} missing feature: dynamic rendering not supported", device_index));
            continue;
        }

        if !validate_device_features(&features, &mut failure_reasons, device_index) {
            continue;
        }

        // Extensions
        if let Err(e) = validate_device_extensions(instance, pdevice, &device_extension_names) {
            failure_reasons.push(format!("Physical device {} not supported: {:?}", device_index, e));
            continue;
        }

        // Queues
        if let Err(e) = validate_queues(instance, pdevice, &mut queue_create_info) {
            failure_reasons.push(format!("Physical device {} not supported: {}", device_index, e));
            continue;
        }

        // Export the data to the outer scope
        all_good = true;
        physical_device = Some(pdevice);

        break;
    }

    if !all_good {
        return Err(backend_init_err!("Engine backend not supported: {:?}", failure_reasons));
    }

    // Create the device
    let extension_names_ptr: Vec<*const u8> = device_extension_names.iter().map(|c| c.to_bytes_with_nul().as_ptr() ).collect();
    let physical_device = physical_device.unwrap();

    let mut device_create_info = vk::DeviceCreateInfo {
        queue_create_info_count: 1,
        p_queue_create_infos: &queue_create_info,
        enabled_extension_count: device_extension_names.len() as _,
        pp_enabled_extension_names: extension_names_ptr.as_ptr(),
        ..Default::default()
    };

    // Link features
    let mut all_features: vk::wrapper::CombinedDeviceFeatures = Default::default();
    all_features.base.features.sampler_anisotropy = 1;
    all_features.dynamic_rendering.dynamic_rendering = 1;
    all_features.timeline_semaphore.timeline_semaphore = 1;
    all_features.descriptor_indexing.shader_sampled_image_array_non_uniform_indexing = 1;
    all_features.descriptor_indexing.runtime_descriptor_array = 1;
    all_features.descriptor_indexing.descriptor_binding_partially_bound = 1;
    all_features.descriptor_indexing.descriptor_binding_variable_descriptor_count = 1;
    all_features.syncronization2.synchronization2 = 1;
    device_create_info.p_next = all_features.features_ptr() as *const _ as _;

    setup.device = instance.create_device(physical_device, &device_create_info)
        .map_err(|err| backend_init_err!("Failed to create vulkan device: {err}") )?
        .into();

    // print_device_name(instance, physical_device);

    Ok(())
}

fn get_device_features(instance: &Instance, pdevice: vk::PhysicalDevice) -> vk::wrapper::CombinedDeviceFeatures {
    let mut features: vk::wrapper::CombinedDeviceFeatures = Default::default();

    instance.get_physical_device_features_2(pdevice, features.features_ptr_mut());

    features
}

fn validate_device_features(features: &vk::wrapper::CombinedDeviceFeatures, failure_reasons: &mut Vec<String>, device_index: usize) -> bool {

    let features = &[
        ("timeline_semaphore", features.timeline_semaphore.timeline_semaphore),
        ("shader_sampled_image_array_non_uniform_indexing", features.descriptor_indexing.shader_sampled_image_array_non_uniform_indexing),
        ("runtime_descriptor_array", features.descriptor_indexing.runtime_descriptor_array),
        ("descriptor_binding_partially_bound", features.descriptor_indexing.descriptor_binding_partially_bound),
        ("descriptor_binding_variable_descriptor_count", features.descriptor_indexing.descriptor_binding_variable_descriptor_count),
        ("dynamic_rendering", features.dynamic_rendering.dynamic_rendering)
    ];

    for (name, supported) in features {
        if *supported != 1 {
            failure_reasons.push(format!("Physical device {device_index} missing feature: {name} not supported"));
        }
    }

    true
}

fn validate_device_extensions(instance: &Instance, pdevice: vk::PhysicalDevice, extensions: &[&CStr]) -> Result<(), String> {
    let available_extensions = match instance.enumerate_device_extension_properties(pdevice) {
        Ok(extensions) => extensions,
        Err(e) => { return Err(format!("Failed to enumerate physical device extensions: {}", e)) }
    };

    let mut missing_extensions = Vec::new();
    'ext: for &ext in extensions.iter() {
        for ext_property in available_extensions.iter() {
            let ext2 = CStr::from_bytes_until_nul(&ext_property.extension_name).unwrap();
            if ext == ext2 {
                continue 'ext;
            }
        }
        
        missing_extensions.push(ext);
        
    }

    match !missing_extensions.is_empty() {
        true => {
            Err(format!("Required device extensions not supported: {:?}", missing_extensions))
        },
        false => {
            Ok(())
        }
    }
}

fn validate_queues(
    instance: &Instance,
    pdevice: vk::PhysicalDevice,
    create_info: &mut vk::DeviceQueueCreateInfo
) -> Result<(), String> {
    static PRIORITY: &[f32] = &[0.0];

    let required_flags = vk::QueueFlags::GRAPHICS;

    let queues_family_properties = instance.get_physical_device_queue_family_properties(pdevice);
    for (queue_family_index, prop) in queues_family_properties.iter().enumerate() {
        // Pretty much all we have to validate until we do something more fancy
        // Surface present support is checked at swapchain creation and it is assumed that it will always work.
        if !prop.queue_flags.contains(required_flags) {
            continue;
        }

        *create_info = vk::DeviceQueueCreateInfo {
            queue_count: 1,
            queue_family_index: queue_family_index as u32,
            p_queue_priorities: PRIORITY.as_ptr(),
            ..Default::default()
        };

        return Ok(());
    }

    Err("No queues with GRAPHICS flags found".to_string())
}

//
// Extensions
//

fn load_extensions(setup: &mut VulkanCtxSetup) {
    use vk::wrapper::{Surface, Swapchain, DynamicRendering, Synchronization2};

    let entry = setup.entry.as_ref().unwrap();
    let instance = setup.instance.as_ref().unwrap();
    let device = setup.device.as_ref().unwrap();

    let surface = Surface::new(entry, instance);
    let swapchain = Swapchain::new(instance, device);
    let dynamic_rendering = DynamicRendering::new(instance, device);
    let synchronization2 = Synchronization2::new(instance, device);

    setup.extensions = Some(VulkanContextExtensions {
        surface,
        swapchain,
        dynamic_rendering,
        synchronization2,

        #[cfg(windows)]
        win32_surface: vk::wrapper::Win32Surface::new(entry, instance),

        #[cfg(target_os="macos")]
        metal_surface: vk::wrapper::MetalSurface::new(entry, instance),

        #[cfg(target_os="linux")]
        linux_surface: crate::context::VulkanLinuxSurfaces { 
            wayland_surface: vk::wrapper::WaylandSurface::new(entry, instance)
        }
    });
}

//
// Other
//

#[allow(dead_code)]
fn print_device_name(instance: &Instance, pdevice: vk::PhysicalDevice) {
    let prop = instance.get_physical_device_properties(pdevice);
    let str = ::std::ffi::CStr::from_bytes_until_nul(&prop.device_name)
        .map(|s| s.to_str().unwrap_or("Unknown") )
        .unwrap_or("Unknown");

    println!("{:?}", str); 
}
