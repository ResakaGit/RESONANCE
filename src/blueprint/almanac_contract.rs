//! EAC1 — Contrato almanaque ↔ constantes críticas.
//! Ver `docs/sprints/ELEMENT_ALMANAC_CANON/README.md` (EAC1 cerrado).

use std::collections::HashMap;
use std::path::Path;

use crate::blueprint::almanac::{
    AlchemicalAlmanac, load_element_defs_with_paths, scan_element_ron_paths,
};
use crate::blueprint::constants::{
    ABIOGENESIS_FLORA_BAND_HZ_HIGH, ABIOGENESIS_FLORA_BAND_HZ_LOW,
    ABIOGENESIS_FLORA_ELEMENT_SYMBOL, ABIOGENESIS_FLORA_PEAK_HZ, ALMANAC_COHERENCE_EPS_HZ,
    EAC1_ERR_DUPLICATE_SYMBOL_PREFIX,
};
use crate::blueprint::element_id::ElementId;

#[inline]
fn require_hz_matches_const(
    actual: f32,
    expected: f32,
    field: &'static str,
    const_name: &'static str,
) -> Result<(), String> {
    if (actual - expected).abs() > ALMANAC_COHERENCE_EPS_HZ {
        Err(format!(
            "drift: {field}={actual} vs {const_name}={expected}"
        ))
    } else {
        Ok(())
    }
}

/// Coherencia Flora: `assets/elements/flora.ron` debe reflejarse en `blueprint::constants` (RON manda datos).
pub fn validate_flora_abiogenesis_coherence(almanac: &AlchemicalAlmanac) -> Result<(), String> {
    let id = ElementId::from_name(ABIOGENESIS_FLORA_ELEMENT_SYMBOL);
    let Some(flora) = almanac.get(id) else {
        return Err(format!(
            "almanaque sin symbol {:?} (¿cargó flora.ron?)",
            ABIOGENESIS_FLORA_ELEMENT_SYMBOL
        ));
    };
    require_hz_matches_const(
        flora.freq_band.0,
        ABIOGENESIS_FLORA_BAND_HZ_LOW,
        "flora.freq_band.0",
        "ABIOGENESIS_FLORA_BAND_HZ_LOW",
    )?;
    require_hz_matches_const(
        flora.freq_band.1,
        ABIOGENESIS_FLORA_BAND_HZ_HIGH,
        "flora.freq_band.1",
        "ABIOGENESIS_FLORA_BAND_HZ_HIGH",
    )?;
    require_hz_matches_const(
        flora.frequency_hz,
        ABIOGENESIS_FLORA_PEAK_HZ,
        "flora.frequency_hz",
        "ABIOGENESIS_FLORA_PEAK_HZ",
    )?;
    Ok(())
}

/// Prohíbe el mismo `symbol` en dos RON distintos (evita “último gana” silencioso en CI).
pub fn validate_unique_element_symbols_in_dir(elements_dir: &Path) -> Result<(), String> {
    if !elements_dir.is_dir() {
        return Err(format!(
            "EAC1: no es directorio: {}",
            elements_dir.display()
        ));
    }
    let paths = scan_element_ron_paths(elements_dir);
    if paths.is_empty() {
        return Err(format!(
            "EAC1: sin archivos .ron en {}",
            elements_dir.display()
        ));
    }
    let parsed = load_element_defs_with_paths(&paths)?;
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for (path, def) in parsed {
        if def.symbol.is_empty() {
            return Err(format!("symbol vacío en {}", path.display()));
        }
        map.entry(def.symbol)
            .or_default()
            .push(path.display().to_string());
    }
    let mut dup_msgs: Vec<String> = map
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(sym, files)| format!("{sym:?} → {}", files.join(", ")))
        .collect();
    dup_msgs.sort();
    if dup_msgs.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{}{}",
            EAC1_ERR_DUPLICATE_SYMBOL_PREFIX,
            dup_msgs.join("; ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::almanac::{elements_asset_dir, test_assets_elements_almanac};

    #[test]
    fn eac1_flora_coherence_with_test_assets_almanac() {
        validate_flora_abiogenesis_coherence(test_assets_elements_almanac()).unwrap();
    }

    #[test]
    fn eac1_unique_symbols_under_assets_elements() {
        validate_unique_element_symbols_in_dir(&elements_asset_dir()).unwrap();
    }
}
