pub(crate) struct StagingBufferCopy {
    pub dst_buffer: vk::Buffer,
    pub copy: vk::BufferCopy  
}

pub(crate) struct StagingImageCopy {
    pub dst_image: vk::Image,
    pub copy: vk::BufferImageCopy,
}

pub struct VulkanStaging {
    pub(crate) memory: vk::DeviceMemory,
    pub(crate) buffer: vk::Buffer,
    pub(crate) mapped_data: Option<*mut u8>,
    pub(crate) upload_offset: vk::DeviceSize,
    pub(crate) buffer_capacity: vk::DeviceSize,
    pub(crate) upload_command_buffer: vk::CommandBuffer,

    pub(crate) vertex_buffer_copies: Vec<StagingBufferCopy>,

    pub(crate) image_barrier_prepare: Vec<vk::ImageMemoryBarrier2>,
    pub(crate) image_copies: Vec<StagingImageCopy>,
    pub(crate) image_barrier_final: Vec<vk::ImageMemoryBarrier2>,
}

impl VulkanStaging {

    pub(crate) fn destroy(mut self, device: &vk::wrapper::Device) {
        self.mapped_data = None;
        device.unmap_memory(self.memory);
        device.destroy_buffer(self.buffer);
        device.free_memory(self.memory);
    }

    pub fn vertex_buffer_copy(&mut self, dst_buffer: vk::Buffer, copy: vk::BufferCopy) {
        self.vertex_buffer_copies.push(StagingBufferCopy { dst_buffer, copy });
    }

    pub fn copy_data_with_align<T: Copy>(&mut self, data: &[T], align: usize) -> vk::DeviceSize {
        let data_ptr = match self.mapped_data {
            Some(ptr) => ptr,
            None => unreachable!("mapped_data must always be mapped at runtime")
        };

        let offset_bytes = crate::helpers::pad_device(self.upload_offset, align as _);
        let size_bytes = (data.len() * size_of::<T>()) as vk::DeviceSize;

        if offset_bytes+size_bytes > self.buffer_capacity {
            Self::not_enough_space_error();
            return 0;
        }

        unsafe {
            let dst_offset = data_ptr.offset(offset_bytes as _);
            let (_, data_aligned, _) = data.align_to::<u8>();

            //println!("Copying {} bytes at {} with alignment {}. Next offset: {}", size_bytes, offset_bytes, align, offset_bytes + size_bytes);

            ::std::ptr::copy_nonoverlapping::<u8>(data_aligned.as_ptr(), dst_offset, size_bytes as usize);
        }

        self.upload_offset = offset_bytes + size_bytes;

        offset_bytes
    }

    pub fn copy_data<T: Copy>(&mut self, data: &[T]) -> vk::DeviceSize {
        self.copy_data_with_align(data, align_of::<T>())
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
            image_barrier_prepare: Vec::with_capacity(8),
            image_copies: Vec::with_capacity(8),
            image_barrier_final: Vec::with_capacity(8),
        }
    }

}
