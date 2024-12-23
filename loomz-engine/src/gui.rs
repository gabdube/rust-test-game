use std::slice;
use loomz_engine_core::{LoomzEngineCore, VulkanContext, alloc::VertexAlloc, pipelines::*};
use loomz_shared::api::{LoomzApi, GuiId};
use loomz_shared::{CommonError, CommonErrorType};
use loomz_shared::{backend_init_err, chain_err};
use super::pipeline_compiler::PipelineCompiler;

const BATCH_LAYOUT_INDEX: u32 = 0;

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
    batch_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    vertex: VertexAlloc<GuiVertex>,
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

#[derive(Copy, Clone, Default)]
struct TempText {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

struct GuiInstance {
    id: GuiId,
    text: Vec<TempText>,
}

struct GuiData {
    instances: Vec<GuiInstance>,
    indices: Vec<u32>,
    vertex: Vec<GuiVertex>,
}

#[derive(Copy, Clone, Default)]
pub struct GuiBatch {
    set: vk::DescriptorSet,
    index_count: u32,
    first_index: u32,
}

pub(crate) struct GuiModule {
    resources: Box<GuiResources>,
    render: Box<GuiRender>,
    data: Box<GuiData>,
    batches: Vec<GuiBatch>,
}

impl GuiModule {

    pub fn init(core: &mut LoomzEngineCore) -> Result<Self, CommonError> {
        let resources = GuiResources {
            batch_layout: vk::DescriptorSetLayout::null(),
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

        let data = GuiData {
            instances: Vec::with_capacity(4),
            vertex: Vec::with_capacity(500),
            indices: Vec::with_capacity(1000)
        };
        
        let mut gui = GuiModule {
            resources: Box::new(resources),
            render: Box::new(render),
            data: Box::new(data),
            batches: Vec::with_capacity(16),
        };

        gui.setup_pipelines(core)?;
        gui.setup_vertex_buffers(core)?;
        gui.setup_render_data();
        gui.setup_test_data(core);

        Ok(gui)
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.resources.text_pipeline.destroy(&core.ctx);
        self.resources.component_pipeline.destroy(&core.ctx);
        self.resources.vertex.free(core);

        core.ctx.device.destroy_pipeline_layout(self.resources.pipeline_layout);
        core.ctx.device.destroy_descriptor_set_layout(self.resources.batch_layout);
    }

    pub fn set_output(&mut self, core: &LoomzEngineCore) {
        let extent = core.info.swapchain_extent;
        self.render.push_constants[0] = GuiPushConstant {
            screen_width: extent.width as f32,
            screen_height: extent.height as f32,
        };
    }

    pub fn rebuild(&mut self, core: &LoomzEngineCore) {
        self.set_output(core);
    }

    //
    // Pipeline setup
    //

    pub fn write_pipeline_create_infos(&mut self, compiler: &mut PipelineCompiler) {
        compiler.add_pipeline_info("gui_component", &mut self.resources.component_pipeline);
        compiler.add_pipeline_info("gui_text", &mut self.resources.text_pipeline);
    }

    pub fn set_pipeline_handle(&mut self, compiler: &PipelineCompiler) {
        let mut handle = compiler.get_pipeline("gui_component");
        self.resources.component_pipeline.set_handle(handle);
        self.render.component_pipeline_handle = handle;

        handle = compiler.get_pipeline("gui_text");
        self.resources.text_pipeline.set_handle(handle);
        self.render.text_pipeline_handle = handle;
    }

    //
    // Rendering
    //

    pub fn render(&self, ctx: &VulkanContext, cmd: vk::CommandBuffer) {
        #[inline(always)]
        fn push_values(constants: &[GuiPushConstant; 1]) -> &[u8] {
            unsafe { constants.align_to::<u8>().1 }
        }

        const GRAPHICS: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;

        let device = &ctx.device;
        let render = *self.render;

        device.cmd_bind_index_buffer(cmd, render.vertex_buffer, render.index_offset, vk::IndexType::UINT32);
        device.cmd_bind_vertex_buffers(cmd, 0, slice::from_ref(&render.vertex_buffer), &render.vertex_offset);
        device.cmd_push_constants(cmd, render.pipeline_layout, PUSH_STAGE_FLAGS, 0, PUSH_SIZE, push_values(&render.push_constants));

        for batch in self.batches.iter() {
            // device.cmd_bind_pipeline(cmd, GRAPHICS, render.component_pipeline_handle);
            // device.cmd_bind_descriptor_sets(cmd, GRAPHICS, render.pipeline_layout, BATCH_LAYOUT_INDEX, slice::from_ref(&batch.set), &[]);
            // device.cmd_draw_indexed(cmd, batch.index_count, batch.first_index, 0, 0, 0);
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

    fn pipeline_descriptor_bindings_batch() -> &'static [PipelineLayoutSetBinding; 1] {
        &[
            PipelineLayoutSetBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                descriptor_count: 1,
            },
        ]
    }

    fn setup_pipelines(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let ctx = &core.ctx;

        // Descriptor set layouts
        let bindings_batch = Self::pipeline_descriptor_bindings_batch();
        let layout_batch = PipelineLayoutSetBinding::build_descriptor_set_layout(&ctx.device, bindings_batch)
            .map_err(|err| backend_init_err!("Failed to create batch descriptor set layout: {}", err) )?;

        let layouts = [layout_batch];

        // Pipeline layout
        let constant_range = vk::PushConstantRange {
            offset: 0,
            size: ::std::mem::size_of::<GuiPushConstant>() as u32,
            stage_flags: vk::ShaderStageFlags::VERTEX,
        };
        let pipeline_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: layouts.len() as u32,
            p_set_layouts: layouts.as_ptr(),
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

        let res = &mut self.resources;
        res.batch_layout = layout_batch;
        res.pipeline_layout = pipeline_layout;
        res.component_pipeline.set_shader_modules(component_modules);
        res.text_pipeline.set_shader_modules(text_modules);
        
        for pipeline in [&mut res.component_pipeline, &mut res.text_pipeline] {
            pipeline.set_vertex_format::<GuiVertex>(&vertex_fields);
            pipeline.set_pipeline_layout(pipeline_layout, true);
            pipeline.set_descriptor_set_layout(BATCH_LAYOUT_INDEX as usize, layout_batch);
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
        let vertex_capacity = 500;
        let index_capacity = 1000;
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

    fn setup_test_data(&mut self, core: &mut LoomzEngineCore) {
        let id = { let id = GuiId::new(); id.bind(0); id };

        let text1 = TempText { x1: 1100.0, y1: 50.0, x2: 1150.0, y2: 100.0 };
        let text2 = TempText { x1: 1100.0, y1: 150.0, x2: 1150.0, y2: 200.0 };
        self.data.instances.push(GuiInstance {
            id,
            text: vec![text1, text2],
        });

        let indices = &mut self.data.indices;
        let vertex = &mut self.data.vertex;

        let mut vertex_count = 0;
        let mut current_batch = GuiBatch::default();
        for instance in self.data.instances.iter() {
            for text in instance.text.iter() {
                let i = vertex_count;
                indices.push(i+0);
                indices.push(i+1);
                indices.push(i+2);
                indices.push(i+2);
                indices.push(i+3);
                indices.push(i+1);

                let uv = [0.0, 0.0];
                vertex.push(GuiVertex { pos: [text.x1, text.y1], uv });
                vertex.push(GuiVertex { pos: [text.x2, text.y1], uv });
                vertex.push(GuiVertex { pos: [text.x1, text.y2], uv });
                vertex.push(GuiVertex { pos: [text.x2, text.y2], uv });

                current_batch.index_count += 6;
                vertex_count += 4;
            }
        }

        self.batches.push(current_batch);
        self.resources.vertex.set_data(core, &self.data.indices, &self.data.vertex);
    }

}
