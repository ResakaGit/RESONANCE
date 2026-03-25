//! I/O sobre `assets/elements/*.ron` (rutas, escaneo, parseo batch).
//! Dominio: filesystem + RON sin Bevy (reutilizable en tests y EAC1).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::element_def::ElementDef;

/// Ruta absoluta `…/assets/elements` (misma raíz que `AssetServer` para `elements/*.ron`).
pub(crate) fn elements_asset_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("elements")
}

/// Rutas absolutas a `*.ron` bajo `dir`, orden lexicográfico (mismo criterio que AssetServer + tests).
pub(crate) fn scan_element_ron_paths(dir: &Path) -> Vec<PathBuf> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.extension() == Some(OsStr::new("ron"))
                && e.file_type().map(|t| t.is_file()).unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();
    paths.sort();
    paths
}

pub(crate) fn read_element_def_from_path(path: &Path) -> Result<ElementDef, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    ron::de::from_bytes::<ElementDef>(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// Parsea cada RON en orden; reutilizable por tests, `build_almanac_*` y validación EAC1.
pub(crate) fn load_element_defs_with_paths(paths: &[PathBuf]) -> Result<Vec<(PathBuf, ElementDef)>, String> {
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let def = read_element_def_from_path(path)?;
        out.push((path.clone(), def));
    }
    Ok(out)
}
