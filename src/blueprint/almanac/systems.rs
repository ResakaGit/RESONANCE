//! Sistemas Bevy: handles de assets, build del almanaque, hot-reload.
//! Dominio: integración runtime con `AssetServer` / `Assets<ElementDef>`.

use bevy::prelude::*;

use super::catalog::AlchemicalAlmanac;
use super::element_def::ElementDef;
use super::paths::{elements_asset_dir, scan_element_ron_paths};

/// Estado de carga del almanac desde `assets/elements/*.ron`.
#[derive(Resource, Debug, Default)]
pub struct AlmanacElementsState {
    pub initialized: bool,
    pub built: bool,
    pub handles: Vec<Handle<ElementDef>>,
}

fn elements_dir_exists() -> bool {
    elements_asset_dir().is_dir()
}

fn list_element_ron_files() -> Vec<String> {
    scan_element_ron_paths(&elements_asset_dir())
        .into_iter()
        .filter_map(|p| p.file_name()?.to_str().map(String::from))
        .collect()
}

/// Sistema: crea handles para todos los `assets/elements/*.ron`.
pub fn init_almanac_elements_system(
    asset_server: Res<AssetServer>,
    mut state: ResMut<AlmanacElementsState>,
    mut almanac: ResMut<AlchemicalAlmanac>,
) {
    if state.initialized {
        return;
    }

    let files = list_element_ron_files();
    if files.is_empty() || !elements_dir_exists() {
        state.initialized = true;
        state.built = true;
        state.handles.clear();
        // Modo 100% data-driven: si no hay assets, el Almanac queda vacío.
        *almanac = AlchemicalAlmanac::default();
        return;
    }

    for file in files {
        // `assets/` es la raíz del AssetServer.
        let asset_path = format!("elements/{file}");
        let handle: Handle<ElementDef> = asset_server.load(asset_path);
        state.handles.push(handle);
    }

    state.initialized = true;
    state.built = false;
}

/// Sistema: reconstruye el `AlchemicalAlmanac` cuando cambia cualquier `.ron`
/// (hot-reload) o cuando todavía no está “completo” en el primer load.
pub fn almanac_hot_reload_system(
    mut state: ResMut<AlmanacElementsState>,
    element_defs: Res<Assets<ElementDef>>,
    mut almanac: ResMut<AlchemicalAlmanac>,
    mut ev: EventReader<AssetEvent<ElementDef>>,
) {
    if !state.initialized || state.handles.is_empty() {
        return;
    }

    // Reconstruimos al menos una vez al boot, y luego en hot-reload.
    let mut should_rebuild = !state.built;
    for event in ev.read() {
        // Cualquier evento de ElementDef obliga a reconstruir el almanaque completo.
        match event {
            AssetEvent::Added { .. }
            | AssetEvent::Modified { .. }
            | AssetEvent::Removed { .. }
            | AssetEvent::Unused { .. }
            | AssetEvent::LoadedWithDependencies { .. } => {
                should_rebuild = true;
            }
        }
    }

    if !should_rebuild {
        return;
    }

    let mut defs: Vec<ElementDef> = Vec::new();

    let mut all_loaded = true;
    for handle in &state.handles {
        if let Some(def) = element_defs.get(handle) {
            defs.push(def.clone());
        } else {
            all_loaded = false;
        }
    }

    // Si todavía no hay ningún ElementDef cargado, no tocamos el Resource.
    if defs.is_empty() {
        return;
    }

    // Swap atómico (reconstruimos todo y luego reemplazamos el resource).
    *almanac = AlchemicalAlmanac::from_defs(defs);
    state.built = all_loaded;
}
