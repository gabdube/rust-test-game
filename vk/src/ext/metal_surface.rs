#![allow(non_camel_case_types, dead_code)]
use crate::*;
use std::{ffi::{c_void, CStr}, mem::transmute};

pub type CAMetalLayer = c_void;

vk_enum!(MetalSurfaceCreateFlagsEXT);

impl StructureType {
    pub const METAL_SURFACE_CREATE_INFO_EXT: Self = Self(1_000_217_000);
}

#[repr(C)]
pub struct MetalSurfaceCreateInfoEXT {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: MetalSurfaceCreateFlagsEXT,
    pub p_layer: *const CAMetalLayer,
}

impl ::core::default::Default for MetalSurfaceCreateInfoEXT {
    #[inline]
    fn default() -> Self {
        Self {
            s_type: StructureType::METAL_SURFACE_CREATE_INFO_EXT,
            p_next: ::core::ptr::null(),
            flags: MetalSurfaceCreateFlagsEXT::default(),
            p_layer: ::std::ptr::null(),
        }
    }
}

pub struct ExtMetalSurfaceFn {
    pub create_metal_surface_ext: PFN_vkCreateMetalSurfaceEXT,
}

impl ExtMetalSurfaceFn {

    pub fn load<F>(cb: F) -> ExtMetalSurfaceFn 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            ExtMetalSurfaceFn {
                create_metal_surface_ext: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkCreateMetalSurfaceEXT\0"))),
            }
        }
    }

}

pub type PFN_vkCreateMetalSurfaceEXT = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const MetalSurfaceCreateInfoEXT,
    p_allocator: *const c_void,
    p_surface: *mut SurfaceKHR,
) -> VkResult;
