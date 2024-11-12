use loomz_shared::{render_record_err, present_err, CommonError};
use super::LoomzEngine;


pub(crate) fn submit(engine: &mut LoomzEngine) -> Result<(), CommonError> {
    submit_commands(engine)?;
    present(engine)?;
    Ok(())
}

fn submit_commands(engine: &mut LoomzEngine) -> Result<(), CommonError> {
    let submit = &mut engine.submit;

    // Note: submit_infos is set in `prepare::prepare_submit`
    engine.ctx.extensions.synchronization2.queue_submit2(submit.graphics_queue, &submit.submit_infos, vk::Fence::null())
        .map_err(|err| render_record_err!("Failed to submit drawing commands: {err}")  )?;

    Ok(())
}

fn present(engine: &mut LoomzEngine) -> Result<(), CommonError> {
    let mut result = vk::VkResult(0);
    let output = &mut engine.output;
    let image_index = output.acquired_image_index;

    let present_info = vk::PresentInfoKHR {
        swapchain_count: 1,
        p_swapchains:    &output.swapchain,
        p_image_indices: &image_index,

        wait_semaphore_count: 1,
        p_wait_semaphores: &output.output_present_ready,

        p_results:         &mut result,
        ..Default::default()
    };

    let present_result = engine.ctx.extensions.swapchain.queue_present(output.queue, &present_info);
    let mut final_present_result = Ok(());

    match present_result {
        Ok(_) => { },
        Err(vk::VkResult::SUBOPTIMAL_KHR) => {
            output.rebuild = true;
        },
        Err(error) => {
            final_present_result = Err(present_err!("Queue present CALL failed: {error}"));
        }
    }

    match result.as_result() {
        Ok(_) => {},
        Err(vk::VkResult::SUBOPTIMAL_KHR) => {
            output.rebuild = true;
        },
        Err(error) =>  {
            final_present_result = match final_present_result {
                Ok(()) => Err(present_err!("Queue present failed: {error}")),
                old => old
            };
        }
    }

    final_present_result
}
