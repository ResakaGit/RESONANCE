//! D4: Thermoregulation systems — extend homeostasis with thermal cost and stability check.

use bevy::prelude::*;

use crate::blueprint::{constants, equations};
use crate::layers::{
    AlchemicalEngine, AmbientPressure, BaseEnergy, ContainedIn, Homeostasis,
    MatterCoherence, OscillatorySignature, SpatialVolume,
};
use crate::simulation::time_compat::simulation_delta_secs;

/// Transient flag: frequency drift exceeds Homeostasis stability band.
/// Future D8 (MorphologicalLayer) consumes this for morphological adaptation.
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct ThermalStressFlag {
    pub drift_hz: f32,
}

/// Computes thermoregulation cost and drains from AlchemicalEngine buffer.
/// Phase: ChemicalLayer, after homeostasis_system.
pub fn thermoregulation_cost_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    hosts: Query<&AmbientPressure>,
    mut query: Query<(
        &ContainedIn,
        &BaseEnergy,
        &SpatialVolume,
        &MatterCoherence,
        &mut AlchemicalEngine,
    )>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }

    for (contained, energy, volume, matter, mut engine) in &mut query {
        let qe = energy.qe();
        if qe <= 0.0 {
            continue;
        }

        let Ok(pressure) = hosts.get(contained.host) else {
            continue;
        };

        let buffer_cap = engine.buffer_cap();
        if buffer_cap <= 0.0 {
            continue;
        }
        let qe_fraction = engine.buffer_level() / buffer_cap;
        if qe_fraction < constants::THERMOREG_MIN_QE_FRACTION {
            continue;
        }

        let density = volume.density(qe);
        let t_core = equations::equivalent_temperature(density);
        let t_env = constants::ENDOTHERM_TARGET_TEMP
            + pressure.delta_qe_constant * constants::THERMOREG_DELTA_TO_TEMP_SCALE;

        let cost = equations::thermoregulation_cost(
            t_core,
            t_env,
            qe,
            matter.thermal_conductivity(),
            constants::INSULATION_BASE,
        ) * dt;

        if cost <= 0.0 {
            continue;
        }

        let drain = cost.min(engine.buffer_level());
        if drain > 0.0 {
            engine.try_subtract_buffer(drain);
        }
    }
}

/// Flags entities whose frequency drifted beyond Homeostasis stability band.
/// Phase: ChemicalLayer, after thermoregulation_cost_system.
pub fn homeostasis_stability_check_system(
    mut commands: Commands,
    hosts: Query<&OscillatorySignature, (With<AmbientPressure>, Without<Homeostasis>)>,
    query: Query<(
        Entity,
        &ContainedIn,
        &Homeostasis,
        &OscillatorySignature,
    )>,
) {
    for (entity, contained, homeostasis, signature) in &query {
        if !homeostasis.enabled {
            continue;
        }

        let Ok(host_wave) = hosts.get(contained.host) else {
            continue;
        };

        let drift = (signature.frequency_hz() - host_wave.frequency_hz()).abs();
        if drift > homeostasis.stability_band_hz {
            commands.entity(entity).insert(ThermalStressFlag { drift_hz: drift });
        } else {
            commands.entity(entity).remove::<ThermalStressFlag>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::DeathEvent;
    use crate::layers::ContactType;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        app.add_systems(
            Update,
            (thermoregulation_cost_system, homeostasis_stability_check_system).chain(),
        );
        app
    }

    fn spawn_host(app: &mut App, delta_qe: f32, freq_hz: f32) -> Entity {
        app.world_mut()
            .spawn((
                AmbientPressure {
                    delta_qe_constant: delta_qe,
                    terrain_viscosity: 1.0,
                },
                OscillatorySignature::new(freq_hz, 0.0),
            ))
            .id()
    }

    // ── thermoregulation_cost_system ──

    #[test]
    fn thermoregulation_drains_buffer_in_hostile_environment() {
        let mut app = test_app();
        let host = spawn_host(&mut app, -5.0, 100.0);

        let initial_buffer = 50.0;
        let entity = app
            .world_mut()
            .spawn((
                ContainedIn { host, contact: ContactType::Immersed },
                BaseEnergy::new(1000.0),
                SpatialVolume::new(2.0),
                MatterCoherence::new(
                    crate::layers::MatterState::Liquid,
                    500.0,
                    0.5,
                ),
                AlchemicalEngine::new(100.0, 10.0, 10.0, initial_buffer),
            ))
            .id();

        // Run several updates to accumulate non-zero dt.
        for _ in 0..3 {
            app.update();
        }

        let engine = app.world().get::<AlchemicalEngine>(entity).unwrap();
        assert!(
            engine.buffer_level() <= initial_buffer,
            "buffer should have been drained or stayed same; got {}",
            engine.buffer_level(),
        );
    }

    #[test]
    fn thermoregulation_skips_when_buffer_fraction_low() {
        let mut app = test_app();
        let host = spawn_host(&mut app, -5.0, 100.0);

        // Buffer fraction = 5/100 = 0.05 < THERMOREG_MIN_QE_FRACTION (0.1)
        let entity = app
            .world_mut()
            .spawn((
                ContainedIn { host, contact: ContactType::Immersed },
                BaseEnergy::new(1000.0),
                SpatialVolume::new(2.0),
                MatterCoherence::new(
                    crate::layers::MatterState::Liquid,
                    500.0,
                    0.5,
                ),
                AlchemicalEngine::new(100.0, 10.0, 10.0, 5.0),
            ))
            .id();

        app.update();
        app.update();

        let engine = app.world().get::<AlchemicalEngine>(entity).unwrap();
        assert!(
            (engine.buffer_level() - 5.0).abs() < 1e-5,
            "buffer should NOT have been drained: {}",
            engine.buffer_level(),
        );
    }

    #[test]
    fn thermoregulation_no_drain_in_neutral_biome() {
        let mut app = test_app();
        // delta_qe=0 → t_env = ENDOTHERM_TARGET_TEMP = 310.
        // If entity density → equivalent_temp = 310, cost = 0.
        let host = spawn_host(&mut app, 0.0, 100.0);

        let entity = app
            .world_mut()
            .spawn((
                ContainedIn { host, contact: ContactType::Immersed },
                // Craft density so equivalent_temperature ≈ 310.
                // equivalent_temperature = density / GAME_BOLTZMANN
                // density = qe / sphere_volume(radius)
                // We pick values that give the neutral temperature.
                BaseEnergy::new(100.0),
                SpatialVolume::new(5.0),
                MatterCoherence::new(
                    crate::layers::MatterState::Liquid,
                    500.0,
                    0.0, // zero conductivity → zero cost
                ),
                AlchemicalEngine::new(100.0, 10.0, 10.0, 50.0),
            ))
            .id();

        app.update();
        app.update();

        let engine = app.world().get::<AlchemicalEngine>(entity).unwrap();
        assert!(
            (engine.buffer_level() - 50.0).abs() < 1e-3,
            "no drain expected with zero conductivity: {}",
            engine.buffer_level(),
        );
    }

    // ── homeostasis_stability_check_system ──

    #[test]
    fn stability_check_flags_drifted_entity() {
        let mut app = test_app();
        let host = spawn_host(&mut app, 0.0, 100.0);

        // Entity frequency = 200, host = 100, stability_band = 10.
        // drift = 100 > 10 → should be flagged.
        let entity = app
            .world_mut()
            .spawn((
                ContainedIn { host, contact: ContactType::Immersed },
                Homeostasis::new(5.0, 1.0, 10.0, true),
                OscillatorySignature::new(200.0, 0.0),
            ))
            .id();

        app.update();

        let flag = app.world().get::<ThermalStressFlag>(entity);
        assert!(flag.is_some(), "entity should be flagged for thermal stress");
        assert!((flag.unwrap().drift_hz - 100.0).abs() < 1e-5);
    }

    #[test]
    fn stability_check_no_flag_when_within_band() {
        let mut app = test_app();
        let host = spawn_host(&mut app, 0.0, 100.0);

        // Entity frequency = 105, host = 100, stability_band = 10.
        // drift = 5 <= 10 → should NOT be flagged.
        let entity = app
            .world_mut()
            .spawn((
                ContainedIn { host, contact: ContactType::Immersed },
                Homeostasis::new(5.0, 1.0, 10.0, true),
                OscillatorySignature::new(105.0, 0.0),
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<ThermalStressFlag>(entity).is_none(),
            "entity within stability band should not be flagged",
        );
    }

    #[test]
    fn stability_check_removes_flag_when_back_in_band() {
        let mut app = test_app();
        let host = spawn_host(&mut app, 0.0, 100.0);

        let entity = app
            .world_mut()
            .spawn((
                ContainedIn { host, contact: ContactType::Immersed },
                Homeostasis::new(5.0, 1.0, 10.0, true),
                OscillatorySignature::new(200.0, 0.0),
                ThermalStressFlag { drift_hz: 100.0 },
            ))
            .id();

        // Manually set frequency within band.
        app.world_mut()
            .get_mut::<OscillatorySignature>(entity)
            .unwrap()
            .set_frequency_hz(105.0);

        app.update();

        assert!(
            app.world().get::<ThermalStressFlag>(entity).is_none(),
            "flag should be removed when drift is within band",
        );
    }

    #[test]
    fn stability_check_skips_disabled_homeostasis() {
        let mut app = test_app();
        let host = spawn_host(&mut app, 0.0, 100.0);

        let entity = app
            .world_mut()
            .spawn((
                ContainedIn { host, contact: ContactType::Immersed },
                Homeostasis::new(5.0, 1.0, 10.0, false),
                OscillatorySignature::new(200.0, 0.0),
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<ThermalStressFlag>(entity).is_none(),
            "disabled homeostasis should not flag",
        );
    }
}
