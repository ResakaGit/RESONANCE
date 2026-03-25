use bevy::prelude::*;

use crate::blueprint::{constants, equations};
use crate::layers::{
    AllometricRadiusAnchor, BaseEnergy, CapabilitySet, GrowthBudget, GrowthIntent, SpatialVolume,
};

/// Escala de tiempo biológico para crecimiento alométrico.
#[derive(Resource, Debug, Clone, Copy)]
pub struct AllometricGrowthTimeScale {
    /// Multiplicador del delta de crecimiento por tick.
    pub growth_multiplier: f32,
}

impl Default for AllometricGrowthTimeScale {
    fn default() -> Self {
        Self {
            growth_multiplier: 1.0,
        }
    }
}

/// Capa 4→5: crecimiento radial por feedback alométrico/logístico.
/// Corre en `Phase::MorphologicalLayer` después de `growth_budget_system`.
/// Nota de orden: no forzar `.before(faction_identity_system)` porque
/// `faction_identity_system` vive en `Phase::MetabolicLayer` y las fases ya
/// están encadenadas (`MetabolicLayer` -> `MorphologicalLayer`) en el pipeline.
pub fn allometric_growth_system(
    mut commands: Commands,
    time_scale: Option<Res<AllometricGrowthTimeScale>>,
    mut query: Query<(
        Entity,
        &BaseEnergy,
        &GrowthBudget,
        &CapabilitySet,
        Option<&GrowthIntent>,
        &mut SpatialVolume,
        Option<&AllometricRadiusAnchor>,
    )>,
) {
    let growth_multiplier = time_scale
        .map(|s| s.growth_multiplier.max(0.0))
        .unwrap_or(1.0);
    for (entity, _energy, budget, cap, intent_opt, mut volume, anchor_opt) in &mut query {
        if !cap.can_grow() {
            continue;
        }
        if budget.biomass_available <= 0.0 {
            continue;
        }
        let base_radius = anchor_opt.map(|a| a.base_radius).unwrap_or(volume.radius);
        let max_radius = equations::allometric_max_radius(base_radius, constants::ALLOMETRIC_MAX_RADIUS_FACTOR);
        let Some(intent) = intent_opt else {
            continue;
        };
        let delta = intent.delta_radius * growth_multiplier;
        let next_radius = if max_radius - volume.radius <= constants::VOLUME_WRITE_EPS {
            max_radius
        } else {
            (volume.radius + delta).min(max_radius)
        };
        if (next_radius - volume.radius).abs() > constants::VOLUME_WRITE_EPS {
            volume.set_radius(next_radius);
        }
        if anchor_opt.is_none() {
            commands
                .entity(entity)
                .insert(AllometricRadiusAnchor::new(base_radius));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::allometric_growth_system;
    use crate::layers::{BaseEnergy, CapabilitySet, GrowthBudget, GrowthIntent, SpatialVolume};
    use bevy::prelude::*;

    #[test]
    fn allometric_growth_increases_radius_when_budget_positive() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, allometric_growth_system);
        let e = app
            .world_mut()
            .spawn((
                GrowthBudget::new(1.0, 0, 1.0),
                BaseEnergy::new(10.0),
                CapabilitySet::new(CapabilitySet::GROW),
                GrowthIntent::new(0.01, 1.0, 1.0),
                SpatialVolume::new(0.5),
            ))
            .id();
        let before = app.world().entity(e).get::<SpatialVolume>().expect("volume").radius;
        app.update();
        let after = app.world().entity(e).get::<SpatialVolume>().expect("volume").radius;
        assert!(after > before, "before={before} after={after}");
    }

    #[test]
    fn allometric_growth_saturates_at_max_radius_factor() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, allometric_growth_system);
        let e = app
            .world_mut()
            .spawn((
                GrowthBudget::new(1_000.0, 0, 1.0),
                BaseEnergy::new(10.0),
                CapabilitySet::new(CapabilitySet::GROW),
                GrowthIntent::new(0.01, 1.0, 1.0),
                SpatialVolume::new(0.5),
            ))
            .id();
        for _ in 0..2_000 {
            app.update();
        }
        let radius = app.world().entity(e).get::<SpatialVolume>().expect("volume").radius;
        assert!(radius <= 1.5 + 1e-3, "radius={radius}");
    }
}
