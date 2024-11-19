use loomz_shared::{synchronize_err, CommonError};
use super::LoomzEngineCore;

pub enum AcquireReturn {
    Invalid,
    Render,
    Rebuild,
}

pub(crate) fn acquire_frame(engine: &mut LoomzEngineCore) -> Result<AcquireReturn, CommonError> {
    if engine.output.swapchain.is_null() {
        return Ok(AcquireReturn::Invalid);
    }

    if engine.output.rebuild {
        return Ok(AcquireReturn::Rebuild);
    }
    
    sync_drawings(engine)?;
    acquire_swapchain_image(engine)
}

fn sync_drawings(engine: &mut LoomzEngineCore) -> Result<(), CommonError> {
    let device = &engine.ctx.device;
    let sync = engine.output.drawings_sync;
    let wait_info = vk::SemaphoreWaitInfo {
        semaphore_count: 1,
        p_semaphores: &sync.handle,
        p_values: &sync.value,
        ..Default::default()
    };

    match device.wait_semaphores(&wait_info, u64::MAX) {
        Ok(_) => { Ok(()) },
        Err(err) => Err(synchronize_err!("failed to sync with rendering commands: {}", err))
    }
}

fn acquire_swapchain_image(engine: &mut LoomzEngineCore) -> Result<AcquireReturn, CommonError> {
    let swapchain = &engine.ctx.extensions.swapchain;
    let output = &mut engine.output;

    let mut image_index = 0;
    let result = swapchain.acquire_next_image(
        output.swapchain,
        u64::MAX,
        output.output_attachment_ready,
        vk::Fence::null(),
        &mut image_index,
    );

    match result {
        Ok(_) => {
            output.acquired_image_index = image_index;
            prepare_recording(engine, image_index);
            prepare_submit(engine);
            Ok(AcquireReturn::Render)
        },
        Err(vk::VkResult::SUBOPTIMAL_KHR) => {
            Ok(AcquireReturn::Rebuild)
        }
        Err(error) => {
            Err(synchronize_err!("Failed to acquire next image in swapchain: {error}"))
        }
    }
}

fn prepare_recording(engine: &mut LoomzEngineCore, image_index: u32) {
    let resources = &engine.resources;

    let i = image_index as usize;
    let att = &resources.attachments;

    assert!(i < att.output.len(), "Acquired image index must be in swapchain image range");

    // Upload
    let staging = &mut engine.staging;
    staging.upload_command_buffer = resources.upload_command_buffers[0];

    // Recording
    let recording = &mut engine.recording;
    recording.drawing_command_buffer = resources.drawing_command_buffers[0];
    recording.output_image = att.output[i].image;
    recording.color_attachment.resolve_image_view = att.output[i].view;
    recording.color_attachment.image_view = att.color.view;
    recording.depth_attachment.image_view = att.depth.view;
    recording.extent = engine.info.swapchain_extent;
}

fn prepare_submit(engine: &mut LoomzEngineCore) {
    let resources = &engine.resources;
    let output = &mut engine.output;
    let submit = &mut engine.submit;

    // Upload
    submit.upload_commands_submit.command_buffer = resources.upload_command_buffers[0];

    submit.upload_semaphore_signal[0].semaphore = output.drawings_sync.handle;
    submit.upload_semaphore_signal[0].value = output.drawings_sync.value + 1;

    // Render
    submit.render_commands_submit.command_buffer = resources.drawing_command_buffers[0];

    submit.render_semaphore_wait[0].semaphore = output.output_attachment_ready;
    submit.render_semaphore_wait[0].stage_mask = vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT;

    submit.render_semaphore_wait[1].semaphore = output.drawings_sync.handle;
    submit.render_semaphore_wait[0].stage_mask = vk::PipelineStageFlags2::VERTEX_INPUT;
    submit.render_semaphore_wait[1].value = output.drawings_sync.value + 1;

    submit.render_semaphore_signal[0].semaphore = output.drawings_sync.handle;
    submit.render_semaphore_signal[0].value = output.drawings_sync.value + 2;

    submit.render_semaphore_signal[1].semaphore = output.output_present_ready;

    output.drawings_sync.value += 2;
}

