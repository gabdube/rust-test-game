#![allow(dead_code)]

use crate::*;
use std::{ptr, mem, ffi::c_void};

define_handle!(Instance, INSTANCE);
define_handle!(PhysicalDevice, PHYSICAL_DEVICE);
define_handle!(Device, DEVICE);
define_handle!(Queue, QUEUE);
define_nondispatchable_handle!(DeviceMemory, DEVICE_MEMORY);
define_nondispatchable_handle!(Semaphore, SEMAPHORE);
define_nondispatchable_handle!(Fence, FENCE);
define_nondispatchable_handle!(Image, IMAGE);
define_nondispatchable_handle!(ImageView, IMAGE_VIEW);
define_nondispatchable_handle!(CommandBuffer, COMMAND_BUFFER);
define_nondispatchable_handle!(CommandPool, COMMAND_POOL);
define_nondispatchable_handle!(RenderPass, RENDER_PASS);
define_nondispatchable_handle!(Framebuffer, FRAMEBUFFER);
define_nondispatchable_handle!(Buffer, BUFFER);
define_nondispatchable_handle!(ShaderModule, SHADER_MODULE);
define_nondispatchable_handle!(PipelineCache, PIPELINE_CACHE);
define_nondispatchable_handle!(PipelineLayout, PIPELINE_LAYOUT);
define_nondispatchable_handle!(Pipeline, PIPELINE);
define_nondispatchable_handle!(DescriptorSet, DESCRIPTOR_SET);
define_nondispatchable_handle!(DescriptorSetLayout, DESCRIPTOR_SET_LAYOUT);
define_nondispatchable_handle!(DescriptorPool, DESCRIPTOR_POOL);
define_nondispatchable_handle!(Sampler, SAMPLER);
define_nondispatchable_handle!(BufferView, BUFFER_VIEW);


pub type Bool32 = u32;
pub type SampleMask = u32;
pub type DeviceSize = u64;

pub const UUID_SIZE: usize = 16;
pub const LUID_SIZE: usize = 8;
pub const MAX_DRIVER_NAME_SIZE: usize = 256;
pub const MAX_DRIVER_INFO_SIZE: usize = 256;
pub const SUBPASS_EXTERNAL: u32 = !0; // 0xffff_ffffu32

pub const fn make_api_version(variant: u32, major: u32, minor: u32, patch: u32) -> u32 {
    ((variant) << 29) | ((major) << 22) | ((minor) << 12) | (patch)
}

#[repr(C)]
pub struct ApplicationInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub p_application_name: *const u8,
    pub application_version: u32,
    pub p_engine_name: *const u8,
    pub engine_version: u32,
    pub api_version: u32,
}

impl Default for ApplicationInfo {
    fn default() -> Self {
        ApplicationInfo {
            s_type: StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: ptr::null(),
            application_version: 0,
            p_engine_name: ptr::null(),
            engine_version: 0,
            api_version: make_api_version(0, 1, 0, 0)
        }
    }
}

#[repr(C)]
pub struct InstanceCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: InstanceCreateFlags,
    pub p_application_info: *const ApplicationInfo,
    pub enabled_layer_count: u32,
    pub pp_enabled_layer_names: *const *const u8,
    pub enabled_extension_count: u32,
    pub pp_enabled_extension_names: *const *const u8,
}

impl Default for InstanceCreateInfo {
    fn default() -> InstanceCreateInfo {
        InstanceCreateInfo {
            s_type: StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: InstanceCreateFlags::default(),
            p_application_info: ptr::null(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: 0,
            pp_enabled_extension_names: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct LayerProperties {
    pub layer_name: [u8; 256],
    pub spec_version: u32,
    pub implementation_version: u32,
    pub description: [u8; 256],
}

#[repr(C)]
#[derive(Clone)]
pub struct ExtensionProperties {
    pub extension_name: [u8; 256],
    pub spec_version: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Offset2D {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Offset3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Rect2D {
    pub offset: Offset2D,
    pub extent: Extent2D,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Extent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct QueueFamilyProperties {
    pub queue_flags: QueueFlags,
    pub queue_count: u32,
    pub timestamp_valid_bits: u32,
    pub min_image_transfer_granularity: Extent3D,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct PhysicalDeviceFeatures {
    pub robust_buffer_access: Bool32,
    pub full_draw_index_uint32: Bool32,
    pub image_cube_array: Bool32,
    pub independent_blend: Bool32,
    pub geometry_shader: Bool32,
    pub tessellation_shader: Bool32,
    pub sample_rate_shading: Bool32,
    pub dual_src_blend: Bool32,
    pub logic_op: Bool32,
    pub multi_draw_indirect: Bool32,
    pub draw_indirect_first_instance: Bool32,
    pub depth_clamp: Bool32,
    pub depth_bias_clamp: Bool32,
    pub fill_mode_non_solid: Bool32,
    pub depth_bounds: Bool32,
    pub wide_lines: Bool32,
    pub large_points: Bool32,
    pub alpha_to_one: Bool32,
    pub multi_viewport: Bool32,
    pub sampler_anisotropy: Bool32,
    pub texture_compression_etc2: Bool32,
    pub texture_compression_astc_ldr: Bool32,
    pub texture_compression_bc: Bool32,
    pub occlusion_query_precise: Bool32,
    pub pipeline_statistics_query: Bool32,
    pub vertex_pipeline_stores_and_atomics: Bool32,
    pub fragment_stores_and_atomics: Bool32,
    pub shader_tessellation_and_geometry_point_size: Bool32,
    pub shader_image_gather_extended: Bool32,
    pub shader_storage_image_extended_formats: Bool32,
    pub shader_storage_image_multisample: Bool32,
    pub shader_storage_image_read_without_format: Bool32,
    pub shader_storage_image_write_without_format: Bool32,
    pub shader_uniform_buffer_array_dynamic_indexing: Bool32,
    pub shader_sampled_image_array_dynamic_indexing: Bool32,
    pub shader_storage_buffer_array_dynamic_indexing: Bool32,
    pub shader_storage_image_array_dynamic_indexing: Bool32,
    pub shader_clip_distance: Bool32,
    pub shader_cull_distance: Bool32,
    pub shader_float64: Bool32,
    pub shader_int64: Bool32,
    pub shader_int16: Bool32,
    pub shader_resource_residency: Bool32,
    pub shader_resource_min_lod: Bool32,
    pub sparse_binding: Bool32,
    pub sparse_residency_buffer: Bool32,
    pub sparse_residency_image2_d: Bool32,
    pub sparse_residency_image3_d: Bool32,
    pub sparse_residency2_samples: Bool32,
    pub sparse_residency4_samples: Bool32,
    pub sparse_residency8_samples: Bool32,
    pub sparse_residency16_samples: Bool32,
    pub sparse_residency_aliased: Bool32,
    pub variable_multisample_rate: Bool32,
    pub inherited_queries: Bool32,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct PhysicalDeviceLimits {
    pub max_image_dimension1_d: u32,
    pub max_image_dimension2_d: u32,
    pub max_image_dimension3_d: u32,
    pub max_image_dimension_cube: u32,
    pub max_image_array_layers: u32,
    pub max_texel_buffer_elements: u32,
    pub max_uniform_buffer_range: u32,
    pub max_storage_buffer_range: u32,
    pub max_push_constants_size: u32,
    pub max_memory_allocation_count: u32,
    pub max_sampler_allocation_count: u32,
    pub buffer_image_granularity: DeviceSize,
    pub sparse_address_space_size: DeviceSize,
    pub max_bound_descriptor_sets: u32,
    pub max_per_stage_descriptor_samplers: u32,
    pub max_per_stage_descriptor_uniform_buffers: u32,
    pub max_per_stage_descriptor_storage_buffers: u32,
    pub max_per_stage_descriptor_sampled_images: u32,
    pub max_per_stage_descriptor_storage_images: u32,
    pub max_per_stage_descriptor_input_attachments: u32,
    pub max_per_stage_resources: u32,
    pub max_descriptor_set_samplers: u32,
    pub max_descriptor_set_uniform_buffers: u32,
    pub max_descriptor_set_uniform_buffers_dynamic: u32,
    pub max_descriptor_set_storage_buffers: u32,
    pub max_descriptor_set_storage_buffers_dynamic: u32,
    pub max_descriptor_set_sampled_images: u32,
    pub max_descriptor_set_storage_images: u32,
    pub max_descriptor_set_input_attachments: u32,
    pub max_vertex_input_attributes: u32,
    pub max_vertex_input_bindings: u32,
    pub max_vertex_input_attribute_offset: u32,
    pub max_vertex_input_binding_stride: u32,
    pub max_vertex_output_components: u32,
    pub max_tessellation_generation_level: u32,
    pub max_tessellation_patch_size: u32,
    pub max_tessellation_control_per_vertex_input_components: u32,
    pub max_tessellation_control_per_vertex_output_components: u32,
    pub max_tessellation_control_per_patch_output_components: u32,
    pub max_tessellation_control_total_output_components: u32,
    pub max_tessellation_evaluation_input_components: u32,
    pub max_tessellation_evaluation_output_components: u32,
    pub max_geometry_shader_invocations: u32,
    pub max_geometry_input_components: u32,
    pub max_geometry_output_components: u32,
    pub max_geometry_output_vertices: u32,
    pub max_geometry_total_output_components: u32,
    pub max_fragment_input_components: u32,
    pub max_fragment_output_attachments: u32,
    pub max_fragment_dual_src_attachments: u32,
    pub max_fragment_combined_output_resources: u32,
    pub max_compute_shared_memory_size: u32,
    pub max_compute_work_group_count: [u32; 3],
    pub max_compute_work_group_invocations: u32,
    pub max_compute_work_group_size: [u32; 3],
    pub sub_pixel_precision_bits: u32,
    pub sub_texel_precision_bits: u32,
    pub mipmap_precision_bits: u32,
    pub max_draw_indexed_index_value: u32,
    pub max_draw_indirect_count: u32,
    pub max_sampler_lod_bias: f32,
    pub max_sampler_anisotropy: f32,
    pub max_viewports: u32,
    pub max_viewport_dimensions: [u32; 2],
    pub viewport_bounds_range: [f32; 2],
    pub viewport_sub_pixel_bits: u32,
    pub min_memory_map_alignment: usize,
    pub min_texel_buffer_offset_alignment: DeviceSize,
    pub min_uniform_buffer_offset_alignment: DeviceSize,
    pub min_storage_buffer_offset_alignment: DeviceSize,
    pub min_texel_offset: i32,
    pub max_texel_offset: u32,
    pub min_texel_gather_offset: i32,
    pub max_texel_gather_offset: u32,
    pub min_interpolation_offset: f32,
    pub max_interpolation_offset: f32,
    pub sub_pixel_interpolation_offset_bits: u32,
    pub max_framebuffer_width: u32,
    pub max_framebuffer_height: u32,
    pub max_framebuffer_layers: u32,
    pub framebuffer_color_sample_counts: SampleCountFlags,
    pub framebuffer_depth_sample_counts: SampleCountFlags,
    pub framebuffer_stencil_sample_counts: SampleCountFlags,
    pub framebuffer_no_attachments_sample_counts: SampleCountFlags,
    pub max_color_attachments: u32,
    pub sampled_image_color_sample_counts: SampleCountFlags,
    pub sampled_image_integer_sample_counts: SampleCountFlags,
    pub sampled_image_depth_sample_counts: SampleCountFlags,
    pub sampled_image_stencil_sample_counts: SampleCountFlags,
    pub storage_image_sample_counts: SampleCountFlags,
    pub max_sample_mask_words: u32,
    pub timestamp_compute_and_graphics: Bool32,
    pub timestamp_period: f32,
    pub max_clip_distances: u32,
    pub max_cull_distances: u32,
    pub max_combined_clip_and_cull_distances: u32,
    pub discrete_queue_priorities: u32,
    pub point_size_range: [f32; 2],
    pub line_width_range: [f32; 2],
    pub point_size_granularity: f32,
    pub line_width_granularity: f32,
    pub strict_lines: Bool32,
    pub standard_sample_locations: Bool32,
    pub optimal_buffer_copy_offset_alignment: DeviceSize,
    pub optimal_buffer_copy_row_pitch_alignment: DeviceSize,
    pub non_coherent_atom_size: DeviceSize,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct PhysicalDeviceSparseProperties {
    pub residency_standard2_d_block_shape: Bool32,
    pub residency_standard2_d_multisample_block_shape: Bool32,
    pub residency_standard3_d_block_shape: Bool32,
    pub residency_aligned_mip_size: Bool32,
    pub residency_non_resident_strict: Bool32,
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct PhysicalDeviceProperties {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: PhysicalDeviceType,
    pub device_name: [u8; 256],
    pub pipeline_cache_uuid: [u8; 16],
    pub limits: PhysicalDeviceLimits,
    pub sparse_properties: PhysicalDeviceSparseProperties,
}

impl Default for PhysicalDeviceProperties {
    fn default() -> PhysicalDeviceProperties {
        PhysicalDeviceProperties {
            api_version: 0,
            driver_version: 0,
            vendor_id: 0,
            device_id: 0,
            device_type: PhysicalDeviceType::default(),
            device_name: unsafe { mem::zeroed() },
            pipeline_cache_uuid: unsafe { mem::zeroed() },
            limits: PhysicalDeviceLimits::default(),
            sparse_properties: PhysicalDeviceSparseProperties::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct MemoryType {
    pub property_flags: MemoryPropertyFlags,
    pub heap_index: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct MemoryHeap {
    pub size: DeviceSize,
    pub flags: MemoryHeapFlags,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct PhysicalDeviceMemoryProperties {
    pub memory_type_count: u32,
    pub memory_types: [MemoryType; 32],
    pub memory_heap_count: u32,
    pub memory_heaps: [MemoryHeap; 16],
}

#[repr(C)]
pub struct DeviceQueueCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: DeviceQueueCreateFlags,
    pub queue_family_index: u32,
    pub queue_count: u32,
    pub p_queue_priorities: *const f32,
}

impl Default for DeviceQueueCreateInfo {
    fn default() -> DeviceQueueCreateInfo {
        DeviceQueueCreateInfo {
            s_type: StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: DeviceQueueCreateFlags::default(),
            queue_family_index: 0,
            queue_count: 0,
            p_queue_priorities: ptr::null(),
        }
    }
}

#[repr(C)]
pub struct DeviceCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: DeviceCreateFlags,
    pub queue_create_info_count: u32,
    pub p_queue_create_infos: *const DeviceQueueCreateInfo,
    pub enabled_layer_count: u32,
    pub pp_enabled_layer_names: *const *const u8,
    pub enabled_extension_count: u32,
    pub pp_enabled_extension_names: *const *const u8,
    pub p_enabled_features: *const PhysicalDeviceFeatures,
}

impl Default for DeviceCreateInfo {
    fn default() -> DeviceCreateInfo {
        DeviceCreateInfo {
            s_type: StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: DeviceCreateFlags::default(),
            queue_create_info_count: 0,
            p_queue_create_infos: ptr::null(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: 0,
            pp_enabled_extension_names: ptr::null(),
            p_enabled_features: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct ComponentMapping {
    pub r: ComponentSwizzle,
    pub g: ComponentSwizzle,
    pub b: ComponentSwizzle,
    pub a: ComponentSwizzle,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct ImageSubresourceRange {
    pub aspect_mask: ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl ImageSubresourceRange {
    pub const fn base_color() -> Self {
        ImageSubresourceRange {
            aspect_mask: ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        }
    }

    pub const fn base_depth() -> Self {
        ImageSubresourceRange {
            aspect_mask: ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ImageViewCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: ImageViewCreateFlags,
    pub image: Image,
    pub view_type: ImageViewType,
    pub format: Format,
    pub components: ComponentMapping,
    pub subresource_range: ImageSubresourceRange,
}

impl Default for ImageViewCreateInfo {
    fn default() -> ImageViewCreateInfo {
        ImageViewCreateInfo {
            s_type: StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: ImageViewCreateFlags::default(),
            image: Image::default(),
            view_type: ImageViewType::default(),
            format: Format::default(),
            components: ComponentMapping::default(),
            subresource_range: ImageSubresourceRange::default(),
        }
    }
}

#[repr(C)]
pub struct SemaphoreCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: SemaphoreCreateFlags,
}
impl Default for SemaphoreCreateInfo {
    fn default() -> SemaphoreCreateInfo {
        SemaphoreCreateInfo {
            s_type: StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: SemaphoreCreateFlags::default(),
        }
    }
}

#[repr(C)]
pub struct FenceCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: FenceCreateFlags,
}

impl Default for FenceCreateInfo {
    fn default() -> FenceCreateInfo {
        FenceCreateInfo {
            s_type: StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: FenceCreateFlags::default(),
        }
    }
}


#[repr(C)]
pub struct CommandPoolCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: CommandPoolCreateFlags,
    pub queue_family_index: u32,
}

impl Default for CommandPoolCreateInfo {
    fn default() -> CommandPoolCreateInfo {
        CommandPoolCreateInfo {
            s_type: StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null_mut(),
            flags: CommandPoolCreateFlags::default(),
            queue_family_index: 0
        }
    }
}

#[repr(C)]
pub struct CommandBufferAllocateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub command_pool: CommandPool,
    pub level: CommandBufferLevel,
    pub command_buffer_count: u32,
}

impl Default for CommandBufferAllocateInfo {
    fn default() -> CommandBufferAllocateInfo {
        CommandBufferAllocateInfo {
            s_type: StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: CommandPool::default(),
            level: CommandBufferLevel::default(),
            command_buffer_count: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct CommandBufferInheritanceInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub render_pass: RenderPass,
    pub subpass: u32,
    pub framebuffer: Framebuffer,
    pub occlusion_query_enable: Bool32,
    pub query_flags: QueryControlFlags,
    pub pipeline_statistics: QueryPipelineStatisticFlags,
}

impl Default for CommandBufferInheritanceInfo {
    fn default() -> CommandBufferInheritanceInfo {
        CommandBufferInheritanceInfo {
            s_type: StructureType::COMMAND_BUFFER_INHERITANCE_INFO,
            p_next: ptr::null(),
            render_pass: RenderPass::default(),
            subpass: 0,
            framebuffer: Framebuffer::default(),
            occlusion_query_enable: 0,  
            query_flags: QueryControlFlags::default(),
            pipeline_statistics: QueryPipelineStatisticFlags::default(),
        }
    }
}

#[repr(C)]
pub struct CommandBufferBeginInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: CommandBufferUsageFlags,
    pub p_inheritance_info: *const CommandBufferInheritanceInfo,
}
impl Default for CommandBufferBeginInfo {
    fn default() -> CommandBufferBeginInfo {
        CommandBufferBeginInfo {
            s_type: StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            flags: CommandBufferUsageFlags::default(),
            p_inheritance_info: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SubmitInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub wait_semaphore_count: u32,
    pub p_wait_semaphores: *const Semaphore,
    pub p_wait_dst_stage_mask: *const PipelineStageFlags,
    pub command_buffer_count: u32,
    pub p_command_buffers: *const CommandBuffer,
    pub signal_semaphore_count: u32,
    pub p_signal_semaphores: *const Semaphore,
}

impl Default for SubmitInfo {
    fn default() -> SubmitInfo {
        SubmitInfo {
            s_type: StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: ptr::null(),
            command_buffer_count: 0,
            p_command_buffers: ptr::null(),
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        }
    }
}

#[repr(C)]
pub struct RenderPassCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: RenderPassCreateFlags,
    pub attachment_count: u32,
    pub p_attachments: *const AttachmentDescription,
    pub subpass_count: u32,
    pub p_subpasses: *const SubpassDescription,
    pub dependency_count: u32,
    pub p_dependencies: *const SubpassDependency,
}

impl Default for RenderPassCreateInfo {
    fn default() -> RenderPassCreateInfo {
        RenderPassCreateInfo {
            s_type: StructureType::RENDER_PASS_CREATE_INFO,
            p_next: ptr::null(),
            flags: RenderPassCreateFlags::default(),
            attachment_count: 0,
            p_attachments: ptr::null(),
            subpass_count: 0,
            p_subpasses: ptr::null(),
            dependency_count: 0,
            p_dependencies: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct SubpassDependency {
    pub src_subpass: u32,
    pub dst_subpass: u32,
    pub src_stage_mask: PipelineStageFlags,
    pub dst_stage_mask: PipelineStageFlags,
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub dependency_flags: DependencyFlags,
}

#[repr(C)]
pub struct AttachmentDescription {
    pub flags: AttachmentDescriptionFlags,
    pub format: Format,
    pub samples: SampleCountFlags,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub stencil_load_op: AttachmentLoadOp,
    pub stencil_store_op: AttachmentStoreOp,
    pub initial_layout: ImageLayout,
    pub final_layout: ImageLayout,
}

impl Default for AttachmentDescription {
    fn default() -> Self {
        AttachmentDescription {
            flags: AttachmentDescriptionFlags(0),
            format: Format::UNDEFINED,
            samples: SampleCountFlags::TYPE_1,
            load_op: AttachmentLoadOp::CLEAR,
            store_op: AttachmentStoreOp::STORE,
            stencil_load_op: AttachmentLoadOp::DONT_CARE,
            stencil_store_op: AttachmentStoreOp::DONT_CARE,
            initial_layout: ImageLayout::UNDEFINED,
            final_layout: ImageLayout::UNDEFINED
        }
    }
}

#[repr(C)]
pub struct SubpassDescription {
    pub flags: SubpassDescriptionFlags,
    pub pipeline_bind_point: PipelineBindPoint,
    pub input_attachment_count: u32,
    pub p_input_attachments: *const AttachmentReference,
    pub color_attachment_count: u32,
    pub p_color_attachments: *const AttachmentReference,
    pub p_resolve_attachments: *const AttachmentReference,
    pub p_depth_stencil_attachment: *const AttachmentReference,
    pub preserve_attachment_count: u32,
    pub p_preserve_attachments: *const u32,
}

impl Default for SubpassDescription {
    fn default() -> SubpassDescription {
        SubpassDescription {
            flags: SubpassDescriptionFlags::default(),
            pipeline_bind_point: PipelineBindPoint::default(),
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            color_attachment_count: 0,
            p_color_attachments: ptr::null(),
            p_resolve_attachments: ptr::null(),
            p_depth_stencil_attachment: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct AttachmentReference {
    pub attachment: u32,
    pub layout: ImageLayout,
}

#[repr(C)]
pub struct FramebufferCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: FramebufferCreateFlags,
    pub render_pass: RenderPass,
    pub attachment_count: u32,
    pub p_attachments: *const ImageView,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
}

impl Default for FramebufferCreateInfo {
    fn default() -> FramebufferCreateInfo {
        FramebufferCreateInfo {
            s_type: StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: FramebufferCreateFlags::default(),
            render_pass: RenderPass::default(),
            attachment_count: 0,
            p_attachments: ptr::null(),
            width: 0,
            height: 0,
            layers: 0,
        }
    }
}

#[repr(C)]
pub struct RenderPassBeginInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub render_pass: RenderPass,
    pub framebuffer: Framebuffer,
    pub render_area: Rect2D,
    pub clear_value_count: u32,
    pub p_clear_values: *const ClearValue,
}
impl Default for RenderPassBeginInfo {
    fn default() -> RenderPassBeginInfo {
        RenderPassBeginInfo {
            s_type: StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: RenderPass::default(),
            framebuffer: Framebuffer::default(),
            render_area: Rect2D::default(),
            clear_value_count: 0,
            p_clear_values: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ClearColorValue {
    pub float32: [f32; 4],
    pub int32: [i32; 4],
    pub uint32: [u32; 4],
}
impl Default for ClearColorValue {
    fn default() -> ClearColorValue {
        unsafe { mem::zeroed() }
    }
}

impl ClearColorValue {

    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        let mut v = Self::default();
        v.float32 = [r, g, b, a];
        v
    }

}


#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct ClearDepthStencilValue {
    pub depth: f32,
    pub stencil: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ClearValue {
    pub color: ClearColorValue,
    pub depth_stencil: ClearDepthStencilValue,
}
impl Default for ClearValue {
    fn default() -> ClearValue {
        unsafe { mem::zeroed() }
    }
}

impl From<ClearColorValue> for ClearValue {

    fn from(color: ClearColorValue) -> Self {
        let mut v = Self::default();
        v.color = color; 
        v
    }

}

impl From<ClearDepthStencilValue> for ClearValue {

    fn from(depth_stencil: ClearDepthStencilValue) -> Self {
        let mut v = Self::default();
        v.depth_stencil = depth_stencil; 
        v
    }

}

#[repr(C)]
pub struct MemoryAllocateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub allocation_size: DeviceSize,
    pub memory_type_index: u32,
}

impl Default for MemoryAllocateInfo {
    fn default() -> MemoryAllocateInfo {
        MemoryAllocateInfo {
            s_type: StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: 0,
            memory_type_index: 0,
        }
    }
}

#[repr(C)]
pub struct BufferCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: BufferCreateFlags,
    pub size: DeviceSize,
    pub usage: BufferUsageFlags,
    pub sharing_mode: SharingMode,
    pub queue_family_index_count: u32,
    pub p_queue_family_indices: *const u32,
}

impl Default for BufferCreateInfo {
    fn default() -> BufferCreateInfo {
        BufferCreateInfo {
            s_type: StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: BufferCreateFlags::default(),
            size: 0,
            usage: BufferUsageFlags::default(),
            sharing_mode: SharingMode::default(),
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        }
    }
}

vk_enum!(BufferUsageFlags);
vk_bitflags!(BufferUsageFlags);
impl BufferUsageFlags {
    pub const TRANSFER_SRC: Self = Self(0b1);
    pub const TRANSFER_DST: Self = Self(0b10);
    pub const UNIFORM_TEXEL_BUFFER: Self = Self(0b100);
    pub const STORAGE_TEXEL_BUFFER: Self = Self(0b1000);
    pub const UNIFORM_BUFFER: Self = Self(0b1_0000);
    pub const STORAGE_BUFFER: Self = Self(0b10_0000);
    pub const INDEX_BUFFER: Self = Self(0b100_0000);
    pub const VERTEX_BUFFER: Self = Self(0b1000_0000);
    pub const INDIRECT_BUFFER: Self = Self(0b1_0000_0000);
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct MemoryRequirements {
    pub size: DeviceSize,
    pub alignment: DeviceSize,
    pub memory_type_bits: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct BufferCopy {
    pub src_offset: DeviceSize,
    pub dst_offset: DeviceSize,
    pub size: DeviceSize,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct BufferImageCopy {
    pub buffer_offset: DeviceSize,
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub image_subresource: ImageSubresourceLayers,
    pub image_offset: Offset3D,
    pub image_extent: Extent3D,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct ImageSubresourceLayers {
    pub aspect_mask: ImageAspectFlags,
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl ImageSubresourceLayers {

    pub fn base_color() -> Self {
        ImageSubresourceLayers {
            aspect_mask: ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        }
    }

}

#[repr(C)]
pub struct ShaderModuleCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: ShaderModuleCreateFlags,
    pub code_size: usize,
    pub p_code: *const u32,
}
impl Default for ShaderModuleCreateInfo {
    fn default() -> ShaderModuleCreateInfo {
        ShaderModuleCreateInfo {
            s_type: StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: ShaderModuleCreateFlags::default(),
            code_size: 0,
            p_code: ptr::null(),
        }
    }
}


#[repr(C)]
pub struct PipelineCacheCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineCacheCreateFlags,
    pub initial_data_size: usize,
    pub p_initial_data: *const c_void,
}

impl Default for PipelineCacheCreateInfo {
    fn default() -> PipelineCacheCreateInfo {
        PipelineCacheCreateInfo {
            s_type: StructureType::PIPELINE_CACHE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineCacheCreateFlags::default(),
            initial_data_size: 0,
            p_initial_data: ptr::null(),
        }
    }
}



#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineShaderStageCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineShaderStageCreateFlags,
    pub stage: ShaderStageFlags,
    pub module: ShaderModule,
    pub p_name: *const u8,
    pub p_specialization_info: *const SpecializationInfo,
}
impl Default for PipelineShaderStageCreateInfo {
    fn default() -> PipelineShaderStageCreateInfo {
        PipelineShaderStageCreateInfo {
            s_type: StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineShaderStageCreateFlags::default(),
            stage: ShaderStageFlags::default(),
            module: ShaderModule::default(),
            p_name: ptr::null(),
            p_specialization_info: ptr::null(),
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct SpecializationMapEntry {
    pub constant_id: u32,
    pub offset: u32,
    pub size: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SpecializationInfo {
    pub map_entry_count: u32,
    pub p_map_entries: *const SpecializationMapEntry,
    pub data_size: usize,
    pub p_data: *const c_void,
}
impl Default for SpecializationInfo {
    fn default() -> SpecializationInfo {
        SpecializationInfo {
            map_entry_count: 0,
            p_map_entries: ptr::null(),
            data_size: 0,
            p_data: ptr::null(),
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct VertexInputBindingDescription {
    pub binding: u32,
    pub stride: u32,
    pub input_rate: VertexInputRate,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct VertexInputAttributeDescription {
    pub location: u32,
    pub binding: u32,
    pub format: Format,
    pub offset: u32,
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineVertexInputStateCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineVertexInputStateCreateFlags,
    pub vertex_binding_description_count: u32,
    pub p_vertex_binding_descriptions: *const VertexInputBindingDescription,
    pub vertex_attribute_description_count: u32,
    pub p_vertex_attribute_descriptions: *const VertexInputAttributeDescription,
}
impl Default for PipelineVertexInputStateCreateInfo {
    fn default() -> PipelineVertexInputStateCreateInfo {
        PipelineVertexInputStateCreateInfo {
            s_type: StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineVertexInputStateCreateFlags::default(),
            vertex_binding_description_count: 0,
            p_vertex_binding_descriptions: ptr::null(),
            vertex_attribute_description_count: 0,
            p_vertex_attribute_descriptions: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineInputAssemblyStateCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineInputAssemblyStateCreateFlags,
    pub topology: PrimitiveTopology,
    pub primitive_restart_enable: Bool32,
}

impl Default for PipelineInputAssemblyStateCreateInfo {
    fn default() -> PipelineInputAssemblyStateCreateInfo {
        PipelineInputAssemblyStateCreateInfo {
            s_type: StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineInputAssemblyStateCreateFlags::default(),
            topology: PrimitiveTopology::default(),
            primitive_restart_enable: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineTessellationStateCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineTessellationStateCreateFlags,
    pub patch_control_points: u32,
}
impl Default for PipelineTessellationStateCreateInfo {
    fn default() -> PipelineTessellationStateCreateInfo {
        PipelineTessellationStateCreateInfo {
            s_type: StructureType::PIPELINE_TESSELLATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineTessellationStateCreateFlags::default(),
            patch_control_points: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineViewportStateCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineViewportStateCreateFlags,
    pub viewport_count: u32,
    pub p_viewports: *const Viewport,
    pub scissor_count: u32,
    pub p_scissors: *const Rect2D,
}
impl Default for PipelineViewportStateCreateInfo {
    fn default() -> PipelineViewportStateCreateInfo {
        PipelineViewportStateCreateInfo {
            s_type: StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineViewportStateCreateFlags::default(),
            viewport_count: 0,
            p_viewports: ptr::null(),
            scissor_count: 0,
            p_scissors: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineRasterizationStateCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineRasterizationStateCreateFlags,
    pub depth_clamp_enable: Bool32,
    pub rasterizer_discard_enable: Bool32,
    pub polygon_mode: PolygonMode,
    pub cull_mode: CullModeFlags,
    pub front_face: FrontFace,
    pub depth_bias_enable: Bool32,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_clamp: f32,
    pub depth_bias_slope_factor: f32,
    pub line_width: f32,
}
impl Default for PipelineRasterizationStateCreateInfo {
    fn default() -> PipelineRasterizationStateCreateInfo {
        PipelineRasterizationStateCreateInfo {
            s_type: StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineRasterizationStateCreateFlags::default(),
            depth_clamp_enable: 0,
            rasterizer_discard_enable: 0,
            polygon_mode: PolygonMode::default(),
            cull_mode: CullModeFlags::default(),
            front_face: FrontFace::default(),
            depth_bias_enable: 0,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            line_width: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineMultisampleStateCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineMultisampleStateCreateFlags,
pub rasterization_samples: SampleCountFlags,
    pub sample_shading_enable: Bool32,
    pub min_sample_shading: f32,
    pub p_sample_mask: *const SampleMask,
    pub alpha_to_coverage_enable: Bool32,
    pub alpha_to_one_enable: Bool32,
}
impl Default for PipelineMultisampleStateCreateInfo {
    fn default() -> PipelineMultisampleStateCreateInfo {
        PipelineMultisampleStateCreateInfo {
            s_type: StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineMultisampleStateCreateFlags::default(),
            rasterization_samples: SampleCountFlags::TYPE_1,
            sample_shading_enable: 0,
            min_sample_shading: 0.0,
            p_sample_mask: ptr::null(),
            alpha_to_coverage_enable: 0,
            alpha_to_one_enable: 0
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct StencilOpState {
    pub fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub compare_op: CompareOp,
    pub compare_mask: u32,
    pub write_mask: u32,
    pub reference: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineDepthStencilStateCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineDepthStencilStateCreateFlags,
    pub depth_test_enable: Bool32,
    pub depth_write_enable: Bool32,
    pub depth_compare_op: CompareOp,
    pub depth_bounds_test_enable: Bool32,
    pub stencil_test_enable: Bool32,
    pub front: StencilOpState,
    pub back: StencilOpState,
    pub min_depth_bounds: f32,
    pub max_depth_bounds: f32,
}
impl Default for PipelineDepthStencilStateCreateInfo {
    fn default() -> PipelineDepthStencilStateCreateInfo {
        PipelineDepthStencilStateCreateInfo {
            s_type: StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineDepthStencilStateCreateFlags::default(),
            depth_test_enable: 0,
            depth_write_enable: 0,
            depth_compare_op: CompareOp::default(),
            depth_bounds_test_enable: 0,
            stencil_test_enable: 0,
            front: StencilOpState::default(),
            back: StencilOpState::default(),
            min_depth_bounds: 0.0,
            max_depth_bounds: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineColorBlendAttachmentState {
    pub blend_enable: Bool32,
    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_op: BlendOp,
    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub alpha_blend_op: BlendOp,
    pub color_write_mask: ColorComponentFlags,
}

impl PipelineColorBlendAttachmentState {
    pub const fn const_default() -> Self {
        PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: BlendFactor::ZERO,
            dst_color_blend_factor: BlendFactor::ZERO,
            color_blend_op: BlendOp::ADD,
            src_alpha_blend_factor: BlendFactor::ZERO,
            dst_alpha_blend_factor: BlendFactor::ZERO,
            alpha_blend_op: BlendOp::ADD,
            color_write_mask: ColorComponentFlags(0b1111),
        }
    }
}

impl Default for PipelineColorBlendAttachmentState {
    fn default() -> PipelineColorBlendAttachmentState {
        Self::const_default()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineColorBlendStateCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineColorBlendStateCreateFlags,
    pub logic_op_enable: Bool32,
    pub logic_op: LogicOp,
    pub attachment_count: u32,
    pub p_attachments: *const PipelineColorBlendAttachmentState,
    pub blend_constants: [f32; 4],
}

impl Default for PipelineColorBlendStateCreateInfo {
    fn default() -> PipelineColorBlendStateCreateInfo {
        PipelineColorBlendStateCreateInfo {
            s_type: StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineColorBlendStateCreateFlags::default(),
            logic_op_enable: 0,
            logic_op: LogicOp::default(),
            attachment_count: 0,
            p_attachments: ptr::null(),
            blend_constants: unsafe { mem::zeroed() },
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineDynamicStateCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineDynamicStateCreateFlags,
    pub dynamic_state_count: u32,
    pub p_dynamic_states: *const DynamicState,
}
impl Default for PipelineDynamicStateCreateInfo {
    fn default() -> PipelineDynamicStateCreateInfo {
        PipelineDynamicStateCreateInfo {
            s_type: StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineDynamicStateCreateFlags::default(),
            dynamic_state_count: 0,
            p_dynamic_states: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GraphicsPipelineCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineCreateFlags,
    pub stage_count: u32,
    pub p_stages: *const PipelineShaderStageCreateInfo,
    pub p_vertex_input_state: *const PipelineVertexInputStateCreateInfo,
    pub p_input_assembly_state: *const PipelineInputAssemblyStateCreateInfo,
    pub p_tessellation_state: *const PipelineTessellationStateCreateInfo,
    pub p_viewport_state: *const PipelineViewportStateCreateInfo,
    pub p_rasterization_state: *const PipelineRasterizationStateCreateInfo,
    pub p_multisample_state: *const PipelineMultisampleStateCreateInfo,
    pub p_depth_stencil_state: *const PipelineDepthStencilStateCreateInfo,
    pub p_color_blend_state: *const PipelineColorBlendStateCreateInfo,
    pub p_dynamic_state: *const PipelineDynamicStateCreateInfo,
    pub layout: PipelineLayout,
    pub render_pass: RenderPass,
    pub subpass: u32,
    pub base_pipeline_handle: Pipeline,
    pub base_pipeline_index: i32,
}

impl Default for GraphicsPipelineCreateInfo {
    fn default() -> GraphicsPipelineCreateInfo {
        GraphicsPipelineCreateInfo {
            s_type: StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineCreateFlags::default(),
            stage_count: 0,
            p_stages: ptr::null(),
            p_vertex_input_state: ptr::null(),
            p_input_assembly_state: ptr::null(),
            p_tessellation_state: ptr::null(),
            p_viewport_state: ptr::null(),
            p_rasterization_state: ptr::null(),
            p_multisample_state: ptr::null(),
            p_depth_stencil_state: ptr::null(),
            p_color_blend_state: ptr::null(),
            p_dynamic_state: ptr::null(),
            layout: PipelineLayout::default(),
            render_pass: RenderPass::default(),
            subpass: 0,
            base_pipeline_handle: Pipeline::default(),
            base_pipeline_index: i32::default(),
        }
    }
}

#[repr(C)]
pub struct PipelineLayoutCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineLayoutCreateFlags,
    pub set_layout_count: u32,
    pub p_set_layouts: *const DescriptorSetLayout,
    pub push_constant_range_count: u32,
    pub p_push_constant_ranges: *const PushConstantRange,
}

impl Default for PipelineLayoutCreateInfo {
    fn default() -> PipelineLayoutCreateInfo {
        PipelineLayoutCreateInfo {
            s_type: StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineLayoutCreateFlags::default(),
            set_layout_count: 0,
            p_set_layouts: ptr::null(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct PushConstantRange {
    pub stage_flags: ShaderStageFlags,
    pub offset: u32,
    pub size: u32,
}

#[repr(C)]
pub struct DescriptorSetLayoutCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: DescriptorSetLayoutCreateFlags,
    pub binding_count: u32,
    pub p_bindings: *const DescriptorSetLayoutBinding,
}

impl Default for DescriptorSetLayoutCreateInfo {
    fn default() -> DescriptorSetLayoutCreateInfo {
        DescriptorSetLayoutCreateInfo {
            s_type: StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: DescriptorSetLayoutCreateFlags::default(),
            binding_count: 0,
            p_bindings: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: ShaderStageFlags,
    pub p_immutable_samplers: *const Sampler,
}

impl Default for DescriptorSetLayoutBinding {
    fn default() -> DescriptorSetLayoutBinding {
        DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: DescriptorType::default(),
            descriptor_count: 0,
            stage_flags: ShaderStageFlags::default(),
            p_immutable_samplers: ptr::null(),
        }
    }
}

#[repr(C)]
pub struct DescriptorPoolCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: DescriptorPoolCreateFlags,
    pub max_sets: u32,
    pub pool_size_count: u32,
    pub p_pool_sizes: *const DescriptorPoolSize,
}

impl Default for DescriptorPoolCreateInfo {
    fn default() -> DescriptorPoolCreateInfo {
        DescriptorPoolCreateInfo {
            s_type: StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: DescriptorPoolCreateFlags::default(),
            max_sets: 0,
            pool_size_count: 0,
            p_pool_sizes: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct DescriptorPoolSize {
    pub ty: DescriptorType,
    pub descriptor_count: u32,
}

#[repr(C)]
pub struct DescriptorSetAllocateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub descriptor_pool: DescriptorPool,
    pub descriptor_set_count: u32,
    pub p_set_layouts: *const DescriptorSetLayout,
}
impl Default for DescriptorSetAllocateInfo {
    fn default() -> DescriptorSetAllocateInfo {
        DescriptorSetAllocateInfo {
            s_type: StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: DescriptorPool::default(),
            descriptor_set_count: 0,
            p_set_layouts: ptr::null(),
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct WriteDescriptorSet {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub dst_set: DescriptorSet,
    pub dst_binding: u32,
    pub dst_array_element: u32,
    pub descriptor_count: u32,
    pub descriptor_type: DescriptorType,
    pub p_image_info: *const DescriptorImageInfo,
    pub p_buffer_info: *const DescriptorBufferInfo,
    pub p_texel_buffer_view: *const BufferView,
}

impl Default for WriteDescriptorSet {
    fn default() -> WriteDescriptorSet {
        WriteDescriptorSet {
            s_type: StructureType::WRITE_DESCRIPTOR_SET,
            p_next: ptr::null(),
            dst_set: DescriptorSet::default(),
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 0,
            descriptor_type: DescriptorType::default(),
            p_image_info: ptr::null(),
            p_buffer_info: ptr::null(),
            p_texel_buffer_view: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct DescriptorImageInfo {
    pub sampler: Sampler,
    pub image_view: ImageView,
    pub image_layout: ImageLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct DescriptorBufferInfo {
    pub buffer: Buffer,
    pub offset: DeviceSize,
    pub range: DeviceSize,
}

#[repr(C)]
pub struct CopyDescriptorSet {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub src_set: DescriptorSet,
    pub src_binding: u32,
    pub src_array_element: u32,
    pub dst_set: DescriptorSet,
    pub dst_binding: u32,
    pub dst_array_element: u32,
    pub descriptor_count: u32,
}
impl Default for CopyDescriptorSet {
    fn default() -> CopyDescriptorSet {
        CopyDescriptorSet {
            s_type: StructureType::COPY_DESCRIPTOR_SET,
            p_next: ptr::null(),
            src_set: DescriptorSet::default(),
            src_binding: 0,
            src_array_element: 0,
            dst_set: DescriptorSet::default(),
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 0,
        }
    }
}

#[repr(C)]
pub struct ImageCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: ImageCreateFlags,
    pub image_type: ImageType,
    pub format: Format,
    pub extent: Extent3D,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: SampleCountFlags,
    pub tiling: ImageTiling,
    pub usage: ImageUsageFlags,
    pub sharing_mode: SharingMode,
    pub queue_family_index_count: u32,
    pub p_queue_family_indices: *const u32,
    pub initial_layout: ImageLayout,
}
impl Default for ImageCreateInfo {
    fn default() -> ImageCreateInfo {
        ImageCreateInfo {
            s_type: StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: ImageCreateFlags::default(),
            image_type: ImageType::default(),
            format: Format::default(),
            extent: Extent3D::default(),
            mip_levels: 1,
            array_layers: 1,
            samples: SampleCountFlags::TYPE_1,
            tiling: ImageTiling::OPTIMAL,
            usage: ImageUsageFlags::default(),
            sharing_mode: SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            initial_layout: ImageLayout::UNDEFINED,
        }
    }
}


#[repr(C)]
pub struct SamplerCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: SamplerCreateFlags,
    pub mag_filter: Filter,
    pub min_filter: Filter,
    pub mipmap_mode: SamplerMipmapMode,
    pub address_mode_u: SamplerAddressMode,
    pub address_mode_v: SamplerAddressMode,
    pub address_mode_w: SamplerAddressMode,
    pub mip_lod_bias: f32,
    pub anisotropy_enable: Bool32,
    pub max_anisotropy: f32,
    pub compare_enable: Bool32,
    pub compare_op: CompareOp,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: BorderColor,
    pub unnormalized_coordinates: Bool32,
}

impl Default for SamplerCreateInfo {
    fn default() -> SamplerCreateInfo {
        SamplerCreateInfo {
            s_type: StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: SamplerCreateFlags::default(),
            mag_filter: Filter::default(),
            min_filter: Filter::default(),
            mipmap_mode: SamplerMipmapMode::default(),
            address_mode_u: SamplerAddressMode::default(),
            address_mode_v: SamplerAddressMode::default(),
            address_mode_w: SamplerAddressMode::default(),
            mip_lod_bias: 0.0,
            anisotropy_enable: 0,
            max_anisotropy: 0.0,
            compare_enable: 0,
            compare_op: CompareOp::default(),
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: BorderColor::default(),
            unnormalized_coordinates: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct FormatProperties {
    pub linear_tiling_features: FormatFeatureFlags,
    pub optimal_tiling_features: FormatFeatureFlags,
    pub buffer_features: FormatFeatureFlags,
}


#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct ImageCopy {
    pub src_subresource: ImageSubresourceLayers,
    pub src_offset: Offset3D,
    pub dst_subresource: ImageSubresourceLayers,
    pub dst_offset: Offset3D,
    pub extent: Extent3D,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ComputePipelineCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineCreateFlags,
    pub stage: PipelineShaderStageCreateInfo,
    pub layout: PipelineLayout,
    pub base_pipeline_handle: Pipeline,
    pub base_pipeline_index: i32,
}

impl Default for ComputePipelineCreateInfo {
    fn default() -> Self {
        Self {
            s_type: StructureType::COMPUTE_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: PipelineCreateFlags::default(),
            stage: PipelineShaderStageCreateInfo::default(),
            layout: PipelineLayout::default(),
            base_pipeline_handle: Pipeline::default(),
            base_pipeline_index: i32::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BufferViewCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: BufferViewCreateFlags,
    pub buffer: Buffer,
    pub format: Format,
    pub offset: DeviceSize,
    pub range: DeviceSize,
}
impl Default for BufferViewCreateInfo {
    fn default() -> Self {
        Self {
            s_type: StructureType::BUFFER_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: BufferViewCreateFlags::default(),
            buffer: Buffer::default(),
            format: Format::default(),
            offset: DeviceSize::default(),
            range: DeviceSize::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct DrawIndexedIndirectCommand {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
    pub first_instance: u32,
}
