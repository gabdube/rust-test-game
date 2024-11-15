use std::marker::PhantomData;
use vk::wrapper::Device;
use loomz_shared::{backend_err, CommonError};

use crate::LoomzEngineCore;


pub const KB: vk::DeviceSize = 1024;
pub const MB: vk::DeviceSize = KB*1000;
pub const GB: vk::DeviceSize = MB*1000;

/// A memory range in a memory allocation
/// Most significant bit in base means the allocation was freed
#[derive(Copy, Clone)]
pub struct MemoryRange {
    base: vk::DeviceSize,
    offset: vk::DeviceSize,
    size: vk::DeviceSize,
}

/// Memory allocator for gpu resources that are not recreated often
pub struct DeviceMemoryAlloc {
    pub handle: vk::DeviceMemory,
    pub size: vk::DeviceSize,
    pub next_offset: vk::DeviceSize,
    pub allocations: Vec<MemoryRange>,
}

impl DeviceMemoryAlloc {

    pub fn new(device: &Device, size: vk::DeviceSize, alloc_capacity: usize, memory_type_index: u32) -> Result<Self, vk::VkResult> {
        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: size,
            memory_type_index,
            ..Default::default()
        };
        let handle = device.allocate_memory(&alloc_info)?;
            
        let alloc = DeviceMemoryAlloc {
            handle,
            size,
            next_offset: 0,
            allocations: Vec::with_capacity(alloc_capacity)
        };

        Ok(alloc)
    }

    pub fn free(self, device: &Device) {
        device.free_memory(self.handle);
    }

    pub fn allocate_memory(&mut self, info: &vk::MemoryRequirements) -> vk::DeviceSize {
        let aligned_offset = crate::helpers::align_device(self.next_offset, info.alignment);
        let next_offset = aligned_offset + info.size;
        if next_offset > self.size {
            panic!("TODO: reallocate memory");
        }

        self.allocations.push(MemoryRange {
            base: self.next_offset,
            offset: aligned_offset,
            size: info.size,
        });

        self.next_offset = next_offset;
        
        aligned_offset
    }

    pub fn free_memory(&mut self, offset: vk::DeviceSize) {
        let index = self.allocations.iter().position(|alloc| alloc.offset == offset );
        match index {
            Some(i) => { self.allocations[i].base |= 0b1 << 63; }
            None => { eprintln!("Allocation at offset {offset} was not found in device memory"); }
        }
    }

}

impl Default for DeviceMemoryAlloc {

    fn default() -> Self {
        DeviceMemoryAlloc {
            handle: vk::DeviceMemory::null(),
            size: 0,
            next_offset: 0,
            allocations: Vec::new(),
        }
    }

}

/// A buffer that combines 32 bits indices and interleaved vertex attributes.
/// Backed by device memory.
pub struct VertexAlloc<V: Copy> {
    pub buffer: vk::Buffer,
    pub offset: vk::DeviceSize,
    pub index_capacity: u32,
    pub vertex_capacity: u32,
    data: PhantomData<V>
}

impl<V: Copy> VertexAlloc<V> {

    pub fn new(core: &mut LoomzEngineCore, index_capacity: u32, vertex_capacity: u32) -> Result<Self, CommonError> {
        let device = &core.ctx.device;
        
        let index_size = index_capacity * (size_of::<u32>() as u32);
        let vertex_size = vertex_capacity * (size_of::<V>() as u32);
        let total_size = index_size + vertex_size;

        let buffer_info = vk::BufferCreateInfo {
            size: total_size as _,
            usage: vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            ..Default::default()
        };

        let buffer = device.create_buffer(&buffer_info)
            .map_err(|err| backend_err!("Failed to create vertex buffer: {err}") )?;

        let buffer_req = device.get_buffer_memory_requirements(buffer);
        
        let vertex_alloc = &mut core.resources.vertex_alloc;
        let offset = vertex_alloc.allocate_memory(&buffer_req);
        device.bind_buffer_memory(buffer, vertex_alloc.handle, offset)
            .map_err(|err| backend_err!("Failed to bind vertex buffer memory: {err}") )?;

        Ok(VertexAlloc {
            buffer,
            offset,
            index_capacity,
            vertex_capacity,
            data: PhantomData
        })
    }

    pub fn free(&self, core: &mut LoomzEngineCore) {
        let device = &core.ctx.device;
        core.resources.vertex_alloc.free_memory(self.offset);
        device.destroy_buffer(self.buffer);
    }
    
    pub fn index_offset(&self) -> vk::DeviceSize {
        0
    }

    pub fn vertex_offset(&self) -> [vk::DeviceSize; 1] {
       [(self.index_capacity as vk::DeviceSize) * (size_of::<V>() as vk::DeviceSize)]
    } 

}

impl<V: Copy> Default for VertexAlloc<V> {
    fn default() -> Self {
        VertexAlloc {
            buffer: vk::Buffer::null(),
            offset: 0,
            index_capacity: 0,
            vertex_capacity: 0,
            data: PhantomData
        }
    }
}
