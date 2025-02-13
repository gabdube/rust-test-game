//! Common data transfer api between loomz-client and loomz-engine
mod base;
pub use base::*;

mod world;
pub use world::*;

mod gui;
pub use gui::*;

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::assets::LoomzAssetsBundle;
use crate::inputs::{SharedInputBuffer, SharedKeysState};
use crate::{CommonError, SizeF32};

struct ApiInner {
    assets: Arc<LoomzAssetsBundle>,
    inputs: SharedInputBuffer,
    keys: SharedKeysState,
    world: WorldApi,
    gui: GuiApi,
    exit: AtomicBool,
}

#[derive(Clone)]
pub struct LoomzApi {
    inner: Arc<ApiInner>
}

impl LoomzApi {

    pub fn init(screen_size: SizeF32) -> Result<Self, CommonError> {
        let inner = ApiInner {
            assets: LoomzAssetsBundle::load()?,
            inputs: SharedInputBuffer::new(screen_size),
            keys: SharedKeysState::new(),
            world: WorldApi::init(),
            gui: GuiApi::init(),
            exit: AtomicBool::new(false),
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

    pub fn inputs(&self) -> SharedInputBuffer {
        self.inner.inputs.clone()
    }

    pub fn inputs_ref(&self) -> &SharedInputBuffer {
        &self.inner.inputs
    }

    pub fn keys(&self) -> SharedKeysState {
        self.inner.keys.clone()
    }

    pub fn keys_ref(&self) -> &SharedKeysState {
        &self.inner.keys
    }

    pub fn world(&self) -> &WorldApi {
        &self.inner.world
    }

    pub fn gui(&self) -> &GuiApi {
        &self.inner.gui
    }

    pub fn exit(&self) {
        self.inner.exit.store(true, Ordering::SeqCst);
    }

    pub fn must_exit(&self) -> bool {
        self.inner.exit.load(Ordering::SeqCst)
    }

    pub fn client_update_finished(&self) {
        self.inner.keys.clear_update_flags();
        self.inner.inputs.clear_update_flags();
    }

}
