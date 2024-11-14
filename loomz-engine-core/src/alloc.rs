use vk::wrapper::Device;

pub const KB: vk::DeviceSize = 1024;
pub const MB: vk::DeviceSize = KB*1000;
pub const GB: vk::DeviceSize = MB*1000;

pub struct VertexAlloc {

}

impl VertexAlloc {

    pub fn new() -> VertexAlloc {
        VertexAlloc {

        }
    }

}

#[derive(Copy, Clone)]
pub struct MemoryRange {
    pub offset: vk::DeviceSize,
    pub size: vk::DeviceSize,
}

/// Memory allocator for gpu resources that are not recreated often
pub struct DeviceMemoryAlloc {
    pub handle: vk::DeviceMemory,
    pub size: vk::DeviceSize,
    pub allocations: Vec<MemoryRange>,
}

impl DeviceMemoryAlloc {

    pub fn new(device: &Device, size: vk::DeviceSize, alloc_capacity: usize, memory_type_index: u32) -> Result<Self, vk::VkResult> {
        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: size,
            memory_type_index,
            ..Default::default()
        };
        let handle = device.allocate_memory(&alloc_info)?;
            
        let alloc = DeviceMemoryAlloc {
            handle,
            size,
            allocations: Vec::with_capacity(alloc_capacity)
        };

        Ok(alloc)
    }

    pub fn free(self, device: &Device) {
        device.free_memory(self.handle);
    }

}

impl Default for DeviceMemoryAlloc {

    fn default() -> Self {
        DeviceMemoryAlloc {
            handle: vk::DeviceMemory::null(),
            size: 0,
            allocations: Vec::new(),
        }
    }

}
