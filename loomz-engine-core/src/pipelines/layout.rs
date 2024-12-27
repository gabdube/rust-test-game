/// A binding in a PipelineLayoutSet
#[derive(Copy, Clone)]
pub struct PipelineLayoutSetBinding {
    pub descriptor_type: vk::DescriptorType,
    pub stage_flags: vk::ShaderStageFlags,
}

impl PipelineLayoutSetBinding {
    pub fn build_descriptor_set_layout<const C: usize>(device: &vk::wrapper::Device, bindings_info: &[Self; C]) -> Result<vk::DescriptorSetLayout, vk::VkResult> {
        let mut bindings: [vk::DescriptorSetLayoutBinding; C] = [vk::DescriptorSetLayoutBinding::default(); C];
        for i in 0..C {
            bindings[i] = vk::DescriptorSetLayoutBinding {
                binding: i as u32,
                descriptor_type: bindings_info[i].descriptor_type,
                descriptor_count: 1,
                stage_flags: bindings_info[i].stage_flags,
                p_immutable_samplers: ::std::ptr::null(),
            };
        }
        
        let create_info = vk::DescriptorSetLayoutCreateInfo {
            binding_count: C as u32,
            p_bindings: bindings.as_ptr(),
            ..Default::default()
        };
        device.create_descriptor_set_layout(&create_info)
    }

}
