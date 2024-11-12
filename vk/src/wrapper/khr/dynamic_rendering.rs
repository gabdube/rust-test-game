use crate as vk;

pub struct DynamicRendering {
    pub dynamic_rendering_fn: vk::DynamicRenderingFn,
}

impl DynamicRendering {

    pub fn new(instance: &vk::wrapper::Instance, device: &vk::wrapper::Device) -> DynamicRendering {
        let dynamic_rendering_fn = vk::DynamicRenderingFn::load(|name| {
            unsafe { instance.get_device_proc_addr(device.handle, name.to_bytes_with_nul().as_ptr()) }
        });

        DynamicRendering {
            dynamic_rendering_fn,
        }
    }

    pub fn begin_rendering(&self, cmd: vk::CommandBuffer, info: &vk::RenderingInfo) {
        unsafe {
            (self.dynamic_rendering_fn.cmd_begin_rendering_khr)(cmd, info)
        }
    }

    pub fn end_rendering(&self, cmd: vk::CommandBuffer) {
        unsafe {
            (self.dynamic_rendering_fn.cmd_end_rendering_khr)(cmd)
        }
    }

}
