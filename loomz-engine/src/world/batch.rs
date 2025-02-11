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

pub(super) fn batch_terrain(world: &mut WorldModule) {
    use loomz_shared::api::{TERRAIN_CHUNK_STRIDE, TERRAIN_CHUNK_SIZE};

    assert!(TERRAIN_CHUNK_STRIDE == 16, "This function assumes the chunk stride is 16");

    let sprites = &mut world.data.terrain_sprites;
    let chunks = &world.data.terrain_chunks;
    let view = world.data.world_view;
    let mut batches_count = 0;

    world.render.terrain.batches.clear();

    for chunk in chunks {
        if !view.intersects(&chunk.view) {
            continue;
        }

        let mut i = TERRAIN_CHUNK_SIZE * batches_count;
        for row in chunk.cells.iter() {
            sprites.write(i+0, row[0]);
            sprites.write(i+1, row[1]);
            sprites.write(i+2, row[2]);
            sprites.write(i+3, row[3]);
            sprites.write(i+4, row[4]);
            sprites.write(i+5, row[5]);
            sprites.write(i+6, row[6]);
            sprites.write(i+7, row[7]);
            sprites.write(i+8, row[8]);
            sprites.write(i+9, row[9]);
            sprites.write(i+10, row[10]);
            sprites.write(i+11, row[11]);
            sprites.write(i+12, row[12]);
            sprites.write(i+13, row[13]);
            sprites.write(i+14, row[14]);
            sprites.write(i+15, row[15]);
            i += 16;
        }

        batches_count += 1;
        world.render.terrain.batches.push([chunk.view.left, chunk.view.top]);
    }
}
