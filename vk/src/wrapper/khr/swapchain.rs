use crate as vk;
use std::ptr;


#[derive(Copy, Clone)]
pub struct Swapchain {
    handle: vk::Device,
    swapchain_fn: vk::KhrSwapchainFn,
}

impl Swapchain {

    pub fn new(instance: &vk::wrapper::Instance, device: &vk::wrapper::Device) -> Swapchain {
        let swapchain_fn = vk::KhrSwapchainFn::load(|name| {
            unsafe { instance.get_device_proc_addr(device.handle, name.to_bytes_with_nul().as_ptr()) }
        });
        
        Swapchain {
            handle: device.handle,
            swapchain_fn,
        }
    }

    pub fn create_swapchain(&self, create_info: &vk::SwapchainCreateInfoKHR) -> Result<vk::SwapchainKHR, vk::VkResult> {
        unsafe {
            let mut swapchain = vk::SwapchainKHR::null();
            (self.swapchain_fn.create_swapchain_khr)(self.handle, create_info, ptr::null(), &mut swapchain)
                .as_result()
                .map(|_| swapchain)
        }
    }

    pub fn destroy_swapchain(&self, swapchain: vk::SwapchainKHR) {
        unsafe {
            (self.swapchain_fn.destroy_swapchain_khr)(self.handle, swapchain, ptr::null());
        }
    }

    pub fn get_swapchain_images(&self, swapchain: vk::SwapchainKHR, images: &mut [vk::Image]) -> Result<(), vk::VkResult> {
        unsafe {
            let mut count = 0;
            (self.swapchain_fn.get_swapchain_images_khr)(self.handle, swapchain, &mut count, ptr::null_mut()).as_result()?;

            assert!(images.len() >= count as usize, "Images slice is not large enough");
            (self.swapchain_fn.get_swapchain_images_khr)(self.handle, swapchain, &mut count, images.as_mut_ptr()).as_result()
        }
    }

    pub fn acquire_next_image(&self, swapchain: vk::SwapchainKHR, timeout: u64, semaphore: vk::Semaphore, fence: vk::Fence, image_index: &mut u32) -> Result<(), vk::VkResult> {
        unsafe {
            (self.swapchain_fn.acquire_next_image_khr)(self.handle, swapchain, timeout, semaphore, fence, image_index)
                .as_result()
        }
    }

    pub fn queue_present(&self, queue: vk::Queue, present_info: &vk::PresentInfoKHR) -> Result<(), vk::VkResult> {
        unsafe {
            (self.swapchain_fn.queue_present_khr)(queue, present_info).as_result()
        }
    }

}
