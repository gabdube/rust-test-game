//! Common data transfer api between loomz-client and loomz-engine
mod base;
pub use base::*;

mod world;
pub use world::*;

mod gui;
pub use gui::*;

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::assets::LoomzAssetsBundle;
use crate::inputs::InputBuffer;
use crate::CommonError;

struct InnerInputs {
    buffer: InputBuffer,
    new_inputs: AtomicBool,
}

struct ApiInner {
    assets: Arc<LoomzAssetsBundle>,
    inputs: InnerInputs,
    world: WorldApi,
    gui: GuiApi,
}

#[derive(Clone)]
pub struct LoomzApi {
    inner: Arc<ApiInner>
}

impl LoomzApi {

    pub fn init() -> Result<Self, CommonError> {
        let assets = LoomzAssetsBundle::load()?;
        let world = WorldApi::init();
        let gui = GuiApi::init();

        let inputs = InnerInputs {
            buffer: InputBuffer::new(),
            new_inputs: AtomicBool::new(false)
        };

        let inner = ApiInner {
            assets,
            inputs,
            world,
            gui,
        };

        let api = LoomzApi {
            inner: Arc::new(inner)
        };

        Ok(api)
    }

    pub fn assets(&self) -> Arc<LoomzAssetsBundle> {
        Arc::clone(&self.inner.assets)
    }

    pub fn assets_ref(&self) -> &LoomzAssetsBundle {
        &self.inner.assets
    }

    pub fn write_inputs(&self) -> InputBuffer {
        self.inner.inputs.new_inputs.store(true, Ordering::Relaxed);
        self.inner.inputs.buffer.clone()
    }

    pub fn read_inputs(&self) -> Option<InputBuffer> {
        match self.inner.inputs.new_inputs.fetch_and(false, Ordering::Relaxed) {
            true => Some(self.inner.inputs.buffer.clone()),
            false => None
        }
    }

    pub fn world(&self) -> &WorldApi {
        &self.inner.world
    }

    pub fn gui(&self) -> &GuiApi {
        &self.inner.gui
    }

}
