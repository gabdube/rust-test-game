use loomz_shared::CommonError;
use loomz_engine_core::descriptors::DescriptorsAllocator;
use super::{WorldModule, WorldBatch, data::WorldActorData, ACTOR_BATCH_LAYOUT_ID, LAYOUT_COUNT};

pub(super) struct WorldBatcher<'a> {
    current_view: vk::ImageView,
    batches: &'a mut Vec<WorldBatch>,
    instances: &'a [WorldActorData],
    descriptors: &'a mut DescriptorsAllocator<LAYOUT_COUNT>,
    sampler: vk::Sampler,
    instance_index: usize,
    batch_index: usize,
}

impl<'a> WorldBatcher<'a> {

    pub(super) fn batch_all(world: &'a mut WorldModule) -> Result<(), CommonError> {
        let mut batcher = WorldBatcher {
            current_view: vk::ImageView::null(),
            batches: &mut world.render.actors.batches,
            instances: &world.data.actors_data,
            descriptors: &mut world.resources.descriptors,
            sampler: world.resources.default_sampler,
            instance_index: 0,
            batch_index: 0,
        };

        batcher.reset_batches();
        batcher.first_batch()?;
        batcher.remaining_batches()?;

        Ok(())
    }

    fn reset_batches(&mut self) {
        self.descriptors.reset_layout::<ACTOR_BATCH_LAYOUT_ID>();
        self.batches.clear();
    }

    fn first_batch(&mut self) -> Result<(), CommonError> {
        let mut found = false;
        let max_instance = self.instances.len();

        while !found && self.instance_index != max_instance {
            let instance = self.instances[self.instance_index];
            if instance.image_view.is_null() {
                // Sprite is not renderable
                self.instance_index += 1;
                continue;
            }

            self.next_batch(instance.image_view)?;
            self.batches[self.batch_index].instances_count += 1;
            self.instance_index += 1;
            found = true;
        }

        Ok(())
    }

    fn remaining_batches(&mut self) -> Result<(), CommonError> {
        let max_instance = self.instances.len();
        while self.instance_index != max_instance {
            let instance = self.instances[self.instance_index];
            if instance.image_view.is_null() {
                // Sprite is not renderable
                self.instance_index += 1;
                continue;
            }

            let image_view = instance.image_view;
            if self.current_view != image_view {
                self.next_batch(image_view)?;
                self.batch_index += 1;
            }

            self.batches[self.batch_index].instances_count += 1;
            self.instance_index += 1;
        }

        Ok(())
    }

    fn next_batch(&mut self, image_view: vk::ImageView) -> Result<(), CommonError> {
        use loomz_engine_core::descriptors::DescriptorWriteBinding;

        let set = self.descriptors.write_set::<ACTOR_BATCH_LAYOUT_ID>(&[
            DescriptorWriteBinding::from_image_and_sampler(
                image_view,
                self.sampler,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            )
        ])?;

        self.current_view = image_view;
        self.batches.push(WorldBatch {
            set,
            instances_count: 0,
            instances_offset: self.instance_index as u32,
        });

        Ok(())
    }

}
