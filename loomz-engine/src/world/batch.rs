use loomz_shared::CommonError;
use super::{WorldModule, WorldBatch, WorldInstance, WorldDescriptors};

pub(super) struct WorldBatcher<'a> {
    current_view: vk::ImageView,
    batches: &'a mut Vec<WorldBatch>,
    instances: &'a mut [WorldInstance],
    descriptors: &'a mut WorldDescriptors,
    instance_index: usize,
    batch_index: usize,
}

impl<'a> WorldBatcher<'a> {

    pub(super) fn build(world: &'a mut WorldModule) -> Result<(), CommonError> {
        let mut batcher = WorldBatcher {
            current_view: vk::ImageView::null(),
            batches: &mut world.batches,
            instances: &mut world.data.instances,
            descriptors: &mut world.descriptors,
            instance_index: 0,
            batch_index: 0,
        };

        batcher.descriptors.reset_batch_layout();
        batcher.batches.clear();

        batcher.first_batch()?;
        batcher.remaining_batches()?;

        Ok(())
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
        let set = self.descriptors.write_batch_texture(image_view)?;

        self.current_view = image_view;
        self.batches.push(WorldBatch {
            set,
            instances_count: 0,
            instances_offset: self.instance_index as u32,
        });

        Ok(())
    }

}
