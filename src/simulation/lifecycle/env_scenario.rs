use bevy::prelude::*;

use crate::blueprint::equations::{
    env_intake_gain, env_maintenance_penalty, env_stress_penalty, organ_base_viability,
    organ_viability_score,
};
use crate::layers::{BaseEnergy, GrowthBudget};

/// Snapshot exógeno normalizado para sandbox ambiental LI8.
#[derive(Resource, Debug, Clone, Copy)]
pub struct EnvScenarioSnapshot {
    pub food_density_t: f32,
    pub predation_pressure_t: f32,
    pub temperature_t: f32,
    pub medium_density_t: f32,
}

impl Default for EnvScenarioSnapshot {
    fn default() -> Self {
        Self {
            food_density_t: 0.5,
            predation_pressure_t: 0.5,
            temperature_t: 0.5,
            medium_density_t: 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct EnvPressureFactors {
    intake_gain: f32,
    maintenance_penalty: f32,
    stress_penalty: f32,
}

/// Cache de viabilidad efectiva para inferencia de órganos.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct EffectiveOrganViability {
    pub value: f32,
}

#[inline]
fn pressure_from_snapshot(snapshot: EnvScenarioSnapshot) -> EnvPressureFactors {
    EnvPressureFactors {
        intake_gain: env_intake_gain(snapshot.food_density_t, snapshot.medium_density_t),
        maintenance_penalty: env_maintenance_penalty(snapshot.temperature_t, snapshot.predation_pressure_t),
        stress_penalty: env_stress_penalty(snapshot.predation_pressure_t, snapshot.medium_density_t),
    }
}

/// Una transformación: viabilidad base + presión -> viabilidad efectiva para morfología.
pub fn organ_viability_with_env_system(
    snapshot: Res<EnvScenarioSnapshot>,
    mut query: Query<(&BaseEnergy, &GrowthBudget, &mut EffectiveOrganViability)>,
) {
    let pressure = pressure_from_snapshot(*snapshot);
    for (energy, growth, mut effective) in &mut query {
        let base_viability = organ_base_viability(energy.qe(), growth.efficiency);
        let next = organ_viability_score(
            base_viability,
            pressure.intake_gain,
            pressure.maintenance_penalty,
            pressure.stress_penalty,
        );
        if effective.value != next {
            effective.value = next;
        }
    }
}

/// Inserta cache de viabilidad efectiva para entidades vivas.
pub fn effective_viability_init_system(
    mut commands: Commands,
    query: Query<Entity, (With<GrowthBudget>, Without<EffectiveOrganViability>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(EffectiveOrganViability::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::EnvContext;

    impl EnvScenarioSnapshot {
        fn abundant_plain() -> Self {
            Self {
                food_density_t: 0.95,
                predation_pressure_t: 0.1,
                temperature_t: 0.5,
                medium_density_t: 0.45,
            }
        }

        fn scarce_cold() -> Self {
            Self {
                food_density_t: 0.15,
                predation_pressure_t: 0.45,
                temperature_t: 0.1,
                medium_density_t: 0.7,
            }
        }

        fn hostile_hot() -> Self {
            Self {
                food_density_t: 0.5,
                predation_pressure_t: 0.9,
                temperature_t: 0.95,
                medium_density_t: 0.6,
            }
        }
    }

    #[test]
    fn abundant_plain_yields_higher_effective_viability_than_scarce_cold() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(EnvScenarioSnapshot::abundant_plain());
        let entity = app
            .world_mut()
            .spawn((
                BaseEnergy::new(300.0),
                GrowthBudget::new(2.0, 0, 0.8),
                EffectiveOrganViability::default(),
            ))
            .id();
        app.add_systems(Update, organ_viability_with_env_system);
        app.update();
        let rich = app
            .world()
            .entity(entity)
            .get::<EffectiveOrganViability>()
            .expect("debe existir cache")
            .value;

        app.insert_resource(EnvScenarioSnapshot::scarce_cold());
        app.update();
        let poor = app
            .world()
            .entity(entity)
            .get::<EffectiveOrganViability>()
            .expect("debe existir cache")
            .value;
        assert!(rich > poor, "abundante debe superar escaso: rich={rich}, poor={poor}");
    }

    #[test]
    fn hostile_hot_profile_is_finite_and_non_negative() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(EnvScenarioSnapshot::hostile_hot());
        let profile = pressure_from_snapshot(*app.world().resource::<EnvScenarioSnapshot>());
        assert!(profile.intake_gain.is_finite() && profile.intake_gain >= 0.0);
        assert!(profile.maintenance_penalty.is_finite() && profile.maintenance_penalty >= 0.0);
        assert!(profile.stress_penalty.is_finite() && profile.stress_penalty >= 0.0);
    }

    #[test]
    fn env_context_from_snapshot_is_clamped() {
        let snapshot = EnvScenarioSnapshot {
            food_density_t: 2.0,
            predation_pressure_t: -1.0,
            temperature_t: f32::NAN,
            medium_density_t: 0.4,
        };
        let ctx = EnvContext::new(
            snapshot.food_density_t,
            snapshot.predation_pressure_t,
            snapshot.temperature_t,
            snapshot.medium_density_t,
            0.7,
        );
        assert_eq!(ctx.food_density_t, 1.0);
        assert_eq!(ctx.predation_pressure_t, 0.0);
        assert_eq!(ctx.temperature_t, 0.0);
        assert_eq!(ctx.medium_density_t, 0.4);
        assert_eq!(ctx.competition_t, 0.7);
    }
}
