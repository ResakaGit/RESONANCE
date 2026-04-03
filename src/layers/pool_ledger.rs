//! EC-6A: Libro contable de conservación de pool. Recomputado cada tick.
//! Solo entidades con EnergyPool que tienen hijos activos.

use bevy::prelude::*;

/// Libro contable de conservación de pool. Derivado cada tick.
/// 4 campos (3 × f32 + 1 × u16) = 16 bytes — cumple DOD max 4.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct PoolConservationLedger {
    /// Energía total extraída por hijos este tick (qe).
    total_extracted: f32,
    /// Energía disipada este tick (segunda ley).
    total_dissipated: f32,
    /// Delta neto del pool este tick: intake_rate - extracted - dissipated.
    net_delta: f32,
    /// Número de hijos activos que extrajeron.
    active_children: u16,
}

impl PoolConservationLedger {
    /// Construye un ledger con los valores del tick.
    pub fn new(
        total_extracted: f32,
        total_dissipated: f32,
        net_delta: f32,
        active_children: u16,
    ) -> Self {
        Self {
            total_extracted: total_extracted.max(0.0),
            total_dissipated: total_dissipated.max(0.0),
            net_delta,
            active_children,
        }
    }

    pub fn total_extracted(&self) -> f32 {
        self.total_extracted
    }
    pub fn total_dissipated(&self) -> f32 {
        self.total_dissipated
    }
    pub fn net_delta(&self) -> f32 {
        self.net_delta
    }
    pub fn active_children(&self) -> u16 {
        self.active_children
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_conservation_ledger_is_copy() {
        let a = PoolConservationLedger::new(200.0, 10.0, -160.0, 3);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn pool_conservation_ledger_size_16_bytes() {
        assert!(std::mem::size_of::<PoolConservationLedger>() <= 16);
    }

    #[test]
    fn pool_conservation_ledger_zero_no_nan() {
        let l = PoolConservationLedger::new(0.0, 0.0, 0.0, 0);
        assert!(!l.total_extracted().is_nan());
        assert!(!l.net_delta().is_nan());
    }

    #[test]
    fn pool_conservation_ledger_net_delta_consistent() {
        let intake = 50.0_f32;
        let extracted = 200.0;
        let dissipated = 10.0;
        let l =
            PoolConservationLedger::new(extracted, dissipated, intake - extracted - dissipated, 3);
        assert!(
            (l.net_delta() - (intake - l.total_extracted() - l.total_dissipated())).abs() < 1e-6
        );
    }
}
