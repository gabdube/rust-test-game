mod builder;
pub use builder::GuiBuilder;

use super::{Id, MessageQueue};
use crate::LoomzApi;

pub struct GuiTag;
pub type GuiId = Id<GuiTag>;

#[derive(Default)]
pub struct Gui {
   
}

impl Gui {
    pub fn build<F: FnOnce(&mut GuiBuilder)>(api: &LoomzApi, cb: F) -> Self {
        let mut builder = GuiBuilder::new(api);

        cb(&mut builder);

        Gui {}
    }

    pub fn update(&mut self) {
        
    }
}

pub struct GuiApi {
    updates: MessageQueue<GuiId, ()>
}

impl GuiApi {
    pub fn init() -> Self {
        GuiApi {
            updates: MessageQueue::with_capacity(16),
        }
    }

    pub fn update_gui(&self, id: &GuiId, data: &Gui) {

    }
}
