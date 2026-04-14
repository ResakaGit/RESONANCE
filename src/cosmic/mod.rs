//! Cosmic Telescope — simulación multi-escala con colapso observacional.
//!
//! Jerarquía: S0 Cosmológico → S1 Estelar → S2 Planetario → S3 Ecológico → S4 Molecular.
//! Solo el nivel observado se simula a resolución completa; los demás corren coarsened.
//!
//! ADR-036.

use bevy::prelude::*;

pub mod bridges;
pub mod constants;
pub mod multiverse;
pub mod observer;
pub mod scale_manager;
pub mod scales;
pub mod zoom;

pub use constants::{scale_label, scale_short, ALL_SCALES};
pub use multiverse::{BranchSnapshot, MultiverseBranch, MultiverseLog, MultiverseSummary};
pub use observer::{
    largest_entity_in, rebranch_observed, seed_universe, zoom_via_bridge,
    zoom_via_bridge_with_seed, BigBangParams,
};
pub use scale_manager::{CosmicEntity, CosmicWorld, ScaleInstance, ScaleManager};
pub use zoom::{
    aggregate_child, collapse_parent, derive_zoom_seed, zoom_in_system, zoom_out_system,
    ZoomConfig, ZoomInEvent, ZoomOutEvent,
};

// ─── ScaleLevel ───────────────────────────────────────────────────────────

/// Nivel de escala espacial. S0 = mayor (cosmos), S4 = menor (molecular).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum ScaleLevel {
    /// S0: clusters galácticos, gravitación N-body.
    Cosmological,
    /// S1: estrellas, nucleosíntesis, protoplanetas.
    Stellar,
    /// S2: superficie planetaria, energy field grid.
    Planetary,
    /// S3: organismos, evolución, ecosistema (escala actual del juego).
    Ecological,
    /// S4: proteínas, moléculas, Go model + REMD.
    Molecular,
}

impl ScaleLevel {
    /// Profundidad 0..4 (S0=0, S4=4).
    pub const fn depth(self) -> u8 {
        match self {
            ScaleLevel::Cosmological => 0,
            ScaleLevel::Stellar => 1,
            ScaleLevel::Planetary => 2,
            ScaleLevel::Ecological => 3,
            ScaleLevel::Molecular => 4,
        }
    }

    /// Escala superior (padre), None si es S0.
    pub const fn parent(self) -> Option<Self> {
        match self {
            ScaleLevel::Cosmological => None,
            ScaleLevel::Stellar => Some(ScaleLevel::Cosmological),
            ScaleLevel::Planetary => Some(ScaleLevel::Stellar),
            ScaleLevel::Ecological => Some(ScaleLevel::Planetary),
            ScaleLevel::Molecular => Some(ScaleLevel::Ecological),
        }
    }

    /// Escala inferior (hijo), None si es S4.
    pub const fn child(self) -> Option<Self> {
        match self {
            ScaleLevel::Cosmological => Some(ScaleLevel::Stellar),
            ScaleLevel::Stellar => Some(ScaleLevel::Planetary),
            ScaleLevel::Planetary => Some(ScaleLevel::Ecological),
            ScaleLevel::Ecological => Some(ScaleLevel::Molecular),
            ScaleLevel::Molecular => None,
        }
    }

    /// dt relativo a S3 (base). Escalas mayores tienen dt mayor.
    pub const fn dt_ratio(self) -> f64 {
        match self {
            ScaleLevel::Cosmological => 1.0e6,
            ScaleLevel::Stellar => 1.0e4,
            ScaleLevel::Planetary => 1.0e2,
            ScaleLevel::Ecological => 1.0,
            ScaleLevel::Molecular => 5.0e-3,
        }
    }

    /// Distancia a otra escala (siempre positiva).
    pub const fn distance_to(self, other: Self) -> u8 {
        let a = self.depth() as i8;
        let b = other.depth() as i8;
        (a - b).unsigned_abs()
    }
}

// ─── CosmicPlugin ─────────────────────────────────────────────────────────

pub struct CosmicPlugin;

impl Plugin for CosmicPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ScaleLevel>()
            .init_resource::<ScaleManager>()
            .init_resource::<MultiverseLog>()
            .add_event::<ZoomInEvent>()
            .add_event::<ZoomOutEvent>();
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_ordered_monotonically() {
        assert_eq!(ScaleLevel::Cosmological.depth(), 0);
        assert_eq!(ScaleLevel::Stellar.depth(), 1);
        assert_eq!(ScaleLevel::Planetary.depth(), 2);
        assert_eq!(ScaleLevel::Ecological.depth(), 3);
        assert_eq!(ScaleLevel::Molecular.depth(), 4);
    }

    #[test]
    fn parent_child_consistent() {
        let all = [
            ScaleLevel::Cosmological,
            ScaleLevel::Stellar,
            ScaleLevel::Planetary,
            ScaleLevel::Ecological,
            ScaleLevel::Molecular,
        ];
        for s in all {
            if let Some(p) = s.parent() {
                assert_eq!(p.child(), Some(s), "parent.child != self for {:?}", s);
            }
            if let Some(c) = s.child() {
                assert_eq!(c.parent(), Some(s), "child.parent != self for {:?}", s);
            }
        }
    }

    #[test]
    fn cosmological_no_parent() {
        assert!(ScaleLevel::Cosmological.parent().is_none());
    }

    #[test]
    fn molecular_no_child() {
        assert!(ScaleLevel::Molecular.child().is_none());
    }

    #[test]
    fn dt_ratio_monotone_with_depth() {
        let mut prev = f64::INFINITY;
        for s in [
            ScaleLevel::Cosmological,
            ScaleLevel::Stellar,
            ScaleLevel::Planetary,
            ScaleLevel::Ecological,
            ScaleLevel::Molecular,
        ] {
            assert!(s.dt_ratio() < prev, "dt not monotone at {:?}", s);
            prev = s.dt_ratio();
        }
    }

    #[test]
    fn distance_symmetric_and_correct() {
        assert_eq!(ScaleLevel::Cosmological.distance_to(ScaleLevel::Molecular), 4);
        assert_eq!(ScaleLevel::Molecular.distance_to(ScaleLevel::Cosmological), 4);
        assert_eq!(ScaleLevel::Stellar.distance_to(ScaleLevel::Stellar), 0);
        assert_eq!(ScaleLevel::Planetary.distance_to(ScaleLevel::Ecological), 1);
    }
}
