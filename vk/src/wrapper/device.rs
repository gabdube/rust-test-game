use crate::*;
use crate::vk100::Device as DeviceHandle;
use super::QueueInfo;
use std::{ptr, ffi};

#[repr(C)]
pub struct Device {
    pub handle: DeviceHandle,
    pub physical_device: PhysicalDevice,
    pub queues: Vec<QueueInfo>,
    pub device_fn_1_0: DeviceFnV1_0,
    pub device_fn_1_2: DeviceFnV1_2,
}

impl Device {

    pub fn load(instance: &wrapper::Instance, device_create_info: &DeviceCreateInfo, physical_device: PhysicalDevice, handle: DeviceHandle) -> Device {
        let device_fn_1_0 = DeviceFnV1_0::load(|name| {
            unsafe { instance.get_device_proc_addr(handle, name.to_bytes_with_nul().as_ptr()) }
        });

        let device_fn_1_2 = DeviceFnV1_2::load(|name| {
            unsafe { instance.get_device_proc_addr(handle, name.to_bytes_with_nul().as_ptr()) }
        });

        let queues_family_properties = instance.get_physical_device_queue_family_properties(physical_device);

        let queue_count = device_create_info.queue_create_info_count as usize;
        let mut queues = Vec::with_capacity(queue_count);
        for i in 0..queue_count {
            let queue_info = unsafe { &*(device_create_info.p_queue_create_infos.add(i)) };

            let queue_family_index = queue_info.queue_family_index;
            let flags = queues_family_properties[queue_family_index as usize].queue_flags;

            let max_queue_local_index = queue_info.queue_count;

            for queue_local_index in 0..max_queue_local_index {
                let mut queue = Queue::null();
                unsafe { (device_fn_1_0.get_device_queue)(handle, queue_family_index, queue_local_index, &mut queue); }

                queues.push(
                    QueueInfo { 
                        handle: queue,
                        family_index: queue_family_index,
                        local_index: queue_local_index,
                        flags
                    }
                );
            }
        }

        Device {
            handle,
            physical_device,
            queues,
            device_fn_1_0,
            device_fn_1_2,
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            (self.device_fn_1_0.destroy_device)(self.handle, ptr::null());
            self.handle = DeviceHandle::null();
        }
    }

    pub fn device_wait_idle(&self) -> Result<(), VkResult> {
        unsafe {
            (self.device_fn_1_0.device_wait_idle)(self.handle).as_result()
        }
    }

    pub fn queue_wait_idle(&self, queue: Queue) -> Result<(), VkResult> {
        unsafe {
            (self.device_fn_1_0.queue_wait_idle)(queue).as_result()
        }
    }

    pub fn get_device_queue(&self, queue_family_index: u32, queue_index: u32) -> Queue {
        unsafe {
            let mut queue = Queue::null();
            (self.device_fn_1_0.get_device_queue)(self.handle, queue_family_index, queue_index, &mut queue);
            queue
        }
    }

    pub fn queue_submit(&self, queue: Queue, submit: &[SubmitInfo], fence: Fence) -> Result<(), VkResult> {
        unsafe {
            (self.device_fn_1_0.queue_submit)(queue, submit.len() as _, submit.as_ptr(), fence).as_result()
        }
    }

    pub fn allocate_memory(&self, alloc_info: &MemoryAllocateInfo) -> Result<DeviceMemory, VkResult> {
        unsafe {
            let mut mem = DeviceMemory::null();
            (self.device_fn_1_0.allocate_memory)(self.handle, alloc_info, ptr::null(), &mut mem)
                .as_result()
                .map(|_| mem)
        }
    }

    pub fn free_memory(&self, memory: DeviceMemory) {
        unsafe {
            (self.device_fn_1_0.free_memory)(self.handle, memory, ptr::null());
        }
    }

    pub fn create_image(&self, create_info: &ImageCreateInfo) -> Result<Image, VkResult> {
        unsafe {
            let mut image = Image::null();
            (self.device_fn_1_0.create_image)(self.handle, create_info, ptr::null(), &mut image)
                .as_result()
                .map(|_| image)
        }
    }

    pub fn destroy_image(&self, image: Image) {
        unsafe {
            (self.device_fn_1_0.destroy_image)(self.handle, image, ptr::null());
        }
    }

    pub fn get_image_memory_requirements(&self, image: Image) -> MemoryRequirements {
        unsafe {
            let mut req = Default::default();
            (self.device_fn_1_0.get_image_memory_requirements)(self.handle, image, &mut req);
            req
        }
    }

    pub fn bind_image_memory(&self, image: Image, memory: DeviceMemory, memory_offset: DeviceSize) -> Result<(), VkResult> {
        unsafe {
            (self.device_fn_1_0.bind_image_memory)(self.handle, image, memory, memory_offset).as_result()
        }
    }

    pub fn create_image_view(&self, create_info: &ImageViewCreateInfo) -> Result<ImageView, VkResult> {
        unsafe {
            let mut view = ImageView::null();
            (self.device_fn_1_0.create_image_view)(self.handle, create_info, ptr::null(), &mut view)
                .as_result()
                .map(|_| view)
        }
    }

    pub fn destroy_image_view(&self, image_view: ImageView) {
        unsafe { 
            (self.device_fn_1_0.destroy_image_view)(self.handle, image_view, ptr::null()); 
        }
    }

    pub fn create_fence(&self, create_info: &FenceCreateInfo) -> Result<Fence, VkResult> {
        unsafe {
            let mut fence = Fence::null();
            (self.device_fn_1_0.create_fence)(self.handle, create_info, ptr::null(), &mut fence)
                .as_result()
                .map(|_| fence)
        }
    }

    pub fn destroy_fence(&self, fence: Fence) {
        unsafe {
            (self.device_fn_1_0.destroy_fence)(self.handle, fence, ptr::null());
        }
    }

    pub fn create_semaphore(&self, create_info: &SemaphoreCreateInfo) -> Result<Semaphore, VkResult> {
        unsafe {
            let mut semaphore = Semaphore::null();
            (self.device_fn_1_0.create_semaphore)(self.handle, create_info, ptr::null(), &mut semaphore)
                .as_result()
                .map(|_| semaphore)
        }
    }

    pub fn destroy_semaphore(&self, semaphore: Semaphore) {
        unsafe {
            (self.device_fn_1_0.destroy_semaphore)(self.handle, semaphore, ptr::null());
        }
    }

    pub fn wait_for_fences(&self, fences: &[Fence], wait_all: bool, timeout: u64) -> Result<(), VkResult> {
        unsafe {
            (self.device_fn_1_0.wait_for_fences)(
                self.handle,
                fences.len() as _,
                fences.as_ptr(),
                wait_all as u32,
                timeout,
            )
            .as_result()
        }
    }

    pub fn reset_fences(&self, fences: &[Fence]) -> Result<(), VkResult> {
        unsafe {
            (self.device_fn_1_0.reset_fences)(self.handle, fences.len() as _, fences.as_ptr())
                .as_result()
        }
    }

    pub fn create_command_pool(&self, create_info: &CommandPoolCreateInfo) -> Result<CommandPool, VkResult> {
        unsafe {
            let mut pool = CommandPool::null();

            (self.device_fn_1_0.create_command_pool)(self.handle, create_info, ptr::null(), &mut pool)
                .as_result()
                .map(|_| pool)
        }
    }

    pub fn destroy_command_pool(&self, pool: CommandPool) {
        unsafe {
            (self.device_fn_1_0.destroy_command_pool)(self.handle, pool, ptr::null());
        }
    }

    pub fn allocate_command_buffers(&self, alloc: &CommandBufferAllocateInfo, cmd: &mut [CommandBuffer]) -> Result<(), VkResult> {
        unsafe {
            assert!(alloc.command_buffer_count as usize <= cmd.len(), "Command buffer array must be larger than command_buffer_count");
            (self.device_fn_1_0.allocate_command_buffers)(self.handle, alloc, cmd.as_mut_ptr()).as_result()
        }
    }

    pub fn begin_command_buffer(&self, cmd: CommandBuffer, begin_info: &CommandBufferBeginInfo) -> Result<(), VkResult> {
        unsafe {
            (self.device_fn_1_0.begin_command_buffer)(cmd, begin_info).as_result()
        }
    }

    pub fn end_command_buffer(&self, cmd: CommandBuffer) -> Result<(), VkResult> {
        unsafe {
            (self.device_fn_1_0.end_command_buffer)(cmd).as_result()
        }
    }

    pub fn create_render_pass(&self, create_info: &RenderPassCreateInfo) -> Result<RenderPass, VkResult> {
        unsafe {
            let mut render_pass = RenderPass::null();
            (self.device_fn_1_0.create_render_pass)(self.handle, create_info, ptr::null(), &mut render_pass)
                .as_result()
                .map(|_| render_pass)
        }
    }

    pub fn destroy_render_pass(&self, render_pass: RenderPass) {
        unsafe {
            (self.device_fn_1_0.destroy_render_pass)(self.handle, render_pass, ptr::null());
        }
    }

    pub fn create_framebuffer(&self, create_info: &FramebufferCreateInfo) -> Result<Framebuffer, VkResult> {
        unsafe {
            let mut framebuffer = Framebuffer::null();
            (self.device_fn_1_0.create_framebuffer)(self.handle, create_info, ptr::null(), &mut framebuffer)
                .as_result()
                .map(|_| framebuffer)
        }
    }

    pub fn destroy_framebuffer(&self, framebuffer: Framebuffer) {
        unsafe { 
            (self.device_fn_1_0.destroy_framebuffer)(self.handle, framebuffer, ptr::null()); 
        }
    }

    pub fn create_buffer(&self, create_info: &BufferCreateInfo) -> Result<Buffer, VkResult> {
        unsafe {
            let mut buffer = Buffer::null();
            (self.device_fn_1_0.create_buffer)(self.handle, create_info, ptr::null(), &mut buffer)
                .as_result()
                .map(|_| buffer)
        }
    }

    pub fn destroy_buffer(&self, buffer: Buffer) {
        unsafe {
            (self.device_fn_1_0.destroy_buffer)(self.handle, buffer, ptr::null());
        }
    }

    pub fn get_buffer_memory_requirements(&self, buffer: Buffer) -> MemoryRequirements {
        unsafe {
            let mut req = Default::default();
            (self.device_fn_1_0.get_buffer_memory_requirements)(self.handle, buffer, &mut req);
            req
        }
    }

    pub fn bind_buffer_memory(&self, buffer: Buffer, mem: DeviceMemory, offset: DeviceSize) -> Result<(), VkResult> {
        unsafe {
            (self.device_fn_1_0.bind_buffer_memory)(self.handle, buffer, mem, offset).as_result()
        }
    }

    pub fn map_memory(&self, device_memory: DeviceMemory, offset: DeviceSize, size: DeviceSize) -> Result<*mut ffi::c_void, VkResult> {
        unsafe {
            let mut out_ptr = ptr::null_mut();
            (self.device_fn_1_0.map_memory)(
                self.handle,
                device_memory,
                offset,
                size,
                MemoryMapFlags(0),
                &mut out_ptr
            )
            .as_result()
            .map(|_| out_ptr)
        }
    }

    pub fn unmap_memory(&self, device_memory: DeviceMemory) {
        unsafe {
            (self.device_fn_1_0.unmap_memory)(self.handle, device_memory);
        }
    }

    pub fn create_shader_module(&self, info: &ShaderModuleCreateInfo) -> Result<ShaderModule, VkResult> {
        unsafe {
            let mut shader = ShaderModule::default();
            (self.device_fn_1_0.create_shader_module)(self.handle, info, ptr::null(), &mut shader)
                .as_result()
                .map(|_| shader)
        }
    }

    pub fn destroy_shader_module(&self, shader: ShaderModule) {
        unsafe {
            (self.device_fn_1_0.destroy_shader_module)(self.handle, shader, ptr::null());
        }
    }

    pub fn create_pipeline_cache(&self, create_info: &PipelineCacheCreateInfo) -> Result<PipelineCache, VkResult> {
        unsafe {
            let mut cache = PipelineCache::null();
            (self.device_fn_1_0.create_pipeline_cache)(self.handle, create_info, ptr::null(), &mut cache)
                .as_result()
                .map(|_| cache)
        }
    }

    pub fn get_pipeline_cache_data(&self, pipeline_cache: PipelineCache) -> Result<Vec<u8>, VkResult> {
        unsafe {
            let mut data_size = 0;
            (self.device_fn_1_0.get_pipeline_cache_data)(self.handle, pipeline_cache, &mut data_size, ptr::null_mut())
                .as_result()?;

            let mut data = vec![0u8; data_size];
            (self.device_fn_1_0.get_pipeline_cache_data)(self.handle, pipeline_cache, &mut data_size, data.as_mut_ptr() as *mut _)
                .as_result()?;

            Ok(data)
        }
    }

    pub fn destroy_pipeline_cache(&self, cache: PipelineCache) {
        unsafe {
            (self.device_fn_1_0.destroy_pipeline_cache)(self.handle, cache, ptr::null());
        }
    }

    pub fn destroy_pipeline(&self, pipeline: Pipeline) {
        unsafe {
            (self.device_fn_1_0.destroy_pipeline)(self.handle, pipeline, ptr::null());
        }
    }

    pub fn create_graphics_pipelines(&self, pipeline_cache: PipelineCache, create_info: &[GraphicsPipelineCreateInfo], pipelines: &mut [Pipeline]) -> Result<(), VkResult> {
        unsafe {
            assert!(create_info.len() <= pipelines.len(), "Create count must be lesser or equal to the pipelines vec count");
            (self.device_fn_1_0.create_graphics_pipelines)(
                self.handle,
                pipeline_cache,
                create_info.len() as _,
                create_info.as_ptr(),
                ptr::null(),
                pipelines.as_mut_ptr()
            )
            .as_result()
        }
    }

    pub fn create_compute_pipelines(&self, pipeline_cache: PipelineCache, create_info: &[ComputePipelineCreateInfo], pipelines: &mut [Pipeline]) -> Result<(), VkResult> {
        unsafe {
            assert!(create_info.len() <= pipelines.len(), "Create count must be lesser or equal to the pipelines vec count");
            (self.device_fn_1_0.create_compute_pipelines)(
                self.handle,
                pipeline_cache,
                create_info.len() as _,
                create_info.as_ptr(),
                ptr::null(),
                pipelines.as_mut_ptr()
            )
            .as_result()
        }
    }

    pub fn create_pipeline_layout(&self, create_info: &PipelineLayoutCreateInfo) -> Result<PipelineLayout, VkResult> {
        unsafe {
            let mut pipeline_layout = PipelineLayout::null();
            (self.device_fn_1_0.create_pipeline_layout)(self.handle, create_info, ptr::null(), &mut pipeline_layout)
                .as_result()
                .map(|_| pipeline_layout)
        }
    }

    pub fn destroy_pipeline_layout(&self, pipeline_layout: PipelineLayout) {
        unsafe {
            (self.device_fn_1_0.destroy_pipeline_layout)(self.handle, pipeline_layout, ptr::null());
        }
    }

    pub fn create_descriptor_set_layout(&self, create_info: &DescriptorSetLayoutCreateInfo) -> Result<DescriptorSetLayout, VkResult> {
        unsafe {
            let mut layout = DescriptorSetLayout::null();
            (self.device_fn_1_0.create_descriptor_set_layout)(self.handle, create_info, ptr::null(), &mut layout)
                    .as_result()
                    .map(|_| layout)
        }
    }

    pub fn destroy_descriptor_set_layout(&self, descriptor_set_layout: DescriptorSetLayout) {
        unsafe {
            (self.device_fn_1_0.destroy_descriptor_set_layout)(self.handle, descriptor_set_layout, ptr::null());
        }
    }

    pub fn create_descriptor_pool(&self, create_info: &DescriptorPoolCreateInfo) -> Result<DescriptorPool, VkResult> {
        unsafe {
            let mut pool = DescriptorPool::null();
            (self.device_fn_1_0.create_descriptor_pool)(self.handle, create_info, ptr::null(), &mut pool)
                .as_result()
                .map(|_| pool)
        }
    }

    pub fn destroy_descriptor_pool(&self, pool: DescriptorPool) {
        unsafe {
            (self.device_fn_1_0.destroy_descriptor_pool)(self.handle, pool, ptr::null());
        }
    }

    pub fn allocate_descriptor_sets(&self, allocate_info: &DescriptorSetAllocateInfo, descriptor_sets: &mut [DescriptorSet]) -> Result<(), VkResult> {
        assert!(allocate_info.descriptor_set_count as usize == descriptor_sets.len(), "Descriptor sets array does not match allocate_info.descriptor_set_count");

        unsafe {
            (self.device_fn_1_0.allocate_descriptor_sets)(
                self.handle,
                allocate_info,
                descriptor_sets.as_mut_ptr()
            )
            .as_result()
        }
    }

    pub fn update_descriptor_sets(&self, writes: &[WriteDescriptorSet], copy: &[CopyDescriptorSet]) {
        let writes_len = writes.len() as u32;
        let copy_len = copy.len() as u32;

        let writes_ptr = match writes_len > 0 {
            true => writes.as_ptr(),
            false => ptr::null()
        };

        let copy_ptr = match copy_len > 0 {
            true => copy.as_ptr(),
            false => ptr::null()
        };
        
        unsafe {
            (self.device_fn_1_0.update_descriptor_sets)(
                self.handle,
                writes_len,
                writes_ptr,
                copy_len,
                copy_ptr,
            );
        }
    }

    pub fn create_sampler(&self, create_info: &SamplerCreateInfo) -> Result<Sampler, VkResult> {
        unsafe {
            let mut sampler = Sampler::null();
            (self.device_fn_1_0.create_sampler)(self.handle, create_info, ptr::null(), &mut sampler)
                .as_result()
                .map(|_| sampler)
        }
    }

    pub fn destroy_sampler(&self, sampler: Sampler) {
        unsafe {
            (self.device_fn_1_0.destroy_sampler)(self.handle, sampler, ptr::null());
        }
    }

    pub fn create_buffer_view(&self, create_info: &BufferViewCreateInfo) -> Result<BufferView, VkResult> {
        unsafe {
            let mut buffer_view = BufferView::null();
            (self.device_fn_1_0.create_buffer_view)(self.handle, create_info, ptr::null(), &mut buffer_view)
                .as_result()
                .map(|_| buffer_view)
        }
    }

    pub fn destroy_buffer_view(&self, buffer_view: BufferView) {
        unsafe {
            (self.device_fn_1_0.destroy_buffer_view)(self.handle, buffer_view, ptr::null());
        }
    }


    pub fn cmd_begin_render_pass(&self, cmd: CommandBuffer, begin_info: &RenderPassBeginInfo, contents: SubpassContents) {
        unsafe {
            (self.device_fn_1_0.cmd_begin_render_pass)(cmd, begin_info, contents);
        }
    }

    pub fn cmd_end_render_pass(&self, cmd: CommandBuffer) {
        unsafe {
            (self.device_fn_1_0.cmd_end_render_pass)(cmd);
        }
    }

    pub fn cmd_copy_buffer(&self, cmd: CommandBuffer, src: Buffer, dst: Buffer, regions: &[BufferCopy]) {
        unsafe {
            (self.device_fn_1_0.cmd_copy_buffer)(cmd, src, dst, regions.len() as _, regions.as_ptr());
        }
    }

    pub fn cmd_copy_buffer_to_image(&self, cmd: CommandBuffer, src: Buffer, dst: Image, dst_layout: ImageLayout, regions: &[BufferImageCopy]) {
        unsafe {
            (self.device_fn_1_0.cmd_copy_buffer_to_image)(cmd, src, dst, dst_layout, regions.len() as _, regions.as_ptr());
        }
    }

    pub fn cmd_bind_pipeline(&self, cmd: CommandBuffer, pipeline_bind_point: PipelineBindPoint, pipeline: Pipeline) {
        unsafe {
            (self.device_fn_1_0.cmd_bind_pipeline)(cmd, pipeline_bind_point, pipeline);
        }
    }

    pub fn cmd_bind_index_buffer(&self, cmd: CommandBuffer, buffer: Buffer, offset: DeviceSize, index_type: IndexType) {
        unsafe {
            (self.device_fn_1_0.cmd_bind_index_buffer)(cmd, buffer, offset, index_type);
        }
    }

    pub fn cmd_bind_vertex_buffers(&self, cmd: CommandBuffer, first_binding: u32, buffers: &[Buffer], offsets: &[DeviceSize]) {
        let binding_count = buffers.len() as u32;
        assert!(buffers.len() == offsets.len(), "buffers array must have the same size as offsets array");
        unsafe {
            (self.device_fn_1_0.cmd_bind_vertex_buffers)(cmd, first_binding, binding_count, buffers.as_ptr(), offsets.as_ptr());
        }
    }

    pub fn cmd_draw_indexed(&self, cmd: CommandBuffer, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) {
        unsafe {
            (self.device_fn_1_0.cmd_draw_indexed)(cmd, index_count, instance_count, first_index, vertex_offset, first_instance);
        }
    }

    pub fn cmd_bind_descriptor_sets(
        &self,
        cmd: CommandBuffer,
        pipeline_bind_point: PipelineBindPoint,
        layout: PipelineLayout,
        first_set: u32,
        descriptor_sets: &[DescriptorSet],
        offsets: &[u32],
    ) {
        unsafe {

            let mut offsets_ptr = ptr::null();
            if !offsets.is_empty() {
                offsets_ptr = offsets.as_ptr();
            }

            (self.device_fn_1_0.cmd_bind_descriptor_sets)(
                cmd,
                pipeline_bind_point,
                layout,
                first_set,
                descriptor_sets.len() as _,
                descriptor_sets.as_ptr(),
                offsets.len() as _,
                offsets_ptr,
            )
        }
    }

    pub fn cmd_set_scissor(&self, cmd: CommandBuffer, first: u32, scissors: &[Rect2D]) {
        unsafe {
            (self.device_fn_1_0.cmd_set_scissor)(cmd, first, scissors.len() as _, scissors.as_ptr());
        }
    }

    pub fn cmd_set_viewport(&self, cmd: CommandBuffer, first: u32, viewports: &[Viewport]) {
        unsafe {
            (self.device_fn_1_0.cmd_set_viewport)(cmd, first, viewports.len() as _, viewports.as_ptr());
        }
    }

    pub fn cmd_draw(&self, cmd: CommandBuffer, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        unsafe {
            (self.device_fn_1_0.cmd_draw)(cmd, vertex_count, instance_count, first_vertex, first_instance);
        }
    }

    pub fn cmd_push_constants(&self, cmd: CommandBuffer, layout: PipelineLayout, stage_flags: ShaderStageFlags, offset: u32, size: u32, values: &[u8]) {
        unsafe {
            assert!(size as usize == values.len(), "Push constant buffer does not equal size");
            (self.device_fn_1_0.cmd_push_constants)(cmd, layout, stage_flags, offset, size, values.as_ptr() as *const _);
        }
    }

    pub fn cmd_copy_image(
        &self,
        cmd: CommandBuffer,
        src_image: Image,
        src_image_layout: ImageLayout,
        dst_image: Image,
        dst_image_layout: ImageLayout,
        regions: &[ImageCopy]
    )
    {
        unsafe {
            let region_count = regions.len() as u32;
            (self.device_fn_1_0.cmd_copy_image)(cmd, src_image, src_image_layout, dst_image, dst_image_layout, region_count, regions.as_ptr());
        }
    }

    pub fn cmd_copy_image_to_buffer(
        &self,
        cmd: CommandBuffer,
        src_image: Image,
        src_image_layout: ImageLayout,
        dst_buffer: Buffer,
        regions: &[BufferImageCopy],
    )
    {
        unsafe {
            let region_count = regions.len() as u32;
            (self.device_fn_1_0.cmd_copy_image_to_buffer)(cmd, src_image, src_image_layout, dst_buffer, region_count, regions.as_ptr());
        }
    }

    pub fn cmd_dispatch(&self, cmd: CommandBuffer, x: u32, y: u32, z: u32) {
        unsafe {
            (self.device_fn_1_0.cmd_dispatch)(cmd, x, y, z)
        }
    }

    //
    // VK 1.2
    //

    pub fn wait_semaphores(&self, info: &SemaphoreWaitInfo, timeout: u64) -> Result<(), VkResult> {
        unsafe {
            (self.device_fn_1_2.wait_semaphores)(self.handle, info, timeout).as_result()
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn cmd_draw_indexed_indirect_count(
        &self,
        cmd: CommandBuffer,
        buffer: Buffer,
        offset: DeviceSize,
        count_buffer: Buffer,
        count_buffer_offset: DeviceSize,
        max_draw_count: u32, 
        stride: u32
    )
    {
        unsafe {
            (self.device_fn_1_2.cmd_draw_indexed_indirect_count)(cmd, buffer, offset, count_buffer, count_buffer_offset, max_draw_count, stride);
        }
    }

}
