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
            panic!("TODO: reallocate memory, {}({}) > {}", next_offset, info.size, self.size);
        }

        self.allocations.push(MemoryRange {
            base: self.next_offset,
            offset: aligned_offset,
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

/**
    Memory allocation for uniforms/storage data. Builds a single buffer that maps a HOST VISIBLE memory allocation.
    [DeviceBox] values are suballocated in that buffer.
*/
pub struct StorageMemoryAlloc {
    pub memory: vk::DeviceMemory,
    pub buffer: vk::Buffer,
    pub size: vk::DeviceSize,
    pub mapped_data: Option<*mut u8>,
    pub next_offset: vk::DeviceSize,
}

impl StorageMemoryAlloc {
    pub fn new(device: &Device, size: vk::DeviceSize, memory_type_index: u32) -> Result<Self, vk::VkResult> {
        let buffer_info = vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::UNIFORM_BUFFER,
            ..Default::default()
        };
        let buffer = device.create_buffer(&buffer_info)?;
        let buffer_req = device.get_buffer_memory_requirements(buffer);

        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: buffer_req.size,
            memory_type_index,
            ..Default::default()
        };
        let memory = device.allocate_memory(&alloc_info)?;
        let mapped_data = device.map_memory(memory, 0, size as _)?;

        device.bind_buffer_memory(buffer, memory, 0)?;

        let alloc = StorageMemoryAlloc {
            memory,
            buffer,
            size,
            mapped_data: Some(mapped_data as *mut u8),
            next_offset: 0,
        };

        Ok(alloc)
    }

    pub fn free(mut self, device: &Device) {
        self.mapped_data = None;
        device.destroy_buffer(self.buffer);
        device.unmap_memory(self.memory);
        device.free_memory(self.memory);
    }

    pub fn allocate_slice<V: Copy>(&mut self, alignment: vk::DeviceSize, capacity: usize) -> *mut V {
        let data_size = (capacity * size_of::<V>()) as vk::DeviceSize;
        let aligned_offset = crate::helpers::align_device(self.next_offset, alignment);
        let next_offset = aligned_offset + data_size;
        if next_offset > self.size {
            panic!("TODO: reallocate memory, {}({}) > {}", next_offset, next_offset, self.size);
        }

        self.next_offset = next_offset;

        unsafe { self.mapped_data.unwrap().add(aligned_offset as usize) as *mut V }
    }

}

impl Default for StorageMemoryAlloc {

    fn default() -> Self {
        StorageMemoryAlloc {
            memory: vk::DeviceMemory::null(),
            buffer: vk::Buffer::null(),
            size: 0,
            mapped_data: None,
            next_offset: 0,
        }
    }

}


/// A buffer that combines 32 bits indices and interleaved vertex attributes.
/// Backed by device memory.
pub struct VertexAlloc<V: Copy> {
    pub buffer: vk::Buffer,
    offset: vk::DeviceSize,
    index_capacity: u32,
    vertex_capacity: u32,
    data: PhantomData<V>
}

impl<V: Copy> VertexAlloc<V> {

    pub fn new(core: &mut LoomzEngineCore, index_capacity: u32, vertex_capacity: u32) -> Result<Self, CommonError> {
        const BUFFER_ALIGN: vk::DeviceSize = 64;
        
        let device = &core.ctx.device;
        
        let mut index_size = index_capacity * (size_of::<u32>() as u32);
        index_size = crate::helpers::align_device(index_size as _, BUFFER_ALIGN) as u32;

        let mut vertex_size = vertex_capacity * (size_of::<V>() as u32);
        vertex_size = crate::helpers::align_device(vertex_size as _, BUFFER_ALIGN) as u32;

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
        let index_size_bytes = (self.index_capacity as vk::DeviceSize) * (size_of::<u32>() as vk::DeviceSize);
        [index_size_bytes]
    } 

    pub fn set_data(&self, core: &mut LoomzEngineCore, index: &[u32], vertex: &[V]) {
        if (self.index_capacity as usize) < index.len() || (self.vertex_capacity as usize) < vertex.len() {
            dbg!("Warning vertex buffer capacity is not large enough to upload data. Data will be truncated");
            dbg!(self.index_capacity, index.len());
            dbg!(self.vertex_capacity, vertex.len());
        }

        if index.len() == 0 || vertex.len() == 0 {
            dbg!("Warning tried to set an empty data set to vertex buffer");
            return;
        }

        let index_count = vk::DeviceSize::min(index.len() as _, self.index_capacity as _);
        let vertex_count = vk::DeviceSize::min(vertex.len() as _, self.vertex_capacity as _);
        let index_offset = self.index_offset();
        let vertex_offset = self.vertex_offset()[0];

        let src_index_offset = core.staging.copy_data(index);
        let src_vertex_offset = core.staging.copy_data(vertex);

        let index_copy = vk::BufferCopy {
            size: index_count * (size_of::<u32>() as vk::DeviceSize),
            src_offset: src_index_offset,
            dst_offset: index_offset,
        };

        let vertex_copy = vk::BufferCopy {
            size: vertex_count * (size_of::<V>() as vk::DeviceSize),
            src_offset: src_vertex_offset,
            dst_offset: vertex_offset
        };

        core.staging.vertex_buffer_copy(self.buffer, index_copy);
        core.staging.vertex_buffer_copy(self.buffer, vertex_copy);
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


/** 
    A slice of data allocated on GPU visible memory.
*/
pub struct DeviceSlice<V> {
    capacity: usize,
    ptr: *mut V
}

impl<V: Default+Copy> DeviceSlice<V> {

    /// Allocate the slice and initialize the values with the types Default value
    pub fn new_default(core: &mut LoomzEngineCore, capacity: usize) -> Self {
        let storage_align = core.info.storage_min_align as vk::DeviceSize;
        let ptr = core.resources.storage_alloc.allocate_slice::<V>(storage_align, capacity);
        let default = V::default();
        for i in 0..capacity {
            unsafe { ptr.add(i).write(default); }
        }
        DeviceSlice { capacity, ptr }
    }

}

impl<V> DeviceSlice<V> {
    /// The total size of the slice
    pub fn range(&self) -> usize {
        size_of::<V>() * self.capacity
    }

    pub fn ptr(&self) -> *mut V {
        self.ptr
    }

    pub fn write(&mut self, index: usize, data: V) {
        assert!(index < self.capacity, "Index is outside capacity");
        unsafe { self.ptr.add(index).write(data); }
    }
}

impl<V> Default for DeviceSlice<V> {
    fn default() -> Self {
        DeviceSlice {
            capacity: 0,
            ptr: ::std::ptr::null_mut(),
        }
    }
}

//     pub fn new(core: &mut LoomzEngineCore) -> Result<Self, CommonError> {
//         let storage_align = core.info.storage_min_align as vk::DeviceSize;
//         let total_size = capacity * size_of::<V>();
//         let ptr = core.resources.storage_alloc.allocate_memory::<V>();
//         Ok(StorageAlloc{
//             ptr
//         })
//     }

//     pub fn free(&self, core: &mut LoomzEngineCore) {
//         let device = &core.ctx.device;
//         core.resources.uniforms_alloc.free_memory(self.offset);
//         device.destroy_buffer(self.buffer);
//     }

//     pub fn write_data(&mut self, index: usize, data: V) {
//         assert!(index < self.capacity, "Tried to write data outside of allocated storage: ({} > {}) ", index, self.capacity);
//         assert!(self.mapped_data.is_some(), "Buffer data is not mapped");
//         let mapped_data_ptr = self.mapped_data.unwrap();
        
//         // Safety, index is not out of bound
//         unsafe {
//             mapped_data_ptr.add(index).write(data);
//         }
//     }

//     pub fn handle(&self) -> vk::Buffer {
//         self.buffer
//     }

//     pub fn bytes_range(&self) -> vk::DeviceSize {
//         (self.capacity as vk::DeviceSize) * (size_of::<V>() as vk::DeviceSize)
//     }
// }
