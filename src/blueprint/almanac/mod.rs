//! Almanaque alquímico (elementos, EAC2/EAC4, carga RON, hot-reload).
//! Estructura por dominio bajo `blueprint/almanac/`.
//! Ver `docs/sprints/ELEMENT_ALMANAC_CANON/README.md` y `docs/arquitectura/blueprint_blueprint_math.md`.

mod catalog;
mod eac;
mod element_def;
mod loader;
mod paths;
mod systems;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

pub use catalog::AlchemicalAlmanac;
pub use element_def::{ElementDef, ElementPhenologyDef};
pub use loader::ElementDefRonLoader;
pub use systems::{almanac_hot_reload_system, init_almanac_elements_system, AlmanacElementsState};

pub(crate) use paths::{load_element_defs_with_paths, scan_element_ron_paths};

/// Solo tests / contrato EAC1 (`almanac_contract`) leen la ruta canónica vía `crate::blueprint::almanac`.
#[cfg(test)]
pub(crate) use paths::elements_asset_dir;

#[cfg(test)]
pub(crate) use test_support::test_assets_elements_almanac;
