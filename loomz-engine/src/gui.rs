use std::{ptr, slice};
use loomz_engine_core::{LoomzEngineCore, VulkanContext, alloc::VertexAlloc, pipelines::*};
use loomz_shared::api::{LoomzApi};
use loomz_shared::{CommonError, CommonErrorType};
use loomz_shared::{backend_init_err, chain_err};

const PUSH_STAGE_FLAGS: vk::ShaderStageFlags = vk::ShaderStageFlags::VERTEX;
const PUSH_SIZE: u32 = size_of::<GuiPushConstant>() as u32;

const GUI_VERT_SRC: &[u8] = include_bytes!("../../assets/shaders/gui.vert.spv");
const GUI_COMPONENT_FRAG_SRC: &[u8] = include_bytes!("../../assets/shaders/gui_component.frag.spv");
const GUI_TEXT_FRAG_SRC: &[u8] = include_bytes!("../../assets/shaders/gui_text.frag.spv");


#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct GuiPushConstant {
    pub screen_width: f32,
    pub screen_height: f32,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct GuiVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

struct GuiResources {
    vertex: VertexAlloc<GuiVertex>,
    pipeline_layout: vk::PipelineLayout,
    text_pipeline: GraphicsPipeline,
    component_pipeline: GraphicsPipeline,
}

/// Data used on rendering
#[derive(Copy, Clone)]
struct GuiRender {
    component_pipeline_handle: vk::Pipeline,
    text_pipeline_handle: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    vertex_buffer: vk::Buffer,
    index_offset: vk::DeviceSize,
    vertex_offset: [vk::DeviceSize; 1],
    push_constants: [GuiPushConstant; 1],
}

pub struct GuiView {
    component_index_count: u32,
    text_index_count: u32,
}

pub(crate) struct GuiModule {
    resources: Box<GuiResources>,
    render: Box<GuiRender>,
    views: Vec<GuiView>,
}

impl GuiModule {

    pub fn init(core: &mut LoomzEngineCore, api: &LoomzApi) -> Result<Box<Self>, CommonError> {
        let resources = GuiResources {
            pipeline_layout: vk::PipelineLayout::null(),
            text_pipeline: GraphicsPipeline::new(),
            component_pipeline: GraphicsPipeline::new(),
            vertex: VertexAlloc::default(),
        };
        
        let render = GuiRender {
            component_pipeline_handle: vk::Pipeline::null(),
            text_pipeline_handle: vk::Pipeline::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            vertex_buffer: vk::Buffer::null(),
            index_offset: 0,
            vertex_offset: [0],
            push_constants: [GuiPushConstant::default(); 1],
        };
        
        let mut gui = GuiModule {
            resources: Box::new(resources),
            render: Box::new(render),
            views: Vec::with_capacity(8),
        };

        gui.setup_pipelines(core)?;
        gui.setup_vertex_buffers(core)?;
        gui.setup_render_data();

        Ok(Box::new(gui))
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.resources.text_pipeline.destroy(&core.ctx);
        self.resources.component_pipeline.destroy(&core.ctx);
        self.resources.vertex.free(core);

        core.ctx.device.destroy_pipeline_layout(self.resources.pipeline_layout);
    }

    pub fn set_output(&mut self, core: &LoomzEngineCore) {
    }

    pub fn rebuild(&mut self, core: &LoomzEngineCore) {
    }

    //
    // Rendering
    //

    pub fn render(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        #[inline(always)]
        fn push_values(constants: &[GuiPushConstant; 1]) -> &[u8] {
            unsafe { constants.align_to::<u8>().1 }
        }

        let device = &ctx.device;
        let render = *self.render;

        device.cmd_bind_index_buffer(cmd, render.vertex_buffer, render.index_offset, vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, slice::from_ref(&render.vertex_buffer), &render.vertex_offset);
        device.cmd_push_constants(cmd, render.pipeline_layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, push_values(&render.push_constants));

        for view in self.views.iter() {
            if view.component_index_count > 0 {
                device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, render.component_pipeline_handle);
                device.cmd_draw_indexed(cmd, view.component_index_count, 1, 0, 0, 0);
            }
            
            if view.text_index_count > 0 {
                device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, render.text_pipeline_handle);
                device.cmd_draw_indexed(cmd, view.text_index_count, 1, 0, 0, 0);
            }
        }
    }

    //
    // Updates
    //

    pub fn update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        Ok(())
    }

    //
    // Setup
    //

    fn setup_pipelines(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let ctx = &core.ctx;

        // Pipeline layout
        let constant_range = vk::PushConstantRange {
            offset: 0,
            size: ::std::mem::size_of::<GuiPushConstant>() as u32,
            stage_flags: vk::ShaderStageFlags::VERTEX,
        };
        let pipeline_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 0,
            p_set_layouts: ptr::null(),
            push_constant_range_count: 1,
            p_push_constant_ranges: &constant_range,
            ..Default::default()
        };
        let pipeline_layout = ctx.device.create_pipeline_layout(&pipeline_create_info)
            .map_err(|err| backend_init_err!("Failed to create pipeline layout: {}", err) )?;
        

        // Shader source
        let component_modules = GraphicsShaderModules::new(ctx, GUI_VERT_SRC, GUI_COMPONENT_FRAG_SRC)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to compute gui component pipeline shader modules") )?;

        let text_modules = GraphicsShaderModules::new(ctx, GUI_VERT_SRC, GUI_TEXT_FRAG_SRC)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to compute gui text pipeline shader modules") )?;

        // Pipeline
        let vertex_fields = [
            PipelineVertexFormat {
                location: 0,
                offset: 0,
                format: vk::Format::R32G32_SFLOAT,
            },
            PipelineVertexFormat {
                location: 1,
                offset: 8,
                format: vk::Format::R32G32_SFLOAT,
            },
        ];

        self.resources.pipeline_layout = pipeline_layout;
        self.resources.component_pipeline.set_shader_modules(component_modules);
        self.resources.text_pipeline.set_shader_modules(text_modules);
        
        for pipeline in [&mut self.resources.component_pipeline, &mut self.resources.text_pipeline] {
            pipeline.set_vertex_format::<GuiVertex>(&vertex_fields);
            pipeline.set_pipeline_layout(pipeline_layout, true);
            pipeline.set_depth_testing(false);
            pipeline.rasterization(&vk::PipelineRasterizationStateCreateInfo {
                polygon_mode: vk::PolygonMode::FILL,
                cull_mode: vk::CullModeFlags::NONE,
                front_face: vk::FrontFace::COUNTER_CLOCKWISE,
                line_width: 1.0,
                ..Default::default()
            });
            pipeline.blending(
                &vk::PipelineColorBlendAttachmentState {
                    blend_enable: 1,
                    src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
                    dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                    src_alpha_blend_factor: vk::BlendFactor::ZERO,
                    dst_alpha_blend_factor: vk::BlendFactor::ONE,
                    ..Default::default()
                },
                &vk::PipelineColorBlendStateCreateInfo {
                    attachment_count: 1,
                    ..Default::default()
                }
            );
    
            let info = &core.info;
            pipeline.set_sample_count(info.sample_count);
            pipeline.set_color_attachment_format(info.color_format);
            pipeline.set_depth_attachment_format(info.depth_format);
        }

        Ok(())
    }

    fn setup_vertex_buffers(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let vertex_capacity = 1000;
        let index_capacity = 1500;
        self.resources.vertex = VertexAlloc::new(core, index_capacity, vertex_capacity)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to create vertex alloc: {err}") )?;

        Ok(())
    }

    fn setup_render_data(&mut self) {
        let render = &mut self.render;

        render.pipeline_layout = self.resources.pipeline_layout;
        render.vertex_buffer = self.resources.vertex.buffer;
        render.index_offset = self.resources.vertex.index_offset();
        render.vertex_offset = self.resources.vertex.vertex_offset();
    }

}
