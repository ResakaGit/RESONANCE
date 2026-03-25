//! Grimorios prearmados para demos MOBA (tres slots en Q/W/E; R sin habilidad en este preset).

use crate::blueprint::ElementId;
use crate::blueprint::recipes::EffectRecipe;
use crate::layers::ModifiedField;
use crate::layers::will::{AbilityCastSpec, AbilityOutput, AbilitySlot, Grimoire, TargetingMode};

/// Tres habilidades del FireMage: Fireball (punto), Ember Shield (self), Lava Surge (punto).
pub fn fire_mage_qwer_grimoire() -> Grimoire {
    let mut g = Grimoire::default();
    let _ = g.push_ability(AbilitySlot {
        name: "Fireball".into(),
        output: AbilityOutput::Projectile {
            element_id: ElementId::from_name("Ignis"),
            radius: 0.5,
            speed: 15.0,
            effect: None,
        },
        cast: AbilityCastSpec {
            cost_qe: 50.0,
            targeting: TargetingMode::PointTarget { range: 28.0 },
            min_channeling_secs: 0.0,
        },
    });
    let _ = g.push_ability(AbilitySlot {
        name: "Ember Shield".into(),
        output: AbilityOutput::SelfBuff {
            effect: EffectRecipe {
                field: ModifiedField::ConductivityMultiplier,
                magnitude: 0.3,
                fuel_qe: 240.0,
                dissipation: 2.0,
            },
        },
        cast: AbilityCastSpec {
            cost_qe: 30.0,
            targeting: TargetingMode::NoTarget,
            min_channeling_secs: 0.0,
        },
    });
    let _ = g.push_ability(AbilitySlot {
        name: "Lava Surge".into(),
        output: AbilityOutput::Projectile {
            element_id: ElementId::from_name("Ignis"),
            radius: 1.0,
            speed: 10.0,
            effect: Some(EffectRecipe {
                field: ModifiedField::DissipationMultiplier,
                magnitude: 3.0,
                fuel_qe: 180.0,
                dissipation: 1.5,
            }),
        },
        cast: AbilityCastSpec {
            cost_qe: 80.0,
            targeting: TargetingMode::PointTarget { range: 22.0 },
            min_channeling_secs: 0.0,
        },
    });
    g
}
