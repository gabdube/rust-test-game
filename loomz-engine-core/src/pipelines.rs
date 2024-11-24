mod layout;
pub use layout::*;

mod shaders;
pub use shaders::*;

use crate::VulkanContext;

static DEFAULT_BLEND_ATTACHMENTS: [vk::PipelineColorBlendAttachmentState; 1] = [
    vk::PipelineColorBlendAttachmentState::const_default(),
];
static DEFAULT_DYNAMIC_STATES: [vk::DynamicState; 2] = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

const MAX_PIPELINE_SET_LAYOUT: usize = 3;

/// Vertex format to use when setting up a pipeline
pub struct PipelineVertexFormat {
    pub location: u32,
    pub offset: u32,
    pub format: vk::Format,
}


struct GraphicsPipelineBuildInfo {
    modules: GraphicsShaderModules,

    stage_count: u32,
    stages:      [vk::PipelineShaderStageCreateInfo; 2],

    vertex_attributes_count: u32,
    vertex_binding_count: u32,

    vertex_attributes: [vk::VertexInputAttributeDescription; 4],
    vertex_bindings:   [vk::VertexInputBindingDescription; 1],
    vertex_input:      vk::PipelineVertexInputStateCreateInfo,

    input_assembly:    vk::PipelineInputAssemblyStateCreateInfo,
    tessellation:      vk::PipelineTessellationStateCreateInfo,
    viewport:          vk::PipelineViewportStateCreateInfo,
    rasterization:     vk::PipelineRasterizationStateCreateInfo,
    multisample:       vk::PipelineMultisampleStateCreateInfo,
    depth_stencil:     vk::PipelineDepthStencilStateCreateInfo,

    blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    color_blend:       vk::PipelineColorBlendStateCreateInfo,

    dynamic_states:    [vk::DynamicState; 2],
    dynamic:           vk::PipelineDynamicStateCreateInfo,

    color_attachments_formats: [vk::Format; 1],
    depth_attachment_format: vk::Format,

    render_info: vk::PipelineRenderingCreateInfo,

    descriptor_set_layouts: [vk::DescriptorSetLayout; MAX_PIPELINE_SET_LAYOUT],
}

pub struct GraphicsPipeline {
    build: Box<GraphicsPipelineBuildInfo>,
    handle: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
}

impl GraphicsPipeline {

    pub fn new() -> Self {
        GraphicsPipeline {
            build: Box::default(),
            handle: vk::Pipeline::null(),
            pipeline_layout: vk::PipelineLayout::null(),
        }
    }

    pub fn destroy(self, ctx: &VulkanContext) {
        let device = &ctx.device;
        device.destroy_pipeline(self.handle);
        device.destroy_pipeline_layout(self.pipeline_layout);
        device.destroy_descriptor_set_layout(self.build.descriptor_set_layouts[0]);
        device.destroy_descriptor_set_layout(self.build.descriptor_set_layouts[1]);
        device.destroy_descriptor_set_layout(self.build.descriptor_set_layouts[2]);
        self.build.modules.destroy(ctx);
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.handle
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }

    pub fn descriptor_set_layout(&self, index: usize) -> vk::DescriptorSetLayout {
        assert!(index < MAX_PIPELINE_SET_LAYOUT, "Max index of descriptor set layout allowed is {} (got {})", MAX_PIPELINE_SET_LAYOUT, index);
        self.build.descriptor_set_layouts[index]
    }

    pub fn set_handle(&mut self, pipeline: vk::Pipeline) {
        self.handle = pipeline;
    }

    pub fn set_pipeline_layout(&mut self, pipeline_layout: vk::PipelineLayout) {
        self.pipeline_layout = pipeline_layout;
    }

    pub fn set_descriptor_set_layout(&mut self, index: usize, layout: vk::DescriptorSetLayout) {
        assert!(index < MAX_PIPELINE_SET_LAYOUT, "Max index of descriptor set layout allowed is {} (got {})", MAX_PIPELINE_SET_LAYOUT, index);
        self.build.descriptor_set_layouts[index] = layout;
    }

    pub fn modules(&self) -> GraphicsShaderModules {
        self.build.modules
    }

    pub fn set_sample_count(&mut self, sample_count: vk::SampleCountFlags) {
        self.build.multisample.rasterization_samples = sample_count;
    }

    pub fn set_color_attachment_format(&mut self, format: vk::Format) {
        self.build.color_attachments_formats[0] = format;
    }

    pub fn set_depth_attachment_format(&mut self, format: vk::Format) {
        self.build.depth_attachment_format = format;
    }

    //
    // Building
    //

    pub fn set_shader_modules(&mut self, modules: GraphicsShaderModules) {
        let build = &mut self.build;
        
        build.modules = modules;

        build.stage_count = 2;
        build.stages = [
            vk::PipelineShaderStageCreateInfo {
                module: build.modules.vert,
                stage: vk::ShaderStageFlags::VERTEX,
                p_name: SHADER_ENTRY.as_ptr(),
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                module: build.modules.frag,
                stage: vk::ShaderStageFlags::FRAGMENT,
                p_name: SHADER_ENTRY.as_ptr(),
                ..Default::default()
            }
        ];
    }

    pub fn set_vertex_format<V: Copy>(&mut self, fmt: &[PipelineVertexFormat]) {
        let build = &mut self.build;

        // Attributes
        build.vertex_attributes_count = fmt.len() as u32;
        for (i, attr) in fmt.iter().enumerate() {
            build.vertex_attributes[i] = vk::VertexInputAttributeDescription {
                binding: 0,
                location: attr.location,
                offset: attr.offset,
                format: attr.format,
            };
        }

        // Renderer only support one packed vertex binding
        build.vertex_binding_count = 1;
        build.vertex_bindings[0] = vk::VertexInputBindingDescription {
            binding: 0,
            stride: ::std::mem::size_of::<V>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        };

        build.vertex_input = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: build.vertex_binding_count,
            p_vertex_binding_descriptions: build.vertex_bindings.as_ptr(),
            vertex_attribute_description_count: build.vertex_attributes_count,
            p_vertex_attribute_descriptions: build.vertex_attributes.as_ptr(),
            ..Default::default()
        };
    }

    pub fn set_depth_testing(&mut self, enabled: bool) {
        self.build.depth_stencil.depth_test_enable = enabled as u32;
        self.build.depth_stencil.depth_write_enable = enabled as u32;
    }

    pub fn rasterization(&mut self, state: &vk::PipelineRasterizationStateCreateInfo) {
        self.build.rasterization = *state;
    }

    pub fn blending(&mut self, attach: &vk::PipelineColorBlendAttachmentState, state: &vk::PipelineColorBlendStateCreateInfo) {
        debug_assert!(state.attachment_count <= 1, "Only one color blend attachment supported");
        self.build.blend_attachments[0] = *attach;
        self.build.color_blend = *state;
    }

    pub fn create_info(&mut self) -> vk::GraphicsPipelineCreateInfo {
        let build = &mut self.build;

        build.vertex_input.vertex_attribute_description_count = build.vertex_attributes_count as _;
        build.vertex_input.p_vertex_attribute_descriptions = build.vertex_attributes.as_ptr();

        build.vertex_input.vertex_binding_description_count = build.vertex_binding_count as _;
        build.vertex_input.p_vertex_binding_descriptions = build.vertex_bindings.as_ptr();

        build.color_blend.attachment_count = build.blend_attachments.len() as _;
        build.color_blend.p_attachments = build.blend_attachments.as_ptr();

        build.dynamic.dynamic_state_count = build.dynamic_states.len() as _;
        build.dynamic.p_dynamic_states = build.dynamic_states.as_ptr();

        build.render_info.depth_attachment_format = build.depth_attachment_format;
        build.render_info.color_attachment_count = build.color_attachments_formats.len() as _;
        build.render_info.p_color_attachment_formats = build.color_attachments_formats.as_ptr();
 
        vk::GraphicsPipelineCreateInfo {
            p_next: &build.render_info as *const vk::PipelineRenderingCreateInfo as _,

            stage_count: build.stage_count,
            p_stages: build.stages.as_ptr(),
            p_vertex_input_state: &build.vertex_input,
            p_input_assembly_state: &build.input_assembly,
            p_tessellation_state: &build.tessellation,
            p_viewport_state: &build.viewport,
            p_rasterization_state: &build.rasterization,
            p_multisample_state: &build.multisample,
            p_depth_stencil_state: &build.depth_stencil,
            p_color_blend_state: &build.color_blend,
            p_dynamic_state: &build.dynamic,

            layout: self.pipeline_layout,
            render_pass: vk::RenderPass::null(),  // Renderer use dynamic rendering, which ignore the render pass

            ..Default::default()
        }
    }

}

impl Default for GraphicsPipelineBuildInfo {
    fn default() -> Self {
        GraphicsPipelineBuildInfo {
            modules: GraphicsShaderModules::default(),
    
            stage_count: 0,
            stages: Default::default(),
    
            vertex_attributes_count: 0,
            vertex_binding_count: 0,
    
            vertex_attributes: Default::default(),
            vertex_bindings: Default::default(),
    
            vertex_input: vk::PipelineVertexInputStateCreateInfo {
                ..Default::default()
            },
    
            input_assembly: vk::PipelineInputAssemblyStateCreateInfo {
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                ..Default::default()
            },
    
            viewport: vk::PipelineViewportStateCreateInfo {
                viewport_count: 1,
                scissor_count: 1,
                ..Default::default()
            },
    
            rasterization: vk::PipelineRasterizationStateCreateInfo {
                polygon_mode: vk::PolygonMode::FILL,
                cull_mode: vk::CullModeFlags::BACK,
                front_face: vk::FrontFace::CLOCKWISE,
                line_width: 1.0,
                ..Default::default()
            },
    
            depth_stencil: vk::PipelineDepthStencilStateCreateInfo {
                depth_test_enable: 1,
                depth_write_enable: 1,
                depth_compare_op: vk::CompareOp::LESS,
                back: vk::StencilOpState {
                    compare_op: vk::CompareOp::ALWAYS,
                    ..Default::default()
                },
                ..Default::default()
            },
    
            multisample: vk::PipelineMultisampleStateCreateInfo {
                rasterization_samples: vk::SampleCountFlags::TYPE_1,
                ..Default::default()
            },
    
            blend_attachments: DEFAULT_BLEND_ATTACHMENTS,
            color_blend: vk::PipelineColorBlendStateCreateInfo::default(),
            
            dynamic_states: DEFAULT_DYNAMIC_STATES,
            dynamic: vk::PipelineDynamicStateCreateInfo::default(),
    
            tessellation: vk::PipelineTessellationStateCreateInfo::default(),

            depth_attachment_format: vk::Format::UNDEFINED,
            color_attachments_formats: [vk::Format::UNDEFINED],
            render_info: vk::PipelineRenderingCreateInfo::default(),

            descriptor_set_layouts: [vk::DescriptorSetLayout::null(); 3],
        }
    }
}

