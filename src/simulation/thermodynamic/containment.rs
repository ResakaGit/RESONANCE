use bevy::prelude::*;

use crate::blueprint::constants::{
    IMMERSION_DEPTH_THRESHOLD_RATIO, RADIATED_HOST_RANGE_MULTIPLIER,
};
use crate::blueprint::equations;
use crate::events::DeathCause;
use crate::layers::{
    AmbientPressure, BaseEnergy, ContactType, ContainedIn, EnergyOps, FlowVector, MatterCoherence,
    ResonanceThermalOverlay, SpatialVolume,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::simulation::time_compat::simulation_delta_secs;

/// Resultado de overlap calculado por `containment_overlap_system`. Efímero (SparseSet).
#[derive(Component, Debug, Clone, Copy, Default)]
#[component(storage = "SparseSet")]
pub struct ContainmentContact {
    pub overlap_area: f32,
    pub drag_factor: f32,
}

/// Clasifica el canal de transferencia para una entidad con radio `entity_radius`
/// respecto de un host de radio `host_radius`, según distancia entre centros.
pub fn infer_contact_type(dist: f32, host_radius: f32, entity_radius: f32) -> Option<ContactType> {
    let d = dist.max(0.0);
    let h = host_radius.max(0.0);
    let e = entity_radius.max(0.0);

    if h <= 0.0 || e <= 0.0 {
        return None;
    }

    let sum = h + e;

    // Intersección (incluye tangencia) => Surface (conducción).
    if d <= sum {
        // Está completamente dentro: distinguimos “superficie cercana” vs “imerso profundo”.
        if d + e <= h {
            let depth = h - (d + e); // 0 => el borde toca la frontera del host.
            if depth <= e * IMMERSION_DEPTH_THRESHOLD_RATIO {
                Some(ContactType::Surface)
            } else {
                Some(ContactType::Immersed)
            }
        } else {
            Some(ContactType::Surface)
        }
    } else if d < h * RADIATED_HOST_RANGE_MULTIPLIER {
        // No interseca, pero está en rango => Radiated.
        Some(ContactType::Radiated)
    } else {
        None
    }
}

/// Sistema: infiere cada frame “qué host me contiene” + “contact channel”.
///
/// Regla V4: Containment no se setea por gameplay; emerge de geometría y
/// contexto espacial.
pub fn containment_system(
    mut commands: Commands,
    layout: Res<SimWorldTransformParams>,
    hosts: Query<(Entity, &Transform, &SpatialVolume, &AmbientPressure)>,
    targets: Query<(
        Entity,
        &Transform,
        &SpatialVolume,
        &BaseEnergy,
        Option<&ContainedIn>,
    )>,
) {
    let xz = layout.use_xz_ground;
    for (entity, e_transform, e_vol, _energy, contained_opt) in &targets {
        let e_pos = sim_plane_pos(e_transform.translation, xz);
        let mut best: Option<(Entity, ContactType, f32)> = None; // (host, contact, priority)

        for (host, h_transform, h_vol, _host_pressure) in &hosts {
            if host == entity {
                continue;
            }

            let dist = e_pos.distance(sim_plane_pos(h_transform.translation, xz));
            let Some(contact) = infer_contact_type(dist, h_vol.radius, e_vol.radius) else {
                continue;
            };

            // Priorizamos el host más cercano (menor dist).
            let priority = -dist;
            let should_replace = best.as_ref().map_or(true, |(best_host, _, best_p)| {
                if priority > *best_p {
                    true
                } else if (priority - *best_p).abs() < f32::EPSILON {
                    host.to_bits() < best_host.to_bits()
                } else {
                    false
                }
            });

            if should_replace {
                best = Some((host, contact, priority));
            }
        }

        match best {
            Some((host, contact, _)) => {
                let same = contained_opt.is_some_and(|ci| ci.host == host && ci.contact == contact);
                if !same {
                    commands
                        .entity(entity)
                        .insert(ContainedIn { host, contact });
                }
            }
            None => {
                if contained_opt.is_some() {
                    commands.entity(entity).remove::<ContainedIn>();
                }
            }
        }
    }
}

/// Sistema: calcula overlap geométrico con todos los hosts activos; escribe `ContainmentContact`.
/// Fase: Phase::ThermodynamicLayer — debe correr antes de `containment_thermal_system`.
pub fn containment_overlap_system(
    mut commands: Commands,
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    hosts: Query<(Entity, &Transform, &SpatialVolume, &AmbientPressure)>,
    targets: Query<(
        Entity,
        &Transform,
        &SpatialVolume,
        Option<&ContainmentContact>,
    )>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    let xz = layout.use_xz_ground;

    for (entity, e_transform, e_vol, contact_opt) in &targets {
        let e_pos = sim_plane_pos(e_transform.translation, xz);
        let mut total_overlap = 0.0_f32;
        let mut drag_factor = 1.0_f32;
        let mut any_host = false;

        for (host_entity, h_transform, h_vol, h_pressure) in &hosts {
            if host_entity == entity {
                continue;
            }
            let h_pos = sim_plane_pos(h_transform.translation, xz);
            let distance = e_pos.distance(h_pos);
            let Some(contact) = infer_contact_type(distance, h_vol.radius, e_vol.radius) else {
                continue;
            };
            any_host = true;
            total_overlap +=
                equations::circle_intersection_area(distance, h_vol.radius, e_vol.radius);
            if contact == ContactType::Surface || contact == ContactType::Immersed {
                let viscosity = h_pressure.terrain_viscosity;
                let host_drag = 1.0 / (1.0 + (viscosity - 1.0) * dt);
                drag_factor *= host_drag;
            }
        }

        if any_host {
            let new_contact = ContainmentContact {
                overlap_area: total_overlap,
                drag_factor,
            };
            let needs_insert = contact_opt.map_or(true, |c| {
                (c.overlap_area - new_contact.overlap_area).abs() > f32::EPSILON
                    || (c.drag_factor - new_contact.drag_factor).abs() > f32::EPSILON
            });
            if needs_insert {
                commands.entity(entity).insert(new_contact);
            }
        } else if contact_opt.is_some() {
            commands.entity(entity).remove::<ContainmentContact>();
        }
    }
}

/// Sistema: transfiere energía térmica por los hosts activos usando `ContainmentContact`.
/// Fase: Phase::ThermodynamicLayer — después de `containment_overlap_system`.
pub fn containment_thermal_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    mut energy_ops: EnergyOps,
    hosts: Query<(
        Entity,
        &Transform,
        &SpatialVolume,
        &AmbientPressure,
        Option<&MatterCoherence>,
        Option<&ResonanceThermalOverlay>,
    )>,
    targets: Query<(
        Entity,
        &ContainedIn,
        &Transform,
        &SpatialVolume,
        Option<&MatterCoherence>,
        Option<&ResonanceThermalOverlay>,
    )>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }
    let fallback_entity_coh = MatterCoherence::default();
    let xz = layout.use_xz_ground;

    for (entity, _contained, e_transform, e_vol, matter_opt, target_overlay_opt) in &targets {
        let e_pos = sim_plane_pos(e_transform.translation, xz);
        let mut entity_coh = matter_opt.cloned().unwrap_or(fallback_entity_coh.clone());
        if let Some(overlay) = target_overlay_opt {
            entity_coh.set_thermal_conductivity(
                entity_coh.thermal_conductivity() * overlay.conductivity_multiplier.max(0.0),
            );
        }
        let mut total_transfer = 0.0_f32;

        for (host_entity, h_transform, h_vol, h_pressure, host_matter_opt, host_overlay_opt) in
            &hosts
        {
            if host_entity == entity {
                continue;
            }
            let h_pos = sim_plane_pos(h_transform.translation, xz);
            let distance = e_pos.distance(h_pos);
            let Some(contact) = infer_contact_type(distance, h_vol.radius, e_vol.radius) else {
                continue;
            };
            let overlap_area =
                equations::circle_intersection_area(distance, h_vol.radius, e_vol.radius);
            let mut host_coh = host_matter_opt.cloned();
            if let Some(overlay) = host_overlay_opt
                && let Some(ref mut coh) = host_coh
            {
                coh.set_thermal_conductivity(
                    coh.thermal_conductivity() * overlay.conductivity_multiplier.max(0.0),
                );
            }
            total_transfer += equations::thermal_transfer(
                contact,
                h_pressure,
                host_coh.as_ref(),
                &entity_coh,
                distance,
                overlap_area,
                dt,
            );
        }

        if total_transfer > 0.0 {
            energy_ops.inject(entity, total_transfer);
        } else if total_transfer < 0.0 {
            energy_ops.drain(entity, -total_transfer, DeathCause::Dissipation);
        }
    }
}

/// Sistema: aplica drag de viscosidad por containment usando `ContainmentContact` cacheado.
/// Fase: Phase::ThermodynamicLayer — después de `containment_thermal_system`.
pub fn containment_drag_system(mut targets: Query<(&ContainmentContact, &mut FlowVector)>) {
    for (contact, mut flow) in &mut targets {
        let drag_factor = contact.drag_factor;
        if (drag_factor - 1.0).abs() > f32::EPSILON {
            let v = flow.velocity();
            let new_v = v * drag_factor;
            if v != new_v {
                flow.set_velocity(new_v, None);
            }
        }
    }
}

/// Sistema: aplica transferencia termodinámica por canal según `ContainedIn`.
#[allow(dead_code)]
pub fn contained_thermal_transfer_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    mut energy_ops: EnergyOps,
    hosts: Query<(
        Entity,
        &Transform,
        &SpatialVolume,
        &AmbientPressure,
        Option<&MatterCoherence>,
        Option<&ResonanceThermalOverlay>,
    )>,
    mut targets: Query<(
        Entity,
        &ContainedIn,
        &Transform,
        &SpatialVolume,
        Option<&mut FlowVector>,
        Option<&MatterCoherence>,
        Option<&ResonanceThermalOverlay>,
    )>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }

    let fallback_entity_coh = MatterCoherence::default();

    let xz = layout.use_xz_ground;
    for (entity, _contained, e_transform, e_vol, flow_opt, matter_opt, target_overlay_opt) in
        &mut targets
    {
        let e_pos = sim_plane_pos(e_transform.translation, xz);
        let mut entity_coh = matter_opt.cloned().unwrap_or(fallback_entity_coh.clone());
        if let Some(overlay) = target_overlay_opt {
            entity_coh.set_thermal_conductivity(
                entity_coh.thermal_conductivity() * overlay.conductivity_multiplier.max(0.0),
            );
        }
        let mut total_transfer = 0.0;
        let mut drag_factor = 1.0;

        // Sprint 04: una entidad puede ser contenido de múltiples hosts simultáneamente.
        // Acumulamos todos los medios activos (host en host) y luego aplicamos el neto.
        for (host_entity, h_transform, h_vol, h_pressure, host_matter_opt, host_overlay_opt) in
            &hosts
        {
            if host_entity == entity {
                continue;
            }
            let h_pos = sim_plane_pos(h_transform.translation, xz);
            let distance = e_pos.distance(h_pos);
            let Some(contact) = infer_contact_type(distance, h_vol.radius, e_vol.radius) else {
                continue;
            };

            let overlap_area =
                equations::circle_intersection_area(distance, h_vol.radius, e_vol.radius);
            let mut host_coh = host_matter_opt.cloned();
            if let Some(overlay) = host_overlay_opt
                && let Some(ref mut coh) = host_coh
            {
                coh.set_thermal_conductivity(
                    coh.thermal_conductivity() * overlay.conductivity_multiplier.max(0.0),
                );
            }
            total_transfer += equations::thermal_transfer(
                contact,
                h_pressure,
                host_coh.as_ref(),
                &entity_coh,
                distance,
                overlap_area,
                dt,
            );

            if contact == ContactType::Surface || contact == ContactType::Immersed {
                let viscosity = h_pressure.terrain_viscosity;
                let host_drag = 1.0 / (1.0 + (viscosity - 1.0) * dt);
                drag_factor *= host_drag;
            }
        }

        if total_transfer > 0.0 {
            energy_ops.inject(entity, total_transfer);
        } else if total_transfer < 0.0 {
            energy_ops.drain(entity, -total_transfer, DeathCause::Dissipation);
        }

        // Mantenemos `ContainedIn` como host dominante para metadata/depuración (Sprint 02-03),
        // pero el arrastre y transferencia térmica se acumulan por todos los medios activos.
        if let Some(mut flow) = flow_opt {
            let v = flow.velocity();
            flow.set_velocity(v * drag_factor, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_contact_radiated() {
        // host=10, entity=2 => sum=12, range=20
        assert_eq!(
            infer_contact_type(15.0, 10.0, 2.0),
            Some(ContactType::Radiated)
        );
    }

    #[test]
    fn test_infer_contact_surface_overlap() {
        // d < h+e => intersección => Surface
        assert_eq!(
            infer_contact_type(11.0, 10.0, 2.0),
            Some(ContactType::Surface)
        );
    }

    #[test]
    fn test_infer_contact_immersed_deep_inside() {
        // inside: d + e <= h
        // depth = h - (d+e) = 10 - (6+2) = 2 > e*0.5 = 1 => Immersed
        assert_eq!(
            infer_contact_type(6.0, 10.0, 2.0),
            Some(ContactType::Immersed)
        );
    }
}
