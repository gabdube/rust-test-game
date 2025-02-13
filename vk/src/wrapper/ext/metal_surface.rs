use crate as vk;
use std::ptr;


pub struct MetalSurface {
    handle: vk::Instance,
    metal_surface_fn: vk::ExtMetalSurfaceFn,
}

impl MetalSurface {

    pub fn new(entry: &vk::wrapper::Entry, instance: &vk::wrapper::Instance) -> MetalSurface {
        let metal_surface_fn = vk::ExtMetalSurfaceFn::load(|name| {
            unsafe { entry.get_instance_proc_addr(instance.handle, name.to_bytes_with_nul().as_ptr()) }
        });
        
        MetalSurface {
            handle: instance.handle,
            metal_surface_fn,
        }
    }

}

impl MetalSurface {

    pub fn create_metal_surface(&self, create_info: &vk::MetalSurfaceCreateInfoEXT) -> Result<vk::SurfaceKHR, vk::VkResult> {
        unsafe {
            let mut handle = vk::SurfaceKHR::null();
            (self.metal_surface_fn.create_metal_surface_ext)(self.handle, create_info, ptr::null(), &mut handle)
                .as_result()
                .map(|_| handle)
        }
    }

}
