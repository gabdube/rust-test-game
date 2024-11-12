use crate::*;
use std::{
    ffi::{c_void, CStr},
    mem::transmute,
};

//
// Entry FN
//

pub struct EntryFnV1_0 {
    pub create_instance: PFN_vkCreateInstance,
    pub enumerate_instance_extension_properties: PFN_vkEnumerateInstanceExtensionProperties,
    pub enumerate_instance_layer_properties: PFN_vkEnumerateInstanceLayerProperties,
}

impl EntryFnV1_0 {

    pub fn load<F>(cb: F) -> EntryFnV1_0 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            let cstr = CStr::from_bytes_with_nul_unchecked;
            EntryFnV1_0 {
                create_instance: transmute(cb(cstr(b"vkCreateInstance\0"))),
                enumerate_instance_extension_properties: transmute(cb(cstr(b"vkEnumerateInstanceExtensionProperties\0"))),
                enumerate_instance_layer_properties: transmute(cb(cstr(b"vkEnumerateInstanceLayerProperties\0")))
            }
        }
    }

}

//
// Instance FN
//

pub struct InstanceFnV1_0 {
    pub destroy_instance: PFN_vkDestroyInstance,
    pub enumerate_physical_devices: PFN_vkEnumeratePhysicalDevices,
    pub enumerate_device_extension_properties: PFN_vkEnumerateDeviceExtensionProperties,
    pub get_physical_device_queue_family_properties: PFN_vkGetPhysicalDeviceQueueFamilyProperties,
    pub get_physical_device_features: PFN_vkGetPhysicalDeviceFeatures,
    pub get_physical_device_properties: PFN_vkGetPhysicalDeviceProperties,
    pub get_physical_device_memory_properties: PFN_vkGetPhysicalDeviceMemoryProperties,
    pub get_physical_device_format_properties: PFN_vkGetPhysicalDeviceFormatProperties,
    pub create_device: PFN_vkCreateDevice,
    pub get_device_proc_addr: PFN_vkGetDeviceProcAddr,
}

impl InstanceFnV1_0 {

    pub fn load<F>(cb: F) -> InstanceFnV1_0 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            let cstr = CStr::from_bytes_with_nul_unchecked;
            InstanceFnV1_0 {
                destroy_instance: transmute(cb(cstr(b"vkDestroyInstance\0"))),
                enumerate_physical_devices: transmute(cb(cstr(b"vkEnumeratePhysicalDevices\0"))),
                enumerate_device_extension_properties: transmute(cb(cstr(b"vkEnumerateDeviceExtensionProperties\0"))),
                get_physical_device_queue_family_properties: transmute(cb(cstr(b"vkGetPhysicalDeviceQueueFamilyProperties\0"))),
                get_physical_device_features: transmute(cb(cstr(b"vkGetPhysicalDeviceFeatures\0"))),
                create_device: transmute(cb(cstr(b"vkCreateDevice\0"))),
                get_device_proc_addr: transmute(cb(cstr(b"vkGetDeviceProcAddr\0"))),
                get_physical_device_properties: transmute(cb(cstr(b"vkGetPhysicalDeviceProperties\0"))),
                get_physical_device_memory_properties: transmute(cb(cstr(b"vkGetPhysicalDeviceMemoryProperties\0"))),
                get_physical_device_format_properties: transmute(cb(cstr(b"vkGetPhysicalDeviceFormatProperties\0"))),
            }
        }
    }

}

//
// Device FN
//

pub struct DeviceFnV1_0 {
    pub destroy_device: PFN_vkDestroyDevice,
    pub device_wait_idle: PFN_vkDeviceWaitIdle,
    pub get_device_queue: PFN_vkGetDeviceQueue,
    pub queue_wait_idle: PFN_vkQueueWaitIdle,
    pub queue_submit: PFN_vkQueueSubmit,

    pub allocate_memory: PFN_vkAllocateMemory,
    pub free_memory: PFN_vkFreeMemory,
    pub map_memory: PFN_vkMapMemory,
    pub unmap_memory: PFN_vkUnmapMemory,

    pub create_image: PFN_vkCreateImage,
    pub destroy_image: PFN_vkDestroyImage,
    pub get_image_memory_requirements: PFN_vkGetImageMemoryRequirements,
    pub bind_image_memory: PFN_vkBindImageMemory,

    pub create_image_view: PFN_vkCreateImageView,
    pub destroy_image_view: PFN_vkDestroyImageView,

    pub create_semaphore: PFN_vkCreateSemaphore,
    pub destroy_semaphore: PFN_vkDestroySemaphore,

    pub create_fence: PFN_vkCreateFence,
    pub destroy_fence: PFN_vkDestroyFence,
    pub wait_for_fences: PFN_vkWaitForFences,
    pub reset_fences: PFN_vkResetFences,

    pub create_command_pool: PFN_vkCreateCommandPool,
    pub destroy_command_pool: PFN_vkDestroyCommandPool,
    pub allocate_command_buffers: PFN_vkAllocateCommandBuffers,
    pub begin_command_buffer: PFN_vkBeginCommandBuffer,
    pub end_command_buffer: PFN_vkEndCommandBuffer,

    pub create_render_pass: PFN_vkCreateRenderPass,
    pub destroy_render_pass: PFN_vkDestroyRenderPass,

    pub create_framebuffer: PFN_vkCreateFramebuffer,
    pub destroy_framebuffer: PFN_vkDestroyFramebuffer,

    pub create_buffer: PFN_vkCreateBuffer,
    pub destroy_buffer: PFN_vkDestroyBuffer,
    pub get_buffer_memory_requirements: PFN_vkGetBufferMemoryRequirements,
    pub bind_buffer_memory: PFN_vkBindBufferMemory,

    pub create_shader_module: PFN_vkCreateShaderModule,
    pub destroy_shader_module: PFN_vkDestroyShaderModule,

    pub create_pipeline_cache: PFN_vkCreatePipelineCache,
    pub destroy_pipeline_cache: PFN_vkDestroyPipelineCache,
    pub get_pipeline_cache_data: PFN_vkGetPipelineCacheData,

    pub create_pipeline_layout: PFN_vkCreatePipelineLayout,
    pub destroy_pipeline_layout: PFN_vkDestroyPipelineLayout,

    pub create_descriptor_set_layout: PFN_vkCreateDescriptorSetLayout,
    pub destroy_descriptor_set_layout: PFN_vkDestroyDescriptorSetLayout,

    pub create_graphics_pipelines: PFN_vkCreateGraphicsPipelines,
    pub create_compute_pipelines: PFN_vkCreateComputePipelines,
    pub destroy_pipeline: PFN_vkDestroyPipeline,

    pub create_descriptor_pool: PFN_vkCreateDescriptorPool,
    pub destroy_descriptor_pool: PFN_vkDestroyDescriptorPool,

    pub allocate_descriptor_sets: PFN_vkAllocateDescriptorSets,
    pub update_descriptor_sets: PFN_vkUpdateDescriptorSets,

    pub create_sampler: PFN_vkCreateSampler,
    pub destroy_sampler: PFN_vkDestroySampler,

    pub create_buffer_view: PFN_vkCreateBufferView,
    pub destroy_buffer_view: PFN_vkDestroyBufferView,

    pub cmd_begin_render_pass: PFN_vkCmdBeginRenderPass,
    pub cmd_end_render_pass: PFN_vkCmdEndRenderPass,
    pub cmd_copy_buffer: PFN_vkCmdCopyBuffer,
    pub cmd_copy_buffer_to_image: PFN_vkCmdCopyBufferToImage,
    pub cmd_bind_pipeline: PFN_vkCmdBindPipeline,
    pub cmd_bind_index_buffer: PFN_vkCmdBindIndexBuffer,
    pub cmd_bind_vertex_buffers: PFN_vkCmdBindDrawingBuffers,
    pub cmd_draw_indexed: PFN_vkCmdDrawIndexed,
    pub cmd_draw: PFN_vkCmdDraw,
    pub cmd_bind_descriptor_sets: PFN_vkCmdBindDescriptorSets,
    pub cmd_set_viewport: PFN_vkCmdSetViewport,
    pub cmd_set_scissor: PFN_vkCmdSetScissor,
    pub cmd_copy_image: PFN_vkCmdCopyImage,
    pub cmd_copy_image_to_buffer: PFN_vkCmdCopyImageToBuffer,
    pub cmd_push_constants: PFN_vkCmdPushConstants,
    pub cmd_dispatch: PFN_vkCmdDispatch,
}

impl DeviceFnV1_0 {

    pub fn load<F>(cb: F) -> DeviceFnV1_0 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            let cstr = CStr::from_bytes_with_nul_unchecked;
            DeviceFnV1_0 {
                destroy_device: transmute(cb(cstr(b"vkDestroyDevice\0"))),
                device_wait_idle: transmute(cb(cstr(b"vkDeviceWaitIdle\0"))),
                get_device_queue: transmute(cb(cstr(b"vkGetDeviceQueue\0"))),
                queue_wait_idle: transmute(cb(cstr(b"vkQueueWaitIdle\0"))),
                queue_submit: transmute(cb(cstr(b"vkQueueSubmit\0"))),

                allocate_memory: transmute(cb(cstr(b"vkAllocateMemory\0"))),
                free_memory: transmute(cb(cstr(b"vkFreeMemory\0"))),
                map_memory: transmute(cb(cstr(b"vkMapMemory\0"))),
                unmap_memory: transmute(cb(cstr(b"vkUnmapMemory\0"))),

                create_image: transmute(cb(cstr(b"vkCreateImage\0"))),
                destroy_image: transmute(cb(cstr(b"vkDestroyImage\0"))),
                get_image_memory_requirements: transmute(cb(cstr(b"vkGetImageMemoryRequirements\0"))),
                bind_image_memory: transmute(cb(cstr(b"vkBindImageMemory\0"))),

                create_image_view: transmute(cb(cstr(b"vkCreateImageView\0"))),
                destroy_image_view: transmute(cb(cstr(b"vkDestroyImageView\0"))),

                create_semaphore: transmute(cb(cstr(b"vkCreateSemaphore\0"))),
                destroy_semaphore: transmute(cb(cstr(b"vkDestroySemaphore\0"))),

                create_fence: transmute(cb(cstr(b"vkCreateFence\0"))),
                destroy_fence: transmute(cb(cstr(b"vkDestroyFence\0"))),
                wait_for_fences: transmute(cb(cstr(b"vkWaitForFences\0"))),
                reset_fences: transmute(cb(cstr(b"vkResetFences\0"))),

                create_command_pool: transmute(cb(cstr(b"vkCreateCommandPool\0"))),
                destroy_command_pool: transmute(cb(cstr(b"vkDestroyCommandPool\0"))),
                allocate_command_buffers: transmute(cb(cstr(b"vkAllocateCommandBuffers\0"))),
                begin_command_buffer: transmute(cb(cstr(b"vkBeginCommandBuffer\0"))),
                end_command_buffer: transmute(cb(cstr(b"vkEndCommandBuffer\0"))),

                create_render_pass: transmute(cb(cstr(b"vkCreateRenderPass\0"))),
                destroy_render_pass: transmute(cb(cstr(b"vkDestroyRenderPass\0"))),

                create_framebuffer: transmute(cb(cstr(b"vkCreateFramebuffer\0"))),
                destroy_framebuffer: transmute(cb(cstr(b"vkDestroyFramebuffer\0"))),

                create_buffer: transmute(cb(cstr(b"vkCreateBuffer\0"))),
                destroy_buffer: transmute(cb(cstr(b"vkDestroyBuffer\0"))),
                get_buffer_memory_requirements: transmute(cb(cstr(b"vkGetBufferMemoryRequirements\0"))),
                bind_buffer_memory: transmute(cb(cstr(b"vkBindBufferMemory\0"))),

                create_shader_module: transmute(cb(cstr(b"vkCreateShaderModule\0"))),
                destroy_shader_module: transmute(cb(cstr(b"vkDestroyShaderModule\0"))),

                create_pipeline_cache: transmute(cb(cstr(b"vkCreatePipelineCache\0"))),
                destroy_pipeline_cache: transmute(cb(cstr(b"vkDestroyPipelineCache\0"))),
                get_pipeline_cache_data: transmute(cb(cstr(b"vkGetPipelineCacheData\0"))),

                create_pipeline_layout: transmute(cb(cstr(b"vkCreatePipelineLayout\0"))),
                destroy_pipeline_layout: transmute(cb(cstr(b"vkDestroyPipelineLayout\0"))),

                create_descriptor_set_layout: transmute(cb(cstr(b"vkCreateDescriptorSetLayout\0"))),
                destroy_descriptor_set_layout: transmute(cb(cstr(b"vkDestroyDescriptorSetLayout\0"))),

                create_graphics_pipelines: transmute(cb(cstr(b"vkCreateGraphicsPipelines\0"))),
                create_compute_pipelines: transmute(cb(cstr(b"vkCreateComputePipelines\0"))),
                destroy_pipeline: transmute(cb(cstr(b"vkDestroyPipeline\0"))),

                create_descriptor_pool: transmute(cb(cstr(b"vkCreateDescriptorPool\0"))),
                destroy_descriptor_pool: transmute(cb(cstr(b"vkDestroyDescriptorPool\0"))),

                allocate_descriptor_sets: transmute(cb(cstr(b"vkAllocateDescriptorSets\0"))),
                update_descriptor_sets: transmute(cb(cstr(b"vkUpdateDescriptorSets\0"))),

                create_sampler: transmute(cb(cstr(b"vkCreateSampler\0"))),
                destroy_sampler: transmute(cb(cstr(b"vkDestroySampler\0"))),

                create_buffer_view: transmute(cb(cstr(b"vkCreateBufferView\0"))),
                destroy_buffer_view: transmute(cb(cstr(b"vkDestroyBufferView\0"))),

                cmd_begin_render_pass: transmute(cb(cstr(b"vkCmdBeginRenderPass\0"))),
                cmd_end_render_pass: transmute(cb(cstr(b"vkCmdEndRenderPass\0"))),
                cmd_copy_buffer: transmute(cb(cstr(b"vkCmdCopyBuffer\0"))),
                cmd_copy_buffer_to_image: transmute(cb(cstr(b"vkCmdCopyBufferToImage\0"))),
                cmd_bind_pipeline: transmute(cb(cstr(b"vkCmdBindPipeline\0"))),
                cmd_draw_indexed: transmute(cb(cstr(b"vkCmdDrawIndexed\0"))),
                cmd_draw: transmute(cb(cstr(b"vkCmdDraw\0"))),
                cmd_bind_index_buffer: transmute(cb(cstr(b"vkCmdBindIndexBuffer\0"))),
                cmd_bind_vertex_buffers: transmute(cb(cstr(b"vkCmdBindVertexBuffers\0"))),
                cmd_bind_descriptor_sets: transmute(cb(cstr(b"vkCmdBindDescriptorSets\0"))),
                cmd_set_viewport: transmute(cb(cstr(b"vkCmdSetViewport\0"))),
                cmd_set_scissor: transmute(cb(cstr(b"vkCmdSetScissor\0"))),
                cmd_copy_image:  transmute(cb(cstr(b"vkCmdCopyImage\0"))),
                cmd_copy_image_to_buffer: transmute(cb(cstr(b"vkCmdCopyImageToBuffer\0"))),
                cmd_push_constants: transmute(cb(cstr(b"vkCmdPushConstants\0"))),
                cmd_dispatch: transmute(cb(cstr(b"vkCmdDispatch\0"))),
            }
        }
    }

}

//
// Functions def
//

pub type PFN_vkVoidFunction = Option<unsafe extern "system" fn()>;

pub type PFN_vkGetInstanceProcAddr =
    unsafe extern "system" fn(instance: Instance, p_name: *const u8) -> PFN_vkVoidFunction;

pub type PFN_vkCreateInstance = unsafe extern "system" fn(
    p_create_info: *const InstanceCreateInfo,
    p_allocator: *const c_void,
    p_instance: *mut Instance,
) -> VkResult;

pub type PFN_vkEnumerateInstanceExtensionProperties = unsafe extern "system" fn(
    p_layer_name: *const u8,
    p_property_count: *mut u32,
    p_properties: *mut ExtensionProperties,
) -> VkResult;

pub type PFN_vkEnumerateInstanceLayerProperties = unsafe extern "system" fn(
    p_property_count: *mut u32,
    p_properties: *mut LayerProperties,
) -> VkResult;

pub type PFN_vkDestroyInstance =
    unsafe extern "system" fn(instance: Instance, p_allocator: *const c_void);

pub type PFN_vkEnumeratePhysicalDevices = unsafe extern "system" fn(
    instance: Instance,
    p_physical_device_count: *mut u32,
    p_physical_devices: *mut PhysicalDevice,
) -> VkResult;

pub type PFN_vkEnumerateDeviceExtensionProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_layer_name: *const u8,
    p_property_count: *mut u32,
    p_properties: *mut ExtensionProperties,
) -> VkResult;

pub type PFN_vkGetPhysicalDeviceQueueFamilyProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_queue_family_property_count: *mut u32,
    p_queue_family_properties: *mut QueueFamilyProperties,
);

pub type PFN_vkGetPhysicalDeviceFeatures = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_features: *mut PhysicalDeviceFeatures,
);

pub type PFN_vkGetPhysicalDeviceProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_properties: *mut PhysicalDeviceProperties,
);

pub type PFN_vkGetPhysicalDeviceMemoryProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_memory_properties: *mut PhysicalDeviceMemoryProperties,
);

pub type PFN_vkCreateDevice = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_create_info: *const DeviceCreateInfo,
    p_allocator: *const c_void,
    p_device: *mut Device,
) -> VkResult;

pub type PFN_vkGetDeviceProcAddr =
    unsafe extern "system" fn(device: Device, p_name: *const u8) -> PFN_vkVoidFunction;

pub type PFN_vkDestroyDevice =
    unsafe extern "system" fn(device: Device, p_allocator: *const c_void);

pub type PFN_vkDeviceWaitIdle = unsafe extern "system" fn(device: Device) -> VkResult;

pub type PFN_vkQueueWaitIdle = unsafe extern "system" fn(queue: Queue) -> VkResult;

pub type PFN_vkGetDeviceQueue = unsafe extern "system" fn(
    device: Device,
    queue_family_index: u32,
    queue_index: u32,
    p_queue: *mut Queue,
);

pub type PFN_vkAllocateMemory = unsafe extern "system" fn(
    device: Device,
    p_allocate_info: *const MemoryAllocateInfo,
    p_allocator: *const c_void,
    p_memory: *mut DeviceMemory,
) -> VkResult;

pub type PFN_vkFreeMemory = unsafe extern "system" fn(
    device: Device,
    memory: DeviceMemory,
    p_allocator: *const c_void,
);

pub type PFN_vkMapMemory = unsafe extern "system" fn(
    device: Device,
    memory: DeviceMemory,
    offset: DeviceSize,
    size: DeviceSize,
    flags: MemoryMapFlags,
    pp_data: *mut *mut c_void,
) -> VkResult;

pub type PFN_vkUnmapMemory = unsafe extern "system" fn(device: Device, memory: DeviceMemory);

pub type PFN_vkCreateImage = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const ImageCreateInfo,
    p_allocator: *const c_void,
    p_image: *mut Image,
) -> VkResult;

pub type PFN_vkDestroyImage = unsafe extern "system" fn(
    device: Device,
    image: Image,
    p_allocator: *const c_void,
);

pub type PFN_vkGetImageMemoryRequirements = unsafe extern "system" fn(
    device: Device,
    image: Image,
    p_memory_requirements: *mut MemoryRequirements,
);

pub type PFN_vkBindImageMemory = unsafe extern "system" fn(
    device: Device,
    image: Image,
    memory: DeviceMemory,
    memory_offset: DeviceSize,
) -> VkResult;

pub type PFN_vkCreateImageView = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const ImageViewCreateInfo,
    p_allocator: *const c_void,
    p_view: *mut ImageView,
) -> VkResult;

pub type PFN_vkDestroyImageView = unsafe extern "system" fn(
    device: Device,
    image_view: ImageView,
    p_allocator: *const c_void,
);

pub type PFN_vkCreateSemaphore = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const SemaphoreCreateInfo,
    p_allocator: *const c_void,
    p_semaphore: *mut Semaphore,
) -> VkResult;

pub type PFN_vkDestroySemaphore = unsafe extern "system" fn(
    device: Device,
    semaphore: Semaphore,
    p_allocator: *const c_void,
);

pub type PFN_vkCreateFence = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const FenceCreateInfo,
    p_allocator: *const c_void,
    p_fence: *mut Fence,
) -> VkResult;

pub type PFN_vkDestroyFence = unsafe extern "system" fn(
    device: Device,
    fence: Fence,
    p_allocator: *const c_void,
);

pub type PFN_vkWaitForFences = unsafe extern "system" fn(
    device: Device,
    fence_count: u32,
    p_fences: *const Fence,
    wait_all: Bool32,
    timeout: u64,
) -> VkResult;

pub type PFN_vkResetFences =
    unsafe extern "system" fn(device: Device, fence_count: u32, p_fences: *const Fence) -> VkResult;

pub type PFN_vkCreateCommandPool = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const CommandPoolCreateInfo,
    p_allocator: *const c_void,
    p_command_pool: *mut CommandPool,
) -> VkResult;

pub type PFN_vkDestroyCommandPool = unsafe extern "system" fn(
    device: Device,
    command_pool: CommandPool,
    p_allocator: *const c_void,
);

pub type PFN_vkAllocateCommandBuffers = unsafe extern "system" fn(
    device: Device,
    p_allocate_info: *const CommandBufferAllocateInfo,
    p_command_buffers: *mut CommandBuffer,
) -> VkResult;  

pub type PFN_vkBeginCommandBuffer = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_begin_info: *const CommandBufferBeginInfo,
) -> VkResult;

pub type PFN_vkEndCommandBuffer =
    unsafe extern "system" fn(command_buffer: CommandBuffer) -> VkResult;

pub type PFN_vkQueueSubmit = unsafe extern "system" fn(
    queue: Queue,
    submit_count: u32,
    p_submits: *const SubmitInfo,
    fence: Fence,
) -> VkResult;

pub type PFN_vkCreateRenderPass = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const RenderPassCreateInfo,
    p_allocator: *const c_void,
    p_render_pass: *mut RenderPass,
) -> VkResult;

pub type PFN_vkDestroyRenderPass = unsafe extern "system" fn(
    device: Device,
    render_pass: RenderPass,
    p_allocator: *const c_void,
);

pub type PFN_vkCreateFramebuffer = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const FramebufferCreateInfo,
    p_allocator: *const c_void,
    p_framebuffer: *mut Framebuffer,
) -> VkResult;

pub type PFN_vkDestroyFramebuffer = unsafe extern "system" fn(
    device: Device,
    framebuffer: Framebuffer,
    p_allocator: *const c_void,
);

pub type PFN_vkCreateBuffer = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const BufferCreateInfo,
    p_allocator: *const c_void,
    p_buffer: *mut Buffer,
) -> VkResult;

pub type PFN_vkDestroyBuffer = unsafe extern "system" fn(
    device: Device,
    buffer: Buffer,
    p_allocator: *const c_void,
);

pub type PFN_vkGetBufferMemoryRequirements = unsafe extern "system" fn(
    device: Device,
    buffer: Buffer,
    p_memory_requirements: *mut MemoryRequirements,
);

pub type PFN_vkBindBufferMemory = unsafe extern "system" fn(
    device: Device,
    buffer: Buffer,
    memory: DeviceMemory,
    memory_offset: DeviceSize,
) -> VkResult;


pub type PFN_vkCmdBeginRenderPass = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_render_pass_begin: *const RenderPassBeginInfo,
    contents: SubpassContents,
);

pub type PFN_vkCmdEndRenderPass = unsafe extern "system" fn(command_buffer: CommandBuffer);

pub type PFN_vkCmdCopyBuffer = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    src_buffer: Buffer,
    dst_buffer: Buffer,
    region_count: u32,
    p_regions: *const BufferCopy,
);

pub type PFN_vkCmdCopyBufferToImage = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    src_buffer: Buffer,
    dst_image: Image,
    dst_image_layout: ImageLayout,
    region_count: u32,
    p_regions: *const BufferImageCopy,
);

pub type PFN_vkCreateShaderModule = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const ShaderModuleCreateInfo,
    p_allocator: *const c_void,
    p_shader_module: *mut ShaderModule,
) -> VkResult;

pub type PFN_vkDestroyShaderModule = unsafe extern "system" fn(
    device: Device,
    shader_module: ShaderModule,
    p_allocator: *const c_void,
);

pub type PFN_vkCreatePipelineCache = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const PipelineCacheCreateInfo,
    p_allocator: *const c_void,
    p_pipeline_cache: *mut PipelineCache,
) -> VkResult;

pub type PFN_vkDestroyPipelineCache = unsafe extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    p_allocator: *const c_void,
);

pub type PFN_vkCreateGraphicsPipelines = unsafe extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    create_info_count: u32,
    p_create_infos: *const GraphicsPipelineCreateInfo,
    p_allocator: *const c_void,
    p_pipelines: *mut Pipeline,
) -> VkResult;

pub type PFN_vkDestroyPipeline = unsafe extern "system" fn(
    device: Device,
    pipeline: Pipeline,
    p_allocator: *const c_void,
);

pub type PFN_vkGetPipelineCacheData = unsafe extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    p_data_size: *mut usize,
    p_data: *mut c_void,
) -> VkResult;

pub type PFN_vkCreatePipelineLayout = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const PipelineLayoutCreateInfo,
    p_allocator: *const c_void,
    p_pipeline_layout: *mut PipelineLayout,
) -> VkResult;

pub type PFN_vkDestroyPipelineLayout = unsafe extern "system" fn(
    device: Device,
    pipeline_layout: PipelineLayout,
    p_allocator: *const c_void,
);

pub type PFN_vkCreateDescriptorSetLayout = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const DescriptorSetLayoutCreateInfo,
    p_allocator: *const c_void,
    p_set_layout: *mut DescriptorSetLayout,
) -> VkResult;

pub type PFN_vkDestroyDescriptorSetLayout = unsafe extern "system" fn(
    device: Device,
    descriptor_set_layout: DescriptorSetLayout,
    p_allocator: *const c_void,
);

pub type PFN_vkCmdBindPipeline = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    pipeline: Pipeline,
);

pub type PFN_vkCmdBindIndexBuffer = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    index_type: IndexType,
); 

pub type PFN_vkCmdBindDrawingBuffers = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_binding: u32,
    binding_count: u32,
    p_buffers: *const Buffer,
    p_offsets: *const DeviceSize,
);

pub type PFN_vkCmdDrawIndexed = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    vertex_offset: i32,
    first_instance: u32,
);

pub type PFN_vkCmdBindDescriptorSets = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    layout: PipelineLayout,
    first_set: u32,
    descriptor_set_count: u32,
    p_descriptor_sets: *const DescriptorSet,
    dynamic_offset_count: u32,
    p_dynamic_offsets: *const u32,
);

pub type PFN_vkCreateDescriptorPool = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const DescriptorPoolCreateInfo,
    p_allocator: *const c_void,
    p_descriptor_pool: *mut DescriptorPool,
) -> VkResult;

pub type PFN_vkDestroyDescriptorPool = unsafe extern "system" fn(
    device: Device,
    descriptor_pool: DescriptorPool,
    p_allocator: *const c_void,
);

pub type PFN_vkAllocateDescriptorSets = unsafe extern "system" fn(
    device: Device,
    p_allocate_info: *const DescriptorSetAllocateInfo,
    p_descriptor_sets: *mut DescriptorSet,
) -> VkResult;

pub type PFN_vkUpdateDescriptorSets = unsafe extern "system" fn(
    device: Device,
    descriptor_write_count: u32,
    p_descriptor_writes: *const WriteDescriptorSet,
    descriptor_copy_count: u32,
    p_descriptor_copies: *const CopyDescriptorSet,
);

pub type PFN_vkCreateSampler = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const SamplerCreateInfo,
    p_allocator: *const c_void,
    p_sampler: *mut Sampler,
) -> VkResult;

pub type PFN_vkDestroySampler = unsafe extern "system" fn(
    device: Device,
    sampler: Sampler,
    p_allocator: *const c_void,
);

pub type PFN_vkCmdSetViewport = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_viewport: u32,
    viewport_count: u32,
    p_viewports: *const Viewport,
);

pub type PFN_vkCmdSetScissor = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_scissor: u32,
    scissor_count: u32,
    p_scissors: *const Rect2D,
);

pub type PFN_vkGetPhysicalDeviceFormatProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    format: Format,
    p_format_properties: *mut FormatProperties,
);

pub type PFN_vkCmdDraw = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
);

pub type PFN_vkCmdCopyImageToBuffer = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    src_image: Image,
    src_image_layout: ImageLayout,
    dst_buffer: Buffer,
    region_count: u32,
    p_regions: *const BufferImageCopy,
);

pub type PFN_vkCmdPushConstants = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    layout: PipelineLayout,
    stage_flags: ShaderStageFlags,
    offset: u32,
    size: u32,
    p_values: *const c_void,
);

pub type PFN_vkCmdCopyImage = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    src_image: Image,
    src_image_layout: ImageLayout,
    dst_image: Image,
    dst_image_layout: ImageLayout,
    region_count: u32,
    p_regions: *const ImageCopy,
);

pub type PFN_vkCmdDispatch = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    group_count_x: u32,
    group_count_y: u32,
    group_count_z: u32,
);

pub type PFN_vkCreateComputePipelines = unsafe extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    create_info_count: u32,
    p_create_infos: *const ComputePipelineCreateInfo,
    p_allocator: *const c_void,
    p_pipelines: *mut Pipeline,
) -> VkResult;

pub type PFN_vkCreateBufferView = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const BufferViewCreateInfo,
    p_allocator: *const c_void,
    p_view: *mut BufferView,
) -> VkResult;

pub type PFN_vkDestroyBufferView = unsafe extern "system" fn(
    device: Device,
    buffer_view: BufferView,
    p_allocator: *const c_void,
);
