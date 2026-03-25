//! IDs persistentes (Sprint G11): newtypes para networking/replay/save.
//! `Entity` de Bevy no es estable entre sesiones ni cliente/servidor.
//!
//! Ver `docs/sprints/GAMEDEV_PATTERNS/SPRINT_G11_STRONG_IDS.md`.
//!
//! Invariante: un valor de `ChampionId` / `WorldEntityId` / `EffectId` a lo sumo una entidad viva
//! (asignación solo vía [`IdGenerator`]). Si se viola, el forward map conserva la última entidad y
//! se limpia el índice inverso de la reemplazada.

mod types;
mod lookup;

pub use types::*;
pub use lookup::*;

#[cfg(test)]
mod tests;
