use crate::{
    vk100::*,
    wrapper,
    error::Error,
};
use libloading::Library;
use std::ptr;


#[cfg(windows)]
const LIB_PATH: &str = "vulkan-1.dll";

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
const LIB_PATH: &str = "libvulkan.so.1";                                                                                                                      

#[cfg(target_os = "android")]
const LIB_PATH: &str = "libvulkan.so";

#[cfg(any(target_os = "macos", target_os = "ios"))]
const LIB_PATH: &str = "libvulkan.dylib";

pub struct Entry {
    _lib: Library,
    get_instance_proc_addr: PFN_vkGetInstanceProcAddr,
    entry_fp_1_0: EntryFnV1_0
}

impl Entry {

    pub fn open() -> Result<Self, Error> {
        let _lib = unsafe { Library::new(LIB_PATH) }
            .map_err(Error::LibraryLoading)?;

        let get_instance_proc_addr: PFN_vkGetInstanceProcAddr = unsafe {
            _lib.get(b"vkGetInstanceProcAddr\0")
                .map(|s| *s)
                .map_err(Error::LibraryLoading)?
        };

        let entry_fp_1_0 = EntryFnV1_0::load(|name| unsafe {
            get_instance_proc_addr(Instance::null(), name.to_bytes_with_nul().as_ptr())
        });

        let entry = Entry {
            _lib,
            get_instance_proc_addr,
            entry_fp_1_0
        };

        Ok(entry)
    }

    pub unsafe fn get_instance_proc_addr(&self, instance: Instance, name: *const u8) -> PFN_vkVoidFunction {
        (self.get_instance_proc_addr)(instance, name)
    }

    pub fn enumerate_instance_extension_properties(&self) -> Result<Vec<ExtensionProperties>, VkResult> {
        unsafe {
            let mut extensions_count = 0;
            
            (self.entry_fp_1_0.enumerate_instance_extension_properties)(ptr::null(), &mut extensions_count, ptr::null_mut()).as_result()?;

            let mut extensions = Vec::with_capacity(extensions_count as usize);
            for _ in 0..extensions_count {
                extensions.push(std::mem::zeroed());
            }

            (self.entry_fp_1_0.enumerate_instance_extension_properties)(ptr::null(), &mut extensions_count, extensions.as_mut_ptr()).as_result()?;

            Ok(extensions)
        }
    }

    pub fn enumerate_instance_layer_properties(&self) -> Result<Vec<LayerProperties>, VkResult> {
        unsafe {
            let mut layer_count = 0;
            
            (self.entry_fp_1_0.enumerate_instance_layer_properties)(&mut layer_count, ptr::null_mut()).as_result()?;

            let mut layers = Vec::with_capacity(layer_count as usize);
            for _ in 0..layer_count {
                layers.push(std::mem::zeroed());
            }

            (self.entry_fp_1_0.enumerate_instance_layer_properties)(&mut layer_count, layers.as_mut_ptr()).as_result()?;

            Ok(layers)
        }
    }

    pub fn create_instance(&self, create_info: &InstanceCreateInfo) -> Result<wrapper::Instance, VkResult> {
        let instance_handle = unsafe {
            let mut handle = Instance::null();
            let result = (self.entry_fp_1_0.create_instance)(create_info, ptr::null(), &mut handle);
            result.as_result()
                .map(|_| handle)?
        };
        
        Ok(wrapper::Instance::load(instance_handle, &self.get_instance_proc_addr))
    }

}
