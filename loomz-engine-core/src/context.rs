use vk::wrapper::{Entry, Instance, Device, Surface, Swapchain, Synchronization2, DynamicRendering};

#[cfg(target_os="linux")]
pub struct VulkanLinuxSurfaces {
    pub wayland_surface: vk::wrapper::WaylandSurface,
}

pub struct VulkanContextInstance {
    pub entry: Entry,
    pub instance: Instance,
}

pub struct VulkanContextExtensions {
    pub surface: Surface,
    pub swapchain: Swapchain,
    pub dynamic_rendering: DynamicRendering,
    pub synchronization2: Synchronization2,

    #[cfg(windows)]
    pub win32_surface: vk::wrapper::Win32Surface,

    #[cfg(target_os="linux")]
    pub linux_surface: VulkanLinuxSurfaces,

    #[cfg(target_os="macos")]
    pub metal_surface: vk::wrapper::MetalSurface,
}

pub struct VulkanContext {
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
