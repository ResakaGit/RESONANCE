use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::blueprint::constants::{
    DERIVED_DEFAULT_DISSIPATION_MULTIPLIER, THERMAL_CONDUCTIVITY_FALLBACK,
};
use crate::blueprint::equations;
use crate::layers::{BaseEnergy, MatterCoherence, MatterState, SpatialVolume};

#[derive(SystemParam)]
pub struct PhysicsOps<'w, 's> {
    energy: Query<'w, 's, &'static BaseEnergy>,
    volume: Query<'w, 's, &'static SpatialVolume>,
    coherence: Query<'w, 's, &'static MatterCoherence>,
}

impl<'w, 's> PhysicsOps<'w, 's> {
    /// Estado de fase actual — usado como pista de histéresis en `BridgedPhysicsOps` (sprint B4).
    #[inline]
    pub fn matter_state(&self, entity: Entity) -> Option<MatterState> {
        self.coherence.get(entity).ok().map(|c| c.state())
    }

    pub fn density(&self, entity: Entity) -> Option<f32> {
        let energy = self.energy.get(entity).ok()?;
        let volume = self.volume.get(entity).ok()?;
        Some(volume.density(energy.qe()))
    }

    pub fn temperature(&self, entity: Entity) -> Option<f32> {
        let density = self.density(entity)?;
        Some(equations::equivalent_temperature(density))
    }

    pub fn velocity_limit(&self, entity: Entity) -> f32 {
        if let Ok(coherence) = self.coherence.get(entity) {
            coherence.velocity_limit().unwrap_or(f32::INFINITY)
        } else {
            f32::INFINITY
        }
    }

    pub fn is_solid(&self, entity: Entity) -> bool {
        matches!(
            self.coherence.get(entity).ok().map(|c| c.state()),
            Some(MatterState::Solid)
        )
    }

    pub fn dissipation_multiplier(&self, entity: Entity) -> f32 {
        self.coherence
            .get(entity)
            .ok()
            .map(|c| c.dissipation_multiplier())
            .unwrap_or(DERIVED_DEFAULT_DISSIPATION_MULTIPLIER)
    }

    pub fn conductivity(&self, entity: Entity) -> f32 {
        self.coherence
            .get(entity)
            .ok()
            .map(|c| c.thermal_conductivity())
            .unwrap_or(THERMAL_CONDUCTIVITY_FALLBACK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{
        DERIVED_DEFAULT_DISSIPATION_MULTIPLIER, THERMAL_CONDUCTIVITY_FALLBACK,
    };
    use crate::blueprint::equations;
    use bevy::ecs::system::SystemState;

    #[test]
    fn density_matches_spatial_volume_formula() {
        let mut world = World::new();
        let e = world
            .spawn((BaseEnergy::new(100.0), SpatialVolume::new(0.5)))
            .id();
        let mut state = SystemState::<PhysicsOps>::new(&mut world);
        let ops = state.get(&world);
        let d = ops.density(e).expect("density");
        let vol = SpatialVolume::new(0.5);
        let expected = vol.density(100.0);
        assert!((d - expected).abs() < 1e-4, "got {d} expected {expected}");
    }

    #[test]
    fn temperature_tracks_equivalent_temperature_of_density() {
        let mut world = World::new();
        let e = world
            .spawn((BaseEnergy::new(100.0), SpatialVolume::new(0.5)))
            .id();
        let mut state = SystemState::<PhysicsOps>::new(&mut world);
        let ops = state.get(&world);
        let rho = ops.density(e).unwrap();
        let t = ops.temperature(e).unwrap();
        assert!((t - equations::equivalent_temperature(rho)).abs() < 1e-4);
    }

    #[test]
    fn velocity_limit_reads_coherence_default_solid() {
        let mut world = World::new();
        let e = world
            .spawn((
                BaseEnergy::default(),
                SpatialVolume::default(),
                MatterCoherence::default(),
            ))
            .id();
        let mut state = SystemState::<PhysicsOps>::new(&mut world);
        let ops = state.get(&world);
        assert_eq!(ops.velocity_limit(e), 0.0);
    }

    #[test]
    fn without_coherence_velocity_unbounded_and_dissipation_fallback() {
        let mut world = World::new();
        let e = world
            .spawn((BaseEnergy::default(), SpatialVolume::default()))
            .id();
        let mut state = SystemState::<PhysicsOps>::new(&mut world);
        let ops = state.get(&world);
        assert_eq!(ops.velocity_limit(e), f32::INFINITY);
        assert_eq!(
            ops.dissipation_multiplier(e),
            DERIVED_DEFAULT_DISSIPATION_MULTIPLIER
        );
        assert_eq!(ops.conductivity(e), THERMAL_CONDUCTIVITY_FALLBACK);
    }
}
