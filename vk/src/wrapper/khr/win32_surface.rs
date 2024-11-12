use crate as vk;
use std::ptr;


pub struct Win32Surface {
    handle: vk::Instance,
    win32_surface_fn: vk::KhrWin32SurfaceFn,
}

impl Win32Surface {

    pub fn new(entry: &vk::wrapper::Entry, instance: &vk::wrapper::Instance) -> Win32Surface {
        let win32_surface_fn = vk::KhrWin32SurfaceFn::load(|name| {
            unsafe { entry.get_instance_proc_addr(instance.handle, name.to_bytes_with_nul().as_ptr()) }
        });
        
        Win32Surface {
            handle: instance.handle,
            win32_surface_fn,
        }
    }

}

impl Win32Surface {

    pub fn create_win32_surface(&self, create_info: &vk::Win32SurfaceCreateInfoKHR) -> Result<vk::SurfaceKHR, vk::VkResult> {
        unsafe {
            let mut handle = vk::SurfaceKHR::null();
            (self.win32_surface_fn.create_win32_surface_khr)(self.handle, create_info, ptr::null(), &mut handle)
                .as_result()
                .map(|_| handle)
        }
    }

}
