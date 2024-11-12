#![allow(dead_code)]

use crate::*;
use std::{ptr, ffi::{c_void, CStr}, mem::transmute};


define_nondispatchable_handle!(SwapchainKHR, SWAPCHAIN_KHR);

impl StructureType {
    pub const SWAPCHAIN_CREATE_INFO_KHR: Self = Self(1_000_001_000);
    pub const PRESENT_INFO_KHR: Self = Self(1_000_001_001);
}

impl VkResult {
    pub const SUBOPTIMAL_KHR: Self = Self(1_000_001_003);
}

vk_enum!(SwapchainCreateFlagsKHR);
vk_bitflags!(SwapchainCreateFlagsKHR);
impl SurfaceTransformFlagsKHR {
    pub const SPLIT_INSTANCE_BIND_REGIONS: Self = Self(0b1);
    pub const PROTECTED: Self = Self(0b10);
    pub const MUTABLE_FORMAT: Self = Self(0b100);
}


#[repr(C)]
pub struct SwapchainCreateInfoKHR {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: SwapchainCreateFlagsKHR,
    pub surface: SurfaceKHR,
    pub min_image_count: u32,
    pub image_format: Format,
    pub image_color_space: ColorSpaceKHR,
    pub image_extent: Extent2D,
    pub image_array_layers: u32,
    pub image_usage: ImageUsageFlags,
    pub image_sharing_mode: SharingMode,
    pub queue_family_index_count: u32,
    pub p_queue_family_indices: *const u32,
    pub pre_transform: SurfaceTransformFlagsKHR,
    pub composite_alpha: CompositeAlphaFlagsKHR,
    pub present_mode: PresentModeKHR,
    pub clipped: Bool32,
    pub old_swapchain: SwapchainKHR,
}

impl Default for SwapchainCreateInfoKHR {
    fn default() -> SwapchainCreateInfoKHR {
        SwapchainCreateInfoKHR {
            s_type: StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: SwapchainCreateFlagsKHR::default(),
            surface: SurfaceKHR::default(),
            min_image_count: 0,
            image_format: Format::default(),
            image_color_space: ColorSpaceKHR::default(),
            image_extent: Extent2D::default(),
            image_array_layers: 0,
            image_usage: ImageUsageFlags::default(),
            image_sharing_mode: SharingMode::default(),
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            pre_transform: SurfaceTransformFlagsKHR::default(),
            composite_alpha: CompositeAlphaFlagsKHR::default(),
            present_mode: PresentModeKHR::default(),
            clipped: 0,
            old_swapchain: SwapchainKHR::default(),
        }
    }
}

#[repr(C)]
pub struct PresentInfoKHR {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub wait_semaphore_count: u32,
    pub p_wait_semaphores: *const Semaphore,
    pub swapchain_count: u32,
    pub p_swapchains: *const SwapchainKHR,
    pub p_image_indices: *const u32,
    pub p_results: *mut VkResult,
}

impl ::std::default::Default for PresentInfoKHR {
    fn default() -> PresentInfoKHR {
        PresentInfoKHR {
            s_type: StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            swapchain_count: 0,
            p_swapchains: ptr::null(),
            p_image_indices: ptr::null(),
            p_results: ptr::null_mut(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct KhrSwapchainFn {
    pub create_swapchain_khr: PFN_vkCreateSwapchainKHR,
    pub destroy_swapchain_khr: PFN_vkDestroySwapchainKHR,
    pub get_swapchain_images_khr: PFN_vkGetSwapchainImagesKHR,
    pub acquire_next_image_khr: PFN_vkAcquireNextImageKHR,
    pub queue_present_khr: PFN_vkQueuePresentKHR,
}

impl KhrSwapchainFn {

    pub fn load<F>(cb: F) -> KhrSwapchainFn 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            KhrSwapchainFn {
                create_swapchain_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkCreateSwapchainKHR\0"))),
                destroy_swapchain_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkDestroySwapchainKHR\0"))),
                get_swapchain_images_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkGetSwapchainImagesKHR\0"))),
                acquire_next_image_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkAcquireNextImageKHR\0"))),
                queue_present_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkQueuePresentKHR\0"))),
            }
        }
    }

}

pub type PFN_vkCreateSwapchainKHR = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const SwapchainCreateInfoKHR,
    p_allocator: *const c_void,
    p_swapchain: *mut SwapchainKHR,
) -> VkResult;

pub type PFN_vkDestroySwapchainKHR = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_allocator: *const c_void,
);

pub type PFN_vkGetSwapchainImagesKHR = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_swapchain_image_count: *mut u32,
    p_swapchain_images: *mut Image,
) -> VkResult;


pub type PFN_vkAcquireNextImageKHR = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    timeout: u64,
    semaphore: Semaphore,
    fence: Fence,
    p_image_index: *mut u32,
) -> VkResult;

pub type PFN_vkQueuePresentKHR =
    unsafe extern "system" fn(queue: Queue, p_present_info: *const PresentInfoKHR) -> VkResult;
