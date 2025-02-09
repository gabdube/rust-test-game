use super::{WorldModule, WorldBatch, data::WorldActorData};

//
// Actors batching
//

fn actors_groups<'a>(actors: &'a [WorldActorData]) -> impl Iterator<Item=(vk::DescriptorSet, u32)> +'a {
    let mut last_descriptor_set = vk::DescriptorSet::null();
    let mut local_instance_count = 0;
    let mut instance_index = 0;

    ::std::iter::from_fn(move || {
        let mut next_descriptor_set = vk::DescriptorSet::null();
        loop {
            let actor = match actors.get(instance_index) {
                Some(actor) => actor,
                None => { break; }
            };

            if last_descriptor_set.is_null() {
                last_descriptor_set = actor.descriptor_set;
            }
            
            if actor.descriptor_set != last_descriptor_set {
                next_descriptor_set = actor.descriptor_set;
                break;
            }

            local_instance_count += 1;
            instance_index += 1;
        }

        let set = last_descriptor_set;
        let count = local_instance_count;

        local_instance_count = 0;
        last_descriptor_set = next_descriptor_set;
        instance_index += 1;

        if count > 0 {
            Some((set, count))
        } else {
            None
        }
    })
}

pub(super) fn batch_actors(world: &mut WorldModule) {
    let actors = &world.data.actors_data;
    let batches = &mut world.render.actors.batches;
    batches.clear();

    let mut global_instance_count = 0;
    for (descriptor_set, instances_count) in actors_groups(actors) {
        batches.push(WorldBatch {
            set: descriptor_set,
            instances_count,
            instances_offset: global_instance_count,
        });

        global_instance_count += instances_count;
    }
}


//
// Terrain batching
//
