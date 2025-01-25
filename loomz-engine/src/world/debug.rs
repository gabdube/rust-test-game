use loomz_shared::api::WorldDebugFlags;
use loomz_shared::{RectF32, RgbaU8, rgb, rect};
use loomz_engine_core::LoomzEngineCore;
use super::{WorldModule, WorldDebugVertex};

impl WorldModule {
    pub(super) fn build_debug_data(&mut self, core: &mut LoomzEngineCore) {
        let grid = self.resources.grid_data;
        if grid.screen_size.width == 0.0 || grid.screen_size.height == 0.0 {
            // Not yet fully initialized
            return;
        }

        let red = rgb(161, 0, 0);
        let blue = rgb(0, 24, 104);
        let half_size = grid.cell_size * 0.5;
        let show_main = self.debug.contains(WorldDebugFlags::SHOW_MAIN_GRID);
        let show_sub = self.debug.contains(WorldDebugFlags::SHOW_SUB_GRID);

        let debug_vertex = &mut self.resources.debug_vertex;

        let mut index = Vec::with_capacity(500);
        let mut vertex = Vec::with_capacity(1000);
        let mut index_count = 0;
        let mut vertex_count = 0;

        let mut line;

        if show_main {
            let mut x = 0.0;
            let mut y = 0.0;
            while x < grid.screen_size.width {
                line = rect(x-0.5, 0.0, x+0.5, grid.screen_size.height);
                write_indices(&mut index, vertex_count);
                write_vertex(&mut vertex, line, red, &mut vertex_count);

                x += grid.cell_size;
                index_count += 6;
            }
    
            while y < grid.screen_size.height {
                line = rect(0.0, y-0.5, grid.screen_size.width, y+0.5);
                write_indices(&mut index, vertex_count);
                write_vertex(&mut vertex, line, red, &mut vertex_count);

                y += grid.cell_size;
                index_count += 6;
            }
        }

        if show_sub {
            let mut x = 0.0;
            let mut y = 0.0;
            while x < grid.screen_size.width {
                line = rect(x+half_size-0.5, 0.0, x+half_size+0.5, grid.screen_size.height);
                write_indices(&mut index, vertex_count);
                write_vertex(&mut vertex, line, blue, &mut vertex_count);
                
                x += grid.cell_size;
                index_count += 6;
            }
    
            while y < grid.screen_size.height {
                line = rect(0.0, y+half_size-0.5, grid.screen_size.width, y+half_size+0.5);
                write_indices(&mut index, vertex_count);
                write_vertex(&mut vertex, line, blue, &mut vertex_count);
                
                y += grid.cell_size;
                index_count += 6;
            }
        }

        if index_count > 0 {
            debug_vertex.set_data(core, &index, &vertex);
            self.render.debug.index_count = index_count;
        }
    }

    pub(super) fn toggle_debug(&mut self, core: &mut LoomzEngineCore) {
        self.render.debug.index_count = 0;
        if self.debug.is_empty() {
            return;
        }

        self.build_debug_data(core);
    }

}

fn write_indices(index: &mut Vec<u32>, i: u32) {
    index.push(i+0);
    index.push(i+1);
    index.push(i+2);
    index.push(i+2);
    index.push(i+3);
    index.push(i+1);
}

fn write_vertex(vertex: &mut Vec<WorldDebugVertex>, rect: RectF32, color: RgbaU8, vertex_count: &mut u32) {
    let [x1, y1, x2, y2] = rect.splat();
    vertex.push(WorldDebugVertex { pos: [x1, y1], color });
    vertex.push(WorldDebugVertex { pos: [x2, y1], color });
    vertex.push(WorldDebugVertex { pos: [x1, y2], color });
    vertex.push(WorldDebugVertex { pos: [x2, y2], color });
    *vertex_count += 4;
}

