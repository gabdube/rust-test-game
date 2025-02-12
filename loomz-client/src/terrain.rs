use bitflags::bitflags;
use loomz_shared::api::{TerrainType, WorldTerrainChunk, TERRAIN_CHUNK_STRIDE};
use loomz_shared::{LoomzApi, RectF32, SizeU32, rect};

bitflags! {
    #[derive(Copy, Clone, Default)]
    struct TerrainUpdateFlags: u8 {
        const UPDATE_VIEW       = 0b0001;
        const UPDATE_SIZE       = 0b0010;
    }
}

pub struct Terrain {
    /// Terrain cells
    batches: Vec<WorldTerrainChunk>,
    /// Indices of the batches that were updated 
    batches_updates: Vec<usize>,    
    view: RectF32,
    size: SizeU32,
    flags: TerrainUpdateFlags,
}

impl Terrain {

    pub fn init() -> Terrain {
        Terrain {
            batches: Vec::with_capacity(16),
            batches_updates: Vec::with_capacity(16),
            view: RectF32::default(),
            size: SizeU32::default(),
            flags: TerrainUpdateFlags::empty(),
        }
    }

    // Resize the world. Clears all the existing data.
    pub fn set_world_size(&mut self, width: u32, height: u32) {
        self.size.width = width;
        self.size.height = height;

        let batch_x = ((width as usize) + (TERRAIN_CHUNK_STRIDE-1)) / TERRAIN_CHUNK_STRIDE;
        let batch_y = ((height as usize) + (TERRAIN_CHUNK_STRIDE-1)) / TERRAIN_CHUNK_STRIDE;
        self.batches.clear();
        self.batches_updates.clear();
        self.flags |= TerrainUpdateFlags::UPDATE_SIZE;

        for y in 0..batch_y {
            for x in 0..batch_x {
                self.batches_updates.push(self.batches.len());
                self.batches.push(WorldTerrainChunk::new(x, y))
            }
        }
    }

    pub fn set_view(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.view = rect(x, y, x+width, y+height);
        self.flags |= TerrainUpdateFlags::UPDATE_VIEW;
    }

    // Copy cells into the target rect. Cells buffer must match the rect
    pub fn set_cells(&mut self, x: u32, y: u32, width: u32, height: u32, cells: &[TerrainType]) {
        let total_cells = (width * height) as usize;

        assert!(total_cells == cells.len(), "Cells count does not fit copy rect");
        assert!(width > 0 && height > 0, "Size must be greater than 0");
        assert!(x + width <= self.size.width && y + height <= self.size.height, "Rectangle is out of terrain range");

        let chunk_stride = TERRAIN_CHUNK_STRIDE as u32;
        let stride = self.size.width / chunk_stride;
        let mut cell_index = 0usize;

        for cell_y in y..(y+height) {
            let batch_y = cell_y / chunk_stride;
            let local_y = cell_y - (batch_y * chunk_stride);

            let mut cell_x = x;
            let mut cells_to_copy = width;
            while cells_to_copy > 0 {
                let batch_x = cell_x / chunk_stride;
                let batch_index = ((batch_y * stride) + batch_x) as usize;
                if !self.batches_updates.iter().any(|&i| i == batch_index) {
                    self.batches_updates.push(batch_index);
                }

                let local_x = cell_x - (batch_x * chunk_stride);
                let cell_count = u32::min(chunk_stride - local_x, cells_to_copy) as usize; 
                let cells_src = &cells[cell_index..(cell_index+cell_count)];

                //println!("{:?} {:?} {:?} {:?}", [batch_x, batch_y], batch_index, [local_x, local_y], cell_count);

                self.set_row_inner(
                    batch_index,
                    local_y as usize,
                    local_x as usize,
                    cells_src
                );

                cell_index += cell_count;
                cells_to_copy -= cell_count as u32;
                cell_x += cell_count as u32;
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_cell(&mut self, x: u32, y: u32) -> TerrainType {
        let chunk_stride = TERRAIN_CHUNK_STRIDE as u32;
        let stride = self.size.width / chunk_stride;
        let batch_x = x / chunk_stride;
        let batch_y = y / chunk_stride;
        
        let batch_index =  ((batch_y * stride) + batch_x) as usize;
        let local_x = x - (batch_x * chunk_stride);
        let local_y = y - (batch_y * chunk_stride);

        self.batches[batch_index].cells[local_y as usize][local_x as usize]
    }

    #[inline(always)]
    fn set_row_inner(&mut self, batch_index: usize, column_index: usize, row_offset: usize, cells: &[TerrainType]) {
        let column = &mut self.batches[batch_index].cells[column_index];
        unsafe {
            ::std::ptr::copy_nonoverlapping(
                cells.as_ptr(),
                column.as_mut_ptr().add(row_offset),
                cells.len()
            );
        }
    }

    pub fn sync(&mut self, api: &LoomzApi) {
        let view = self.view;
        let world = api.world();

        if self.flags.contains(TerrainUpdateFlags::UPDATE_SIZE) {
            world.set_world_size(self.size);
        }

        if self.flags.contains(TerrainUpdateFlags::UPDATE_VIEW) {
            world.set_world_view(view);
        }

        for &batch_index in self.batches_updates.iter() {
            let batch = &self.batches[batch_index];
            if view.intersects(&batch.view) {
                world.update_terrain(&batch);
            }
        }

        self.batches_updates.clear();
        self.flags = TerrainUpdateFlags::empty();
    }

}

impl loomz_shared::store::StoreAndLoad for Terrain {
    fn load(reader: &mut loomz_shared::store::SaveFileReaderBase) -> Self {
        let mut terrain = Terrain::init();
        terrain.view = reader.read();
        terrain.size = reader.read();
        terrain.flags = TerrainUpdateFlags::from_bits(reader.read_u32() as u8).unwrap_or_default();
        terrain.batches = reader.read_slice().to_vec();
        terrain
    }

    fn store(&self, writer: &mut loomz_shared::store::SaveFileWriterBase) {
        writer.write(&self.view);
        writer.write(&self.size);
        writer.write_u32(self.flags.bits() as u32);
        writer.write_slice(&self.batches);
    }
}

#[cfg(test)]
mod terrain_tests {
    use super::*;

    #[test]
    fn set_cells() {
        let mut terrain = Terrain::init();
        terrain.set_world_size(32, 32);

        terrain.set_cells(1, 1, 3, 1, &[TerrainType::Sand; 3]);
        assert!(terrain.batches[0].cells[1][0] == TerrainType::Grass);
        assert!(terrain.batches[0].cells[1][1] == TerrainType::Sand);
        assert!(terrain.batches[0].cells[1][2] == TerrainType::Sand);
        assert!(terrain.batches[0].cells[1][3] == TerrainType::Sand);
        assert!(terrain.batches[0].cells[1][4] == TerrainType::Grass);

        terrain.set_cells(15, 15, 2, 2, &[TerrainType::Sand; 4]);
        assert!(terrain.batches[0].cells[15][15] == TerrainType::Sand);
        assert!(terrain.batches[1].cells[15][0] == TerrainType::Sand);
        assert!(terrain.batches[2].cells[0][15] == TerrainType::Sand);
        assert!(terrain.batches[3].cells[0][0] == TerrainType::Sand);
    }

}
