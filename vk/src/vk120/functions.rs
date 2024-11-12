use crate::*;
use std::{
    ffi::CStr,
    mem::transmute,
};

//
// Instance FN
//

pub struct DeviceFnV1_2 {
    pub wait_semaphores: PFN_vkWaitSemaphores,
    pub cmd_draw_indexed_indirect_count: PFN_vkCmdDrawIndexedIndirectCount,
}

impl DeviceFnV1_2 {

    pub fn load<F>(cb: F) -> DeviceFnV1_2 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            let cstr = CStr::from_bytes_with_nul_unchecked;
            DeviceFnV1_2 {
                wait_semaphores: transmute(cb(cstr(b"vkWaitSemaphores\0"))),
                cmd_draw_indexed_indirect_count: transmute(cb(cstr(b"vkCmdDrawIndexedIndirectCount\0"))),
            }
        }
    }
}


//
// Functions def
//

pub type PFN_vkWaitSemaphores = unsafe extern "system" fn(
    device: Device,
    wait_info: &SemaphoreWaitInfo,
    timeout: u64,
) -> VkResult;

pub type PFN_vkCmdDrawIndexedIndirectCount = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    count_buffer: Buffer,
    count_buffer_offset: DeviceSize,
    max_draw_count: u32, 
    stride: u32
);
