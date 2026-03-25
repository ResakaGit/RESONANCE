use bevy::prelude::*;

use crate::blueprint::{AlchemicalAlmanac, constants, equations};
use crate::layers::{
    BaseEnergy, IrradianceReceiver, NutrientProfile, OscillatorySignature, SpatialVolume,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::simulation::time_compat::simulation_delta_secs;
use crate::worldgen::{EnergyNucleus, Materialized, NUTRIENT_WRITE_EPS};

type IrradianceEntityQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static OscillatorySignature,
        Option<&'static IrradianceReceiver>,
    ),
    (With<Materialized>, With<BaseEnergy>),
>;

#[derive(Resource, Debug, Default)]
pub struct IrradianceUpdateCursor {
    offset: usize,
}

#[inline]
fn is_lux_frequency(freq_hz: f32) -> bool {
    (constants::LUX_BAND_MIN_HZ..=constants::LUX_BAND_MAX_HZ).contains(&freq_hz)
}

#[inline]
fn receiver_differs(a: IrradianceReceiver, b: IrradianceReceiver) -> bool {
    (a.photon_density - b.photon_density).abs() > constants::IRRADIANCE_MIN_EFFECTIVE
        || (a.absorbed_fraction - b.absorbed_fraction).abs() > constants::IRRADIANCE_MIN_EFFECTIVE
}

/// Capa 1 (extensión TL4A/TL4C): actualiza irradiancia recibida desde núcleos Lux.
/// Runs in `Phase::ThermodynamicLayer`.
pub fn irradiance_update_system(
    mut commands: Commands,
    layout: Res<SimWorldTransformParams>,
    almanac: Res<AlchemicalAlmanac>,
    mut cursor: ResMut<IrradianceUpdateCursor>,
    nuclei: Query<(Entity, &EnergyNucleus, &Transform)>,
    entities: IrradianceEntityQuery,
) {
    let mut lux_sources: Vec<(u64, Vec2, f32, f32)> = Vec::new();
    for (entity, nucleus, transform) in &nuclei {
        if !is_lux_frequency(nucleus.frequency_hz()) {
            continue;
        }
        lux_sources.push((
            entity.to_bits(),
            sim_plane_pos(transform.translation, layout.use_xz_ground),
            nucleus.emission_rate_qe_s(),
            nucleus.propagation_radius(),
        ));
    }
    lux_sources.sort_by_key(|(id, _, _, _)| *id);

    let mut ordered: Vec<(Entity, Vec2, f32, Option<IrradianceReceiver>)> = entities
        .iter()
        .map(|(entity, transform, signature, receiver)| {
            (
                entity,
                sim_plane_pos(transform.translation, layout.use_xz_ground),
                signature.frequency_hz(),
                receiver.copied(),
            )
        })
        .collect();
    ordered.sort_by_key(|(entity, _, _, _)| entity.to_bits());

    if ordered.is_empty() {
        cursor.offset = 0;
        return;
    }
    if lux_sources.is_empty() {
        for (entity, _, _, existing) in &ordered {
            if existing.is_some() {
                commands.entity(*entity).remove::<IrradianceReceiver>();
            }
        }
        cursor.offset = 0;
        return;
    }

    let mut processed: u32 = 0;
    let len = ordered.len();
    for i in 0..len {
        if processed >= constants::MAX_IRRADIANCE_PER_FRAME {
            break;
        }
        let idx = (cursor.offset + i) % len;
        let (entity, position, freq_hz, existing) = ordered[idx];

        let mut photon_density = 0.0_f32;
        for (_, source_pos, source_emission, source_radius) in &lux_sources {
            let delta = position - *source_pos;
            let distance_sq = delta.length_squared();
            if distance_sq > *source_radius * *source_radius {
                continue;
            }
            let contribution = equations::irradiance_at_distance_sq(
                *source_emission,
                distance_sq,
                constants::IRRADIANCE_LUX_DECAY,
            );
            if contribution.is_finite() {
                photon_density =
                    (photon_density + contribution).min(constants::PHOTO_MAX_PHOTON_DENSITY);
            }
        }

        let absorbed_fraction = almanac
            .find_stable_band(freq_hz)
            .map(|element| element.visibility)
            .unwrap_or_else(|| equations::frequency_visibility(freq_hz));

        if photon_density > constants::IRRADIANCE_MIN_EFFECTIVE {
            let next = IrradianceReceiver::new(photon_density, absorbed_fraction);
            match existing {
                Some(current) if !receiver_differs(current, next) => {}
                _ => {
                    commands.entity(entity).insert(next);
                }
            }
        } else if existing.is_some() {
            commands.entity(entity).remove::<IrradianceReceiver>();
        }

        processed += 1;
    }

    cursor.offset = (cursor.offset + processed as usize) % len;
}

/// Capa 4 (TL4C): convierte irradiancia en `qe` y consume agua del perfil de nutrientes.
/// Runs in `Phase::ChemicalLayer`, after `nutrient_uptake_system`.
pub fn photosynthetic_contribution_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    mut entities: Query<(
        &mut BaseEnergy,
        &mut NutrientProfile,
        &IrradianceReceiver,
        &SpatialVolume,
    )>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }

    for (mut energy, mut nutrient, irradiance, volume) in &mut entities {
        let photon_effective =
            irradiance.photon_density.max(0.0) * irradiance.absorbed_fraction.max(0.0);
        if photon_effective <= constants::IRRADIANCE_MIN_EFFECTIVE {
            continue;
        }

        let density = volume.density(energy.qe());
        let temp_norm = (equations::equivalent_temperature(density)
            / constants::PHOTO_TEMP_NORM_REFERENCE)
            .clamp(0.0, 1.0);
        let yield_per_sec = equations::photosynthetic_yield(
            photon_effective,
            nutrient.water_norm,
            nutrient.carbon_norm,
            temp_norm,
        ) * constants::PHOTO_YIELD_SCALE;
        let qe_gain = (yield_per_sec * dt).max(0.0);
        if qe_gain <= 0.0 {
            continue;
        }

        energy.inject(qe_gain);

        let water_after = (nutrient.water_norm
            - qe_gain * constants::PHOTO_WATER_CONSUMPTION_PER_QE)
            .clamp(0.0, 1.0);
        if (water_after - nutrient.water_norm).abs() > NUTRIENT_WRITE_EPS {
            nutrient.water_norm = water_after;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IrradianceUpdateCursor, irradiance_update_system, photosynthetic_contribution_system,
    };
    use crate::blueprint::almanac::ElementDef;
    use crate::blueprint::constants;
    use crate::blueprint::{AlchemicalAlmanac, ElementId};
    use crate::layers::{
        BaseEnergy, IrradianceReceiver, NutrientProfile, OscillatorySignature, SpatialVolume,
    };
    use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
    use crate::worldgen::{EnergyNucleus, Materialized, PropagationDecay, WorldArchetype};
    use bevy::prelude::*;
    use std::time::Duration;

    #[test]
    fn irradiance_update_adds_receiver_when_entity_is_near_lux_nucleus() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimWorldTransformParams::default());
        app.init_resource::<IrradianceUpdateCursor>();
        app.insert_resource(AlchemicalAlmanac::from_defs(vec![ElementDef {
            name: "Lux".to_string(),
            symbol: "Lux".to_string(),
            atomic_number: 1,
            frequency_hz: 1000.0,
            freq_band: (900.0, 1100.0),
            bond_energy: 1000.0,
            conductivity: 0.5,
            visibility: 1.0,
            matter_state: crate::layers::MatterState::Gas,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (1.0, 1.0, 1.0),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        }]));
        app.add_systems(Update, irradiance_update_system);

        app.world_mut().spawn((
            EnergyNucleus::new(1000.0, 100.0, 10.0, PropagationDecay::Flat),
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));
        let entity = app
            .world_mut()
            .spawn((
                BaseEnergy::new(20.0),
                OscillatorySignature::new(1000.0, 0.0),
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                Transform::from_xyz(1.0, 0.0, 0.0),
                GlobalTransform::default(),
            ))
            .id();

        app.update();

        let receiver = app
            .world()
            .entity(entity)
            .get::<IrradianceReceiver>()
            .copied()
            .expect("must receive irradiance");
        assert!(receiver.photon_density > 0.0);
        assert!(receiver.absorbed_fraction > 0.0);
    }

    #[test]
    fn photosynthetic_contribution_increases_energy_and_consumes_water() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Time::<Fixed>::default());
        app.add_systems(Update, photosynthetic_contribution_system);

        let entity = app
            .world_mut()
            .spawn((
                BaseEnergy::new(10.0),
                SpatialVolume::new(1.0),
                NutrientProfile::new(1.0, 1.0, 1.0, 1.0),
                IrradianceReceiver::new(100.0, 1.0),
                OscillatorySignature::new(1000.0, 0.0),
                ElementId::from_name("Lux"),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0));
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f32(1.0));
        app.update();
        app.update();

        let world = app.world();
        let energy = world
            .entity(entity)
            .get::<BaseEnergy>()
            .expect("energy")
            .qe();
        let nutrient = world
            .entity(entity)
            .get::<NutrientProfile>()
            .copied()
            .expect("nutrient");
        assert!(energy > 10.0, "energy={energy}");
        assert!(nutrient.water_norm < 1.0, "water={}", nutrient.water_norm);
    }

    #[test]
    fn irradiance_update_removes_receiver_without_lux_sources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimWorldTransformParams::default());
        app.init_resource::<IrradianceUpdateCursor>();
        app.insert_resource(AlchemicalAlmanac::default());
        app.add_systems(Update, irradiance_update_system);

        let entity = app
            .world_mut()
            .spawn((
                BaseEnergy::new(10.0),
                OscillatorySignature::new(1000.0, 0.0),
                IrradianceReceiver::new(1.0, 1.0),
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                GlobalTransform::default(),
            ))
            .id();

        app.update();
        assert!(
            app.world()
                .entity(entity)
                .get::<IrradianceReceiver>()
                .is_none()
        );
    }

    #[test]
    fn irradiance_update_respects_frame_budget() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimWorldTransformParams::default());
        app.init_resource::<IrradianceUpdateCursor>();
        app.insert_resource(AlchemicalAlmanac::default());
        app.add_systems(Update, irradiance_update_system);

        app.world_mut().spawn((
            EnergyNucleus::new(1000.0, 100.0, 1000.0, PropagationDecay::Flat),
            Transform::default(),
            GlobalTransform::default(),
        ));
        for i in 0..(constants::MAX_IRRADIANCE_PER_FRAME + 10) {
            app.world_mut().spawn((
                BaseEnergy::new(10.0),
                OscillatorySignature::new(1000.0, 0.0),
                Materialized {
                    cell_x: i as i32,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
                GlobalTransform::default(),
            ));
        }

        app.update();

        let updated = app
            .world_mut()
            .query::<&IrradianceReceiver>()
            .iter(app.world())
            .count() as u32;
        assert_eq!(updated, constants::MAX_IRRADIANCE_PER_FRAME);
    }

    #[test]
    fn entity_far_from_lux_nucleus_does_not_receive_irradiance() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimWorldTransformParams::default());
        app.init_resource::<IrradianceUpdateCursor>();
        app.insert_resource(AlchemicalAlmanac::default());
        app.add_systems(Update, irradiance_update_system);

        app.world_mut().spawn((
            EnergyNucleus::new(1000.0, 100.0, 1.0, PropagationDecay::Flat),
            Transform::default(),
            GlobalTransform::default(),
        ));
        let entity = app
            .world_mut()
            .spawn((
                BaseEnergy::new(10.0),
                OscillatorySignature::new(1000.0, 0.0),
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
                GlobalTransform::default(),
            ))
            .id();

        app.update();
        assert!(
            app.world()
                .entity(entity)
                .get::<IrradianceReceiver>()
                .is_none()
        );
    }

    #[test]
    fn irradiance_ignores_non_lux_nucleus_even_in_range() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimWorldTransformParams::default());
        app.init_resource::<IrradianceUpdateCursor>();
        app.insert_resource(AlchemicalAlmanac::default());
        app.add_systems(Update, irradiance_update_system);

        app.world_mut().spawn((
            EnergyNucleus::new(450.0, 100.0, 20.0, PropagationDecay::Flat),
            Transform::default(),
            GlobalTransform::default(),
        ));
        let entity = app
            .world_mut()
            .spawn((
                BaseEnergy::new(10.0),
                OscillatorySignature::new(1000.0, 0.0),
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
                GlobalTransform::default(),
            ))
            .id();

        app.update();
        assert!(
            app.world()
                .entity(entity)
                .get::<IrradianceReceiver>()
                .is_none()
        );
    }
}
