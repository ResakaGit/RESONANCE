//! ET-5: Symbiosis effects — apply mutualism/parasitism drain/benefit between linked entities.

use bevy::prelude::*;

use crate::blueprint::equations::emergence::symbiosis::{
    is_symbiosis_stable, mutualism_benefit, parasitism_drain,
};
use crate::layers::{BaseEnergy, SymbiosisLink, SymbiosisType};

/// Fracción de qe propia usada como ingreso base en mutualismo.
/// Fraction of own qe used as base intake in mutualism.
const MUTUALISM_INTAKE_FRACTION: f32 = 0.01;

/// Fracción de pérdida por transferencia en mutualismo (Axiom 4: disipación).
/// Transfer loss fraction in mutualism (Axiom 4: dissipation).
const MUTUALISM_TRANSFER_LOSS: f32 = 0.05;

/// Fracción de pérdida por transferencia en parasitismo (Axiom 4: disipación).
/// Transfer loss fraction in parasitism (Axiom 4: dissipation).
const PARASITISM_TRANSFER_LOSS: f32 = 0.1;

/// Fracción de qe propia usada como ingreso base en comensalismo.
/// Fraction of own qe used as base intake in commensalism.
const COMMENSALISM_INTAKE_FRACTION: f32 = 0.005;

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
        if qe != new_qe { energy.set_qe(new_qe); }

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
