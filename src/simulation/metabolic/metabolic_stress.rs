//! Estrés metabólico: muerte por inanición cuando `qe` cae por debajo del umbral adaptativo.
//!
//! Emite [`DeathEvent`]; el despawn lo resuelve `faction_identity_system`.
//! Fase: [`Phase::MetabolicLayer`], después de `growth_budget_system`.
//!
//! **Contrato:** muerte por inanición cuando `0 < qe < umbral_adaptativo`. Si `qe == 0`, la muerte
//! por L0 la declara [`EnergyOps::drain`](crate::layers::energy::EnergyOps::drain) — evita
//! `DeathEvent` duplicado el mismo tick.

use bevy::prelude::*;

use crate::blueprint::constants::METABOLIC_STARVATION_BASE_THRESHOLD_QE;
use crate::blueprint::equations;
use crate::events::{DeathCause, DeathEvent};
use crate::layers::{BaseEnergy, InferenceProfile};

/// Evalúa umbral adaptativo y emite `DeathEvent` por inanición (qe residual bajo mínimo).
pub fn metabolic_stress_death_system(
    query: Query<(Entity, &BaseEnergy, Option<&InferenceProfile>)>,
    mut death_events: EventWriter<DeathEvent>,
) {
    for (entity, energy, profile_opt) in &query {
        let resilience = InferenceProfile::resilience_effective(profile_opt);
        let threshold =
            equations::starvation_threshold(METABOLIC_STARVATION_BASE_THRESHOLD_QE, resilience);
        let qe = energy.qe();
        // `qe == 0`: `EnergyOps` ya emitió `DeathEvent`; no duplicar.
        if qe > 0.0 && equations::metabolic_viability(qe, threshold) < 1.0 {
            death_events.send(DeathEvent {
                entity,
                cause: DeathCause::Dissipation,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::test_support::drain_death_events;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        app.add_systems(Update, metabolic_stress_death_system);
        app
    }

    #[test]
    fn healthy_entity_does_not_die() {
        let mut app = test_app();
        app.world_mut().spawn((
            BaseEnergy::new(100.0),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.update();
        assert!(drain_death_events(&mut app).is_empty());
    }

    #[test]
    fn starving_entity_emits_death_event() {
        let mut app = test_app();
        // resilience 0.5 → threshold = 5 * 0.6 = 3.0; qe=1 queda bajo el umbral (>0).
        let e = app
            .world_mut()
            .spawn((
                BaseEnergy::new(1.0),
                InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
            ))
            .id();
        app.update();
        let deaths = drain_death_events(&mut app);
        assert_eq!(deaths.len(), 1);
        assert_eq!(deaths[0].entity, e);
    }

    #[test]
    fn zero_qe_does_not_emit_metabolic_death() {
        let mut app = test_app();
        app.world_mut().spawn((
            BaseEnergy::new(0.0),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.update();
        assert!(
            drain_death_events(&mut app).is_empty(),
            "qe=0 lo maneja EnergyOps, no duplicar aquí"
        );
    }

    #[test]
    fn entity_without_inference_profile_uses_default_resilience() {
        let mut app = test_app();
        let e = app.world_mut().spawn(BaseEnergy::new(1.0)).id();
        app.update();
        let deaths = drain_death_events(&mut app);
        assert_eq!(deaths.len(), 1);
        assert_eq!(deaths[0].entity, e);
    }

    #[test]
    fn high_resilience_survives_longer() {
        let mut app = test_app();
        // threshold = 5 * (1 - 0.95*0.8) = 1.2; qe=2 no cumple qe < threshold → no muere
        app.world_mut().spawn((
            BaseEnergy::new(2.0),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.95),
        ));
        app.update();
        assert!(
            drain_death_events(&mut app).is_empty(),
            "High resilience should survive low qe"
        );
    }
}
