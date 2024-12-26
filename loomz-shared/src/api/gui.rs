mod component;
use component::*;
pub use component::GuiComponentTextGlyph;

mod gui_data;
pub use gui_data::Gui;
use gui_data::GuiFontStyle;

mod builder;
pub use builder::GuiBuilder;

use parking_lot::Mutex;
use crate::{assets::MsdfFontId, msdf_font::ComputedGlyph};
use super::{Id, MessageQueue};

pub struct GuiTextTag;
pub type GuiTextId = Id<GuiTextTag>;

pub enum GuiTextUpdate {
    Font(MsdfFontId),
    Glyphs(&'static [ComputedGlyph]),
}

// Todo: replace this with a special message queue
struct GuiApiGlyphBuffer {
    length: usize,
    buffer: Box<[ComputedGlyph]>
}

pub struct GuiApi {
    glyphs_buffer: Mutex<GuiApiGlyphBuffer>,
    text: MessageQueue<GuiTextId, GuiTextUpdate>
}

impl GuiApi {
    pub fn init() -> Self {
        let buffer = GuiApiGlyphBuffer {
            length: 0,
            buffer: vec![ComputedGlyph::default(); 100].into_boxed_slice(),
        };

        GuiApi {
            glyphs_buffer: Mutex::new(buffer),
            text: MessageQueue::with_capacity(16),
        }
    }

    pub fn update_text_font(&self, id: &GuiTextId, font: MsdfFontId) {
        self.text.push(id, GuiTextUpdate::Font(font));
    }

    pub fn update_text_glyphs(&self, id: &GuiTextId, glyphs: &[GuiComponentTextGlyph]) {
        let mut glyphs_buffer = self.glyphs_buffer.lock();
        let start = glyphs_buffer.length;
        let stop = start + glyphs.len();
        if stop > glyphs_buffer.buffer.len() {
            eprintln!("Not enough space left to upload text glyph. Increase buffer size");
            return;
        }

        for g in glyphs {
            let i = glyphs_buffer.length;
            let mut glyph = g.glyph;
            glyph.position.left += g.offset.x;
            glyph.position.right += g.offset.x;
            glyph.position.top += g.offset.y;
            glyph.position.bottom += g.offset.y;
            glyphs_buffer.buffer[i] = glyph;
            glyphs_buffer.length += 1;
        }

        // Safety: slice will be valid for the duration of the program
        // Safety: access to glyphs will synchronized by the api
        let slice = unsafe { ::std::mem::transmute(&glyphs_buffer.buffer[start..stop]) };
        self.text.push(id, GuiTextUpdate::Glyphs(slice));
    }

    pub fn text_updates<'a>(&'a self) -> Option<impl Iterator<Item = (GuiTextId, GuiTextUpdate)> + 'a> {
        let mut glyphs_buffer = self.glyphs_buffer.lock();
        glyphs_buffer.length = 0;

        self.text.read_values()
            .map(|mut it|
                ::std::iter::from_fn(move || {
                    let _ = &glyphs_buffer; // Keep alive for safety
                    it.next()
                })
            )
    }
}
