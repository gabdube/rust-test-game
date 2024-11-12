use crate::*;
use std::{
    ffi::CStr,
    mem::transmute,
};

//
// Instance FN
//

pub struct InstanceFnV1_1 {
    pub get_physical_device_features_2: PFN_vkGetPhysicalDeviceFeatures2,
}

impl InstanceFnV1_1 {

    pub fn load<F>(cb: F) -> InstanceFnV1_1 
    where
        F: Fn(&CStr) -> PFN_vkVoidFunction
    {
        unsafe {
            let cstr = CStr::from_bytes_with_nul_unchecked;
            InstanceFnV1_1 {
                get_physical_device_features_2: transmute(cb(cstr(b"vkGetPhysicalDeviceFeatures2\0"))),
            }
        }
    }
}

//
// Functions def
//

pub type PFN_vkGetPhysicalDeviceFeatures2 = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    features: &PhysicalDeviceFeatures2,
);
