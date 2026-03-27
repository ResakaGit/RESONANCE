//! ET-5: Symbiosis effects — apply mutualism/parasitism drain/benefit between linked entities.

use bevy::prelude::*;

use crate::blueprint::equations::emergence::symbiosis::{
    is_symbiosis_stable, mutualism_benefit, parasitism_drain,
};
use crate::layers::{BaseEnergy, SymbiosisLink, SymbiosisType};

/// Applies symbiosis effects each tick. Removes unstable links.
pub fn symbiosis_effect_system(
    mut commands: Commands,
    mut query: Query<(Entity, &SymbiosisLink, &mut BaseEnergy)>,
) {
    for (entity, link, mut energy) in &mut query {
        let qe = energy.qe();
        let (benefit_self, cost) = match link.relationship {
            SymbiosisType::Mutualism => {
                // mutualism_benefit(own_intake, partner_bonus_factor)
                let b = mutualism_benefit(qe * 0.01, link.bonus_factor);
                (b, b * 0.05) // 5% transfer loss (Axiom 4)
            }
            SymbiosisType::Parasitism => {
                // parasitism_drain(host_qe, drain_rate) — self is parasite, gains
                let drain = parasitism_drain(qe, link.drain_rate);
                (drain, drain * 0.1)
            }
            SymbiosisType::Commensalism => {
                let b = mutualism_benefit(qe * 0.005, link.bonus_factor);
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
