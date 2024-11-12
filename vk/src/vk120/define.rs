use crate::*;
use std::ptr;
use std::ffi::c_void;

#[repr(C)]
#[derive(Debug)]
pub struct PhysicalDeviceTimelineSemaphoreFeatures {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub timeline_semaphore: Bool32,
}

impl ::std::default::Default for PhysicalDeviceTimelineSemaphoreFeatures {
    fn default() -> PhysicalDeviceTimelineSemaphoreFeatures {
        PhysicalDeviceTimelineSemaphoreFeatures {
            s_type: StructureType::PHYSICAL_DEVICE_TIMELINE_SEMAPHORE_FEATURES,
            p_next: ptr::null(),
            timeline_semaphore: 0
        }
    }
}

#[repr(C)]
pub struct SemaphoreTypeCreateInfo {
    pub s_type : StructureType,
	pub p_next : *const c_void,
	pub semaphore_type: SemaphoreType,
    pub initial_value: u64, 
}

impl ::std::default::Default for SemaphoreTypeCreateInfo {
    fn default() -> SemaphoreTypeCreateInfo {
        SemaphoreTypeCreateInfo {
            s_type: StructureType::SEMAPHORE_TYPE_CREATE_INFO,
            p_next: ptr::null(),
            semaphore_type: SemaphoreType::BINARY,
            initial_value: 0,
        }
    }
}

#[repr(C)]
pub struct SemaphoreWaitInfo {
	pub s_type: StructureType,
	pub p_next: *const c_void,
	pub flags: SemaphoreWaitFlagsBits,
	pub semaphore_count: u32,
	pub p_semaphores: *const Semaphore,
	pub p_values: *const u64,
}

impl ::std::default::Default for SemaphoreWaitInfo {
    fn default() -> SemaphoreWaitInfo {
        SemaphoreWaitInfo {
            s_type: StructureType::SEMAPHORE_WAIT_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            semaphore_count: 0,
            p_semaphores: ptr::null(),
            p_values: ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TimelineSemaphoreSubmitInfo {
	pub s_type: StructureType,
	pub p_next: *const c_void,
	pub wait_semaphore_value_count: u32,
	pub p_wait_semaphore_values: *const u64,
	pub signal_semaphore_value_count: u32,
	pub p_signal_semaphore_values: *const u64,
}

impl ::std::default::Default for TimelineSemaphoreSubmitInfo {
    fn default() -> TimelineSemaphoreSubmitInfo {
        TimelineSemaphoreSubmitInfo {
            s_type: StructureType::TIMELINE_SEMAPHORE_SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_value_count: 0,
            p_wait_semaphore_values: ptr::null(),
            signal_semaphore_value_count: 0,
            p_signal_semaphore_values: ptr::null(),
        }
    }
}
