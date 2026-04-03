use bevy::prelude::*;

use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;

use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::ElementId;
use crate::blueprint::constants::{DETECTION_THRESHOLD, MAX_VISION_RADIUS, OVERLOAD_FACTOR};
use crate::blueprint::equations;
use crate::blueprint::equations::terrain_blocks_vision;
use crate::events::DeathCause;
use crate::layers::{
    AlchemicalEngine, AlchemicalInjector, BaseEnergy, EnergyOps, MobaIdentity, ModifiedField,
    OscillatorySignature, ProjectedQeFromEnergy, ResonanceFlowOverlay, ResonanceLink,
    ResonanceMotorOverlay, ResonanceThermalOverlay, SpatialVolume,
};
use crate::simulation::time_compat::simulation_delta_secs;
use crate::topology::TerrainField;
use crate::world::{PerceptionCache, SpatialIndex};

/// Sistema: Tick del Motor Alquímico (Capa 5).
/// Fase: Phase::ThermodynamicLayer
pub fn engine_processing_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    mut energy_ops: EnergyOps,
    mut query: Query<(
        Entity,
        &mut AlchemicalEngine,
        Option<&SpatialVolume>,
        Option<&ResonanceMotorOverlay>,
    )>,
) {
    let dt = simulation_delta_secs(fixed, &time);

    for (entity, mut engine, volume_opt, overlay_opt) in &mut query {
        let qe_available = energy_ops.qe(entity).unwrap_or(0.0);
        if qe_available <= 0.0 {
            continue;
        }

        let intake_mult = overlay_opt
            .map(|overlay| overlay.motor_intake_multiplier)
            .unwrap_or(1.0)
            .max(0.0);
        let intake_valve = engine.valve_in_rate() * intake_mult;
        let intake = if let Some(volume) = volume_opt {
            equations::engine_intake_allometric(
                intake_valve,
                dt,
                qe_available,
                engine.buffer_level(),
                engine.buffer_cap(),
                volume.radius,
            )
        } else {
            equations::engine_intake(
                intake_valve,
                dt,
                qe_available,
                engine.buffer_level(),
                engine.buffer_cap(),
            )
        };
        if intake <= 0.0 {
            continue;
        }

        let drained = energy_ops.drain(entity, intake, DeathCause::Overload);
        if drained > 0.0 {
            engine.absorb(drained);
        }

        if engine.buffer_level() > engine.buffer_cap() * OVERLOAD_FACTOR {
            let remaining = energy_ops.qe(entity).unwrap_or(0.0);
            if remaining > 0.0 {
                energy_ops.drain(entity, remaining, DeathCause::Overload);
            }
        }
    }
}

// --- ResonanceLink → overlays (un patrón por familia de componente; Bevy exige Query separadas) ---

#[derive(Clone, Copy)]
enum FlowOverlayField {
    VelocityMultiplier,
    DissipationMultiplier,
}

fn apply_resonance_flow_overlay(
    commands: &mut Commands,
    q: &mut Query<&mut ResonanceFlowOverlay>,
    target: Entity,
    magnitude: f32,
    field: FlowOverlayField,
) {
    if let Ok(mut o) = q.get_mut(target) {
        match field {
            FlowOverlayField::VelocityMultiplier => {
                let next = o.velocity_multiplier * magnitude;
                if next != o.velocity_multiplier {
                    o.velocity_multiplier = next;
                }
            }
            FlowOverlayField::DissipationMultiplier => {
                let next = o.dissipation_multiplier * magnitude;
                if next != o.dissipation_multiplier {
                    o.dissipation_multiplier = next;
                }
            }
        }
    } else {
        let mut o = ResonanceFlowOverlay::default();
        match field {
            FlowOverlayField::VelocityMultiplier => o.velocity_multiplier = magnitude,
            FlowOverlayField::DissipationMultiplier => o.dissipation_multiplier = magnitude,
        }
        commands.entity(target).insert(o);
    }
}

#[derive(Clone, Copy)]
enum MotorOverlayField {
    Intake,
    Output,
}

fn apply_resonance_motor_overlay(
    commands: &mut Commands,
    q: &mut Query<&mut ResonanceMotorOverlay>,
    target: Entity,
    magnitude: f32,
    field: MotorOverlayField,
) {
    if let Ok(mut o) = q.get_mut(target) {
        match field {
            MotorOverlayField::Intake => {
                let next = o.motor_intake_multiplier * magnitude;
                if next != o.motor_intake_multiplier {
                    o.motor_intake_multiplier = next;
                }
            }
            MotorOverlayField::Output => {
                let next = o.motor_output_multiplier * magnitude;
                if next != o.motor_output_multiplier {
                    o.motor_output_multiplier = next;
                }
            }
        }
    } else {
        let mut o = ResonanceMotorOverlay::default();
        match field {
            MotorOverlayField::Intake => o.motor_intake_multiplier = magnitude,
            MotorOverlayField::Output => o.motor_output_multiplier = magnitude,
        }
        commands.entity(target).insert(o);
    }
}

#[derive(Clone, Copy)]
enum ThermalOverlayField {
    BondEnergy,
    Conductivity,
}

fn apply_resonance_thermal_overlay(
    commands: &mut Commands,
    q: &mut Query<&mut ResonanceThermalOverlay>,
    target: Entity,
    magnitude: f32,
    field: ThermalOverlayField,
) {
    if let Ok(mut o) = q.get_mut(target) {
        match field {
            ThermalOverlayField::BondEnergy => {
                let next = o.bond_energy_multiplier * magnitude;
                if next != o.bond_energy_multiplier {
                    o.bond_energy_multiplier = next;
                }
            }
            ThermalOverlayField::Conductivity => {
                let next = o.conductivity_multiplier * magnitude;
                if next != o.conductivity_multiplier {
                    o.conductivity_multiplier = next;
                }
            }
        }
    } else {
        let mut o = ResonanceThermalOverlay::default();
        match field {
            ThermalOverlayField::BondEnergy => o.bond_energy_multiplier = magnitude,
            ThermalOverlayField::Conductivity => o.conductivity_multiplier = magnitude,
        }
        commands.entity(target).insert(o);
    }
}

/// Resetea overlays efímeros de Capa 10 (tres componentes ortogonales).
pub fn reset_resonance_overlay_system(
    mut flow: Query<&mut ResonanceFlowOverlay>,
    mut motor: Query<&mut ResonanceMotorOverlay>,
    mut thermal: Query<&mut ResonanceThermalOverlay>,
) {
    let flow_default = ResonanceFlowOverlay::default();
    for mut overlay in &mut flow {
        overlay.set_if_neq(flow_default);
    }
    let motor_default = ResonanceMotorOverlay::default();
    for mut overlay in &mut motor {
        overlay.set_if_neq(motor_default);
    }
    let thermal_default = ResonanceThermalOverlay::default();
    for mut overlay in &mut thermal {
        overlay.set_if_neq(thermal_default);
    }
}

/// Compone enlaces de resonancia (Capa 10) en overlays efímeros.
pub fn resonance_link_system(
    mut commands: Commands,
    links: Query<(Entity, &ResonanceLink, &BaseEnergy)>,
    mut flow_q: Query<&mut ResonanceFlowOverlay>,
    mut motor_q: Query<&mut ResonanceMotorOverlay>,
    mut thermal_q: Query<&mut ResonanceThermalOverlay>,
) {
    for (_effect_entity, link, effect_energy) in &links {
        if effect_energy.qe() <= 0.0 {
            continue;
        }

        let target = link.target;

        match link.modified_field {
            ModifiedField::VelocityMultiplier => apply_resonance_flow_overlay(
                &mut commands,
                &mut flow_q,
                target,
                link.magnitude,
                FlowOverlayField::VelocityMultiplier,
            ),
            ModifiedField::DissipationMultiplier => apply_resonance_flow_overlay(
                &mut commands,
                &mut flow_q,
                target,
                link.magnitude,
                FlowOverlayField::DissipationMultiplier,
            ),
            ModifiedField::MotorIntakeMultiplier => apply_resonance_motor_overlay(
                &mut commands,
                &mut motor_q,
                target,
                link.magnitude,
                MotorOverlayField::Intake,
            ),
            ModifiedField::MotorOutputMultiplier => apply_resonance_motor_overlay(
                &mut commands,
                &mut motor_q,
                target,
                link.magnitude,
                MotorOverlayField::Output,
            ),
            ModifiedField::BondEnergyMultiplier => apply_resonance_thermal_overlay(
                &mut commands,
                &mut thermal_q,
                target,
                link.magnitude,
                ThermalOverlayField::BondEnergy,
            ),
            ModifiedField::ConductivityMultiplier => apply_resonance_thermal_overlay(
                &mut commands,
                &mut thermal_q,
                target,
                link.magnitude,
                ThermalOverlayField::Conductivity,
            ),
        }
    }
}

/// Sistema: Percepción basada en índice espacial + frecuencia elemental.
/// Fase: Phase::ThermodynamicLayer
pub fn perception_system(
    index: Res<SpatialIndex>,
    layout: Res<SimWorldTransformParams>,
    almanac: Res<AlchemicalAlmanac>,
    perceivers: Query<(&Transform, &MobaIdentity)>,
    targets: Query<(&BaseEnergy, &OscillatorySignature, &ElementId)>,
    terrain: Option<Res<TerrainField>>,
    mut cache: ResMut<PerceptionCache>,
) {
    cache.clear();

    let xz = layout.use_xz_ground;
    for (transform, identity) in &perceivers {
        let origin = sim_plane_pos(transform.translation, xz);
        let nearby = index.query_radius(origin, MAX_VISION_RADIUS);

        for entry in nearby {
            let Ok((energy, target_wave, element_id)) = targets.get(entry.entity) else {
                continue;
            };
            if let Some(terrain_field) = terrain.as_ref() {
                if terrain_blocks_vision(origin, entry.position, terrain_field.as_ref()) {
                    continue;
                }
            }

            // Intensidad de señal: qe ponderado por visibilidad + pureza.
            // Hot-path: evitamos `find_stable_band()` (O(|elements|) en EAC2) y usamos `ElementId` directo.
            let def = almanac.get(*element_id);
            let visibility = def.map(|d| d.visibility).unwrap_or(0.5);
            let purity = def
                .map(|d| d.purity(target_wave.frequency_hz()))
                .unwrap_or(0.25);

            let distance_sq = origin.distance_squared(entry.position);
            let signal =
                equations::perception_signal_weighted(energy.qe(), visibility, purity, distance_sq);

            if signal >= DETECTION_THRESHOLD {
                cache.mark_visible(identity.faction(), entry.entity);
            }
        }
    }
}

/// Mantiene consistencia: `AlchemicalInjector.projected_qe` refleja
/// la energía actual del spell (`BaseEnergy.qe`).
///
/// Gateado por `ProjectedQeFromEnergy` para no romper hechizos “fijos”
/// (ej. `lava_knight`) cuyo `projected_qe` no debe seguir a `BaseEnergy`.
/// Fase: Phase::ThermodynamicLayer
pub fn sync_injector_projected_qe_system(
    mut query: Query<(&BaseEnergy, &mut AlchemicalInjector), With<ProjectedQeFromEnergy>>,
) {
    for (energy, mut injector) in &mut query {
        let qe = energy.qe();
        if injector.projected_qe != qe {
            injector.projected_qe = qe;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::{ElementDef, constants::LINK_NEUTRAL_MULTIPLIER};
    use crate::layers::{Faction, MatterState, SpatialVolume};
    use crate::topology::TerrainField;
    use crate::world::SpatialEntry;
    use std::time::Duration;

    fn test_almanac() -> AlchemicalAlmanac {
        AlchemicalAlmanac::from_defs(vec![ElementDef {
            name: "Terra".to_string(),
            symbol: "Terra".to_string(),
            atomic_number: 14,
            frequency_hz: 75.0,
            freq_band: (50.0, 84.0),
            bond_energy: 3000.0,
            conductivity: 0.4,
            visibility: 1.0,
            matter_state: MatterState::Solid,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (0.45, 0.34, 0.20),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        }])
    }

    fn neutral_identity() -> MobaIdentity {
        MobaIdentity {
            faction: Faction::Neutral,
            relational_tags: 0,
            critical_multiplier: LINK_NEUTRAL_MULTIPLIER,
        }
    }

    #[test]
    fn perception_system_blocks_target_when_terrain_occludes() {
        let mut app = App::new();
        app.insert_resource(SimWorldTransformParams::default());
        app.insert_resource(test_almanac());
        app.insert_resource(PerceptionCache::default());
        app.insert_resource(SpatialIndex::new(5.0));
        app.insert_resource({
            let mut terrain = TerrainField::new(3, 1, 1.0, Vec2::ZERO, 1);
            terrain.altitude = vec![0.0, 2.0, 0.0];
            terrain
        });
        app.world_mut().spawn((
            Transform::from_xyz(0.5, 0.5, 0.0),
            neutral_identity(),
            SpatialVolume::new(0.5),
        ));
        let target = app
            .world_mut()
            .spawn((
                BaseEnergy::new(500.0),
                OscillatorySignature::new(75.0, 0.0),
                ElementId::from_name("Terra"),
                Transform::from_xyz(2.5, 0.5, 0.0),
            ))
            .id();
        app.world_mut()
            .resource_mut::<SpatialIndex>()
            .insert(SpatialEntry {
                entity: target,
                position: Vec2::new(2.5, 0.5),
                radius: 0.5,
            });

        app.add_systems(Update, perception_system);
        app.update();

        let cache = app.world().resource::<PerceptionCache>();
        assert!(!cache.is_visible_to(Faction::Neutral, target));
    }

    #[test]
    fn perception_system_without_terrain_keeps_legacy_visibility() {
        let mut app = App::new();
        app.insert_resource(SimWorldTransformParams::default());
        app.insert_resource(test_almanac());
        app.insert_resource(PerceptionCache::default());
        app.insert_resource(SpatialIndex::new(5.0));
        app.world_mut().spawn((
            Transform::from_xyz(0.5, 0.5, 0.0),
            neutral_identity(),
            SpatialVolume::new(0.5),
        ));
        let target = app
            .world_mut()
            .spawn((
                BaseEnergy::new(500.0),
                OscillatorySignature::new(75.0, 0.0),
                ElementId::from_name("Terra"),
                Transform::from_xyz(2.5, 0.5, 0.0),
            ))
            .id();
        app.world_mut()
            .resource_mut::<SpatialIndex>()
            .insert(SpatialEntry {
                entity: target,
                position: Vec2::new(2.5, 0.5),
                radius: 0.5,
            });

        app.add_systems(Update, perception_system);
        app.update();

        let cache = app.world().resource::<PerceptionCache>();
        assert!(cache.is_visible_to(Faction::Neutral, target));
    }

    #[test]
    fn reset_resonance_overlay_skips_default_multiples() {
        let mut app = App::new();
        let e = app.world_mut().spawn(ResonanceFlowOverlay::default()).id();
        app.add_systems(Update, reset_resonance_overlay_system);
        app.update();
        let o = app
            .world()
            .entity(e)
            .get::<ResonanceFlowOverlay>()
            .expect("overlay");
        assert_eq!(*o, ResonanceFlowOverlay::default());
    }

    #[test]
    fn engine_processing_uses_allometric_with_volume_and_legacy_without_volume() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<crate::events::DeathEvent>();
        app.add_systems(Update, engine_processing_system);

        let e_allometric = app
            .world_mut()
            .spawn((
                BaseEnergy::new(500.0),
                AlchemicalEngine::new(1_000.0, 10.0, 50.0, 0.0),
                SpatialVolume::new(2.0),
            ))
            .id();
        let e_legacy = app
            .world_mut()
            .spawn((
                BaseEnergy::new(500.0),
                AlchemicalEngine::new(1_000.0, 10.0, 50.0, 0.0),
            ))
            .id();

        app.update();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0));
        app.update();

        let eng_allometric = app
            .world()
            .entity(e_allometric)
            .get::<AlchemicalEngine>()
            .expect("engine allometric");
        let eng_legacy = app
            .world()
            .entity(e_legacy)
            .get::<AlchemicalEngine>()
            .expect("engine legacy");

        assert!(eng_allometric.buffer_level() > 0.0);
        assert!(eng_legacy.buffer_level() > 0.0);
        assert!(eng_allometric.buffer_level() <= 500.0);
        assert!(eng_legacy.buffer_level() <= 500.0);
        assert!(eng_allometric.buffer_level() <= 1000.0);
        assert!(eng_legacy.buffer_level() <= 1000.0);
        assert!(eng_allometric.buffer_level() > eng_legacy.buffer_level());
    }
}
