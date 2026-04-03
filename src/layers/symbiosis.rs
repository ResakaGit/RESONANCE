//! ET-5: Obligate Symbiosis — SymbiosisLink component. Capa T2-1.

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Reflect, PartialEq)]
pub enum SymbiosisType {
    Mutualism,
    Parasitism,
    Commensalism,
}

/// Capa T2-1: SymbiosisLink — dependencia energética par a par.
/// SparseSet: relaciones simbióticas son pocas y transientes.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct SymbiosisLink {
    pub partner_id: u32,
    pub relationship: SymbiosisType,
    pub bonus_factor: f32,
    pub drain_rate: f32,
}

impl Default for SymbiosisLink {
    fn default() -> Self {
        Self {
            partner_id: 0,
            relationship: SymbiosisType::Commensalism,
            bonus_factor: 0.2,
            drain_rate: 0.05,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_relationship_is_commensalism() {
        let link = SymbiosisLink::default();
        assert_eq!(link.relationship, SymbiosisType::Commensalism);
    }

    #[test]
    fn default_bonus_factor_and_drain_rate() {
        let link = SymbiosisLink::default();
        assert!((link.bonus_factor - 0.2).abs() < 1e-5);
        assert!((link.drain_rate - 0.05).abs() < 1e-5);
    }

    #[test]
    fn symbiosis_type_equality() {
        assert_eq!(SymbiosisType::Mutualism, SymbiosisType::Mutualism);
        assert_ne!(SymbiosisType::Mutualism, SymbiosisType::Parasitism);
        assert_ne!(SymbiosisType::Parasitism, SymbiosisType::Commensalism);
    }
}
