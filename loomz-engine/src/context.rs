use vk::wrapper::{Entry, Instance, Device, Surface, Swapchain, Synchronization2, DynamicRendering};

pub(crate) struct VulkanContextInstance {
    pub entry: Entry,
    pub instance: Instance,
}

pub(crate) struct VulkanContextExtensions {
    pub surface: Surface,
    pub swapchain: Swapchain,
    pub dynamic_rendering: DynamicRendering,
    pub synchronization2: Synchronization2,

    #[cfg(windows)]
    pub win32_surface: vk::wrapper::Win32Surface,

    #[cfg(target_os="linux")]
    pub linux_surface: vk::wrapper::WaylandSurface,
}

pub(crate) struct VulkanContext {
    pub device: Device,
    pub extensions: VulkanContextExtensions,
    pub instance: VulkanContextInstance,
}

impl VulkanContextInstance {

    pub fn destroy(mut self) {
        self.instance.destroy();
        drop(self.entry);
    }

}
