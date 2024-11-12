use crate as vk;
use std::ptr;


pub struct WaylandSurface {
    handle: vk::Instance,
    wayland_surface_fn: vk::KhrWaylandSurfaceFn,
}

impl WaylandSurface {

    pub fn new(entry: &vk::wrapper::Entry, instance: &vk::wrapper::Instance) -> WaylandSurface {
        let wayland_surface_fn = vk::KhrWaylandSurfaceFn::load(|name| {
            unsafe { entry.get_instance_proc_addr(instance.handle, name.to_bytes_with_nul().as_ptr()) }
        });
        
        WaylandSurface {
            handle: instance.handle,
            wayland_surface_fn,
        }
    }

}

impl WaylandSurface {

    pub fn create_wayland_surface(&self, create_info: &vk::WaylandSurfaceCreateInfoKHR) -> Result<vk::SurfaceKHR, vk::VkResult> {
        unsafe {
            let mut handle = vk::SurfaceKHR::null();
            (self.wayland_surface_fn.create_wayland_surface_khr)(self.handle, create_info, ptr::null(), &mut handle)
                .as_result()
                .map(|_| handle)
        }
    }

}
