use crate::*;
use std::{ptr, ffi::c_void};

impl StructureType {
    pub const DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO: Self = Self(1_000_161_000);
    pub const PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES: Self = Self(1_000_161_001);
    pub const DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_ALLOCATE_INFO: Self = Self(1_000_161_003);
}

vk_enum!(DescriptorBindingFlags);
vk_bitflags!(DescriptorBindingFlags);
impl DescriptorBindingFlags {
    pub const UPDATE_AFTER_BIND: Self = Self(0b1);
    pub const UPDATE_UNUSED_WHILE_PENDING: Self = Self(0b10);
    pub const PARTIALLY_BOUND: Self = Self(0b100);
    pub const VARIABLE_DESCRIPTOR_COUNT: Self = Self(0b1000);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PhysicalDeviceDescriptorIndexingFeatures {
    pub s_type: StructureType,
    pub p_next: *mut c_void,
    pub shader_input_attachment_array_dynamic_indexing: Bool32,
    pub shader_uniform_texel_buffer_array_dynamic_indexing: Bool32,
    pub shader_storage_texel_buffer_array_dynamic_indexing: Bool32,
    pub shader_uniform_buffer_array_non_uniform_indexing: Bool32,
    pub shader_sampled_image_array_non_uniform_indexing: Bool32,
    pub shader_storage_buffer_array_non_uniform_indexing: Bool32,
    pub shader_storage_image_array_non_uniform_indexing: Bool32,
    pub shader_input_attachment_array_non_uniform_indexing: Bool32,
    pub shader_uniform_texel_buffer_array_non_uniform_indexing: Bool32,
    pub shader_storage_texel_buffer_array_non_uniform_indexing: Bool32,
    pub descriptor_binding_uniform_buffer_update_after_bind: Bool32,
    pub descriptor_binding_sampled_image_update_after_bind: Bool32,
    pub descriptor_binding_storage_image_update_after_bind: Bool32,
    pub descriptor_binding_storage_buffer_update_after_bind: Bool32,
    pub descriptor_binding_uniform_texel_buffer_update_after_bind: Bool32,
    pub descriptor_binding_storage_texel_buffer_update_after_bind: Bool32,
    pub descriptor_binding_update_unused_while_pending: Bool32,
    pub descriptor_binding_partially_bound: Bool32,
    pub descriptor_binding_variable_descriptor_count: Bool32,
    pub runtime_descriptor_array: Bool32,
}

impl Default for PhysicalDeviceDescriptorIndexingFeatures {
    fn default() -> Self {
        Self {
            s_type: StructureType::PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES,
            p_next: ptr::null_mut(),
            shader_input_attachment_array_dynamic_indexing: 0,
            shader_uniform_texel_buffer_array_dynamic_indexing: 0,
            shader_storage_texel_buffer_array_dynamic_indexing: 0,
            shader_uniform_buffer_array_non_uniform_indexing: 0,
            shader_sampled_image_array_non_uniform_indexing: 0,
            shader_storage_buffer_array_non_uniform_indexing: 0,
            shader_storage_image_array_non_uniform_indexing: 0,
            shader_input_attachment_array_non_uniform_indexing: 0,
            shader_uniform_texel_buffer_array_non_uniform_indexing: 0,
            shader_storage_texel_buffer_array_non_uniform_indexing: 0,
            descriptor_binding_uniform_buffer_update_after_bind: 0,
            descriptor_binding_sampled_image_update_after_bind: 0,
            descriptor_binding_storage_image_update_after_bind: 0,
            descriptor_binding_storage_buffer_update_after_bind: 0,
            descriptor_binding_uniform_texel_buffer_update_after_bind: 0,
            descriptor_binding_storage_texel_buffer_update_after_bind: 0,
            descriptor_binding_update_unused_while_pending: 0,
            descriptor_binding_partially_bound: 0,
            descriptor_binding_variable_descriptor_count: 0,
            runtime_descriptor_array: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DescriptorSetLayoutBindingFlagsCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub binding_count: u32,
    pub p_binding_flags: *const DescriptorBindingFlags,
}
impl Default for DescriptorSetLayoutBindingFlagsCreateInfo {
    fn default() -> Self {
        Self {
            s_type: StructureType::DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO,
            p_next: ptr::null(),
            binding_count: 0,
            p_binding_flags: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DescriptorSetVariableDescriptorCountAllocateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub descriptor_set_count: u32,
    pub p_descriptor_counts: *const u32,
}
impl Default for DescriptorSetVariableDescriptorCountAllocateInfo {
    fn default() -> Self {
        Self {
            s_type: StructureType::DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_set_count: 0,
            p_descriptor_counts: ptr::null(),
        }
    }
}
