use crate as vk;

pub struct Synchronization2 {
    pub synchronization2_fn: vk::Synchronization2Fn,
}

impl Synchronization2 {

    pub fn new(instance: &vk::wrapper::Instance, device: &vk::wrapper::Device) -> Synchronization2 {
        let synchronization2_fn = vk::Synchronization2Fn::load(|name| {
            unsafe { instance.get_device_proc_addr(device.handle, name.to_bytes_with_nul().as_ptr()) }
        });

        Synchronization2 {
            synchronization2_fn,
        }
    }

    pub fn cmd_pipeline_barrier2(&self, cmd: vk::CommandBuffer, dependency_info: &vk::DependencyInfo) {
        unsafe { (self.synchronization2_fn.cmd_pipeline_barrier_2_khr)(cmd, dependency_info) }
    }

    pub fn queue_submit2(&self, queue: vk::Queue, submit_info: &[vk::SubmitInfo2], fence: vk::Fence) -> Result<(), vk::VkResult> {
        unsafe { (self.synchronization2_fn.cmd_queue_submit_2_khr)(queue, submit_info.len() as u32, submit_info.as_ptr(), fence).as_result() }
    }

}
