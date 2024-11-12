#![allow(non_camel_case_types, dead_code)]

use crate::*;
use std::{ffi::{c_void, CStr}, mem::transmute};

define_nondispatchable_handle!(SurfaceKHR, SURFACE_KHR);

vk_enum!(SurfaceTransformFlagsKHR);
vk_bitflags!(SurfaceTransformFlagsKHR);
impl SurfaceTransformFlagsKHR {
    pub const IDENTITY: Self = Self(0b1);
    pub const ROTATE_90: Self = Self(0b10);
    pub const ROTATE_180: Self = Self(0b100);
    pub const ROTATE_270: Self = Self(0b1000);
    pub const HORIZONTAL_MIRROR: Self = Self(0b1_0000);
    pub const HORIZONTAL_MIRROR_ROTATE_90: Self = Self(0b10_0000);
    pub const HORIZONTAL_MIRROR_ROTATE_180: Self = Self(0b100_0000);
    pub const HORIZONTAL_MIRROR_ROTATE_270: Self = Self(0b1000_0000);
    pub const INHERIT: Self = Self(0b1_0000_0000);
}

vk_enum!(CompositeAlphaFlagsKHR);
vk_bitflags!(CompositeAlphaFlagsKHR);
impl CompositeAlphaFlagsKHR {
    pub const OPAQUE: Self = Self(0b1);
    pub const PRE_MULTIPLIED: Self = Self(0b10);
    pub const POST_MULTIPLIED: Self = Self(0b100);
    pub const INHERIT: Self = Self(0b1000);
}

vk_enum!(PresentModeKHR);
impl PresentModeKHR {
    pub const IMMEDIATE: Self = Self(0);
    pub const MAILBOX: Self = Self(1);
    pub const FIFO: Self = Self(2);
    pub const FIFO_RELAXED: Self = Self(3);
}

vk_enum!(ColorSpaceKHR);
impl ColorSpaceKHR {
    pub const SRGB_NONLINEAR: Self = Self(0);
}


#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct SurfaceFormatKHR {
    pub format: Format,
    pub color_space: ColorSpaceKHR,
}

#[repr(C)]
pub struct SurfaceCapabilitiesKHR {
    pub min_image_count: u32,
    pub max_image_count: u32,
    pub current_extent: Extent2D,
    pub min_image_extent: Extent2D,
    pub max_image_extent: Extent2D,
    pub max_image_array_layers: u32,
    pub supported_transforms: SurfaceTransformFlagsKHR,
    pub current_transform: SurfaceTransformFlagsKHR,
    pub supported_composite_alpha: CompositeAlphaFlagsKHR,
    pub supported_usage_flags: ImageUsageFlags,
}

pub struct KhrSurfaceFn {
    pub destroy_surface_khr: PFN_vkDestroySurfaceKHR,
    pub get_physical_device_surface_support_khr: PFN_vkGetPhysicalDeviceSurfaceSupportKHR,
    pub get_physical_device_surface_capabilities_khr: PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR,
    pub get_physical_device_surface_formats_khr: PFN_vkGetPhysicalDeviceSurfaceFormatsKHR,
    pub get_physical_device_surface_present_modes_khr: PFN_vkGetPhysicalDeviceSurfacePresentModesKHR,
}

impl KhrSurfaceFn {

    pub fn load<F>(cb: F) -> KhrSurfaceFn 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            KhrSurfaceFn {
                destroy_surface_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkDestroySurfaceKHR\0"))),
                get_physical_device_surface_support_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkGetPhysicalDeviceSurfaceSupportKHR\0"))),
                get_physical_device_surface_capabilities_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkGetPhysicalDeviceSurfaceCapabilitiesKHR\0"))),
                get_physical_device_surface_formats_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkGetPhysicalDeviceSurfaceFormatsKHR\0"))),
                get_physical_device_surface_present_modes_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkGetPhysicalDeviceSurfacePresentModesKHR\0"))),
            }
        }
    }

}


pub type PFN_vkDestroySurfaceKHR = unsafe extern "system" fn(
    instance: Instance,
    surface: SurfaceKHR,
    p_allocator: *const c_void,
);

pub type PFN_vkGetPhysicalDeviceSurfaceSupportKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    queue_family_index: u32,
    surface: SurfaceKHR,
    p_supported: *mut Bool32,
) -> VkResult;

pub type PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    p_surface_capabilities: *mut SurfaceCapabilitiesKHR,
) -> VkResult;

pub type PFN_vkGetPhysicalDeviceSurfaceFormatsKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    p_surface_format_count: *mut u32,
    p_surface_formats: *mut SurfaceFormatKHR,
) -> VkResult;

pub type PFN_vkGetPhysicalDeviceSurfacePresentModesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    p_present_mode_count: *mut u32,
    p_present_modes: *mut PresentModeKHR,
) -> VkResult;

