use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::blueprint::{constants, equations};
use crate::events::{DeathCause, HomeostasisAdaptEvent, StructuralLinkBreakEvent};
use crate::layers::{
    AmbientPressure, BaseEnergy, ContainedIn, EnergyOps, FlowVector, Homeostasis,
    OscillatorySignature, StructuralLink, TensionField,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::runtime_platform::simulation_tick::SimulationElapsed;
use crate::simulation::time_compat::simulation_delta_secs;
use crate::world::SpatialIndex;

/// Capa 13: aplica restricción estructural, transferencia y ruptura de enlaces.
pub fn structural_constraint_system(
    mut commands: Commands,
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    mut energy_ops: EnergyOps,
    mut ev_break: EventWriter<StructuralLinkBreakEvent>,
    links: Query<(Entity, &StructuralLink, &Transform)>,
    targets: Query<(Entity, &Transform)>,
    mut flows: Query<&mut FlowVector>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }

    let xz = layout.use_xz_ground;
    for (source, link, source_t) in &links {
        let Ok((_target_e, target_t)) = targets.get(link.target) else {
            commands.entity(source).remove::<StructuralLink>();
            continue;
        };
        if source == link.target {
            commands.entity(source).remove::<StructuralLink>();
            continue;
        }

        let delta =
            sim_plane_pos(target_t.translation, xz) - sim_plane_pos(source_t.translation, xz);
        let distance = delta.length();
        let extension = distance - link.rest_length.max(0.0);
        let stress = equations::structural_stress(extension, constants::STRUCTURAL_DEFAULT_THERMAL_LOAD);
        if stress > link.break_stress {
            ev_break.send(StructuralLinkBreakEvent {
                source,
                target: link.target,
                stress,
            });
            commands.entity(source).remove::<StructuralLink>();
            continue;
        }

        let spring_force = equations::spring_force(delta, link.rest_length, link.stiffness);
        let source_qe = energy_ops.qe(source).unwrap_or(0.0).max(0.01);
        let target_qe = energy_ops.qe(link.target).unwrap_or(0.0).max(0.01);

        if let Ok(mut source_flow) = flows.get_mut(source) {
            source_flow.add_velocity((spring_force / source_qe) * dt, None);
        }
        if let Ok(mut target_flow) = flows.get_mut(link.target) {
            target_flow.add_velocity(-(spring_force / target_qe) * dt, None);
        }

        // Difusión conservativa simple de energía a través del enlace.
        let source_qe_now = energy_ops.qe(source).unwrap_or(0.0);
        let target_qe_now = energy_ops.qe(link.target).unwrap_or(0.0);
        let qe_delta = source_qe_now - target_qe_now;
        if qe_delta.abs() > 0.0 {
            let transfer =
                equations::structural_link_qe_transfer(qe_delta.abs(), link.stiffness, dt);
            if transfer > 0.0 {
                if qe_delta > 0.0 {
                    let drained =
                        energy_ops.drain(source, transfer, DeathCause::StructuralCollapse);
                    if drained > 0.0 {
                        energy_ops.inject(link.target, drained);
                    }
                } else {
                    let drained =
                        energy_ops.drain(link.target, transfer, DeathCause::StructuralCollapse);
                    if drained > 0.0 {
                        energy_ops.inject(source, drained);
                    }
                }
            }
        }
    }
}

/// Capa 11: campo de tensión a distancia (gravedad + acople magneto-oscilatorio).
pub fn tension_field_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    sim_elapsed: Option<Res<SimulationElapsed>>,
    index: Res<SpatialIndex>,
    emitters: Query<(
        Entity,
        &Transform,
        &BaseEnergy,
        &TensionField,
        Option<&OscillatorySignature>,
    )>,
    targets_read: Query<(
        Entity,
        &Transform,
        &BaseEnergy,
        Option<&OscillatorySignature>,
    )>,
    mut targets_flow: Query<&mut FlowVector>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }

    let phase_t = sim_elapsed.map(|s| s.secs).unwrap_or(0.0);
    let xz = layout.use_xz_ground;

    let mut accumulated: BTreeMap<Entity, Vec2> = BTreeMap::new();
    for (source, source_t, source_energy, field, source_wave_opt) in &emitters {
        if source_energy.qe() <= 0.0 || field.radius <= 0.0 {
            continue;
        }
        let nearby = index.query_radius(sim_plane_pos(source_t.translation, xz), field.radius);
        for entry in nearby {
            if entry.entity == source {
                continue;
            }
            let Ok((target, target_t, target_energy, target_wave_opt)) =
                targets_read.get(entry.entity)
            else {
                continue;
            };
            if target_energy.qe() <= 0.0 {
                continue;
            }

            let delta =
                sim_plane_pos(source_t.translation, xz) - sim_plane_pos(target_t.translation, xz);
            let interference = match (source_wave_opt, target_wave_opt) {
                (Some(sw), Some(tw)) => equations::interference(
                    sw.frequency_hz(),
                    sw.phase(),
                    tw.frequency_hz(),
                    tw.phase(),
                    phase_t,
                ),
                _ => 0.0,
            };

            let acceleration = equations::tension_field_acceleration(
                source_energy.qe(),
                target_energy.qe(),
                delta,
                field.gravity_gain,
                field.magnetic_gain,
                interference,
                field.falloff_mode,
                constants::TENSION_FIELD_SOFTENING_EPS,
            );
            if acceleration != Vec2::ZERO {
                let acc = accumulated.entry(target).or_insert(Vec2::ZERO);
                *acc += acceleration;
            }
        }
    }

    for (entity, acceleration) in accumulated {
        if let Ok(mut flow) = targets_flow.get_mut(entity) {
            flow.add_velocity(acceleration * dt, None);
        }
    }
}

/// Capa 12: adapta frecuencia hacia banda estable con costo energético.
pub fn homeostasis_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    mut energy_ops: EnergyOps,
    hosts: Query<&AmbientPressure>,
    host_waves: Query<&OscillatorySignature, (With<AmbientPressure>, Without<Homeostasis>)>,
    mut query: Query<(
        Entity,
        &ContainedIn,
        &Homeostasis,
        &mut OscillatorySignature,
    )>,
    mut ev_homeostasis: EventWriter<HomeostasisAdaptEvent>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }

    for (entity, contained, homeostasis, mut signature) in &mut query {
        if !homeostasis.enabled || energy_ops.qe(entity).unwrap_or(0.0) <= 0.0 {
            continue;
        }

        let Ok(host_pressure) = hosts.get(contained.host) else {
            continue;
        };
        // Hostilidad sostenida => presión negativa.
        if host_pressure.delta_qe_constant >= 0.0 {
            continue;
        }
        let host_wave_opt = host_waves.get(contained.host).ok();
        let Some(host_wave) = host_wave_opt else {
            continue;
        };

        let target_hz = host_wave.frequency_hz();
        let diff = target_hz - signature.frequency_hz();
        if diff.abs() <= homeostasis.stability_band_hz {
            continue;
        }

        let delta_hz = equations::homeostasis_delta_hz(
            signature.frequency_hz(),
            target_hz,
            homeostasis.adapt_rate_hz,
            dt,
        );
        if delta_hz.abs() <= 0.0 {
            continue;
        }

        let qe_cost = equations::homeostasis_qe_cost(delta_hz, homeostasis.qe_cost_per_hz);
        if energy_ops.qe(entity).unwrap_or(0.0) < qe_cost {
            continue;
        }

        let from_hz = signature.frequency_hz();
        let drained = energy_ops.drain(entity, qe_cost, DeathCause::Dissipation);
        if drained < qe_cost {
            continue;
        }
        let new_hz = signature.frequency_hz() + delta_hz;
        signature.set_frequency_hz(new_hz);

        ev_homeostasis.send(HomeostasisAdaptEvent {
            entity,
            from_hz,
            to_hz: signature.frequency_hz(),
            qe_cost,
        });
    }
}
