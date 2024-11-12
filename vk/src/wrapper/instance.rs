use crate::*;
use crate::vk100::Instance as InstanceHandle;
use std::ptr;


#[allow(dead_code)]
pub struct Instance {
    pub handle: InstanceHandle,
    instance_fp_1_0: InstanceFnV1_0,
    instance_fp_1_1: InstanceFnV1_1,
}

impl Instance {

    pub fn load(handle: InstanceHandle, loader: &PFN_vkGetInstanceProcAddr) -> Instance {
        let instance_fp_1_0 = InstanceFnV1_0::load(|name| unsafe {
            loader(handle, name.to_bytes_with_nul().as_ptr())
        });

        let instance_fp_1_1 = InstanceFnV1_1::load(|name| unsafe {
            loader(handle, name.to_bytes_with_nul().as_ptr())
        });
        
        Instance {
            handle,
            instance_fp_1_0,
            instance_fp_1_1,
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            (self.instance_fp_1_0.destroy_instance)(self.handle, ptr::null());
            self.handle = InstanceHandle::null();
        }
    }

    pub fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, VkResult> {
        unsafe {
            let mut device_count = 0;

            (self.instance_fp_1_0.enumerate_physical_devices)(self.handle, &mut device_count, ptr::null_mut())
                .as_result()?;

            let mut physical_devices = vec![PhysicalDevice::null(); device_count as usize];

            (self.instance_fp_1_0.enumerate_physical_devices)(self.handle, &mut device_count, physical_devices.as_mut_ptr())
                .as_result()
                .map(|_| physical_devices)
        }
    }

    pub fn enumerate_device_extension_properties(&self, pdevice: PhysicalDevice) -> Result<Vec<ExtensionProperties>, VkResult> {
        unsafe {
            let mut extensions_count = 0;

            (self.instance_fp_1_0.enumerate_device_extension_properties)(pdevice, ptr::null(), &mut extensions_count, ptr::null_mut())
                .as_result()?;

            let mut extensions = Vec::with_capacity(extensions_count as usize);
            for _ in 0..extensions_count {
                extensions.push(std::mem::zeroed());
            }

            (self.instance_fp_1_0.enumerate_device_extension_properties)(pdevice, ptr::null(), &mut extensions_count, extensions.as_mut_ptr())
                .as_result()
                .map(|_| extensions)
        }
    
    }

    pub fn get_physical_device_queue_family_properties(&self, pdevice: PhysicalDevice) -> Vec<QueueFamilyProperties> {
        unsafe {
            let mut properties_count = 0;
            
            (self.instance_fp_1_0.get_physical_device_queue_family_properties)(pdevice, &mut properties_count, ptr::null_mut());

            let mut properties = Vec::with_capacity(properties_count as usize);
            for _ in 0..properties_count {
                properties.push(QueueFamilyProperties::default());
            }

            (self.instance_fp_1_0.get_physical_device_queue_family_properties)(pdevice, &mut properties_count, properties.as_mut_ptr());
        
            properties
        }
    }

    pub fn get_physical_device_memory_properties(&self, pdevice: PhysicalDevice) -> PhysicalDeviceMemoryProperties {
        unsafe {
            let mut mem = Default::default();
            (self.instance_fp_1_0.get_physical_device_memory_properties)(pdevice, &mut mem);
            mem
        }
    }

    pub fn get_physical_device_format_properties(&self, pdevice: PhysicalDevice, format: Format) -> FormatProperties {
        unsafe {
            let mut props = FormatProperties::default();
            (self.instance_fp_1_0.get_physical_device_format_properties)(pdevice, format, &mut props);
            props
        }
    }

    pub fn get_physical_device_properties(&self, pdevice: PhysicalDevice) -> PhysicalDeviceProperties {
        unsafe {
            let mut prop = Default::default();
            (self.instance_fp_1_0.get_physical_device_properties)(pdevice, &mut prop);
            prop
        }
    }

    pub fn get_physical_device_features_2(&self, pdevice: PhysicalDevice, features: &mut PhysicalDeviceFeatures2) {
        unsafe {
            (self.instance_fp_1_1.get_physical_device_features_2)(pdevice, features);
        }
    }   

    pub fn create_device(&self, pdevice: PhysicalDevice, create_info: &DeviceCreateInfo) -> Result<wrapper::Device, VkResult> {
        unsafe {
            let mut device_handle = Device::null();
            (self.instance_fp_1_0.create_device)(pdevice, create_info, ptr::null(), &mut device_handle)
                .as_result()?;

            Ok(wrapper::Device::load(self, create_info, pdevice, device_handle))
        }
    }

    pub unsafe fn get_device_proc_addr(&self, device: Device, name: *const u8) -> PFN_vkVoidFunction {
        unsafe {
            (self.instance_fp_1_0.get_device_proc_addr)(device, name)
        }
    }

}

