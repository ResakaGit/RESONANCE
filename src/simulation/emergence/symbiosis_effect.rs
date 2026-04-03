//! ET-5: Symbiosis effects — apply mutualism/parasitism drain/benefit between linked entities.

use bevy::prelude::*;

use crate::blueprint::constants::DISSIPATION_MULT_SOLID;
use crate::blueprint::equations::derived_thresholds::{DISSIPATION_GAS, DISSIPATION_LIQUID};
use crate::blueprint::equations::emergence::symbiosis::{
    is_symbiosis_stable, mutualism_benefit, parasitism_drain,
};
use crate::layers::{BaseEnergy, SymbiosisLink, SymbiosisType};

/// Fracción de qe propia usada como ingreso base en mutualismo (2× disipación sólida).
/// Fraction of own qe used as base intake in mutualism (2× solid dissipation).
const MUTUALISM_INTAKE_FRACTION: f32 = DISSIPATION_MULT_SOLID * 2.0;

/// Pérdida por transferencia en mutualismo = 2.5× disipación líquida (Axiom 4).
/// Cooperación tiene pérdida moderada — análogo al flujo en fase líquida.
/// Transfer loss in mutualism = 2.5× liquid dissipation (Axiom 4).
const MUTUALISM_TRANSFER_LOSS: f32 = DISSIPATION_LIQUID * 2.5;

/// Pérdida por transferencia en parasitismo = 1.25× disipación gas (Axiom 4).
/// Extracción forzada — pérdida alta análoga a fase gaseosa.
/// Transfer loss in parasitism = 1.25× gas dissipation (Axiom 4).
const PARASITISM_TRANSFER_LOSS: f32 = DISSIPATION_GAS * 1.25;

/// Fracción de qe propia usada como ingreso base en comensalismo.
/// Derivada de DISSIPATION_SOLID — el mínimo de interacción energética (Axiom 4).
/// Derived from DISSIPATION_SOLID — minimum energy interaction (Axiom 4).
const COMMENSALISM_INTAKE_FRACTION: f32 = DISSIPATION_MULT_SOLID;

/// Applies symbiosis effects each tick. Removes unstable links.
pub fn symbiosis_effect_system(
    mut commands: Commands,
    mut query: Query<(Entity, &SymbiosisLink, &mut BaseEnergy)>,
) {
    for (entity, link, mut energy) in &mut query {
        let qe = energy.qe();
        let (benefit_self, cost) = match link.relationship {
            SymbiosisType::Mutualism => {
                let b = mutualism_benefit(qe * MUTUALISM_INTAKE_FRACTION, link.bonus_factor);
                (b, b * MUTUALISM_TRANSFER_LOSS)
            }
            SymbiosisType::Parasitism => {
                let drain = parasitism_drain(qe, link.drain_rate);
                (drain, drain * PARASITISM_TRANSFER_LOSS)
            }
            SymbiosisType::Commensalism => {
                let b = mutualism_benefit(qe * COMMENSALISM_INTAKE_FRACTION, link.bonus_factor);
                (b, 0.0)
            }
        };

        let new_qe = (qe + benefit_self - cost).max(0.0);
        if qe != new_qe {
            energy.set_qe(new_qe);
        }

        // is_symbiosis_stable(a_with_b, a_without_b, b_with_a, b_without_a)
        // Stable if both are at least as well off together as apart
        let a_with = qe + benefit_self;
        let a_without = qe;
        let b_with = qe; // simplified: partner assumed symmetric
        let b_without = qe;
        if !is_symbiosis_stable(a_with, a_without, b_with, b_without) {
            commands.entity(entity).remove::<SymbiosisLink>();
        }
    }
}
