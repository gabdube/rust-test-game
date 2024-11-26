//! Common data transfer api between loomz-client and loomz-engine
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use parking_lot::{Mutex, MutexGuard};
use crate::{CommonError, base_types::{PosF32, SizeF32}, assets::{LoomzAssetsBundle, TextureId}};

const WORLD_COMPONENT_UPDATES_CAP: usize = 100;
type ComponentUpdates = Box<[Option<WorldComponent>; WORLD_COMPONENT_UPDATES_CAP]>;

#[derive(Copy, Clone, Debug)]
pub struct WorldComponent {
    pub position: PosF32,
    pub size: SizeF32,
    pub texture_id: TextureId,
}

pub struct WorldApi {
    components_length: AtomicUsize,
    components: Mutex<ComponentUpdates>
}

impl WorldApi {

    pub fn init() -> Self {
        WorldApi {
            components_length: AtomicUsize::new(0),
            components: Mutex::new(new_boxed_array()),
        }
    }

    pub fn update_component(&self, component: WorldComponent) {
        let mut components = self.components.lock();
        let index = self.components_length.fetch_add(1, Ordering::Relaxed);
        assert!(index < WORLD_COMPONENT_UPDATES_CAP, "Increase components buffer cap");
        components[index] = Some(component);
    }

    pub fn components(&self) -> Option<MutexGuard<ComponentUpdates>> {
        match self.components_length.load(Ordering::Relaxed) {
            0 => None,
            _ => {
                let components_guard = self.components.lock();
                self.components_length.store(0, Ordering::Relaxed);
                Some(components_guard)
            },
        }
    }

}

struct ApiInner {
    assets: Arc<LoomzAssetsBundle>,
    world: WorldApi,
}

#[derive(Clone)]
pub struct LoomzApi {
    inner: Arc<ApiInner>
}

impl LoomzApi {

    pub fn init() -> Result<Self, CommonError> {
        let assets = LoomzAssetsBundle::load()?;
        let world = WorldApi::init();

        let inner = ApiInner {
            assets,
            world,
        };

        let api = LoomzApi {
            inner: Arc::new(inner)
        };

        Ok(api)
    }

    pub fn assets(&self) -> Arc<LoomzAssetsBundle> {
        Arc::clone(&self.inner.assets)
    }

    pub fn world(&self) -> &WorldApi {
        &self.inner.world
    }

}

fn new_boxed_array<const S: usize, T: Default>() -> Box<[T; S]> {
    let mut data = Box::new_uninit();
    unsafe {
        let data_ptr = data.as_mut_ptr() as *mut T;
        for i in 0..WORLD_COMPONENT_UPDATES_CAP {
            data_ptr.offset(i as isize).write(T::default());
        }

        data.assume_init()
    }
}
