//! Common data transfer api between loomz-client and loomz-engine
use kanal::{Sender, Receiver};
use std::sync::Arc;
use crate::{CommonError, RgbaU8, base_types::PosF32, assets::{LoomzAssetsBundle, TextureId}};

#[derive(Copy, Clone, Debug)]
pub struct WorldComponent {
    pub position: PosF32,
    pub color: RgbaU8,
    pub texture: TextureId,
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
    assets: Arc<LoomzAssetsBundle>,
    client: Option<LoomzClientApi>,
    engine: Option<LoomzEngineApi>,
}

impl LoomzApi {

    pub fn init() -> Result<Self, CommonError> {
        let assets = LoomzAssetsBundle::init()?;
        let (world_component_senders, world_component_receiver) = kanal::unbounded::<WorldComponent>();

        let api = LoomzApi {
            assets,
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
        };

        Ok(api)
    }

    pub fn assets(&self) -> Arc<LoomzAssetsBundle> {
        Arc::clone(&self.assets)
    }

    pub fn client_api(&mut self) -> LoomzClientApi {
        self.client.take().unwrap()
    }

    pub fn engine_api(&mut self) -> LoomzEngineApi {
        self.engine.take().unwrap()
    }

}
