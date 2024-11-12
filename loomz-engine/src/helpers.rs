use loomz_shared::{backend_init_err, CommonError};

#[derive(Default, Copy, Clone)]
pub struct TimelineSemaphore {
    pub handle: vk::Semaphore,
    pub value: u64,
}

impl TimelineSemaphore {

    pub fn new(device: &vk::wrapper::Device) -> Result<Self, CommonError> {
        let sm_timeline_create_info = vk::SemaphoreTypeCreateInfo { 
            semaphore_type: vk::SemaphoreType::TIMELINE,
            initial_value: 0,
            ..Default::default()
        };

        let sm_create_info = vk::SemaphoreCreateInfo { 
            p_next: &sm_timeline_create_info as *const vk::SemaphoreTypeCreateInfo as _,
            ..Default::default()
        };

        let handle = device.create_semaphore(&sm_create_info)
            .map_err(|err| backend_init_err!("Failed to create timeline semaphore: {:?}", err) )?;

        Ok(TimelineSemaphore { handle, value: 0 })
    }

}

#[derive(Copy, Clone, Default)]
pub struct Attachment {
    pub image: vk::Image,
    pub view: vk::ImageView,
}

#[derive(Default)]
pub struct RenderAttachments {
    pub memory: vk::DeviceMemory,
    pub color: Attachment,
    pub depth: Attachment,

    // Output attachments size depends on the number of image in the swapchain
    pub output: Vec<Attachment>,
}

impl RenderAttachments {
    pub fn free(&self, device: &vk::wrapper::Device) {
        device.destroy_image_view(self.color.view);
        device.destroy_image_view(self.depth.view);
        for out in self.output.iter() {
            device.destroy_image_view(out.view);
        }

        device.destroy_image(self.color.image);
        device.destroy_image(self.depth.image);
        device.free_memory(self.memory);
    }
}

/// align must be a power of 2
#[inline]
pub const fn align_device(addr: vk::DeviceSize, align: vk::DeviceSize) -> vk::DeviceSize {
    let addr = addr as isize;
    let align = align as isize;
    ((addr + (align - 1)) & -align) as vk::DeviceSize
}

#[inline]
pub(crate) fn begin_record(device: &vk::wrapper::Device, cmd: vk::CommandBuffer) -> Result<(), vk::VkResult> {
    let begin_info = vk::CommandBufferBeginInfo {
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
    };
    device.begin_command_buffer(cmd, &begin_info)
}

#[inline]
pub(crate) fn end_record(device: &vk::wrapper::Device, cmd: vk::CommandBuffer) -> Result<(), vk::VkResult> {
    device.end_command_buffer(cmd)
}

pub fn fetch_attachments_memory_index(instance: &vk::wrapper::Instance, physical_device: vk::PhysicalDevice) -> u32 {
    let properties = instance.get_physical_device_memory_properties(physical_device);
    
    let count = properties.memory_type_count as usize;
    let mut memory_type_index = None;

    for (i, memory_type) in properties.memory_types[0..count].iter().enumerate() {
        if memory_type.property_flags == vk::MemoryPropertyFlags::DEVICE_LOCAL {
            return i as u32;
        }

        if memory_type.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) {
            memory_type_index = Some(i as u32);
        }
    }

    match memory_type_index {
        Some(i) => i,
        None => unreachable!("There must be a memory type DEVICE_LOCAL")
    }
}
