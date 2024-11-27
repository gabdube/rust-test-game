//! Common data transfer api between loomz-client and loomz-engine
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, AtomicU32, Ordering}};
use parking_lot::{MutexGuard, Mutex};
use crate::assets::{LoomzAssetsBundle, TextureId};
use crate::store::{StoreAndLoad, SaveFileReaderBase, SaveFileWriterBase};
use crate::inputs::InputBuffer;
use crate::base_types::_2d::{Position, Size};
use crate::CommonError;

#[derive(Clone)]
pub struct Uid(Arc<AtomicU32>);

impl Uid {
    pub fn new() -> Self {
        Uid(Arc::new(AtomicU32::new(u32::MAX)))
    }

    #[inline]
    pub fn bind(&self, val: u32) {
        self.0.store(val, Ordering::SeqCst);
    }

    #[inline]
    pub fn is_unbound(&self) -> bool {
        self.0.load(Ordering::SeqCst) == u32::MAX
    }

    #[inline]
    pub fn value(&self) -> u32 {
        self.0.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn bound_value(&self) -> Option<usize> {
        let value = self.0.load(Ordering::SeqCst) as usize;
        match value == (u32::MAX as usize) {
            true => None,
            false => Some(value)
        }
    }
}

impl StoreAndLoad for Uid {
    fn load(reader: &mut SaveFileReaderBase) -> Self {
        Uid(Arc::new(AtomicU32::new(reader.read_u32())))
    }

    fn store(&self, writer: &mut SaveFileWriterBase) {
        writer.write_u32(self.0.load(Ordering::Relaxed));
    }
}

impl Default for Uid {
    fn default() -> Self {
        Self::new()
    }
}


const WORLD_COMPONENT_UPDATES_CAP: usize = 100;
type ComponentUpdates = Box<[Option<WorldComponentUpdate>; WORLD_COMPONENT_UPDATES_CAP]>;

pub struct WorldComponentUpdate {
    pub uid: Uid,
    pub component:WorldComponent,
}

#[derive(Copy, Clone, Debug)]
pub struct WorldComponent {
    pub position: Position<f32>,
    pub size: Size<f32>,
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

    pub fn update_component(&self, uid: &Uid, component: WorldComponent) {
        let mut components = self.components.lock();
        let index = self.components_length.fetch_add(1, Ordering::SeqCst);
        assert!(index < WORLD_COMPONENT_UPDATES_CAP, "Increase components buffer cap");
        components[index] = Some(WorldComponentUpdate {
            uid: uid.clone(),
            component,
        });
    }

    pub fn components(&self) -> Option<MutexGuard<ComponentUpdates>> {
        match self.components_length.load(Ordering::SeqCst) {
            0 => None,
            _ => {
                self.components_length.store(0, Ordering::Relaxed);
                let components_guard = self.components.lock();
                Some(components_guard)
            },
        }
    }

}

struct InnerInputs {
    buffer: Mutex<InputBuffer>,
    new_inputs: AtomicBool,
}

struct ApiInner {
    assets: Arc<LoomzAssetsBundle>,
    inputs: InnerInputs,
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

        let inputs = InnerInputs {
            buffer: Mutex::new(InputBuffer::new()),
            new_inputs: AtomicBool::new(false)
        };

        let inner = ApiInner {
            assets,
            inputs,
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

    pub fn inputs<'a>(&'a self) -> MutexGuard<'a, InputBuffer> {
        self.inner.inputs.new_inputs.store(true, Ordering::Relaxed);
        self.inner.inputs.buffer.lock()
    }

    pub fn new_inputs<'a>(&'a self) -> Option<MutexGuard<'a, InputBuffer>> {
        match self.inner.inputs.new_inputs.fetch_not(Ordering::Relaxed) {
            true => Some(self.inner.inputs.buffer.lock()),
            false => None
        }
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
