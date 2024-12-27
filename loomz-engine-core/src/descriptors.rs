use std::sync::Arc;
use parking_lot::Mutex;
use loomz_shared::{backend_err, CommonError};
use crate::alloc::StorageAlloc;
use crate::{LoomzEngineCore, VulkanDescriptorSubmit};

pub struct DescriptorsAllocation<'a> {
    pub layout: vk::DescriptorSetLayout,
    pub binding_types: &'a [vk::DescriptorType],
    pub count: u32,
}

#[derive(Copy, Clone)]
pub enum DescriptorWriteBinding {
    Image { image_view: vk::ImageView, sampler: vk::Sampler, image_layout: vk::ImageLayout },
    Buffer { buffer: vk::Buffer, offset: vk::DeviceSize, range: vk::DeviceSize },
}

impl DescriptorWriteBinding {
    pub fn from_storage_buffer<T: Copy>(buffer: &StorageAlloc<T>) -> Self {
        DescriptorWriteBinding::Buffer { 
            buffer: buffer.handle(),
            offset: buffer.offset(),
            range: buffer.bytes_range(),
        }
    }

    pub fn from_image_and_sampler(image_view: vk::ImageView, sampler: vk::Sampler, image_layout: vk::ImageLayout) -> Self {
        DescriptorWriteBinding::Image {
            image_view,
            sampler,
            image_layout,
        }
    }
}

/// A collection of preallocated descriptor set in a [DescriptorsAllocator]
pub struct DescriptorSetsCollection {
    next: usize,
    binding_types: Box<[vk::DescriptorType]>,
    sets: Box<[vk::DescriptorSet]>,
}

/**
    A structure that can allocate and update descriptor sets
*/
pub struct DescriptorsAllocator<const C: usize> {
    pool: vk::DescriptorPool,
    collections: Option<Box<[DescriptorSetsCollection; C]>>,
    updates: Option<Arc<Mutex<VulkanDescriptorSubmit>>>,
}

impl<const C: usize> DescriptorsAllocator<C> where
    [DescriptorSetsCollection; C]: Default
{

    pub fn new(
        core: &mut LoomzEngineCore,
        allocations: &[DescriptorsAllocation; C],
    ) -> Result<Self, CommonError> {
        let mut max_sets = 0;
        let mut pool_size_count = 0;
        let mut pool_sizes = [vk::DescriptorPoolSize::default(); 10];
        Self::compile_allocations(allocations, &mut max_sets, &mut pool_size_count, &mut pool_sizes)?;

        let mut pool = vk::DescriptorPool::null();
        Self::create_descriptor_pool(core, max_sets, &pool_sizes[0..pool_size_count], &mut pool)?;

        let mut collections: Box<[DescriptorSetsCollection; C]> = Box::default();
        Self::allocate_collections(core, pool, allocations, &mut collections)?;

        let alloc = DescriptorsAllocator {
            pool,
            collections: Some(collections),
            updates: Some(Arc::clone(&core.descriptors))
        };

        Ok(alloc)
    }

    pub fn destroy(self, core: &mut LoomzEngineCore) {
        core.ctx.device.destroy_descriptor_pool(self.pool);
    }

    /// Locally "resets" the descriptor allocations for layout `C2`, allowing them to be used by `write_set`
    /// Layout must be within `C` collections bounds.
    pub fn reset_layout<const C2: u32>(&mut self) {
        assert!((C2 as usize) < C, "Descriptor alloc was created with {} layouts but access to index {} was requested", C, C2);
        assert!(self.collections.is_some(), "Descriptor alloc was not initialized");
        
        let collections = self.collections.as_mut().unwrap();
        collections[C2 as usize].next = 0;
    }

    /// Writes and returns a new descriptor set from the layout at index `C2`
    /// Layout must be within `C` collections bounds.
    /// Bindings type must match the layout bindings.
    /// Returns a backend error if not more sets can be allocated
    pub fn write_set<const C2: u32>(&mut self, binding_writes: &[DescriptorWriteBinding]) -> Result<vk::DescriptorSet, CommonError> {
        assert!((C2 as usize) < C, "Descriptor alloc was created with {} layouts but access to index {} was requested", C, C2);
        assert!(self.collections.is_some(), "Descriptor alloc was not initialized");

        let updates = self.updates.as_ref().unwrap();
        let mut updates_guard = updates.lock();

        let collections = self.collections.as_mut().unwrap();
        let collection = &mut collections[C2 as usize];
        if collection.next == collection.sets.len() {
            return Err(backend_err!("No more descriptor sets in layout {C2}"));
        }

        let set = collection.sets[collection.next];
        for (binding_id, binding_write) in binding_writes.iter().enumerate() {
            match *binding_write {
                DescriptorWriteBinding::Buffer { buffer, offset, range } => {
                    updates_guard.write_buffer(
                        set,
                        buffer,
                        offset,
                        range,
                        binding_id as u32,
                        collection.binding_types[binding_id],
                    );
                },
                DescriptorWriteBinding::Image { image_view, sampler, image_layout } => {
                    updates_guard.write_image(
                        set,
                        image_view,
                        sampler,
                        image_layout,
                        binding_id as u32,
                        collection.binding_types[binding_id],
                    );
                }
            }
        }

        collection.next += 1;

        Ok(set)
    }

    //
    // Initial setup
    //

    fn compile_allocations(
        allocations: &[DescriptorsAllocation; C],
        max_sets_: &mut u32,
        pool_size_count_: &mut usize,
        pool_sizes: &mut [vk::DescriptorPoolSize; 10]
    ) -> Result<(), CommonError> {
        let mut pool_size_count = (*pool_size_count_) as usize;
        let mut max_sets = *max_sets_;

        for alloc in allocations {
            max_sets += alloc.count;

            for &descriptor_type in alloc.binding_types {
                let index = pool_sizes[0.. pool_size_count].iter().position(|ps| ps.ty == descriptor_type );
                match index {
                    Some(i) => {
                        pool_sizes[i].descriptor_count += alloc.count;
                    },
                    None => {
                        pool_sizes[pool_size_count].ty = descriptor_type;
                        pool_sizes[pool_size_count].descriptor_count = alloc.count;
                        pool_size_count += 1;
                    }
                }
            }
        }

        if C == 0 || max_sets == 0 {
            return Err(backend_err!("Tried to initialize a descriptor allocator with no capacity"));
        }

        *max_sets_ = max_sets;
        *pool_size_count_ = pool_size_count;

        Ok(())
    }

    fn create_descriptor_pool(
        core: &mut LoomzEngineCore,
        max_sets: u32,
        pool_sizes: &[vk::DescriptorPoolSize],
        pool: &mut vk::DescriptorPool,
    ) -> Result<(), CommonError> {
        let pool_create_info = vk::DescriptorPoolCreateInfo {
            max_sets,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
            ..Default::default()
        };

        *pool = core.ctx.device.create_descriptor_pool(&pool_create_info)
            .map_err(|err| backend_err!("Failed to create descriptor pool: {err}") )?;

        Ok(())
    }

    fn allocate_collections(
        core: &mut LoomzEngineCore,
        descriptor_pool: vk::DescriptorPool,
        allocations: &[DescriptorsAllocation; C],
        collections: &mut [DescriptorSetsCollection; C]
    ) -> Result<(), CommonError> {
        let device = &core.ctx.device;
       
        let mut sets = Vec::with_capacity(32);
        let mut layouts = Vec::with_capacity(32);
        let mut allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool,
            ..Default::default()
        };

        for (alloc, col) in allocations.iter().zip(collections.iter_mut()) {
            for _ in 0..alloc.count {
                sets.push(vk::DescriptorSet::null());
                layouts.push(alloc.layout);
            }

            allocate_info.descriptor_set_count = alloc.count;
            allocate_info.p_set_layouts = layouts.as_ptr();
            device.allocate_descriptor_sets(&allocate_info, &mut sets)
                .map_err(|err| backend_err!("Failed to preallocate descriptor sets: {err}") )?;

            col.sets = sets.clone().into_boxed_slice();
            col.binding_types = alloc.binding_types.to_vec().into_boxed_slice();

            sets.clear();
            layouts.clear();
        }

        Ok(())
    }
}

impl<const C: usize> Default for DescriptorsAllocator<C> {
    fn default() -> Self {
        DescriptorsAllocator {
            pool: vk::DescriptorPool::null(),
            collections: None,
            updates: None,
        }
    }
}

impl Default for DescriptorSetsCollection {
    fn default() -> Self {
        DescriptorSetsCollection {
            next: 0,
            binding_types: Box::new([]),
            sets: Box::new([]),
        }
    }
}
