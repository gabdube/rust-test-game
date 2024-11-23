use std::slice;
use loomz_shared::{render_record_err, CommonError};
use crate::{staging::VulkanStaging, LoomzEngineCore};


pub(crate) fn upload(engine: &mut LoomzEngineCore) -> Result<(), CommonError> {
    let device = &engine.ctx.device;
    let staging = &mut engine.staging;
    let cmd = staging.upload_command_buffer;

    begin_record(device, cmd)?;

    for buffer_copy in staging.vertex_buffer_copies.iter() {
        device.cmd_copy_buffer(cmd, staging.buffer, buffer_copy.dst_buffer, slice::from_ref(&buffer_copy.copy));
    }

    image_layout_transfer(&engine.ctx, &staging.image_barrier_prepare, cmd);

    for image_copy in staging.image_copies.iter() {
        device.cmd_copy_buffer_to_image(cmd, staging.buffer, image_copy.dst_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, slice::from_ref(&image_copy.copy));
    }

    image_layout_transfer(&engine.ctx, &staging.image_barrier_final, cmd);

    end_record(device, cmd)?;
    clear_data(staging);

    Ok(())
}

fn clear_data(staging: &mut VulkanStaging) {
    staging.vertex_buffer_copies.clear();
    staging.image_barrier_prepare.clear();
    staging.image_barrier_final.clear();
    staging.image_copies.clear();
    staging.upload_offset = 0;
}

fn begin_record(device: &vk::wrapper::Device, cmd: vk::CommandBuffer) -> Result<(), CommonError> {
    let begin_info = vk::CommandBufferBeginInfo {
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
    };
    device.begin_command_buffer(cmd, &begin_info)
        .map_err(|err| render_record_err!("Begin command buffer failed: {err}") )
}

fn end_record(device: &vk::wrapper::Device, cmd: vk::CommandBuffer) -> Result<(), CommonError> {
    device.end_command_buffer(cmd)
        .map_err(|err| render_record_err!("End command buffer failed: {err}") )
}

fn image_layout_transfer(ctx: &crate::VulkanContext, barriers: &[vk::ImageMemoryBarrier2], cmd: vk::CommandBuffer) {
    let dependency = vk::DependencyInfo {
        image_memory_barrier_count: barriers.len() as u32,
        image_memory_barrier: barriers.as_ptr(),
        ..Default::default()
    };

    ctx.extensions.synchronization2.cmd_pipeline_barrier2(cmd, &dependency);
}
