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
/// Layer 10: Resonance Link — Effect Entities
///
/// Buff/debuff temporal: modifica un campo del target mientras la fuente viva.
/// Temporary buff/debuff: modifies a target field while the source lives.
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ResonanceLink {
    /// Entidad cuyo estado será modificado.
    #[serde(skip, default = "placeholder_entity")]
    pub target: Entity,

    /// Qué campo del target se modifica.
    pub modified_field: ModifiedField,

    /// Magnitud del modificador (ej. 0.5 = slow, 2.0 = haste).
    /// Invariante: `>= 0` (Ax4/Ax5 — un multiplicador negativo sobre
    /// disipación/energía crearía qe). Construir vía `new`/`set_magnitude`.
    pub(crate) magnitude: f32,
}

impl ResonanceLink {
    /// Crea un enlace con `magnitude` clampeada a `>= 0`.
    pub fn new(target: Entity, modified_field: ModifiedField, magnitude: f32) -> Self {
        Self {
            target,
            modified_field,
            magnitude: magnitude.max(0.0),
        }
    }

    #[inline]
    pub fn magnitude(&self) -> f32 {
        self.magnitude
    }

    /// Asigna `magnitude` clampeada a `>= 0`; no muta si el valor no cambia
    /// (evita falsos positivos de `Changed<ResonanceLink>`).
    pub fn set_magnitude(&mut self, magnitude: f32) {
        let next = magnitude.max(0.0);
        if self.magnitude != next {
            self.magnitude = next;
        }
    }
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
        let link = ResonanceLink::new(target, ModifiedField::MotorIntakeMultiplier, 1.25);
        assert_eq!(link.target, target);
        assert_eq!(link.modified_field, ModifiedField::MotorIntakeMultiplier);
        assert!((link.magnitude() - 1.25).abs() < 1e-5);
    }

    #[test]
    fn new_clamps_negative_magnitude_to_zero() {
        let link = ResonanceLink::new(Entity::PLACEHOLDER, ModifiedField::DissipationMultiplier, -2.0);
        assert_eq!(link.magnitude(), 0.0, "Ax4/Ax5: multiplicador no puede ser negativo");
    }

    #[test]
    fn set_magnitude_clamps_and_is_idempotent() {
        let mut link = ResonanceLink::new(Entity::PLACEHOLDER, ModifiedField::VelocityMultiplier, 1.0);
        link.set_magnitude(-5.0);
        assert_eq!(link.magnitude(), 0.0);
        link.set_magnitude(2.0);
        assert!((link.magnitude() - 2.0).abs() < 1e-6);
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
