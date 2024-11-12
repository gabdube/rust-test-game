use std::slice;
use loomz_shared::{render_record_err, CommonError};
use super::helpers::{begin_record, end_record};
use crate::{context::VulkanContext, LoomzEngine, VulkanRecordingInfo};

macro_rules! wrap {
    ($call:expr, $message:expr) => {
        $call.map_err(|err| render_record_err!("{}: {}", $message, err) )
    };
}

pub(crate) fn record_commands(engine: &mut LoomzEngine) -> Result<(), CommonError> {
    let ctx = &engine.ctx;
    let recording = &engine.recording;
    let cmd = recording.drawing_command_buffer;

    debug_assert!(!cmd.is_null(), "Drawing command buffer was not set during render preparing phase");

    wrap!(begin_record(&ctx.device, cmd), "Begin record failed")?;
    prepare_attachments(ctx, cmd, recording.output_image);
    begin_render_main(ctx, cmd, recording);
    end_render_main(ctx, cmd);
    finalize_attachments(ctx, cmd, recording.output_image);
    wrap!(end_record(&ctx.device, cmd), "End record failed")?;

    Ok(())
}

fn begin_render_main(ctx: &VulkanContext, cmd: vk::CommandBuffer, recording: &VulkanRecordingInfo) {
    let extent = recording.extent;
    let rendering_info = vk::RenderingInfo {
        render_area: vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        },
        layer_count: 1,
        color_attachment_count: 1,
        color_attachments: &recording.color_attachment,
        depth_attachment: &recording.depth_attachment,
        ..Default::default()
    };

    ctx.extensions.dynamic_rendering.begin_rendering(cmd, &rendering_info);
   
    let viewport = vk::Viewport { x: 0.0, y: 0.0, width: extent.width as f32, height: extent.height as f32, min_depth: 0.0, max_depth: 1.0 };
    ctx.device.cmd_set_viewport(cmd, 0, slice::from_ref(&viewport));

    let scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent };
    ctx.device.cmd_set_scissor(cmd, 0, slice::from_ref(&scissor));
}

fn end_render_main(ctx: &VulkanContext, cmd: vk::CommandBuffer) {
    ctx.extensions.dynamic_rendering.end_rendering(cmd);
}

fn prepare_attachments(ctx: &VulkanContext, cmd: vk::CommandBuffer, image: vk::Image) {
    let subresource_range = vk::ImageSubresourceRange::base_color();
    let attachment_barrier = vk::ImageMemoryBarrier2 {
        image,
        old_layout: vk::ImageLayout::UNDEFINED,
        new_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        src_access_mask: vk::AccessFlags2::NONE,
        src_stage_mask: vk::PipelineStageFlags2::NONE,
        dst_access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        dst_stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        subresource_range,
        ..Default::default()
    };
    let dependency = vk::DependencyInfo {
        image_memory_barrier_count: 1,
        image_memory_barrier: &attachment_barrier,
        ..Default::default()
    };

    ctx.extensions.synchronization2.cmd_pipeline_barrier2(cmd, &dependency);
}

fn finalize_attachments(ctx: &VulkanContext, cmd: vk::CommandBuffer, image: vk::Image) {
    let subresource_range = vk::ImageSubresourceRange::base_color();
    let attachment_barrier = vk::ImageMemoryBarrier2 {
        image,
        old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        src_access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        src_stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        dst_access_mask: vk::AccessFlags2::HOST_READ,
        dst_stage_mask: vk::PipelineStageFlags2::HOST,
        subresource_range,
        ..Default::default()
    };
    let dependency = vk::DependencyInfo {
        image_memory_barrier_count: 1,
        image_memory_barrier: &attachment_barrier,
        ..Default::default()
    };
    ctx.extensions.synchronization2.cmd_pipeline_barrier2(cmd, &dependency);
}
