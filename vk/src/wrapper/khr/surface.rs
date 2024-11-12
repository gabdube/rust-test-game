use crate as vk;
use std::ptr;

pub struct Surface {
    handle: vk::Instance,
    surface_fn: vk::KhrSurfaceFn,
}

impl Surface {

    pub fn new(entry: &vk::wrapper::Entry, instance: &vk::wrapper::Instance) -> Surface {
        let surface_fn = vk::KhrSurfaceFn::load(|name| {
            unsafe { entry.get_instance_proc_addr(instance.handle, name.to_bytes_with_nul().as_ptr()) }
        });
        
        Surface {
            handle: instance.handle,
            surface_fn,
        }
    }

    pub fn destroy_surface(&self, surface: vk::SurfaceKHR) {
        unsafe {
            (self.surface_fn.destroy_surface_khr)(self.handle, surface, ptr::null());
        }
    }

    pub fn get_physical_device_surface_support(&self, pdevice: vk::PhysicalDevice, queue_family_index: u32, surface: vk::SurfaceKHR) -> Result<bool, vk::VkResult> {
        unsafe {
            let mut supported = 0;
            (self.surface_fn.get_physical_device_surface_support_khr)(pdevice, queue_family_index, surface, &mut supported)
                .as_result()
                .map(|_| supported > 0)
        }
    }

    pub fn get_physical_device_surface_capabilities(&self, pdevice: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Result<vk::SurfaceCapabilitiesKHR, vk::VkResult> {
        unsafe {
            let mut caps = ::std::mem::zeroed();
            (self.surface_fn.get_physical_device_surface_capabilities_khr)(pdevice, surface, &mut caps)
                .as_result()
                .map(|_| caps)
        }
    }

    pub fn get_physical_device_surface_present_modes(&self, pdevice: vk::PhysicalDevice, surface: vk::SurfaceKHR, present_modes: &mut Vec<vk::PresentModeKHR>) -> Result<(), vk::VkResult> {
        unsafe {
            let mut count = 0;

            (self.surface_fn.get_physical_device_surface_present_modes_khr)(pdevice, surface, &mut count, ptr::null_mut()).as_result()?;

            *present_modes = vec![Default::default(); count as usize];

            (self.surface_fn.get_physical_device_surface_present_modes_khr)(pdevice, surface, &mut count, present_modes.as_mut_ptr()).as_result()
        }
    }

    pub fn get_physical_device_surface_formats(&self, pdevice: vk::PhysicalDevice, surface: vk::SurfaceKHR, formats: &mut Vec<vk::SurfaceFormatKHR>) -> Result<(), vk::VkResult> {
        unsafe {
            let mut count = 0;

            (self.surface_fn.get_physical_device_surface_formats_khr)(pdevice, surface, &mut count, ptr::null_mut()).as_result()?;

            *formats = vec![Default::default(); count as usize];

            (self.surface_fn.get_physical_device_surface_formats_khr)(pdevice, surface, &mut count, formats.as_mut_ptr()).as_result()
        }
    }

}
