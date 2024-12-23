use loomz_shared::{backend_err, CommonError};
use crate::pipelines::PipelineLayoutSetBinding;
use crate::LoomzEngineCore;

#[derive(Copy, Clone)]
pub enum DescriptorWriteParam {
    Image(vk::DescriptorImageInfo),
    Buffer(vk::DescriptorBufferInfo)
}

#[derive(Copy, Clone)]
pub struct DescriptorWrite {
    pub dst_set: vk::DescriptorSet,
    pub dst_binding: u32,
    pub descriptor_type: vk::DescriptorType,
    pub param: DescriptorWriteParam,
}

#[derive(Copy, Clone, Default)]
pub struct DescriptorWriteImageParams {
    pub sampler: vk::Sampler,
    pub image_layout: vk::ImageLayout,
    pub dst_binding: u32,
    pub descriptor_type: vk::DescriptorType,
}

/// Helpers structure to handle descriptors update
pub struct DescriptorWriteBuffer {
    writes: Vec<DescriptorWrite>,
}

impl DescriptorWriteBuffer {
    pub fn clear(&mut self) {
        self.writes.clear();
    }

    pub fn write_simple_image(&mut self, dst_set: vk::DescriptorSet, image_view: vk::ImageView, params: &DescriptorWriteImageParams) {
        self.writes.push(DescriptorWrite {
            dst_set,
            dst_binding: params.dst_binding,
            descriptor_type: params.descriptor_type,
            param: DescriptorWriteParam::Image(vk::DescriptorImageInfo {
                sampler: params.sampler,
                image_view,
                image_layout: params.image_layout,
            })
        });
    }

    pub fn write_storage_buffer<T: Copy>(&mut self, dst_set: vk::DescriptorSet, store: &crate::alloc::StorageAlloc<T>, dst_binding: u32) {
        self.writes.push(DescriptorWrite {
            dst_set,
            dst_binding,
            descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
            param: DescriptorWriteParam::Buffer(vk::DescriptorBufferInfo {
                buffer: store.buffer,
                offset: 0,
                range: store.bytes_range(),
            })
        });
    }

    pub fn submit(&mut self, engine: &mut LoomzEngineCore) {
        engine.descriptors.updates.append(&mut self.writes);
    }

}

/// Allocation info when creating a new [DescriptorsAllocator]
pub struct DescriptorsAllocation<'a> {
    pub layout: vk::DescriptorSetLayout,
    pub bindings: &'a [PipelineLayoutSetBinding],
    pub count: u32,
}

/// A collection of preallocated descriptor set in a [DescriptorsAllocator]
struct DescriptorSetsCollection {
    next: usize,
    sets: Box<[vk::DescriptorSet]>,
}

/// An utility to allocate vulkan descriptor sets
pub struct DescriptorsAllocator {
    pool: vk::DescriptorPool,
    sets: Vec<DescriptorSetsCollection>,
}

impl DescriptorsAllocator {

    pub fn new(core: &mut LoomzEngineCore, allocations: &[DescriptorsAllocation]) -> Result<Self, CommonError> {
        let device = &core.ctx.device;
        let mut alloc = DescriptorsAllocator::default();

        let mut max_sets = 0;
        let mut pool_size_count = 0;
        let mut pool_sizes = [vk::DescriptorPoolSize::default(); 10];
        Self::compile_allocations(allocations, &mut max_sets, &mut pool_size_count, &mut pool_sizes);

        if max_sets == 0 {
            return Ok(alloc);
        }
    
        let pool_create_info = vk::DescriptorPoolCreateInfo {
            max_sets,
            pool_size_count,
            p_pool_sizes: pool_sizes.as_ptr(),
            ..Default::default()
        };

        alloc.pool = device.create_descriptor_pool(&pool_create_info)
            .map_err(|err| backend_err!("Failed to create descriptor pool: {err}") )?;

        alloc.preallocate_sets(core, allocations)
            .map_err(|err| backend_err!("Failed to preallocate descriptor sets: {err}") )?;

        Ok(alloc)
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        core.ctx.device.destroy_descriptor_pool(self.pool);
    }

    pub fn clear_sets(&mut self, index: u32) {
        let index = index as usize;
        assert!(index < self.sets.len(), "Descriptor alloc was created with {} collections but access to index {} was requested", self.sets.len(), index);
        self.sets[index].next = 0;
    }

    pub fn next_set(&mut self, index: u32) -> vk::DescriptorSet {
        let index = index as usize;
        assert!(index < self.sets.len(), "Descriptor alloc was created with {} collections but access to index {} was requested", self.sets.len(), index);

        let set_collection = &mut self.sets[index];
        assert!(set_collection.next < set_collection.sets.len(), "All descriptor sets in set collection {} were allocated", index);

        let descriptor_set = set_collection.sets[set_collection.next];
        set_collection.next += 1;

        descriptor_set
    }

    fn compile_allocations(
        allocations: &[DescriptorsAllocation],
        max_sets_: &mut u32,
        pool_size_count_: &mut u32,
        pool_sizes: &mut [vk::DescriptorPoolSize; 10]
    ) {
        let mut pool_size_count = (*pool_size_count_) as usize;
        let mut max_sets = *max_sets_;

        for alloc in allocations {
            max_sets += alloc.count;

            for binding in alloc.bindings {
                let index = pool_sizes[0.. pool_size_count].iter().position(|ps| ps.ty == binding.descriptor_type );
                match index {
                    Some(i) => {
                        pool_sizes[i].descriptor_count += binding.descriptor_count * alloc.count;
                    },
                    None => {
                        pool_sizes[pool_size_count].ty = binding.descriptor_type;
                        pool_sizes[pool_size_count].descriptor_count = binding.descriptor_count * alloc.count;
                        pool_size_count += 1;
                    }
                }
            }
        }

        *max_sets_ = max_sets;
        *pool_size_count_ = pool_size_count as u32;
    }

    fn preallocate_sets(&mut self, core: &mut LoomzEngineCore, allocations: &[DescriptorsAllocation]) -> Result<(), vk::VkResult> {
        let device = &core.ctx.device;
       
        let mut allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: self.pool,
            ..Default::default()
        };

        for alloc in allocations.iter() {
            let max = alloc.count as usize;
            let mut sets = vec![vk::DescriptorSet::null(); max];
            let mut layouts = vec![vk::DescriptorSetLayout::null(); max];

            for i in 0..max {
                layouts[i] = alloc.layout;
            }

            allocate_info.descriptor_set_count = alloc.count;
            allocate_info.p_set_layouts = layouts.as_ptr();
            device.allocate_descriptor_sets(&allocate_info, &mut sets)?;

            self.sets.push(DescriptorSetsCollection {
                next: 0,
                sets: sets.clone().into_boxed_slice(),
            });
        }

        Ok(())
    }

}

impl Default for DescriptorsAllocator {

    fn default() -> Self {
        DescriptorsAllocator {
            pool: vk::DescriptorPool::null(),
            sets: Vec::with_capacity(3),
        }
    }

}

impl Default for DescriptorWriteBuffer {
    fn default() -> Self {
        DescriptorWriteBuffer {
            writes: Vec::with_capacity(16),
        }
    }
}
