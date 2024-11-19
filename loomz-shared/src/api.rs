//! Common data transfer api between loomz-client and loomz-engine
use kanal::{Sender, Receiver};
use crate::{RgbaU8, base_types::PosF32};

#[derive(Copy, Clone, Debug)]
pub struct WorldComponent {
    pub position: PosF32,
    pub color: RgbaU8,
}

pub struct WorldClientApi {
    components: Sender<WorldComponent>,
}

impl WorldClientApi {

    pub fn update_component(&self, component: WorldComponent) {
        match self.components.send(component) {
            Ok(_) => {},
            Err(_) => unreachable!("engine api receiver should always be open")
        }
    }

}

/// API used from the client side
pub struct LoomzClientApi {
    pub world: WorldClientApi,
}


pub struct WorldEngineApi {
    pub components: Receiver<WorldComponent>,
}

impl WorldEngineApi {

    pub fn recv_component(&self) -> Option<WorldComponent> {
        match self.components.try_recv() {
            Ok(v) => v,
            Err(_) => unreachable!("engine api receiver should always be open")
        }
    }

}

/// API used from the engine side
pub struct LoomzEngineApi {
    pub world: WorldEngineApi,
}

pub struct LoomzApi {
    pub client: Option<LoomzClientApi>,
    pub engine: Option<LoomzEngineApi>,
}

impl LoomzApi {

    pub fn init() -> Self {
        let (world_component_senders, world_component_receiver) = kanal::unbounded::<WorldComponent>();

        LoomzApi {
            client: Some(LoomzClientApi {
                world: WorldClientApi {
                    components: world_component_senders,
                }
            }),
            engine: Some(LoomzEngineApi { 
                world: WorldEngineApi {
                    components: world_component_receiver,
                }
            }),
        }
    }

    pub fn client_api(&mut self) -> LoomzClientApi {
        self.client.take().unwrap()
    }

    pub fn engine_api(&mut self) -> LoomzEngineApi {
        self.engine.take().unwrap()
    }

}
