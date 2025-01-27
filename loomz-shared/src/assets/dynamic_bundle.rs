use parking_lot::Mutex;
use std::sync::Arc;
use crate::{CommonError, CommonErrorType, chain_err};
use super::{AssetId, AssetsTextureData, AssetsMsdfFontData, ShaderData, TextureId, ShaderId, JsonId, MsdfFontId};

struct AssetReloadState {
    textures: Vec<String>,
    json: Vec<String>,
    msdf_fonts: Vec<(String, String)>,
    shader: Vec<(String, String)>,
    paths_to_id: Vec<(String, AssetId)>,
    
}

fn init_asset_reload() -> Result<AssetReloadState, CommonError> {
    // Saves the paths from the assets csv for filtering
    let meta_csv = super::static_bundle::LoomzAssetsBundle::load_asset_metadata()?;

    let mut state = AssetReloadState {
        textures: Vec::with_capacity(4),
        json: Vec::with_capacity(4),
        msdf_fonts: Vec::with_capacity(4),
        shader: Vec::with_capacity(4),
        paths_to_id: Vec::with_capacity(12),
    };

    super::static_bundle::LoomzAssetsBundle::split_csv(&meta_csv, |args| {
        match args[0] {
            "TEXTURE" => {
                let id = AssetId::Texture(TextureId(state.textures.len() as u32));
                let src = args[2].to_string();
                state.paths_to_id.push((src.clone(), id));
                state.json.push(src);
            },
            "JSON" => {
                let id = AssetId::Json(JsonId(state.json.len() as u32));
                let src = args[2].to_string();
                state.paths_to_id.push((src.clone(), id));
                state.json.push(src);
            },
            "MSDF_FONT" => {
                let id = AssetId::MsdfFont(MsdfFontId(state.msdf_fonts.len() as u32));
                let image_src = args[2].to_string();
                let bin_src = args[3].to_string();
                state.paths_to_id.push((image_src.clone(), id));
                state.paths_to_id.push((bin_src.clone(), id));
                state.msdf_fonts.push((image_src, bin_src));
            },
            "SHADER" => {
                let id = AssetId::ShaderId(ShaderId(state.shader.len() as u32));
                let vert_src = args[2].to_string();
                let frag_src = args[3].to_string();
                state.paths_to_id.push((vert_src.clone(), id));
                state.paths_to_id.push((frag_src.clone(), id));
                state.shader.push((vert_src, frag_src));
            }
            _ => unreachable!("Assets type are already validated during the initial assets load")
        }
    });

    Ok(state)
}

fn filter_asset_change(state: &AssetReloadState, dedup: &mut fnv::FnvHashSet<AssetId>, event: notify::Event) {
    let path = event.paths.first().and_then(|p| p.to_str() ).unwrap_or("");

    // Only allow modify events
    if !matches!(event.kind, notify::EventKind::Modify(_)) {
        return;
    }

    // Filter dev assets
    if path.contains("assets/dev") {
        return;
    }

    // Filter files in the assets csv
    let id = match state.paths_to_id.iter().find(|(path2, _)| path.contains(path2) ) {
        Some((_, id)) => *id,
        None => { return; }
    };

    // Filter duplicated events
    dedup.insert(id);
}

fn reload_asset(state: &AssetReloadState, assets: &mut super::static_bundle::LoomzAssetsBundle, id: AssetId) {
    match id {
        AssetId::Texture(_) => { /* TODO */ },
        AssetId::Json(_) => { /* TODO */ }
        AssetId::MsdfFont(_) => { /* TODO */ }
        AssetId::ShaderId(ShaderId(id)) => {
            let shader = match assets.shaders.get_mut(id as usize) {
                Some(shader) => shader,
                None => {  return; }
            };

            let (vert_src, frag_src) = match state.shader.get(id as usize) {
                Some(shader) => shader,
                None => {  return; }
            };

            let vert_path = format!("./assets/shaders/{}", vert_src);
            if let Ok(data) = ::std::fs::read(&vert_path) {
                shader.vert = data;
            }

            let frag_path = format!("./assets/shaders/{}", frag_src);
            if let Ok(data) = ::std::fs::read(&frag_path) {
                shader.frag = data;
            }
        }
    }
}

fn start_assets_watcher(bundle: Arc<LoomzAssetsBundle>) -> Result<(), CommonError> {
    use notify::{Event, RecursiveMode, Result, Watcher};
    use std::{thread, time, sync::mpsc, path::PathBuf};
    use crate::system_err;

    let (sender, receiver) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(sender)
        .map_err(|err| system_err!("Failed to create watcher: {err:?}") )?;

    thread::spawn(move || {
        let mut watch_path = PathBuf::new();
        watch_path.push("assets");
        watcher.watch(&watch_path, RecursiveMode::Recursive).unwrap();

        let mut dedup = fnv::FnvHashSet::default();
        let state = match init_asset_reload() {
            Ok(state) => state,
            Err(err) => {
                let err = chain_err!(err, CommonErrorType::Assets, "Failed to initialize asset reload state");
                println!("{err:?}");
                return;
            }
        };
 
        'outer: loop {
            loop {
                match receiver.recv_timeout(time::Duration::from_millis(200)) {
                    Ok(Ok(event)) => filter_asset_change(&state, &mut dedup, event),
                    Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Disconnected) => { break 'outer; },
                    Err(mpsc::RecvTimeoutError::Timeout) => { break; },
                }
            }

            if dedup.len() > 0 {
                let mut assets = bundle.bundle.lock();
                let mut changed = bundle.changed.lock();
                for &id in dedup.iter() {
                    changed.insert(id);
                    reload_asset(&state, &mut assets, id);
                }

                dedup.clear();
            }
        }

        println!("Assets watcher closed");
    });

    Ok(())
}


/// Asset bundle referencing all the assets in the program. With automatic reloading
pub struct LoomzAssetsBundle {
    bundle: Mutex<super::static_bundle::LoomzAssetsBundle>,
    changed: Mutex<fnv::FnvHashSet<AssetId>>,
}

impl LoomzAssetsBundle {

    pub fn load() -> Result<Arc<Self>, CommonError> {
        let inner_bundle = super::static_bundle::LoomzAssetsBundle::load_base_bundle()?;

        let bundle = Arc::new(LoomzAssetsBundle {
            bundle: Mutex::new(inner_bundle),
            changed: Mutex::new(fnv::FnvHashSet::default())
        });

        let watcher_bundle = Arc::clone(&bundle);
        start_assets_watcher(watcher_bundle)?;

        Ok(bundle)
    }

    pub fn texture_id_by_name(&self, name: &str) -> Option<TextureId> {
        self.bundle.lock().texture_id_by_name(name)
    }

    pub fn texture<'a>(&'a self, id: TextureId) -> Option<AssetsTextureData> {
        self.bundle.lock().texture(id).cloned()
    }

    pub fn json_id_by_name(&self, name: &str) -> Option<JsonId> {
        self.bundle.lock().json_id_by_name(name)
    }

    pub fn json_by_name(&self, name: &str) -> Option<String> {
        self.bundle.lock().json_by_name(name).cloned()
    }

    pub fn font_id_by_name(&self, name: &str) -> Option<MsdfFontId> {
        self.bundle.lock().font_id_by_name(name)
    }

    pub fn font<'a>(&'a self, id: MsdfFontId) -> Option<AssetsMsdfFontData> {
        self.bundle.lock().font(id).cloned()
    }

    pub fn default_font_id(&self) -> Option<MsdfFontId> {
        self.bundle.lock().default_font_id()
    }

    pub fn shader_id_by_name(&self, name: &str) -> Option<ShaderId> {
        self.bundle.lock().shader_id_by_name(name)
    }

    pub fn shader(&self, id: ShaderId) -> Option<ShaderData> {
        self.bundle.lock().shader(id).cloned()
    }

    pub fn changed_assets(&self) -> Option<Vec<AssetId>> {
        let mut changed = self.changed.lock();
        if changed.len() > 0 {
            let ids = changed.iter().copied().collect();
            changed.clear();
            Some(ids)
        } else {
            None
        }
    }

}
