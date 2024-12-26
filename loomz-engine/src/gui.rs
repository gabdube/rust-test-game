use fnv::FnvHashMap;
use std::{slice, sync::Arc};
use loomz_engine_core::{LoomzEngineCore, VulkanContext, Texture, alloc::VertexAlloc, descriptors::*, pipelines::*};
use loomz_shared::api::{LoomzApi, GuiTextId, GuiTextUpdate, GuiComponentTextGlyph};
use loomz_shared::assets::{LoomzAssetsBundle, MsdfFontId, msdf_font::ComputedGlyph};
use loomz_shared::{CommonError, CommonErrorType};
use loomz_shared::{backend_init_err, assets_err, chain_err};
use super::pipeline_compiler::PipelineCompiler;

const BATCH_LAYOUT_INDEX: u32 = 0;
const TEXTURE_BINDING: u32 = 0;

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

#[derive(Default)]
struct GuiModuleDescriptors {
    alloc: DescriptorsAllocator,
    updates: DescriptorWriteBuffer,
    texture_params: DescriptorWriteImageParams,
}

struct GuiResources {
    assets: Arc<LoomzAssetsBundle>,
    batch_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    text_pipeline: GraphicsPipeline,
    component_pipeline: GraphicsPipeline,
    vertex: VertexAlloc<GuiVertex>,
    textures: FnvHashMap<MsdfFontId, Texture>,
    descriptors: GuiModuleDescriptors,
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

struct GuiText {
    font_id: Option<MsdfFontId>,
    font_view: vk::ImageView,
    glyphs: Vec<ComputedGlyph>,
}

struct GuiData {
    text: Vec<GuiText>,
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
    update_batches: bool,
}

impl GuiModule {

    pub fn init(core: &mut LoomzEngineCore, api: &LoomzApi) -> Result<Self, CommonError> {
        let resources = GuiResources {
            assets: api.assets(),
            batch_layout: vk::DescriptorSetLayout::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            text_pipeline: GraphicsPipeline::new(),
            component_pipeline: GraphicsPipeline::new(),
            vertex: VertexAlloc::default(),
            textures: FnvHashMap::default(),
            descriptors: GuiModuleDescriptors::default(),
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
            text: Vec::with_capacity(16),
            vertex: vec![GuiVertex::default(); 500],
            indices: vec![0; 1000]
        };
        
        let mut gui = GuiModule {
            resources: Box::new(resources),
            render: Box::new(render),
            data: Box::new(data),
            batches: Vec::with_capacity(16),
            update_batches: false,
        };

        gui.setup_pipelines(core)?;
        gui.setup_vertex_buffers(core)?;
        gui.setup_descriptors(core)?;
        gui.setup_render_data();

        Ok(gui)
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        self.resources.text_pipeline.destroy(&core.ctx);
        self.resources.component_pipeline.destroy(&core.ctx);
        self.resources.vertex.free(core);
        self.resources.descriptors.alloc.destroy(core);

        core.ctx.device.destroy_pipeline_layout(self.resources.pipeline_layout);
        core.ctx.device.destroy_descriptor_set_layout(self.resources.batch_layout);

        for texture in self.resources.textures.values() {
            core.destroy_texture(*texture);
        }
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
            device.cmd_bind_pipeline(cmd, GRAPHICS, render.text_pipeline_handle);
            device.cmd_bind_descriptor_sets(cmd, GRAPHICS, render.pipeline_layout, BATCH_LAYOUT_INDEX, slice::from_ref(&batch.set), &[]);
            device.cmd_draw_indexed(cmd, batch.index_count, 1, batch.first_index, 0, 0);
        }
    }

    //
    // Updates
    //

    fn create_text(&mut self, text_id: GuiTextId) -> usize {
        let id = self.data.text.len();
        self.data.text.push(GuiText {
            font_id: None,
            font_view: vk::ImageView::null(),
            glyphs: Vec::new(),
        });

        text_id.bind(id as u32);

        id
    }

    fn update_text_font(&mut self, core: &mut LoomzEngineCore, text_index: usize, font: MsdfFontId) -> Result<(), CommonError> {
        let old_font_id = self.data.text[text_index].font_id;
        if old_font_id != Some(font) {
            let font_view = self.fetch_font_texture_view(core, font)?;
            self.data.text[text_index].font_id = Some(font);
            self.data.text[text_index].font_view = font_view;
            self.update_batches = true;
        }
        Ok(())
    }

    fn update_text_glyphs(&mut self, text_index: usize, glyphs: &'static [GuiComponentTextGlyph]) {
        let text = &mut self.data.text[text_index];
        text.glyphs.clear();

        if text.glyphs.capacity() < glyphs.len() {
            text.glyphs.reserve(glyphs.len());
        }

        for g in glyphs {
            let mut glyph = g.glyph;
            glyph.position.left += g.offset.x;
            glyph.position.right += g.offset.x;
            glyph.position.top += g.offset.y;
            glyph.position.bottom += g.offset.y;
            text.glyphs.push(glyph);
        }

        self.update_batches = true;
    }

    fn update_text(&mut self, core: &mut LoomzEngineCore, text_index: usize, update: GuiTextUpdate) -> Result<(), CommonError> {
        match update {
            GuiTextUpdate::Font(font) => {
                self.update_text_font(core, text_index, font)?;
            },
            GuiTextUpdate::Glyphs(glyphs) => {
                self.update_text_glyphs(text_index, glyphs);
            }
        }

        Ok(())
    }

    fn api_update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        if let Some(updates) = api.gui().text_updates() {
            for (id, update) in updates {
                let index = match id.bound_value() {
                    Some(index) => index,
                    None => self.create_text(id),
                };

                self.update_text(core, index, update)?;
            }
        }

        Ok(())
    }

    fn batches_update(&mut self, core: &mut LoomzEngineCore) {
        GuiBatcher::build(self, core);
        self.update_batches = false;
    }

    pub fn update(&mut self, api: &LoomzApi, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        self.api_update(api, core)?;

        if self.update_batches {
            self.batches_update(core);
        }

        Ok(())
    }

    //
    // Data
    //

    fn fetch_font_texture_view(&mut self, core: &mut LoomzEngineCore, id: MsdfFontId) -> Result<vk::ImageView, CommonError> {
        if let Some(texture) = self.resources.textures.get(&id) {
            return Ok(texture.view);
        }

        let texture_asset = self.resources.assets.font(id)
            .ok_or_else(|| assets_err!("Unkown asset with ID {id:?}") )?;

        let texture = core.create_texture_from_font_asset(&texture_asset)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendGeneric, "Failed to create image from asset") )?;

        self.resources.textures.insert(id, texture);

        Ok(texture.view)
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

    fn setup_descriptors(&mut self, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        use loomz_engine_core::descriptors::DescriptorsAllocation;
        
        let allocations = [
            DescriptorsAllocation {
                layout: self.resources.batch_layout,
                bindings: Self::pipeline_descriptor_bindings_batch(),
                count: 1,
            },
        ];

        self.resources.descriptors.alloc = DescriptorsAllocator::new(core, &allocations)
            .map_err(|err| chain_err!(err, CommonErrorType::BackendInit, "Failed to prellocate descriptor sets") )?;

        self.resources.descriptors.texture_params = DescriptorWriteImageParams {
            sampler: core.resources.linear_sampler,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            dst_binding: TEXTURE_BINDING,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };

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

struct GuiBatcher<'a> {
    core: &'a mut LoomzEngineCore,
    data: &'a mut GuiData,
    resources: &'a mut GuiResources,
    batches: &'a mut Vec<GuiBatch>,
    current_view: vk::ImageView,
    text_index: usize,
    batch_index: usize,
    index_count: isize,
    vertex_count: u32,
}

impl<'a> GuiBatcher<'a> {

    fn build(gui: &mut GuiModule, core: &mut LoomzEngineCore) {
        let mut batcher = GuiBatcher {
            core,
            current_view: vk::ImageView::null(),
            data: &mut gui.data,
            resources: &mut gui.resources,
            batches: &mut gui.batches,
            text_index: 0,
            batch_index: 0,
            index_count: 0,
            vertex_count: 0,
        };
        batcher.resources.descriptors.alloc.clear_sets(BATCH_LAYOUT_INDEX);
        batcher.batches.clear();

        batcher.first_batch();
        batcher.remaining_batches();
        batcher.upload_vertex();

        gui.resources.descriptors.updates.submit(core);
    }

    fn first_batch(&mut self) {
        let mut found = false;
        let max_instance = self.data.text.len();
        while !found && self.text_index != max_instance {
            let text = &self.data.text[self.text_index];
            let font_view = text.font_view;
            let index_count = (text.glyphs.len() * 6) as u32;
            if index_count == 0 || font_view.is_null() {
                self.text_index += 1;
                continue;
            }

            self.next_batch(font_view);
            self.write_text_indices();
            self.write_text_vertex();

            self.text_index += 1;
            found = true;
        }
    }

    fn remaining_batches(&mut self) {
        let max_instance = self.data.text.len();
        while self.text_index != max_instance {
            let text = &self.data.text[self.text_index];
            let font_view = text.font_view;
            let index_count = (text.glyphs.len() * 6) as u32;
            if index_count == 0 || font_view.is_null() {
                self.text_index += 1;
                continue;
            }

            if self.current_view != font_view {
                self.next_batch(font_view);
                self.batch_index += 1;
            }

            self.write_text_indices();
            self.write_text_vertex();

            self.text_index += 1;
        }
    }

    fn next_batch(&mut self, font_view: vk::ImageView) {
        let set = self.resources.descriptors.alloc.next_set(BATCH_LAYOUT_INDEX);
        self.resources.descriptors.updates.write_simple_image(set, font_view, &self.resources.descriptors.texture_params);
    
        self.current_view = font_view;

        self.batches.push(GuiBatch {
            set,
            first_index: self.index_count as u32,
            index_count: 0,
        });
    }

    fn write_text_indices(&mut self) {
        let glyph_count = self.data.text[self.text_index].glyphs.len() as isize;
        let index_count = glyph_count * 6;

        // Safety, indices capacity will be greater than written range
        let mut i = self.index_count;
        let mut v = self.vertex_count;
        assert!(i + index_count < self.data.indices.len() as isize);

        unsafe {
            for _ in 0..glyph_count {
                let indices = self.data.indices.as_mut_ptr();
                indices.offset(i+0).write(v+0);
                indices.offset(i+1).write(v+1);
                indices.offset(i+2).write(v+2);
                indices.offset(i+3).write(v+2);
                indices.offset(i+4).write(v+3);
                indices.offset(i+5).write(v+1);

                i += 6;
                v += 4;
            }
        }

        self.index_count += index_count;
        self.batches[self.batch_index].index_count += index_count as u32;
    }

    fn write_text_vertex(&mut self) {
        let glyphs = &self.data.text[self.text_index].glyphs;
        let glyph_count = glyphs.len() as isize;
        let vertex_count = glyph_count * 4;

        // Safety, vertex capacity will be greater than written range
        let mut v = self.vertex_count as isize;
        assert!(v + vertex_count < self.data.vertex.len() as isize);

        unsafe {
            let vertex = self.data.vertex.as_mut_ptr();            
            for glyph in glyphs {
                let [x1, y1, x2, y2] = glyph.position.splat();
                let [x3, y3, x4, y4] = glyph.texcoord.splat();
                vertex.offset(v+0).write(GuiVertex { pos: [x1, y1], uv: [x3, y3] });
                vertex.offset(v+1).write(GuiVertex { pos: [x2, y1], uv: [x4, y3] });
                vertex.offset(v+2).write(GuiVertex { pos: [x1, y2], uv: [x3, y4] });
                vertex.offset(v+3).write(GuiVertex { pos: [x2, y2], uv: [x4, y4] });
                v += 4;
            }
        }

        self.vertex_count += vertex_count as u32;
    }

    fn upload_vertex(&mut self) {
        let i = self.index_count as usize;
        let v = self.vertex_count as usize;
        self.resources.vertex.set_data(
            self.core,
            &self.data.indices[0..i],
            &self.data.vertex[0..v],
        );
    }
}
