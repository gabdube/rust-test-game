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

    end_record(device, cmd)?;
    clear_data(staging);

    Ok(())
}

fn clear_data(staging: &mut VulkanStaging) {
    staging.vertex_buffer_copies.clear();
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

