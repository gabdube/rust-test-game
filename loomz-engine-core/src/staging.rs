

pub(crate) struct StagingBufferCopy {
    pub dst_buffer: vk::Buffer,
    pub copy: vk::BufferCopy  
}

pub struct VulkanStaging {
    pub(crate) memory: vk::DeviceMemory,
    pub(crate) buffer: vk::Buffer,
    pub(crate) mapped_data: Option<*mut u8>,
    pub(crate) upload_offset: vk::DeviceSize,
    pub(crate) buffer_capacity: vk::DeviceSize,
    pub(crate) upload_command_buffer: vk::CommandBuffer,
    pub(crate) vertex_buffer_copies: Vec<StagingBufferCopy>,
}

impl VulkanStaging {

    pub(crate) fn destroy(self, device: &vk::wrapper::Device) {
        device.unmap_memory(self.memory);
        device.destroy_buffer(self.buffer);
        device.free_memory(self.memory);
    }

    pub fn vertex_buffer_copy(&mut self, dst_buffer: vk::Buffer, copy: vk::BufferCopy) {
        self.vertex_buffer_copies.push(StagingBufferCopy { dst_buffer, copy });
    }

    pub fn copy_data<T: Copy>(&mut self, data: &[T]) -> vk::DeviceSize {
        let data_ptr = match self.mapped_data {
            Some(ptr) => ptr,
            None => unreachable!("mapped_data must always be there at runtime")
        };

        let offset = crate::helpers::align_device(self.upload_offset, align_of::<T>() as _);
        let size_bytes = (data.len() * size_of::<T>()) as vk::DeviceSize;

        if offset+size_bytes > self.buffer_capacity {
            Self::not_enough_space_error();
            return 0;
        }
        
        unsafe {
            let dst_offset = data_ptr.offset(offset as _);
            ::std::ptr::copy_nonoverlapping(data.as_ptr(), dst_offset as *mut T, data.len());
        }

        self.upload_offset += size_bytes;

        offset
    }

    #[cold]
    #[inline(never)]
    fn not_enough_space_error() {
        eprintln!("Not enough space left in staging");
    }

}

impl Default for VulkanStaging {

    fn default() -> Self {
        VulkanStaging {
            memory: vk::DeviceMemory::null(),
            buffer: vk::Buffer::null(),
            mapped_data: None,
            upload_offset: 0,
            buffer_capacity: 0,
            upload_command_buffer: vk::CommandBuffer::null(),
            vertex_buffer_copies: Vec::with_capacity(16),
        }
    }

}
