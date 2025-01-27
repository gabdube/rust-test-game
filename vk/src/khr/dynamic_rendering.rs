use crate::*;
use std::{ptr, ffi::{c_void, CStr}, mem::transmute};

impl StructureType {
    pub const RENDERING_INFO: Self = Self(1_000_044_000);
    pub const RENDERING_ATTACHMENT_INFO: Self = Self(1_000_044_001);
    pub const PIPELINE_RENDERING_CREATE_INFO: Self = Self(1_000_044_002);
    pub const PHYSICAL_DEVICE_DYNAMIC_RENDERING_FEATURES: Self = Self(1_000_044_003);
}

vk_enum!(RenderingFlagBitsKHR);
vk_bitflags!(RenderingFlagBitsKHR);
impl RenderingFlagBitsKHR {
    pub const RENDERING_CONTENTS_SECONDARY_COMMAND_BUFFERS: Self = Self(0x001);
    pub const RENDERING_SUSPENDING: Self = Self(0x002);
    pub const RENDERING_RESUMING: Self = Self(0x004);
}

#[repr(C)]
pub struct PhysicalDeviceDynamicRenderingFeatures {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub dynamic_rendering: Bool32,
}

impl ::std::default::Default for PhysicalDeviceDynamicRenderingFeatures {
    fn default() -> PhysicalDeviceDynamicRenderingFeatures {
        PhysicalDeviceDynamicRenderingFeatures {
            s_type: StructureType::PHYSICAL_DEVICE_DYNAMIC_RENDERING_FEATURES,
            p_next: ptr::null(),
            dynamic_rendering: 0
        }
    }
}

#[repr(C)]
pub struct RenderingAttachmentInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub image_view: ImageView,
    pub image_layout: ImageLayout,
    pub resolve_mode: ResolveModeFlagsBits,
    pub resolve_image_view: ImageView,
    pub resolve_image_layout: ImageLayout,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub clear_value: ClearValue, 
}

impl ::std::default::Default for RenderingAttachmentInfo {
    fn default() -> RenderingAttachmentInfo {
        RenderingAttachmentInfo {
            s_type: StructureType::RENDERING_ATTACHMENT_INFO,
            p_next: ptr::null(),
            image_view: ImageView::null(),
            image_layout: ImageLayout::UNDEFINED,
            resolve_mode: ResolveModeFlagsBits::NONE,
            resolve_image_view: ImageView::null(),
            resolve_image_layout: ImageLayout::UNDEFINED,
            load_op: AttachmentLoadOp::DONT_CARE,
            store_op: AttachmentStoreOp::DONT_CARE,
            clear_value: ClearValue::default()
        }
    }
}

#[repr(C)]
pub struct RenderingInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: RenderingFlagBitsKHR,
    pub render_area: Rect2D,
    pub layer_count: u32,
    pub view_mask: u32,
    pub color_attachment_count: u32,
    pub color_attachments: *const RenderingAttachmentInfo,
    pub depth_attachment: *const RenderingAttachmentInfo,
    pub stencil_attachment: *const RenderingAttachmentInfo,
}

impl ::std::default::Default for RenderingInfo {
    fn default() -> RenderingInfo {
        RenderingInfo {
            s_type: StructureType::RENDERING_INFO,
            p_next: ptr::null(),
            flags: RenderingFlagBitsKHR::default(),
            render_area: Rect2D::default(),
            layer_count: 0,
            view_mask: 0,
            color_attachment_count: 0,
            color_attachments: ptr::null(),
            depth_attachment: ptr::null(),
            stencil_attachment: ptr::null()
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PipelineRenderingCreateInfo {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub view_mask: u32,
    pub color_attachment_count: u32,
    pub p_color_attachment_formats: *const Format,
    pub depth_attachment_format: Format,
    pub stencil_attachment_format: Format,
}
impl Default for PipelineRenderingCreateInfo {
    fn default() -> Self {
        Self {
            s_type: StructureType::PIPELINE_RENDERING_CREATE_INFO,
            p_next: ptr::null(),
            view_mask: 0,
            color_attachment_count: 0,
            p_color_attachment_formats: ptr::null(),
            depth_attachment_format: Format::UNDEFINED,
            stencil_attachment_format: Format::UNDEFINED,
        }
    }
}

pub struct DynamicRenderingFn {
    pub cmd_begin_rendering_khr: PFN_vkBeginRenderingKHR,
    pub cmd_end_rendering_khr: PFN_vkEndRenderingKHR,
}

impl DynamicRenderingFn {

    pub fn load<F>(cb: F) -> DynamicRenderingFn 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            DynamicRenderingFn {
                cmd_begin_rendering_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkCmdBeginRenderingKHR\0"))),
                cmd_end_rendering_khr: transmute(cb(CStr::from_bytes_with_nul_unchecked(b"vkCmdEndRenderingKHR\0"))),
            }
        }
    }

}

pub type PFN_vkBeginRenderingKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    rendering_info: *const RenderingInfo
);

pub type PFN_vkEndRenderingKHR = unsafe extern "system" fn(command_buffer: CommandBuffer);
