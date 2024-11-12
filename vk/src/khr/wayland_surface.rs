#![allow(non_camel_case_types, dead_code)]
use crate::*;
use std::{ffi::{c_void, CStr}, mem::transmute};

vk_enum!(WaylandSurfaceCreateFlagsKHR);

impl StructureType {
    pub const WAYLAND_SURFACE_CREATE_INFO_KHR: Self = Self(1_000_006_000);
}

#[repr(C)]
pub struct WaylandSurfaceCreateInfoKHR {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: WaylandSurfaceCreateFlagsKHR,
    pub display: *mut c_void,
    pub surface: *mut c_void,
}

impl ::std::default::Default for WaylandSurfaceCreateInfoKHR {
    fn default() -> WaylandSurfaceCreateInfoKHR {
        WaylandSurfaceCreateInfoKHR {
            s_type: StructureType::WAYLAND_SURFACE_CREATE_INFO_KHR,
            p_next: ::std::ptr::null(),
            flags: WaylandSurfaceCreateFlagsKHR::default(),
            display: unsafe { ::std::mem::zeroed() },
            surface: unsafe { ::std::mem::zeroed() },
        }
    }
}


pub struct KhrWaylandSurfaceFn {
    pub create_wayland_surface_khr: PFN_vkCreateWaylandSurfaceKHR,
    pub get_physical_device_wayland_presentation_support: PFN_vkGetPhysicalDeviceWaylandPresentationSupportKHR,
}


impl KhrWaylandSurfaceFn {

    pub fn load<F>(cb: F) -> KhrWaylandSurfaceFn 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            KhrWaylandSurfaceFn {
                create_wayland_surface_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkCreateWaylandSurfaceKHR\0"))),
                get_physical_device_wayland_presentation_support: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkGetPhysicalDeviceWaylandPresentationSupportKHR\0"))),
            }
        }
    }

}

pub type PFN_vkCreateWaylandSurfaceKHR = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const WaylandSurfaceCreateInfoKHR,
    p_allocator: *const c_void,
    p_surface: *mut SurfaceKHR,
) -> VkResult;

pub type PFN_vkGetPhysicalDeviceWaylandPresentationSupportKHR =
    unsafe extern "system" fn(physical_device: PhysicalDevice, queue_family_index: u32, display: *mut c_void) -> Bool32;
