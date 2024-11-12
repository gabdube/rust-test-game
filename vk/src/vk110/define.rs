use crate::*;
use std::ptr;
use std::ffi::c_void;

#[repr(C)]
pub struct PhysicalDeviceFeatures2 {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub features: PhysicalDeviceFeatures,
}

impl ::std::default::Default for PhysicalDeviceFeatures2 {
    fn default() -> PhysicalDeviceFeatures2 {
        PhysicalDeviceFeatures2 {
            s_type: StructureType::PHYSICAL_DEVICE_FEATURES_2,
            p_next: ptr::null(),
            features: PhysicalDeviceFeatures::default(),
        }
    }
}
