#![allow(non_camel_case_types, dead_code)]
use crate::*;
use std::{ffi::{c_void, CStr}, mem::transmute};

vk_enum!(Win32SurfaceCreateFlagsKHR);

impl StructureType {
    pub const WIN32_SURFACE_CREATE_INFO_KHR: Self = Self(1_000_009_000);
}

#[repr(C)]
pub struct Win32SurfaceCreateInfoKHR {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: Win32SurfaceCreateFlagsKHR,
    pub hinstance: *mut c_void,
    pub hwnd: *mut c_void,
}

impl ::std::default::Default for Win32SurfaceCreateInfoKHR {
    fn default() -> Win32SurfaceCreateInfoKHR {
        Win32SurfaceCreateInfoKHR {
            s_type: StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            p_next: ::std::ptr::null(),
            flags: Win32SurfaceCreateFlagsKHR::default(),
            hinstance: unsafe { ::std::mem::zeroed() },
            hwnd: unsafe { ::std::mem::zeroed() },
        }
    }
}

pub struct KhrWin32SurfaceFn {
    pub create_win32_surface_khr: PFN_vkCreateWin32SurfaceKHR,
    pub get_physical_device_win32_presentation_support_khr: PFN_vkGetPhysicalDeviceWin32PresentationSupportKHR,
}

impl KhrWin32SurfaceFn {

    pub fn load<F>(cb: F) -> KhrWin32SurfaceFn 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            KhrWin32SurfaceFn {
                create_win32_surface_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkCreateWin32SurfaceKHR\0"))),
                get_physical_device_win32_presentation_support_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkGetPhysicalDeviceWin32PresentationSupportKHR\0"))),
            }
        }
    }

}

pub type PFN_vkCreateWin32SurfaceKHR = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const Win32SurfaceCreateInfoKHR,
    p_allocator: *const c_void,
    p_surface: *mut SurfaceKHR,
) -> VkResult;

pub type PFN_vkGetPhysicalDeviceWin32PresentationSupportKHR =
    unsafe extern "system" fn(physical_device: PhysicalDevice, queue_family_index: u32) -> Bool32;
