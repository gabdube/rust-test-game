use crate::*;
use std::{ptr, ffi::{c_void, CStr}, mem::transmute};

impl StructureType {
    pub const MEMORY_BARRIER_2: Self = Self(1_000_314_000);
    pub const BUFFER_MEMORY_BARRIER_2: Self = Self(1_000_314_001);
    pub const IMAGE_MEMORY_BARRIER_2: Self = Self(1_000_314_002);
    pub const DEPENDENCY_INFO: Self = Self(1_000_314_003);
    pub const SUBMIT_INFO_2: Self = Self(1_000_314_004);
    pub const SEMAPHORE_SUBMIT_INFO: Self = Self(1_000_314_005);
    pub const COMMAND_BUFFER_SUBMIT_INFO: Self = Self(1_000_314_006);
    pub const PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES: Self = Self(1_000_314_007);
}

vk_enum!(SubmitFlags);
vk_bitflags!(SubmitFlags);
impl SubmitFlags {
    pub const PROTECTED: Self = Self(0b1);
    pub const PROTECTED_KHR: Self = Self::PROTECTED;
}

vk_enum64!(PipelineStageFlags2);
vk_bitflags!(PipelineStageFlags2);
impl PipelineStageFlags2 {
    pub const NONE: Self = Self(0);
    pub const TOP_OF_PIPE: Self = Self(0b1);
    pub const DRAW_INDIRECT: Self = Self(0b10);
    pub const VERTEX_INPUT: Self = Self(0b100);
    pub const VERTEX_SHADER: Self = Self(0b1000);
    pub const TESSELLATION_CONTROL_SHADER: Self = Self(0b1_0000);
    pub const TESSELLATION_EVALUATION_SHADER: Self = Self(0b10_0000);
    pub const GEOMETRY_SHADER: Self = Self(0b100_0000);
    pub const FRAGMENT_SHADER: Self = Self(0b1000_0000);
    pub const EARLY_FRAGMENT_TESTS: Self = Self(0b1_0000_0000);
    pub const LATE_FRAGMENT_TESTS: Self = Self(0b10_0000_0000);
    pub const COLOR_ATTACHMENT_OUTPUT: Self = Self(0b100_0000_0000);
    pub const COMPUTE_SHADER: Self = Self(0b1000_0000_0000);
    pub const ALL_TRANSFER: Self = Self(0b1_0000_0000_0000);
    pub const BOTTOM_OF_PIPE: Self = Self(0b10_0000_0000_0000);
    pub const HOST: Self = Self(0b100_0000_0000_0000);
    pub const ALL_GRAPHICS: Self = Self(0b1000_0000_0000_0000);
    pub const ALL_COMMANDS: Self = Self(0b1_0000_0000_0000_0000);
    pub const COPY: Self = Self(0b1_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const RESOLVE: Self = Self(0b10_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const BLIT: Self = Self(0b100_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const CLEAR: Self = Self(0b1000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const INDEX_INPUT: Self = Self(0b1_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const VERTEX_ATTRIBUTE_INPUT: Self = Self(0b10_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const PRE_RASTERIZATION_SHADERS: Self = Self(0b100_0000_0000_0000_0000_0000_0000_0000_0000_0000);
}

vk_enum64!(AccessFlags2);
vk_bitflags!(AccessFlags2);
impl AccessFlags2 {
    pub const NONE: Self = Self(0);
    pub const INDIRECT_COMMAND_READ: Self = Self(0b1);
    pub const INDEX_READ: Self = Self(0b10);
    pub const VERTEX_ATTRIBUTE_READ: Self = Self(0b100);
    pub const UNIFORM_READ: Self = Self(0b1000);
    pub const INPUT_ATTACHMENT_READ: Self = Self(0b1_0000);
    pub const SHADER_READ: Self = Self(0b10_0000);
    pub const SHADER_WRITE: Self = Self(0b100_0000);
    pub const COLOR_ATTACHMENT_READ: Self = Self(0b1000_0000);
    pub const COLOR_ATTACHMENT_WRITE: Self = Self(0b1_0000_0000);
    pub const DEPTH_STENCIL_ATTACHMENT_READ: Self = Self(0b10_0000_0000);
    pub const DEPTH_STENCIL_ATTACHMENT_WRITE: Self = Self(0b100_0000_0000);
    pub const TRANSFER_READ: Self = Self(0b1000_0000_0000);
    pub const TRANSFER_WRITE: Self = Self(0b1_0000_0000_0000);
    pub const HOST_READ: Self = Self(0b10_0000_0000_0000);
    pub const HOST_WRITE: Self = Self(0b100_0000_0000_0000);
    pub const MEMORY_READ: Self = Self(0b1000_0000_0000_0000);
    pub const MEMORY_WRITE: Self = Self(0b1_0000_0000_0000_0000);
    pub const SHADER_SAMPLED_READ: Self = Self(0b1_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const SHADER_STORAGE_READ: Self = Self(0b10_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const SHADER_STORAGE_WRITE: Self = Self(0b100_0000_0000_0000_0000_0000_0000_0000_0000);
}

#[repr(C)]
pub struct PhysicalDeviceSynchronization2Features {
    pub s_type: StructureType,
    pub p_next: *mut c_void,
    pub synchronization2: Bool32,
}

impl Default for PhysicalDeviceSynchronization2Features {
    fn default() -> Self {
        Self {
            s_type: StructureType::PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES,
            p_next: ::core::ptr::null_mut(),
            synchronization2: 0,
        }
    }
}

#[repr(C)]
pub struct MemoryBarrier2 {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub src_stage_mask: PipelineStageFlags2,
    pub src_access_mask: AccessFlags2,
    pub dst_stage_mask: PipelineStageFlags2,
    pub dst_access_mask: AccessFlags2,
}

impl Default for MemoryBarrier2 {
    #[inline]
    fn default() -> Self {
        Self {
            s_type: StructureType::MEMORY_BARRIER_2,
            p_next: ptr::null(),
            src_stage_mask: PipelineStageFlags2::default(),
            src_access_mask: AccessFlags2::default(),
            dst_stage_mask: PipelineStageFlags2::default(),
            dst_access_mask: AccessFlags2::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BufferMemoryBarrier2 {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub src_stage_mask: PipelineStageFlags2,
    pub src_access_mask: AccessFlags2,
    pub dst_stage_mask: PipelineStageFlags2,
    pub dst_access_mask: AccessFlags2,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub buffer: Buffer,
    pub offset: DeviceSize,
    pub size: DeviceSize,
}

impl Default for BufferMemoryBarrier2 {
    #[inline]
    fn default() -> Self {
        Self {
            s_type: StructureType::BUFFER_MEMORY_BARRIER_2,
            p_next: ptr::null(),
            src_stage_mask: PipelineStageFlags2::default(),
            src_access_mask: AccessFlags2::default(),
            dst_stage_mask: PipelineStageFlags2::default(),
            dst_access_mask: AccessFlags2::default(),
            src_queue_family_index: u32::default(),
            dst_queue_family_index: u32::default(),
            buffer: Buffer::default(),
            offset: DeviceSize::default(),
            size: DeviceSize::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ImageMemoryBarrier2 {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub src_stage_mask: PipelineStageFlags2,
    pub src_access_mask: AccessFlags2,
    pub dst_stage_mask: PipelineStageFlags2,
    pub dst_access_mask: AccessFlags2,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub image: Image,
    pub subresource_range: ImageSubresourceRange,
}

impl Default for ImageMemoryBarrier2 {
    #[inline]
    fn default() -> Self {
        Self {
            s_type: StructureType::IMAGE_MEMORY_BARRIER_2,
            p_next: ptr::null(),
            src_stage_mask: PipelineStageFlags2::default(),
            src_access_mask: AccessFlags2::default(),
            dst_stage_mask: PipelineStageFlags2::default(),
            dst_access_mask: AccessFlags2::default(),
            old_layout: ImageLayout::default(),
            new_layout: ImageLayout::default(),
            src_queue_family_index: u32::default(),
            dst_queue_family_index: u32::default(),
            image: Image::default(),
            subresource_range: ImageSubresourceRange::default(),
        }
    }
}

#[repr(C)]
pub struct DependencyInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub dependency_flags: DependencyFlags,
    pub memory_barrier_count: u32,
    pub memory_barrier: *const MemoryBarrier2,
    pub buffer_memory_barrier_count: u32,
    pub buffer_memory_barrier: *const BufferMemoryBarrier2,
    pub image_memory_barrier_count: u32,
    pub image_memory_barrier: *const ImageMemoryBarrier2,
}

impl Default for DependencyInfo {
    #[inline]
    fn default() -> Self {
        DependencyInfo {
            s_type: StructureType::DEPENDENCY_INFO,
            p_next: ptr::null(),
            dependency_flags: DependencyFlags(0),
            memory_barrier_count: 0,
            memory_barrier: ptr::null(),
            buffer_memory_barrier_count: 0,
            buffer_memory_barrier: ptr::null(),
            image_memory_barrier_count: 0,
            image_memory_barrier: ptr::null(),
        }
    }

}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SemaphoreSubmitInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub semaphore: Semaphore,
    pub value: u64,
    pub stage_mask: PipelineStageFlags2,
    pub device_index: u32,
}

impl Default for SemaphoreSubmitInfo {
    fn default() -> Self {
        Self {
            s_type: StructureType::SEMAPHORE_SUBMIT_INFO,
            p_next: ptr::null(),
            semaphore: Semaphore::default(),
            value: 0,
            stage_mask: PipelineStageFlags2::default(),
            device_index: 0,
        }
    }
}

#[repr(C)]
pub struct CommandBufferSubmitInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub command_buffer: CommandBuffer,
    pub device_mask: u32,
}

impl Default for CommandBufferSubmitInfo {
    fn default() -> Self {
        Self {
            s_type: StructureType::COMMAND_BUFFER_SUBMIT_INFO,
            p_next: ptr::null(),
            command_buffer: CommandBuffer::default(),
            device_mask: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SubmitInfo2 {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: SubmitFlags,
    pub wait_semaphore_info_count: u32,
    pub p_wait_semaphore_infos: *const SemaphoreSubmitInfo,
    pub command_buffer_info_count: u32,
    pub p_command_buffer_infos: *const CommandBufferSubmitInfo,
    pub signal_semaphore_info_count: u32,
    pub p_signal_semaphore_infos: *const SemaphoreSubmitInfo,
}

impl Default for SubmitInfo2 {
    #[inline]
    fn default() -> Self {
        Self {
            s_type: StructureType::SUBMIT_INFO_2,
            p_next: ptr::null(),
            flags: SubmitFlags::default(),
            wait_semaphore_info_count: 0,
            p_wait_semaphore_infos: ::core::ptr::null(),
            command_buffer_info_count: 0,
            p_command_buffer_infos: ::core::ptr::null(),
            signal_semaphore_info_count: 0,
            p_signal_semaphore_infos: ::core::ptr::null(),
        }
    }
}

pub struct Synchronization2Fn {
    pub cmd_pipeline_barrier_2_khr: PFN_vkPipelineBarrier2KHR,
    pub cmd_queue_submit_2_khr: PFN_vkQueueSubmit2KHR,
}

impl Synchronization2Fn {

    pub fn load<F>(cb: F) -> Synchronization2Fn 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            Synchronization2Fn {
                cmd_pipeline_barrier_2_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkCmdPipelineBarrier2KHR\0"))),
                cmd_queue_submit_2_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkQueueSubmit2KHR\0"))),
            }
        }
    }

}


pub type PFN_vkPipelineBarrier2KHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    dependency_info: *const DependencyInfo
);

pub type PFN_vkQueueSubmit2KHR = unsafe extern "system" fn(
    queue: Queue,
    submit_count: u32,
    submit_info: *const SubmitInfo2,
    fence: Fence
) -> VkResult;
