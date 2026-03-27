use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blueprint::constants::LINK_NEUTRAL_MULTIPLIER;

fn placeholder_entity() -> Entity {
    Entity::PLACEHOLDER
}

/// Campo de una entidad target que puede ser modificado por una entidad-efecto.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum ModifiedField {
    VelocityMultiplier,
    BondEnergyMultiplier,
    MotorIntakeMultiplier,
    MotorOutputMultiplier,
    DissipationMultiplier,
    ConductivityMultiplier,
}

/// Capa 10: Enlace de Resonancia — Entidades-efecto tipo B.
///
/// Une una entidad-efecto (fuente) con un target y aplica un modificador
/// temporal mientras la entidad-efecto tenga energía.
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ResonanceLink {
    /// Entidad cuyo estado será modificado.
    #[serde(skip, default = "placeholder_entity")]
    pub target: Entity,

    /// Qué campo del target se modifica.
    pub modified_field: ModifiedField,

    /// Magnitud del modificador (ej. 0.5 = slow, 2.0 = haste).
    pub magnitude: f32,
}

// --- Overlays efímeros (DoD: máx. 4 campos por componente) ---

/// Multiplicadores de cinemática / disipación (Capa 10 → overlay).
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ResonanceFlowOverlay {
    pub velocity_multiplier: f32,
    pub dissipation_multiplier: f32,
}

impl Default for ResonanceFlowOverlay {
    fn default() -> Self {
        Self {
            velocity_multiplier: LINK_NEUTRAL_MULTIPLIER,
            dissipation_multiplier: LINK_NEUTRAL_MULTIPLIER,
        }
    }
}

/// Multiplicadores del motor alquímico (Capa 5).
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ResonanceMotorOverlay {
    pub motor_intake_multiplier: f32,
    pub motor_output_multiplier: f32,
}

impl Default for ResonanceMotorOverlay {
    fn default() -> Self {
        Self {
            motor_intake_multiplier: LINK_NEUTRAL_MULTIPLIER,
            motor_output_multiplier: LINK_NEUTRAL_MULTIPLIER,
        }
    }
}

/// Multiplicadores térmicos / coherencia (Capa 4 + conducción).
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ResonanceThermalOverlay {
    pub bond_energy_multiplier: f32,
    pub conductivity_multiplier: f32,
}

impl Default for ResonanceThermalOverlay {
    fn default() -> Self {
        Self {
            bond_energy_multiplier: LINK_NEUTRAL_MULTIPLIER,
            conductivity_multiplier: LINK_NEUTRAL_MULTIPLIER,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Entity;

    #[test]
    fn resonance_link_stores_target_and_field() {
        let target = Entity::from_raw(42);
        let link = ResonanceLink {
            target,
            modified_field: ModifiedField::MotorIntakeMultiplier,
            magnitude: 1.25,
        };
        assert_eq!(link.target, target);
        assert_eq!(link.modified_field, ModifiedField::MotorIntakeMultiplier);
        assert!((link.magnitude - 1.25).abs() < 1e-5);
    }

    #[test]
    fn flow_overlay_default_is_neutral_multipliers() {
        let o = ResonanceFlowOverlay::default();
        assert!((o.velocity_multiplier - LINK_NEUTRAL_MULTIPLIER).abs() < 1e-5);
        assert!((o.dissipation_multiplier - LINK_NEUTRAL_MULTIPLIER).abs() < 1e-5);
    }

    #[test]
    fn motor_and_thermal_overlay_defaults_neutral() {
        let m = ResonanceMotorOverlay::default();
        let t = ResonanceThermalOverlay::default();
        assert!((m.motor_intake_multiplier - LINK_NEUTRAL_MULTIPLIER).abs() < 1e-5);
        assert!((t.bond_energy_multiplier - LINK_NEUTRAL_MULTIPLIER).abs() < 1e-5);
    }
}
