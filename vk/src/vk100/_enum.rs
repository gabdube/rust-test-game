#![allow(dead_code)]

use std::fmt;

vk_enum!(InstanceCreateFlags);
vk_enum!(DeviceQueueCreateFlags);
vk_enum!(DeviceCreateFlags);
vk_enum!(ImageViewCreateFlags);
vk_enum!(SemaphoreCreateFlags);
vk_enum!(SubpassDescriptionFlags);
vk_enum!(FramebufferCreateFlags);
vk_enum!(MemoryMapFlags);
vk_enum!(ShaderModuleCreateFlags);
vk_enum!(PipelineCacheCreateFlags);
vk_enum!(PipelineShaderStageCreateFlags);
vk_enum!(PipelineVertexInputStateCreateFlags);
vk_enum!(PipelineInputAssemblyStateCreateFlags);
vk_enum!(PipelineTessellationStateCreateFlags);
vk_enum!(PipelineViewportStateCreateFlags);
vk_enum!(PipelineRasterizationStateCreateFlags);
vk_enum!(PipelineMultisampleStateCreateFlags);
vk_enum!(PipelineDepthStencilStateCreateFlags);
vk_enum!(PipelineColorBlendStateCreateFlags);
vk_enum!(PipelineDynamicStateCreateFlags);
vk_enum!(PipelineLayoutCreateFlags);
vk_enum!(SamplerCreateFlags);
vk_enum!(BufferViewCreateFlags);


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
#[must_use]
pub struct VkResult(pub i32);
impl VkResult {
    pub const SUCCESS: Self = Self(0);
    pub const NOT_READY: Self = Self(1);
    pub const TIMEOUT: Self = Self(2);
    pub const EVENT_SET: Self = Self(3);
    pub const EVENT_RESET: Self = Self(4);
    pub const INCOMPLETE: Self = Self(5);
    pub const ERROR_OUT_OF_HOST_MEMORY: Self = Self(-1);
    pub const ERROR_OUT_OF_DEVICE_MEMORY: Self = Self(-2);
    pub const ERROR_INITIALIZATION_FAILED: Self = Self(-3);
    pub const ERROR_DEVICE_LOST: Self = Self(-4);
    pub const ERROR_MEMORY_MAP_FAILED: Self = Self(-5);
    pub const ERROR_LAYER_NOT_PRESENT: Self = Self(-6);
    pub const ERROR_EXTENSION_NOT_PRESENT: Self = Self(-7);
    pub const ERROR_FEATURE_NOT_PRESENT: Self = Self(-8);
    pub const ERROR_INCOMPATIBLE_DRIVER: Self = Self(-9);
    pub const ERROR_TOO_MANY_OBJECTS: Self = Self(-10);
    pub const ERROR_FORMAT_NOT_SUPPORTED: Self = Self(-11);
    pub const ERROR_FRAGMENTED_POOL: Self = Self(-12);
    pub const ERROR_UNKNOWN: Self = Self(-13);

    #[inline(always)]
    pub fn as_result(self) -> Result<(), VkResult> {
        match self.0 == 0 {
            true => Ok(()),
            false => Err(self)
        }
    }

}

impl fmt::Display for VkResult {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let name = match *self {
            VkResult::SUCCESS => Some("Command completed successfully"),
            VkResult::NOT_READY => Some("A fence or query has not yet completed"),
            VkResult::TIMEOUT => Some("A wait operation has not completed in the specified time"),
            VkResult::EVENT_SET => Some("An event is signaled"),
            VkResult::EVENT_RESET => Some("An event is unsignaled"),
            VkResult::INCOMPLETE => Some("A return array was too small for the result"),
            VkResult::ERROR_OUT_OF_HOST_MEMORY => Some("A host memory allocation has failed"),
            VkResult::ERROR_OUT_OF_DEVICE_MEMORY => Some("A device memory allocation has failed"),
            VkResult::ERROR_INITIALIZATION_FAILED => Some("Initialization of a object has failed"),
            VkResult::ERROR_DEVICE_LOST => {
                Some("The logical device has been lost. See <<devsandqueues-lost-device>>")
            }
            VkResult::ERROR_MEMORY_MAP_FAILED => Some("Mapping of a memory object has failed"),
            VkResult::ERROR_LAYER_NOT_PRESENT => Some("Layer specified does not exist"),
            VkResult::ERROR_EXTENSION_NOT_PRESENT => Some("Extension specified does not exist"),
            VkResult::ERROR_FEATURE_NOT_PRESENT => {
                Some("Requested feature is not available on this device")
            }
            VkResult::ERROR_INCOMPATIBLE_DRIVER => Some("Unable to find a Vulkan driver"),
            VkResult::ERROR_TOO_MANY_OBJECTS => {
                Some("Too many objects of the type have already been created")
            }
            VkResult::ERROR_FORMAT_NOT_SUPPORTED => {
                Some("Requested format is not supported on this device")
            }
            VkResult::ERROR_FRAGMENTED_POOL => Some(
                "A requested pool allocation has failed due to fragmentation of the pool's memory",
            ),
            VkResult::ERROR_UNKNOWN => {
                Some("An unknown error has occurred, due to an implementation or application bug")
            }
            _ => None,
        };
        if let Some(x) = name {
            fmt.write_str(x)
        } else {
            <Self as fmt::Debug>::fmt(self, fmt)
        }
    }
}

vk_enum!(StructureType);
impl StructureType {
    pub const APPLICATION_INFO: Self = Self(0);
    pub const INSTANCE_CREATE_INFO: Self = Self(1);
    pub const DEVICE_QUEUE_CREATE_INFO: Self = Self(2);
    pub const DEVICE_CREATE_INFO: Self = Self(3);
    pub const SUBMIT_INFO: Self = Self(4);
    pub const MEMORY_ALLOCATE_INFO: Self = Self(5);
    pub const MAPPED_MEMORY_RANGE: Self = Self(6);
    pub const BIND_SPARSE_INFO: Self = Self(7);
    pub const FENCE_CREATE_INFO: Self = Self(8);
    pub const SEMAPHORE_CREATE_INFO: Self = Self(9);
    pub const EVENT_CREATE_INFO: Self = Self(10);
    pub const QUERY_POOL_CREATE_INFO: Self = Self(11);
    pub const BUFFER_CREATE_INFO: Self = Self(12);
    pub const BUFFER_VIEW_CREATE_INFO: Self = Self(13);
    pub const IMAGE_CREATE_INFO: Self = Self(14);
    pub const IMAGE_VIEW_CREATE_INFO: Self = Self(15);
    pub const SHADER_MODULE_CREATE_INFO: Self = Self(16);
    pub const PIPELINE_CACHE_CREATE_INFO: Self = Self(17);
    pub const PIPELINE_SHADER_STAGE_CREATE_INFO: Self = Self(18);
    pub const PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO: Self = Self(19);
    pub const PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO: Self = Self(20);
    pub const PIPELINE_TESSELLATION_STATE_CREATE_INFO: Self = Self(21);
    pub const PIPELINE_VIEWPORT_STATE_CREATE_INFO: Self = Self(22);
    pub const PIPELINE_RASTERIZATION_STATE_CREATE_INFO: Self = Self(23);
    pub const PIPELINE_MULTISAMPLE_STATE_CREATE_INFO: Self = Self(24);
    pub const PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO: Self = Self(25);
    pub const PIPELINE_COLOR_BLEND_STATE_CREATE_INFO: Self = Self(26);
    pub const PIPELINE_DYNAMIC_STATE_CREATE_INFO: Self = Self(27);
    pub const GRAPHICS_PIPELINE_CREATE_INFO: Self = Self(28);
    pub const COMPUTE_PIPELINE_CREATE_INFO: Self = Self(29);
    pub const PIPELINE_LAYOUT_CREATE_INFO: Self = Self(30);
    pub const SAMPLER_CREATE_INFO: Self = Self(31);
    pub const DESCRIPTOR_SET_LAYOUT_CREATE_INFO: Self = Self(32);
    pub const DESCRIPTOR_POOL_CREATE_INFO: Self = Self(33);
    pub const DESCRIPTOR_SET_ALLOCATE_INFO: Self = Self(34);
    pub const WRITE_DESCRIPTOR_SET: Self = Self(35);
    pub const COPY_DESCRIPTOR_SET: Self = Self(36);
    pub const FRAMEBUFFER_CREATE_INFO: Self = Self(37);
    pub const RENDER_PASS_CREATE_INFO: Self = Self(38);
    pub const COMMAND_POOL_CREATE_INFO: Self = Self(39);
    pub const COMMAND_BUFFER_ALLOCATE_INFO: Self = Self(40);
    pub const COMMAND_BUFFER_INHERITANCE_INFO: Self = Self(41);
    pub const COMMAND_BUFFER_BEGIN_INFO: Self = Self(42);
    pub const RENDER_PASS_BEGIN_INFO: Self = Self(43);
    pub const BUFFER_MEMORY_BARRIER: Self = Self(44);
    pub const IMAGE_MEMORY_BARRIER: Self = Self(45);
    pub const MEMORY_BARRIER: Self = Self(46);
    pub const LOADER_INSTANCE_CREATE_INFO: Self = Self(47);
    pub const LOADER_DEVICE_CREATE_INFO: Self = Self(48);
    pub const PHYSICAL_DEVICE_VULKAN_1_1_FEATURES: Self = Self(49);
    pub const PHYSICAL_DEVICE_VULKAN_1_1_PROPERTIES: Self = Self(50);
    pub const PHYSICAL_DEVICE_VULKAN_1_2_FEATURES: Self = Self(51);
    pub const PHYSICAL_DEVICE_VULKAN_1_2_PROPERTIES: Self = Self(52);
}

vk_enum!(QueueFlags);
vk_bitflags!(QueueFlags);
impl QueueFlags {
    pub const GRAPHICS: Self = Self(0b1);
    pub const COMPUTE: Self = Self(0b10);
    pub const TRANSFER: Self = Self(0b100);
    pub const SPARSE_BINDING: Self = Self(0b1000);
}

vk_enum!(PhysicalDeviceType);
impl PhysicalDeviceType {
    pub const OTHER: Self = Self(0);
    pub const INTEGRATED_GPU: Self = Self(1);
    pub const DISCRETE_GPU: Self = Self(2);
    pub const VIRTUAL_GPU: Self = Self(3);
    pub const CPU: Self = Self(4);
}

vk_enum!(SampleCountFlags);
vk_bitflags!(SampleCountFlags);
impl SampleCountFlags {
    pub const TYPE_1: Self = Self(0b1);
    pub const TYPE_2: Self = Self(0b10);
    pub const TYPE_4: Self = Self(0b100);
    pub const TYPE_8: Self = Self(0b1000);
    pub const TYPE_16: Self = Self(0b1_0000);
    pub const TYPE_32: Self = Self(0b10_0000);
    pub const TYPE_64: Self = Self(0b100_0000);
}

vk_enum!(MemoryPropertyFlags);
vk_bitflags!(MemoryPropertyFlags);
impl MemoryPropertyFlags {
    pub const DEVICE_LOCAL: Self = Self(0b1);
    pub const HOST_VISIBLE: Self = Self(0b10);
    pub const HOST_COHERENT: Self = Self(0b100);
    pub const HOST_CACHED: Self = Self(0b1000);
    pub const LAZILY_ALLOCATED: Self = Self(0b1_0000);
}

vk_enum!(MemoryHeapFlags);
vk_bitflags!(MemoryHeapFlags);
impl MemoryHeapFlags {
    pub const DEVICE_LOCAL: Self = Self(0b1);
}

vk_enum!(ImageUsageFlags);
vk_bitflags!(ImageUsageFlags);
impl ImageUsageFlags {
    pub const TRANSFER_SRC: Self = Self(0b1);
    pub const TRANSFER_DST: Self = Self(0b10);
    pub const SAMPLED: Self = Self(0b100);
    pub const STORAGE: Self = Self(0b1000);
    pub const COLOR_ATTACHMENT: Self = Self(0b1_0000);
    pub const DEPTH_STENCIL_ATTACHMENT: Self = Self(0b10_0000);
    pub const TRANSIENT_ATTACHMENT: Self = Self(0b100_0000);
    pub const INPUT_ATTACHMENT: Self = Self(0b1000_0000);
}

vk_enum!(Format);
impl Format {
    pub const UNDEFINED: Self = Self(0);
    pub const R4G4_UNORM_PACK8: Self = Self(1);
    pub const R4G4B4A4_UNORM_PACK16: Self = Self(2);
    pub const B4G4R4A4_UNORM_PACK16: Self = Self(3);
    pub const R5G6B5_UNORM_PACK16: Self = Self(4);
    pub const B5G6R5_UNORM_PACK16: Self = Self(5);
    pub const R5G5B5A1_UNORM_PACK16: Self = Self(6);
    pub const B5G5R5A1_UNORM_PACK16: Self = Self(7);
    pub const A1R5G5B5_UNORM_PACK16: Self = Self(8);
    pub const R8_UNORM: Self = Self(9);
    pub const R8_SNORM: Self = Self(10);
    pub const R8_USCALED: Self = Self(11);
    pub const R8_SSCALED: Self = Self(12);
    pub const R8_UINT: Self = Self(13);
    pub const R8_SINT: Self = Self(14);
    pub const R8_SRGB: Self = Self(15);
    pub const R8G8_UNORM: Self = Self(16);
    pub const R8G8_SNORM: Self = Self(17);
    pub const R8G8_USCALED: Self = Self(18);
    pub const R8G8_SSCALED: Self = Self(19);
    pub const R8G8_UINT: Self = Self(20);
    pub const R8G8_SINT: Self = Self(21);
    pub const R8G8_SRGB: Self = Self(22);
    pub const R8G8B8_UNORM: Self = Self(23);
    pub const R8G8B8_SNORM: Self = Self(24);
    pub const R8G8B8_USCALED: Self = Self(25);
    pub const R8G8B8_SSCALED: Self = Self(26);
    pub const R8G8B8_UINT: Self = Self(27);
    pub const R8G8B8_SINT: Self = Self(28);
    pub const R8G8B8_SRGB: Self = Self(29);
    pub const B8G8R8_UNORM: Self = Self(30);
    pub const B8G8R8_SNORM: Self = Self(31);
    pub const B8G8R8_USCALED: Self = Self(32);
    pub const B8G8R8_SSCALED: Self = Self(33);
    pub const B8G8R8_UINT: Self = Self(34);
    pub const B8G8R8_SINT: Self = Self(35);
    pub const B8G8R8_SRGB: Self = Self(36);
    pub const R8G8B8A8_UNORM: Self = Self(37);
    pub const R8G8B8A8_SNORM: Self = Self(38);
    pub const R8G8B8A8_USCALED: Self = Self(39);
    pub const R8G8B8A8_SSCALED: Self = Self(40);
    pub const R8G8B8A8_UINT: Self = Self(41);
    pub const R8G8B8A8_SINT: Self = Self(42);
    pub const R8G8B8A8_SRGB: Self = Self(43);
    pub const B8G8R8A8_UNORM: Self = Self(44);
    pub const B8G8R8A8_SNORM: Self = Self(45);
    pub const B8G8R8A8_USCALED: Self = Self(46);
    pub const B8G8R8A8_SSCALED: Self = Self(47);
    pub const B8G8R8A8_UINT: Self = Self(48);
    pub const B8G8R8A8_SINT: Self = Self(49);
    pub const B8G8R8A8_SRGB: Self = Self(50);
    pub const A8B8G8R8_UNORM_PACK32: Self = Self(51);
    pub const A8B8G8R8_SNORM_PACK32: Self = Self(52);
    pub const A8B8G8R8_USCALED_PACK32: Self = Self(53);
    pub const A8B8G8R8_SSCALED_PACK32: Self = Self(54);
    pub const A8B8G8R8_UINT_PACK32: Self = Self(55);
    pub const A8B8G8R8_SINT_PACK32: Self = Self(56);
    pub const A8B8G8R8_SRGB_PACK32: Self = Self(57);
    pub const A2R10G10B10_UNORM_PACK32: Self = Self(58);
    pub const A2R10G10B10_SNORM_PACK32: Self = Self(59);
    pub const A2R10G10B10_USCALED_PACK32: Self = Self(60);
    pub const A2R10G10B10_SSCALED_PACK32: Self = Self(61);
    pub const A2R10G10B10_UINT_PACK32: Self = Self(62);
    pub const A2R10G10B10_SINT_PACK32: Self = Self(63);
    pub const A2B10G10R10_UNORM_PACK32: Self = Self(64);
    pub const A2B10G10R10_SNORM_PACK32: Self = Self(65);
    pub const A2B10G10R10_USCALED_PACK32: Self = Self(66);
    pub const A2B10G10R10_SSCALED_PACK32: Self = Self(67);
    pub const A2B10G10R10_UINT_PACK32: Self = Self(68);
    pub const A2B10G10R10_SINT_PACK32: Self = Self(69);
    pub const R16_UNORM: Self = Self(70);
    pub const R16_SNORM: Self = Self(71);
    pub const R16_USCALED: Self = Self(72);
    pub const R16_SSCALED: Self = Self(73);
    pub const R16_UINT: Self = Self(74);
    pub const R16_SINT: Self = Self(75);
    pub const R16_SFLOAT: Self = Self(76);
    pub const R16G16_UNORM: Self = Self(77);
    pub const R16G16_SNORM: Self = Self(78);
    pub const R16G16_USCALED: Self = Self(79);
    pub const R16G16_SSCALED: Self = Self(80);
    pub const R16G16_UINT: Self = Self(81);
    pub const R16G16_SINT: Self = Self(82);
    pub const R16G16_SFLOAT: Self = Self(83);
    pub const R16G16B16_UNORM: Self = Self(84);
    pub const R16G16B16_SNORM: Self = Self(85);
    pub const R16G16B16_USCALED: Self = Self(86);
    pub const R16G16B16_SSCALED: Self = Self(87);
    pub const R16G16B16_UINT: Self = Self(88);
    pub const R16G16B16_SINT: Self = Self(89);
    pub const R16G16B16_SFLOAT: Self = Self(90);
    pub const R16G16B16A16_UNORM: Self = Self(91);
    pub const R16G16B16A16_SNORM: Self = Self(92);
    pub const R16G16B16A16_USCALED: Self = Self(93);
    pub const R16G16B16A16_SSCALED: Self = Self(94);
    pub const R16G16B16A16_UINT: Self = Self(95);
    pub const R16G16B16A16_SINT: Self = Self(96);
    pub const R16G16B16A16_SFLOAT: Self = Self(97);
    pub const R32_UINT: Self = Self(98);
    pub const R32_SINT: Self = Self(99);
    pub const R32_SFLOAT: Self = Self(100);
    pub const R32G32_UINT: Self = Self(101);
    pub const R32G32_SINT: Self = Self(102);
    pub const R32G32_SFLOAT: Self = Self(103);
    pub const R32G32B32_UINT: Self = Self(104);
    pub const R32G32B32_SINT: Self = Self(105);
    pub const R32G32B32_SFLOAT: Self = Self(106);
    pub const R32G32B32A32_UINT: Self = Self(107);
    pub const R32G32B32A32_SINT: Self = Self(108);
    pub const R32G32B32A32_SFLOAT: Self = Self(109);
    pub const R64_UINT: Self = Self(110);
    pub const R64_SINT: Self = Self(111);
    pub const R64_SFLOAT: Self = Self(112);
    pub const R64G64_UINT: Self = Self(113);
    pub const R64G64_SINT: Self = Self(114);
    pub const R64G64_SFLOAT: Self = Self(115);
    pub const R64G64B64_UINT: Self = Self(116);
    pub const R64G64B64_SINT: Self = Self(117);
    pub const R64G64B64_SFLOAT: Self = Self(118);
    pub const R64G64B64A64_UINT: Self = Self(119);
    pub const R64G64B64A64_SINT: Self = Self(120);
    pub const R64G64B64A64_SFLOAT: Self = Self(121);
    pub const B10G11R11_UFLOAT_PACK32: Self = Self(122);
    pub const E5B9G9R9_UFLOAT_PACK32: Self = Self(123);
    pub const D16_UNORM: Self = Self(124);
    pub const X8_D24_UNORM_PACK32: Self = Self(125);
    pub const D32_SFLOAT: Self = Self(126);
    pub const S8_UINT: Self = Self(127);
    pub const D16_UNORM_S8_UINT: Self = Self(128);
    pub const D24_UNORM_S8_UINT: Self = Self(129);
    pub const D32_SFLOAT_S8_UINT: Self = Self(130);
    pub const BC1_RGB_UNORM_BLOCK: Self = Self(131);
    pub const BC1_RGB_SRGB_BLOCK: Self = Self(132);
    pub const BC1_RGBA_UNORM_BLOCK: Self = Self(133);
    pub const BC1_RGBA_SRGB_BLOCK: Self = Self(134);
    pub const BC2_UNORM_BLOCK: Self = Self(135);
    pub const BC2_SRGB_BLOCK: Self = Self(136);
    pub const BC3_UNORM_BLOCK: Self = Self(137);
    pub const BC3_SRGB_BLOCK: Self = Self(138);
    pub const BC4_UNORM_BLOCK: Self = Self(139);
    pub const BC4_SNORM_BLOCK: Self = Self(140);
    pub const BC5_UNORM_BLOCK: Self = Self(141);
    pub const BC5_SNORM_BLOCK: Self = Self(142);
    pub const BC6H_UFLOAT_BLOCK: Self = Self(143);
    pub const BC6H_SFLOAT_BLOCK: Self = Self(144);
    pub const BC7_UNORM_BLOCK: Self = Self(145);
    pub const BC7_SRGB_BLOCK: Self = Self(146);
    pub const ETC2_R8G8B8_UNORM_BLOCK: Self = Self(147);
    pub const ETC2_R8G8B8_SRGB_BLOCK: Self = Self(148);
    pub const ETC2_R8G8B8A1_UNORM_BLOCK: Self = Self(149);
    pub const ETC2_R8G8B8A1_SRGB_BLOCK: Self = Self(150);
    pub const ETC2_R8G8B8A8_UNORM_BLOCK: Self = Self(151);
    pub const ETC2_R8G8B8A8_SRGB_BLOCK: Self = Self(152);
    pub const EAC_R11_UNORM_BLOCK: Self = Self(153);
    pub const EAC_R11_SNORM_BLOCK: Self = Self(154);
    pub const EAC_R11G11_UNORM_BLOCK: Self = Self(155);
    pub const EAC_R11G11_SNORM_BLOCK: Self = Self(156);
    pub const ASTC_4X4_UNORM_BLOCK: Self = Self(157);
    pub const ASTC_4X4_SRGB_BLOCK: Self = Self(158);
    pub const ASTC_5X4_UNORM_BLOCK: Self = Self(159);
    pub const ASTC_5X4_SRGB_BLOCK: Self = Self(160);
    pub const ASTC_5X5_UNORM_BLOCK: Self = Self(161);
    pub const ASTC_5X5_SRGB_BLOCK: Self = Self(162);
    pub const ASTC_6X5_UNORM_BLOCK: Self = Self(163);
    pub const ASTC_6X5_SRGB_BLOCK: Self = Self(164);
    pub const ASTC_6X6_UNORM_BLOCK: Self = Self(165);
    pub const ASTC_6X6_SRGB_BLOCK: Self = Self(166);
    pub const ASTC_8X5_UNORM_BLOCK: Self = Self(167);
    pub const ASTC_8X5_SRGB_BLOCK: Self = Self(168);
    pub const ASTC_8X6_UNORM_BLOCK: Self = Self(169);
    pub const ASTC_8X6_SRGB_BLOCK: Self = Self(170);
    pub const ASTC_8X8_UNORM_BLOCK: Self = Self(171);
    pub const ASTC_8X8_SRGB_BLOCK: Self = Self(172);
    pub const ASTC_10X5_UNORM_BLOCK: Self = Self(173);
    pub const ASTC_10X5_SRGB_BLOCK: Self = Self(174);
    pub const ASTC_10X6_UNORM_BLOCK: Self = Self(175);
    pub const ASTC_10X6_SRGB_BLOCK: Self = Self(176);
    pub const ASTC_10X8_UNORM_BLOCK: Self = Self(177);
    pub const ASTC_10X8_SRGB_BLOCK: Self = Self(178);
    pub const ASTC_10X10_UNORM_BLOCK: Self = Self(179);
    pub const ASTC_10X10_SRGB_BLOCK: Self = Self(180);
    pub const ASTC_12X10_UNORM_BLOCK: Self = Self(181);
    pub const ASTC_12X10_SRGB_BLOCK: Self = Self(182);
    pub const ASTC_12X12_UNORM_BLOCK: Self = Self(183);
    pub const ASTC_12X12_SRGB_BLOCK: Self = Self(184);
}

vk_enum!(SharingMode);
impl SharingMode {
    pub const EXCLUSIVE: Self = Self(0);
    pub const CONCURRENT: Self = Self(1);
}

vk_enum!(ImageViewType);
impl ImageViewType {
    pub const TYPE_1D: Self = Self(0);
    pub const TYPE_2D: Self = Self(1);
    pub const TYPE_3D: Self = Self(2);
    pub const CUBE: Self = Self(3);
    pub const TYPE_1D_ARRAY: Self = Self(4);
    pub const TYPE_2D_ARRAY: Self = Self(5);
    pub const CUBE_ARRAY: Self = Self(6);
}

vk_enum!(ComponentSwizzle);
impl ComponentSwizzle {
    pub const IDENTITY: Self = Self(0);
    pub const ZERO: Self = Self(1);
    pub const ONE: Self = Self(2);
    pub const R: Self = Self(3);
    pub const G: Self = Self(4);
    pub const B: Self = Self(5);
    pub const A: Self = Self(6);
}

vk_enum!(ImageAspectFlags);
vk_bitflags!(ImageAspectFlags);
impl ImageAspectFlags {
    pub const COLOR: Self = Self(0b1);
    pub const DEPTH: Self = Self(0b10);
    pub const STENCIL: Self = Self(0b100);
    pub const METADATA: Self = Self(0b1000);
}

vk_enum!(FenceCreateFlags);
vk_bitflags!(FenceCreateFlags);
impl FenceCreateFlags {
    pub const SIGNALED: Self = Self(0b1);
}

vk_enum!(CommandPoolCreateFlags);
vk_bitflags!(CommandPoolCreateFlags);
impl CommandPoolCreateFlags {
    pub const TRANSIENT: Self = Self(0b1);
    pub const RESET_COMMAND_BUFFER: Self = Self(0b10);
}

vk_enum!(CommandBufferLevel);
impl CommandBufferLevel {
    pub const PRIMARY: Self = Self(0);
    pub const SECONDARY: Self = Self(1);
}

vk_enum!(CommandBufferUsageFlags);
vk_bitflags!(CommandBufferUsageFlags);
impl CommandBufferUsageFlags {
    pub const ONE_TIME_SUBMIT: Self = Self(0b1);
    pub const RENDER_PASS_CONTINUE: Self = Self(0b10);
    pub const SIMULTANEOUS_USE: Self = Self(0b100);
}

vk_enum!(QueryControlFlags);
vk_bitflags!(QueryControlFlags);
impl QueryControlFlags {
    pub const PRECISE: Self = Self(0b1);
}

vk_enum!(QueryPipelineStatisticFlags);
vk_bitflags!(QueryPipelineStatisticFlags);
impl QueryPipelineStatisticFlags  {
    pub const INPUT_ASSEMBLY_VERTICES: Self = Self(0b1);
    pub const INPUT_ASSEMBLY_PRIMITIVES: Self = Self(0b10);
    pub const VERTEX_SHADER_INVOCATIONS: Self = Self(0b100);
    pub const GEOMETRY_SHADER_INVOCATIONS: Self = Self(0b1000);
    pub const GEOMETRY_SHADER_PRIMITIVES: Self = Self(0b1_0000);
    pub const CLIPPING_INVOCATIONS: Self = Self(0b10_0000);
    pub const CLIPPING_PRIMITIVES: Self = Self(0b100_0000);
    pub const FRAGMENT_SHADER_INVOCATIONS: Self = Self(0b1000_0000);
    pub const TESSELLATION_CONTROL_SHADER_PATCHES: Self = Self(0b1_0000_0000);
    pub const TESSELLATION_EVALUATION_SHADER_INVOCATIONS: Self = Self(0b10_0000_0000);
    pub const COMPUTE_SHADER_INVOCATIONS: Self = Self(0b100_0000_0000);
}

vk_enum!(PipelineStageFlags);
vk_bitflags!(PipelineStageFlags);
impl PipelineStageFlags {
    pub const TOP_OF_PIPE: Self = Self(0b1);
    pub const DRAW_INDIRECT: Self = Self(0b10);
    pub const VERTEX_INPUT: Self = Self(0b100);
    pub const VERTEX_SHADER: Self = Self(0b1000);
    pub const TESSELLATION_CONTROL_SHADER: Self = Self(0b1_0000);
    pub const TESSELLATION_EVALUATION_SHADER: Self = Self(0b10_0000);
    pub const GEOMETRY_SHADER: Self = Self(0b100_0000);
    pub const FRAGMENT_SHADER: Self = Self(0b1000_0000);
    pub const EARLY_FRAGMENT_TESTS: Self = Self(0b1_0000_0000);
    pub const LATE_FRAGMENT_TESTS: Self = Self(0b10_0000_0000);
    pub const COLOR_ATTACHMENT_OUTPUT: Self = Self(0b100_0000_0000);
    pub const COMPUTE_SHADER: Self = Self(0b1000_0000_0000);
    pub const TRANSFER: Self = Self(0b1_0000_0000_0000);
    pub const BOTTOM_OF_PIPE: Self = Self(0b10_0000_0000_0000);
    pub const HOST: Self = Self(0b100_0000_0000_0000);
    pub const ALL_GRAPHICS: Self = Self(0b1000_0000_0000_0000);
    pub const ALL_COMMANDS: Self = Self(0b1_0000_0000_0000_0000);
}

vk_enum!(RenderPassCreateFlags);
vk_bitflags!(RenderPassCreateFlags);
impl FenceCreateFlags {
    pub const RESERVED_0_KHR: Self = Self(0b1);
    pub const TRANSFORM_QCOM: Self = Self(0b10);
}

vk_enum!(AccessFlags);
vk_bitflags!(AccessFlags);
impl AccessFlags {
    pub const INDIRECT_COMMAND_READ: Self = Self(0b1);
    pub const INDEX_READ: Self = Self(0b10);
    pub const VERTEX_ATTRIBUTE_READ: Self = Self(0b100);
    pub const UNIFORM_READ: Self = Self(0b1000);
    pub const INPUT_ATTACHMENT_READ: Self = Self(0b1_0000);
    pub const SHADER_READ: Self = Self(0b10_0000);
    pub const SHADER_WRITE: Self = Self(0b100_0000);
    pub const COLOR_ATTACHMENT_READ: Self = Self(0b1000_0000);
    pub const COLOR_ATTACHMENT_WRITE: Self = Self(0b1_0000_0000);
    pub const DEPTH_STENCIL_ATTACHMENT_READ: Self = Self(0b10_0000_0000);
    pub const DEPTH_STENCIL_ATTACHMENT_WRITE: Self = Self(0b100_0000_0000);
    pub const TRANSFER_READ: Self = Self(0b1000_0000_0000);
    pub const TRANSFER_WRITE: Self = Self(0b1_0000_0000_0000);
    pub const HOST_READ: Self = Self(0b10_0000_0000_0000);
    pub const HOST_WRITE: Self = Self(0b100_0000_0000_0000);
    pub const MEMORY_READ: Self = Self(0b1000_0000_0000_0000);
    pub const MEMORY_WRITE: Self = Self(0b1_0000_0000_0000_0000);
}

vk_enum!(DependencyFlags);
vk_bitflags!(DependencyFlags);
impl DependencyFlags {
    pub const BY_REGION: Self = Self(0b1);
}

vk_enum!(AttachmentDescriptionFlags);
vk_bitflags!(AttachmentDescriptionFlags);
impl FenceCreateFlags {
    pub const MAY_ALIAS: Self = Self(0b1);
}

vk_enum!(AttachmentLoadOp);
impl AttachmentLoadOp {
    pub const LOAD: Self = Self(0);
    pub const CLEAR: Self = Self(1);
    pub const DONT_CARE: Self = Self(2);
    pub const NONE_EXT: Self = Self(1_000_400_000);
}

vk_enum!(AttachmentStoreOp);
impl AttachmentStoreOp {
    pub const STORE: Self = Self(0);
    pub const DONT_CARE: Self = Self(1);
    pub const NONE_EXT: Self = Self(1_000_301_000);
    pub const NONE_QCOM: Self = Self::NONE_EXT;
}

vk_enum!(ImageLayout);
impl ImageLayout {
    pub const UNDEFINED: Self = Self(0);
    pub const GENERAL: Self = Self(1);
    pub const COLOR_ATTACHMENT_OPTIMAL: Self = Self(2);
    pub const DEPTH_STENCIL_ATTACHMENT_OPTIMAL: Self = Self(3);
    pub const DEPTH_STENCIL_READ_ONLY_OPTIMAL: Self = Self(4);
    pub const SHADER_READ_ONLY_OPTIMAL: Self = Self(5);
    pub const TRANSFER_SRC_OPTIMAL: Self = Self(6);
    pub const TRANSFER_DST_OPTIMAL: Self = Self(7);
    pub const PREINITIALIZED: Self = Self(8);
    pub const PRESENT_SRC_KHR: Self = Self(1_000_001_002);
}

vk_enum!(PipelineBindPoint);
impl PipelineBindPoint {
    pub const GRAPHICS: Self = Self(0);
    pub const COMPUTE: Self = Self(1);
}

vk_enum!(SubpassContents);
impl SubpassContents {
    pub const INLINE: Self = Self(0);
    pub const SECONDARY_COMMAND_BUFFERS: Self = Self(1);
}

vk_enum!(BufferCreateFlags);
vk_bitflags!(BufferCreateFlags);
impl BufferCreateFlags {
    pub const SPARSE_BINDING: Self = Self(0b1);
    pub const SPARSE_RESIDENCY: Self = Self(0b10);
    pub const SPARSE_ALIASED: Self = Self(0b100);
}

vk_enum!(PipelineCreateFlags);
vk_bitflags!(PipelineCreateFlags);
impl PipelineCreateFlags {
    pub const DISABLE_OPTIMIZATION: Self = Self(0b1);
    pub const ALLOW_DERIVATIVES: Self = Self(0b10);
    pub const DERIVATIVE: Self = Self(0b100);
}

vk_enum!(ShaderStageFlags);
vk_bitflags!(ShaderStageFlags);
impl ShaderStageFlags {
    pub const VERTEX: Self = Self(0b1);
    pub const TESSELLATION_CONTROL: Self = Self(0b10);
    pub const TESSELLATION_EVALUATION: Self = Self(0b100);
    pub const GEOMETRY: Self = Self(0b1000);
    pub const FRAGMENT: Self = Self(0b1_0000);
    pub const COMPUTE: Self = Self(0b10_0000);
    pub const ALL_GRAPHICS: Self = Self(0x0000_001F);
    pub const ALL: Self = Self(0x7FFF_FFFF);
}

vk_enum!(VertexInputRate);
impl VertexInputRate {
    pub const VERTEX: Self = Self(0);
    pub const INSTANCE: Self = Self(1);
}


vk_enum!(PrimitiveTopology);
impl PrimitiveTopology {
    pub const POINT_LIST: Self = Self(0);
    pub const LINE_LIST: Self = Self(1);
    pub const LINE_STRIP: Self = Self(2);
    pub const TRIANGLE_LIST: Self = Self(3);
    pub const TRIANGLE_STRIP: Self = Self(4);
    pub const TRIANGLE_FAN: Self = Self(5);
    pub const LINE_LIST_WITH_ADJACENCY: Self = Self(6);
    pub const LINE_STRIP_WITH_ADJACENCY: Self = Self(7);
    pub const TRIANGLE_LIST_WITH_ADJACENCY: Self = Self(8);
    pub const TRIANGLE_STRIP_WITH_ADJACENCY: Self = Self(9);
    pub const PATCH_LIST: Self = Self(10);
}

vk_enum!(PolygonMode);
impl PolygonMode {
    pub const FILL: Self = Self(0);
    pub const LINE: Self = Self(1);
    pub const POINT: Self = Self(2);
}

vk_enum!(FrontFace);
impl FrontFace {
    pub const COUNTER_CLOCKWISE: Self = Self(0);
    pub const CLOCKWISE: Self = Self(1);
}

vk_enum!(CullModeFlags);
vk_bitflags!(CullModeFlags);
impl CullModeFlags {
    pub const NONE: Self = Self(0);
    pub const FRONT: Self = Self(0b1);
    pub const BACK: Self = Self(0b10);
    pub const FRONT_AND_BACK: Self = Self(0x0000_0003);
}

vk_enum!(CompareOp);
impl CompareOp {
    pub const NEVER: Self = Self(0);
    pub const LESS: Self = Self(1);
    pub const EQUAL: Self = Self(2);
    pub const LESS_OR_EQUAL: Self = Self(3);
    pub const GREATER: Self = Self(4);
    pub const NOT_EQUAL: Self = Self(5);
    pub const GREATER_OR_EQUAL: Self = Self(6);
    pub const ALWAYS: Self = Self(7);
}

vk_enum!(StencilOp);
impl StencilOp {
    pub const KEEP: Self = Self(0);
    pub const ZERO: Self = Self(1);
    pub const REPLACE: Self = Self(2);
    pub const INCREMENT_AND_CLAMP: Self = Self(3);
    pub const DECREMENT_AND_CLAMP: Self = Self(4);
    pub const INVERT: Self = Self(5);
    pub const INCREMENT_AND_WRAP: Self = Self(6);
    pub const DECREMENT_AND_WRAP: Self = Self(7);
}

vk_enum!(LogicOp);
impl LogicOp {
    pub const CLEAR: Self = Self(0);
    pub const AND: Self = Self(1);
    pub const AND_REVERSE: Self = Self(2);
    pub const COPY: Self = Self(3);
    pub const AND_INVERTED: Self = Self(4);
    pub const NO_OP: Self = Self(5);
    pub const XOR: Self = Self(6);
    pub const OR: Self = Self(7);
    pub const NOR: Self = Self(8);
    pub const EQUIVALENT: Self = Self(9);
    pub const INVERT: Self = Self(10);
    pub const OR_REVERSE: Self = Self(11);
    pub const COPY_INVERTED: Self = Self(12);
    pub const OR_INVERTED: Self = Self(13);
    pub const NAND: Self = Self(14);
    pub const SET: Self = Self(15);
}

vk_enum!(BlendFactor);
impl BlendFactor {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const SRC_COLOR: Self = Self(2);
    pub const ONE_MINUS_SRC_COLOR: Self = Self(3);
    pub const DST_COLOR: Self = Self(4);
    pub const ONE_MINUS_DST_COLOR: Self = Self(5);
    pub const SRC_ALPHA: Self = Self(6);
    pub const ONE_MINUS_SRC_ALPHA: Self = Self(7);
    pub const DST_ALPHA: Self = Self(8);
    pub const ONE_MINUS_DST_ALPHA: Self = Self(9);
    pub const CONSTANT_COLOR: Self = Self(10);
    pub const ONE_MINUS_CONSTANT_COLOR: Self = Self(11);
    pub const CONSTANT_ALPHA: Self = Self(12);
    pub const ONE_MINUS_CONSTANT_ALPHA: Self = Self(13);
    pub const SRC_ALPHA_SATURATE: Self = Self(14);
    pub const SRC1_COLOR: Self = Self(15);
    pub const ONE_MINUS_SRC1_COLOR: Self = Self(16);
    pub const SRC1_ALPHA: Self = Self(17);
    pub const ONE_MINUS_SRC1_ALPHA: Self = Self(18);
}

vk_enum!(BlendOp);
impl BlendOp {
    pub const ADD: Self = Self(0);
    pub const SUBTRACT: Self = Self(1);
    pub const REVERSE_SUBTRACT: Self = Self(2);
    pub const MIN: Self = Self(3);
    pub const MAX: Self = Self(4);
}

vk_enum!(ColorComponentFlags);
vk_bitflags!(ColorComponentFlags);
impl ColorComponentFlags {
    pub const R: Self = Self(0b1);
    pub const G: Self = Self(0b10);
    pub const B: Self = Self(0b100);
    pub const A: Self = Self(0b1000);
}

vk_enum!(DynamicState);
impl DynamicState {
    pub const VIEWPORT: Self = Self(0);
    pub const SCISSOR: Self = Self(1);
    pub const LINE_WIDTH: Self = Self(2);
    pub const DEPTH_BIAS: Self = Self(3);
    pub const BLEND_CONSTANTS: Self = Self(4);
    pub const DEPTH_BOUNDS: Self = Self(5);
    pub const STENCIL_COMPARE_MASK: Self = Self(6);
    pub const STENCIL_WRITE_MASK: Self = Self(7);
    pub const STENCIL_REFERENCE: Self = Self(8);
}

vk_enum!(DescriptorSetLayoutCreateFlags);
vk_bitflags!(DescriptorSetLayoutCreateFlags);
impl DescriptorSetLayoutCreateFlags {
    pub const PUSH_DESCRIPTOR_KHR: Self = Self(0b1);
}

vk_enum!(DescriptorType);
impl DescriptorType {
    pub const SAMPLER: Self = Self(0);
    pub const COMBINED_IMAGE_SAMPLER: Self = Self(1);
    pub const SAMPLED_IMAGE: Self = Self(2);
    pub const STORAGE_IMAGE: Self = Self(3);
    pub const UNIFORM_TEXEL_BUFFER: Self = Self(4);
    pub const STORAGE_TEXEL_BUFFER: Self = Self(5);
    pub const UNIFORM_BUFFER: Self = Self(6);
    pub const STORAGE_BUFFER: Self = Self(7);
    pub const UNIFORM_BUFFER_DYNAMIC: Self = Self(8);
    pub const STORAGE_BUFFER_DYNAMIC: Self = Self(9);
    pub const INPUT_ATTACHMENT: Self = Self(10);
}

impl fmt::Display for DescriptorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = match *self {
            DescriptorType::SAMPLER => "SAMPLER",
            DescriptorType::COMBINED_IMAGE_SAMPLER => "COMBINED_IMAGE_SAMPLER",
            DescriptorType::SAMPLED_IMAGE => "SAMPLED_IMAGE",
            DescriptorType::STORAGE_IMAGE => "STORAGE_IMAGE",
            DescriptorType::UNIFORM_TEXEL_BUFFER => "UNIFORM_TEXEL_BUFFER",
            DescriptorType::STORAGE_TEXEL_BUFFER => "STORAGE_TEXEL_BUFFER",
            DescriptorType::UNIFORM_BUFFER => "UNIFORM_BUFFER",
            DescriptorType::STORAGE_BUFFER => "STORAGE_BUFFER",
            DescriptorType::UNIFORM_BUFFER_DYNAMIC => "UNIFORM_BUFFER_DYNAMIC",
            DescriptorType::STORAGE_BUFFER_DYNAMIC => "STORAGE_BUFFER_DYNAMIC",
            DescriptorType::INPUT_ATTACHMENT => "INPUT_ATTACHMENT",
            _ => "BAD_VALUE"
        };

        f.write_str(id)
    }
}

vk_enum!(IndexType);
impl IndexType {
    pub const UINT16: Self = Self(0);
    pub const UINT32: Self = Self(1);
}

vk_enum!(DescriptorPoolCreateFlags);
vk_bitflags!(DescriptorPoolCreateFlags);
impl DescriptorPoolCreateFlags {
    pub const FREE_DESCRIPTOR_SET: Self = Self(0b1);
}

vk_enum!(ImageCreateFlags);
vk_bitflags!(ImageCreateFlags);
impl ImageCreateFlags {
    pub const SPARSE_BINDING: Self = Self(0b1);
    pub const SPARSE_RESIDENCY: Self = Self(0b10);
    pub const SPARSE_ALIASED: Self = Self(0b100);
    pub const MUTABLE_FORMAT: Self = Self(0b1000);
    pub const CUBE_COMPATIBLE: Self = Self(0b1_0000);
}

vk_enum!(ImageType);
impl ImageType {
    pub const TYPE_1D: Self = Self(0);
    pub const TYPE_2D: Self = Self(1);
    pub const TYPE_3D: Self = Self(2);
}

vk_enum!(ImageTiling);
impl ImageTiling {
    pub const OPTIMAL: Self = Self(0);
    pub const LINEAR: Self = Self(1);
}

vk_enum!(Filter);
impl Filter {
    pub const NEAREST: Self = Self(0);
    pub const LINEAR: Self = Self(1);
}

vk_enum!(BorderColor);
impl BorderColor {
    pub const FLOAT_TRANSPARENT_BLACK: Self = Self(0);
    pub const INT_TRANSPARENT_BLACK: Self = Self(1);
    pub const FLOAT_OPAQUE_BLACK: Self = Self(2);
    pub const INT_OPAQUE_BLACK: Self = Self(3);
    pub const FLOAT_OPAQUE_WHITE: Self = Self(4);
    pub const INT_OPAQUE_WHITE: Self = Self(5);
}

vk_enum!(SamplerMipmapMode);
impl SamplerMipmapMode {
    pub const NEAREST: Self = Self(0);
    pub const LINEAR: Self = Self(1);
}

vk_enum!(SamplerAddressMode);
impl SamplerAddressMode {
    pub const REPEAT: Self = Self(0);
    pub const MIRRORED_REPEAT: Self = Self(1);
    pub const CLAMP_TO_EDGE: Self = Self(2);
    pub const CLAMP_TO_BORDER: Self = Self(3);
}

vk_enum!(FormatFeatureFlags);
vk_bitflags!(FormatFeatureFlags);
impl FormatFeatureFlags {
    pub const SAMPLED_IMAGE: Self = Self(0b1);
    pub const STORAGE_IMAGE: Self = Self(0b10);
    pub const STORAGE_IMAGE_ATOMIC: Self = Self(0b100);
    pub const UNIFORM_TEXEL_BUFFER: Self = Self(0b1000);
    pub const STORAGE_TEXEL_BUFFER: Self = Self(0b1_0000);
    pub const STORAGE_TEXEL_BUFFER_ATOMIC: Self = Self(0b10_0000);
    pub const VERTEX_BUFFER: Self = Self(0b100_0000);
    pub const COLOR_ATTACHMENT: Self = Self(0b1000_0000);
    pub const COLOR_ATTACHMENT_BLEND: Self = Self(0b1_0000_0000);
    pub const DEPTH_STENCIL_ATTACHMENT: Self = Self(0b10_0000_0000);
    pub const BLIT_SRC: Self = Self(0b100_0000_0000);
    pub const BLIT_DST: Self = Self(0b1000_0000_0000);
    pub const SAMPLED_IMAGE_FILTER_LINEAR: Self = Self(0b1_0000_0000_0000);
}
