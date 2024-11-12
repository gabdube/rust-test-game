//! Helpers struct

use crate as vk;

/// A device queue definition
#[derive(Copy, Clone, Debug, Default)]
pub struct QueueInfo {
    pub handle: vk::Queue,
    pub family_index: u32,
    pub flags: vk::QueueFlags,
    pub local_index: u32,
}

#[allow(unused)]
impl QueueInfo {

    pub fn is_null(&self) -> bool {
        self.handle == vk::Queue::null()
    }

    pub fn is_valid(&self) -> bool {
        self.handle != vk::Queue::null()
    }

}


/// Combine all the features between vulkan versions and extensions
pub struct CombinedDeviceFeatures {
    pub base: vk::PhysicalDeviceFeatures2,
    pub dynamic_rendering: vk::PhysicalDeviceDynamicRenderingFeatures,
    pub timeline_semaphore: vk::PhysicalDeviceTimelineSemaphoreFeatures,
    pub descriptor_indexing: vk::PhysicalDeviceDescriptorIndexingFeatures,
    pub syncronization2: vk::PhysicalDeviceSynchronization2Features,
}

impl CombinedDeviceFeatures {
    pub fn features_ptr<'a>(&'a mut self) -> &'a vk::PhysicalDeviceFeatures2 {
        self.base.p_next = &self.dynamic_rendering as *const vk::PhysicalDeviceDynamicRenderingFeatures as _;
        self.dynamic_rendering.p_next = &self.timeline_semaphore as *const vk::PhysicalDeviceTimelineSemaphoreFeatures as _;
        self.timeline_semaphore.p_next = &self.descriptor_indexing  as *const vk::PhysicalDeviceDescriptorIndexingFeatures as _;
        self.descriptor_indexing.p_next = &self.syncronization2 as *const vk::PhysicalDeviceSynchronization2Features as _;
        &self.base
    }

    pub fn features_ptr_mut<'a>(&'a mut self) -> &'a mut vk::PhysicalDeviceFeatures2 {
        self.base.p_next = &self.dynamic_rendering as *const vk::PhysicalDeviceDynamicRenderingFeatures as _;
        self.dynamic_rendering.p_next = &self.timeline_semaphore as *const vk::PhysicalDeviceTimelineSemaphoreFeatures as _;
        self.timeline_semaphore.p_next = &self.descriptor_indexing  as *const vk::PhysicalDeviceDescriptorIndexingFeatures as _;
        self.descriptor_indexing.p_next = &self.syncronization2 as *const vk::PhysicalDeviceSynchronization2Features as _;
        &mut self.base
    }
}

impl Default for CombinedDeviceFeatures {
    fn default() -> Self {
        CombinedDeviceFeatures { 
            base: Default::default(),
            dynamic_rendering: Default::default(),
            timeline_semaphore: Default::default(),
            descriptor_indexing: Default::default(),
            syncronization2: Default::default(),
        }
    }
}

#[inline]
pub fn next_ptr<T>(value: &T) -> *const ::std::ffi::c_void {
    value as *const T as *const ::std::ffi::c_void
}
