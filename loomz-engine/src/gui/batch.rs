use loomz_shared::CommonError;
use loomz_engine_core::LoomzEngineCore;
use super::{GuiModule, GuiBatch, GuiDescriptor, GuiData, GuiVertex};


pub(super) struct GuiBatcher<'a> {
    core: &'a mut LoomzEngineCore,
    data: &'a mut GuiData,
    descriptors: &'a mut GuiDescriptor,
    batches: &'a mut Vec<GuiBatch>,
    current_view: vk::ImageView,
    text_index: usize,
    batch_index: usize,
    index_count: isize,
    vertex_count: u32,
}

impl<'a> GuiBatcher<'a> {

    pub(super) fn build(gui: &mut GuiModule, core: &mut LoomzEngineCore) -> Result<(), CommonError> {
        let mut batcher = GuiBatcher {
            core,
            current_view: vk::ImageView::null(),
            data: &mut gui.data,
            descriptors: &mut gui.descriptors,
            batches: &mut gui.batches,
            text_index: 0,
            batch_index: 0,
            index_count: 0,
            vertex_count: 0,
        };
        batcher.descriptors.reset_batch_layout();
        batcher.batches.clear();

        batcher.first_batch()?;
        batcher.remaining_batches()?;
        batcher.upload_vertex();

        Ok(())
    }

    fn first_batch(&mut self) -> Result<(), CommonError> {
        let mut found = false;
        let max_instance = self.data.text.len();
        while !found && self.text_index != max_instance {
            let text = &self.data.text[self.text_index];
            let font_view = text.font_view;
            let index_count = (text.glyphs.len() * 6) as u32;
            if index_count == 0 || font_view.is_null() {
                self.text_index += 1;
                continue;
            }

            self.next_batch(font_view)?;
            self.write_text_indices();
            self.write_text_vertex();

            self.text_index += 1;
            found = true;
        }

        Ok(())
    }

    fn remaining_batches(&mut self) -> Result<(), CommonError> {
        let max_instance = self.data.text.len();
        while self.text_index != max_instance {
            let text = &self.data.text[self.text_index];
            let font_view = text.font_view;
            let index_count = (text.glyphs.len() * 6) as u32;
            if index_count == 0 || font_view.is_null() {
                self.text_index += 1;
                continue;
            }

            if self.current_view != font_view {
                self.next_batch(font_view)?;
                self.batch_index += 1;
            }

            self.write_text_indices();
            self.write_text_vertex();

            self.text_index += 1;
        }

        Ok(())
    }

    fn next_batch(&mut self, font_view: vk::ImageView) -> Result<(), CommonError> {
        let set = self.descriptors.write_batch_texture(font_view)?;

        self.current_view = font_view;

        self.batches.push(GuiBatch {
            set,
            first_index: self.index_count as u32,
            index_count: 0,
        });

        Ok(())
    }

    fn write_text_indices(&mut self) {
        let glyph_count = self.data.text[self.text_index].glyphs.len() as isize;
        let index_count = glyph_count * 6;

        // Safety, indices capacity will be greater than written range
        let mut i = self.index_count;
        let mut v = self.vertex_count;
        assert!(i + index_count < self.data.indices.len() as isize);

        unsafe {
            for _ in 0..glyph_count {
                let indices = self.data.indices.as_mut_ptr();
                indices.offset(i+0).write(v+0);
                indices.offset(i+1).write(v+1);
                indices.offset(i+2).write(v+2);
                indices.offset(i+3).write(v+2);
                indices.offset(i+4).write(v+3);
                indices.offset(i+5).write(v+1);

                i += 6;
                v += 4;
            }
        }

        self.index_count += index_count;
        self.batches[self.batch_index].index_count += index_count as u32;
    }

    fn write_text_vertex(&mut self) {
        let glyphs = &self.data.text[self.text_index].glyphs;
        let glyph_count = glyphs.len() as isize;
        let vertex_count = glyph_count * 4;

        // Safety, vertex capacity will be greater than written range
        let mut v = self.vertex_count as isize;
        assert!(v + vertex_count < self.data.vertex.len() as isize);

        unsafe {
            let vertex = self.data.vertex.as_mut_ptr();            
            for glyph in glyphs {
                let [x1, y1, x2, y2] = glyph.position.splat();
                let [x3, y3, x4, y4] = glyph.texcoord.splat();
                vertex.offset(v+0).write(GuiVertex { pos: [x1, y1], uv: [x3, y3] });
                vertex.offset(v+1).write(GuiVertex { pos: [x2, y1], uv: [x4, y3] });
                vertex.offset(v+2).write(GuiVertex { pos: [x1, y2], uv: [x3, y4] });
                vertex.offset(v+3).write(GuiVertex { pos: [x2, y2], uv: [x4, y4] });
                v += 4;
            }
        }

        self.vertex_count += vertex_count as u32;
    }

    fn upload_vertex(&mut self) {
        let i = self.index_count as usize;
        let v = self.vertex_count as usize;
        self.data.vertex_alloc.set_data(
            self.core,
            &self.data.indices[0..i],
            &self.data.vertex[0..v],
        );
    }
}
