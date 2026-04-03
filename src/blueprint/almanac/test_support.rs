//! Fixtures de test: almanaque estático desde `assets/elements` (una lectura por proceso).

use std::path::Path;

use super::catalog::AlchemicalAlmanac;
use super::element_def::ElementDef;
use super::paths::{load_element_defs_with_paths, scan_element_ron_paths};

fn build_almanac_from_manifest_ron_files(
    relative_to_manifest: &str,
) -> Result<AlchemicalAlmanac, String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_to_manifest);
    let paths = scan_element_ron_paths(&dir);
    if paths.is_empty() {
        return Err(format!("ningún .ron en {}", dir.display()));
    }
    let defs: Vec<ElementDef> = load_element_defs_with_paths(&paths)?
        .into_iter()
        .map(|(_, d)| d)
        .collect();
    Ok(AlchemicalAlmanac::from_defs(defs))
}

/// Almanac de `assets/elements` cacheado (una lectura por proceso de test).
pub(crate) fn test_assets_elements_almanac() -> &'static AlchemicalAlmanac {
    use std::sync::OnceLock;
    static CACHE: OnceLock<AlchemicalAlmanac> = OnceLock::new();
    CACHE.get_or_init(|| {
        build_almanac_from_manifest_ron_files("assets/elements")
            .unwrap_or_else(|msg| panic!("almanac de test: {msg}"))
    })
}
