use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use crate::blueprint::{IdGenerator, equations};
use crate::eco::context_lookup::ContextLookup;
use crate::entities::EffectConfig;
use crate::entities::archetypes::spawn_effect;
use crate::events::{
    CatalysisEvent, CatalysisRequest, DeathCause, DeltaEnergyCommit, PhaseTransitionEvent,
};
use crate::layers::compute_interference_total;
use crate::layers::{
    AlchemicalInjector, BaseEnergy, DespawnOnContact, EnergyOps, MatterCoherence, MobaIdentity,
    OnContactEffect, OscillatorySignature, ResonanceThermalOverlay, SpatialVolume,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::runtime_platform::simulation_tick::SimulationElapsed;
use crate::simulation::Phase;
use crate::simulation::structural_runtime;
use crate::world::SpatialIndex;

const CATALYSIS_MIN_EFFECT_QE: f32 = 0.01;

/// Marcador para entidades que son hechizos activos.
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct SpellMarker {
    pub caster: Option<Entity>,
}

/// Sistema: Transiciones de fase basadas en temperatura equivalente.
/// Fase: Phase::ChemicalLayer
pub fn state_transitions_system(
    layout: Res<SimWorldTransformParams>,
    ctx_lookup: ContextLookup,
    mut query: Query<(
        Entity,
        &BaseEnergy,
        &SpatialVolume,
        &Transform,
        &mut MatterCoherence,
        Option<&ResonanceThermalOverlay>,
    )>,
    mut ev_transition: EventWriter<PhaseTransitionEvent>,
) {
    let xz = layout.use_xz_ground;
    for (entity, energy, volume, transform, mut matter, overlay_opt) in query.iter_mut() {
        let dens = volume.density(energy.qe());
        let pos = sim_plane_pos(transform.translation, xz);
        let temp =
            equations::equivalent_temperature(dens) + ctx_lookup.context_at(pos).temperature_base;
        let bond_mult = overlay_opt
            .map(|overlay| overlay.bond_energy_multiplier)
            .unwrap_or(1.0)
            .max(0.0);
        let effective_bond = matter.bond_energy_eb() * bond_mult;
        let new_state = equations::state_from_temperature(temp, effective_bond);

        if new_state != matter.state() {
            let previous = matter.state();
            matter.set_state(new_state);

            ev_transition.send(PhaseTransitionEvent {
                entity,
                previous_state: previous,
                new_state,
            });
        }
    }
}

/// Escaneo (Filtro Espacial C2): Detecta colisiones físicas sin matemática compleja.
pub fn catalysis_spatial_filter_system(
    layout: Res<SimWorldTransformParams>,
    ctx_lookup: ContextLookup,
    spatial_index: Res<SpatialIndex>,
    spells: Query<(
        Entity,
        &Transform,
        &AlchemicalInjector,
        &SpellMarker,
        Option<&OnContactEffect>,
        Option<&DespawnOnContact>,
    )>,
    targets: Query<(Entity, &Transform, &SpatialVolume), Without<SpellMarker>>,
    mut ev_request: EventWriter<CatalysisRequest>,
) {
    let xz = layout.use_xz_ground;
    let mut spells_sorted: Vec<_> = spells.iter().collect();
    spells_sorted.sort_by_key(|(spell_ent, ..)| spell_ent.to_bits());

    for (spell_ent, spell_transform, injector, marker, effect_opt, despawn_opt) in spells_sorted {
        let spell_pos = sim_plane_pos(spell_transform.translation, xz);
        let caster = marker.caster;
        let on_contact_effect = effect_opt.map(|e| e.recipe.clone());
        let despawn_on_contact = despawn_opt.is_some();

        let mut nearby = spatial_index.query_radius(spell_pos, injector.influence_radius);
        nearby.retain(|e| e.entity != spell_ent);
        nearby.sort_by_key(|e| e.entity.to_bits());

        for entry in nearby {
            let Ok((target_ent, target_transform, target_vol)) = targets.get(entry.entity) else {
                continue;
            };

            let target_pos = sim_plane_pos(target_transform.translation, xz);
            let distance = (spell_pos - target_pos).length();

            if distance > injector.influence_radius + target_vol.radius {
                continue;
            }

            if ctx_lookup.should_skip_catalysis_at(target_pos) {
                continue;
            }

            ev_request.send(CatalysisRequest {
                spell: spell_ent,
                target: target_ent,
                caster,
                on_contact_effect: on_contact_effect.clone(),
                despawn_on_contact,
            });

            if despawn_on_contact {
                break;
            }
        }
    }
}

/// Evaluador (Strategy C3): Consume el request, evalúa la termodinámica dura de catálisis.
pub fn catalysis_math_strategy_system(
    mut ev_request: EventReader<CatalysisRequest>,
    sim_elapsed: Option<Res<SimulationElapsed>>,
    spells: Query<(&AlchemicalInjector, &OscillatorySignature)>,
    targets: Query<
        (Entity, &OscillatorySignature, Option<&MobaIdentity>),
        Without<SpellMarker>,
    >,
    identities: Query<&MobaIdentity>,
    mut ev_commit: EventWriter<DeltaEnergyCommit>,
) {
    let phase_t = sim_elapsed.map(|r| r.secs).unwrap_or(0.0);

    for req in ev_request.read() {
        let Ok((injector, spell_sig)) = spells.get(req.spell) else {
            continue;
        };

        let Ok((_ent, target_signature, identity_opt)) = targets.get(req.target) else {
            continue;
        };

        let faction_mod = match (req.caster, identity_opt) {
            (Some(caster_ent), Some(target_identity)) => identities
                .get(caster_ent)
                .ok()
                .map(|caster_identity| caster_identity.faction_modifier(target_identity))
                .unwrap_or(0.0),
            _ => 0.0,
        };

        let interf = compute_interference_total(
            spell_sig.frequency_hz(),
            spell_sig.phase(),
            target_signature.frequency_hz(),
            target_signature.phase(),
            phase_t,
            faction_mod,
        );

        let multiplier = identity_opt.map(|id| id.critical_multiplier()).unwrap_or(1.0);
        let result = equations::catalysis_result(injector.projected_qe, interf, multiplier);

        if result.abs() < CATALYSIS_MIN_EFFECT_QE {
            continue;
        }

        let positive_freq_delta = if result > 0.0 {
            Some(equations::frequency_lock_delta(
                injector.forced_frequency,
                target_signature.frequency_hz(),
            ))
        } else {
            None
        };

        let bond_weakening_factor = if result < 0.0 {
            Some(equations::weakening_factor(interf))
        } else {
            None
        };

        ev_commit.send(DeltaEnergyCommit {
            spell: req.spell,
            target: req.target,
            caster: req.caster,
            result_qe: result,
            interference: interf,
            positive_freq_delta,
            bond_weakening_factor,
            on_contact_effect: req.on_contact_effect.clone(),
            despawn_on_contact: req.despawn_on_contact,
        });
    }
}

/// Aplicador (Reducer C4): Inyecta o roba ergios ciegamente basándose en la decisión del Strategy.
pub fn catalysis_energy_reducer_system(
    mut ev_commit: EventReader<DeltaEnergyCommit>,
    mut energy_ops: EnergyOps,
    mut targets: Query<
        (Entity, &mut OscillatorySignature, Option<&mut MatterCoherence>),
        Without<SpellMarker>,
    >,
) {
    for commit in ev_commit.read() {
        let Ok((target_ent, mut target_signature, mut matter_opt)) = targets.get_mut(commit.target)
        else {
            continue;
        };

        if commit.result_qe > 0.0 {
            energy_ops.inject(target_ent, commit.result_qe);
            if let Some(delta) = commit.positive_freq_delta {
                let hz = target_signature.frequency_hz() + delta;
                if target_signature.frequency_hz() != hz {
                    target_signature.set_frequency_hz(hz);
                }
            }
        } else {
            energy_ops.drain(target_ent, commit.result_qe.abs(), DeathCause::Destruction);
            if let Some(w) = commit.bond_weakening_factor {
                if let Some(ref mut matter) = matter_opt {
                    let eb = matter.bond_energy_eb();
                    let new_eb = equations::bond_weakening(eb, w);
                    if eb != new_eb {
                        matter.set_bond_energy_eb(new_eb);
                    }
                }
            }
        }
    }
}

/// Side-effects C4: spawn/despawn + telemetría de catálisis.
pub fn catalysis_side_effects_system(
    mut commands: Commands,
    mut id_gen: ResMut<IdGenerator>,
    mut ev_commit: EventReader<DeltaEnergyCommit>,
    mut energy_ops: EnergyOps,
    mut ev_catalysis: EventWriter<CatalysisEvent>,
) {
    for commit in ev_commit.read() {
        if let Some(recipe) = commit.on_contact_effect.as_ref() {
            let cfg = EffectConfig {
                target: commit.target,
                modified_field: recipe.field,
                magnitude: recipe.magnitude,
                fuel_qe: recipe.fuel_qe,
                dissipation_rate: recipe.dissipation,
            };
            spawn_effect(&mut commands, &mut id_gen, cfg);
        }

        if commit.despawn_on_contact {
            let qe_now = energy_ops.qe(commit.spell).unwrap_or(0.0);
            if qe_now > 0.0 {
                energy_ops.drain(commit.spell, qe_now, DeathCause::Destruction);
            }
        }

        ev_catalysis.send(CatalysisEvent {
            caster: commit.caster,
            target: commit.target,
            spell: commit.spell,
            interference: commit.interference,
            applied_qe: commit.result_qe,
        });
    }
}

/// Registra la cadena `Phase::ChemicalLayer` (orden fijo Event-Driven).
pub fn register_reactions_phase_systems<S: ScheduleLabel + Clone>(app: &mut App, schedule: S) {
    app.add_systems(
        schedule,
        (
            crate::simulation::osmosis::osmotic_diffusion_system,
            crate::simulation::nutrient_uptake::nutrient_regen_system,
            crate::simulation::nutrient_uptake::nutrient_uptake_system,
            crate::simulation::competitive_exclusion::competitive_exclusion_system,
            crate::simulation::photosynthesis::photosynthetic_contribution_system,
            crate::simulation::nutrient_uptake::nutrient_depletion_system,
            state_transitions_system,
            catalysis_spatial_filter_system,
            catalysis_math_strategy_system,
            catalysis_energy_reducer_system,
            catalysis_side_effects_system,
            structural_runtime::homeostasis_system,
            crate::simulation::homeostasis_thermo::thermoregulation_cost_system,
            crate::simulation::homeostasis_thermo::homeostasis_stability_check_system,
            crate::simulation::nutrient_uptake::nutrient_return_on_death_system,
        )
            .chain()
            .in_set(Phase::ChemicalLayer),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    use crate::events::{CatalysisRequest, DeathEvent, DeltaEnergyCommit};
    use crate::layers::{
        AlchemicalInjector, Faction, MobaIdentity, OscillatorySignature,
    };
    use crate::runtime_platform::simulation_tick::SimulationElapsed;

    /// Minimal Bevy app with only the events and resources needed by
    /// `catalysis_math_strategy_system`.
    fn strategy_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<CatalysisRequest>();
        app.add_event::<DeltaEnergyCommit>();
        app.add_event::<DeathEvent>();
        app.insert_resource(SimulationElapsed { secs: 0.0 });
        app.add_systems(Update, catalysis_math_strategy_system);
        app
    }

    fn drain_commits(app: &mut App) -> Vec<DeltaEnergyCommit> {
        app.world_mut()
            .resource_mut::<Events<DeltaEnergyCommit>>()
            .drain()
            .collect()
    }

    fn send_catalysis_request(app: &mut App, req: CatalysisRequest) {
        app.world_mut()
            .resource_mut::<Events<CatalysisRequest>>()
            .send(req);
    }

    #[test]
    fn catalysis_positive_result_commits_freq_shift() {
        let mut app = strategy_test_app();

        // Spell and target share frequency+phase at t=0 -> interference = cos(0) = 1.0
        // (constructive). projected_qe = 80 -> result = +80.
        let spell = app
            .world_mut()
            .spawn((
                AlchemicalInjector::new(80.0, 450.0, 5.0),
                OscillatorySignature::new(450.0, 0.0),
                SpellMarker { caster: None },
            ))
            .id();

        let target = app
            .world_mut()
            .spawn((OscillatorySignature::new(450.0, 0.0),))
            .id();

        send_catalysis_request(&mut app, CatalysisRequest {
            spell,
            target,
            caster: None,
            on_contact_effect: None,
            despawn_on_contact: false,
        });

        app.update();

        let commits = drain_commits(&mut app);
        assert_eq!(commits.len(), 1, "exactly one commit expected");
        let c = &commits[0];
        assert!(c.result_qe > 0.0, "constructive catalysis must yield positive result_qe");
        assert!(
            c.positive_freq_delta.is_some(),
            "positive result must carry a freq_delta"
        );
        assert!(
            c.bond_weakening_factor.is_none(),
            "positive result must NOT carry bond_weakening"
        );
        assert_eq!(c.spell, spell);
        assert_eq!(c.target, target);
    }

    #[test]
    fn catalysis_negative_result_commits_bond_weakening() {
        let mut app = strategy_test_app();

        // Opposing phases (0 vs PI) at same frequency, t=0 -> interference = cos(-PI) = -1.0
        // (destructive). projected_qe = 80 -> result = -80.
        let spell = app
            .world_mut()
            .spawn((
                AlchemicalInjector::new(80.0, 450.0, 5.0),
                OscillatorySignature::new(450.0, 0.0),
                SpellMarker { caster: None },
            ))
            .id();

        let target = app
            .world_mut()
            .spawn((OscillatorySignature::new(450.0, PI),))
            .id();

        send_catalysis_request(&mut app, CatalysisRequest {
            spell,
            target,
            caster: None,
            on_contact_effect: None,
            despawn_on_contact: false,
        });

        app.update();

        let commits = drain_commits(&mut app);
        assert_eq!(commits.len(), 1, "exactly one commit expected");
        let c = &commits[0];
        assert!(c.result_qe < 0.0, "destructive catalysis must yield negative result_qe");
        assert!(
            c.bond_weakening_factor.is_some(),
            "negative result must carry bond_weakening_factor"
        );
        assert!(
            c.positive_freq_delta.is_none(),
            "negative result must NOT carry freq_delta"
        );
    }

    #[test]
    fn catalysis_below_threshold_ignored() {
        let mut app = strategy_test_app();

        // projected_qe = 0.005 -> result = 0.005 * interf (at most 0.005) < CATALYSIS_MIN_EFFECT_QE (0.01)
        let spell = app
            .world_mut()
            .spawn((
                AlchemicalInjector::new(0.005, 450.0, 5.0),
                OscillatorySignature::new(450.0, 0.0),
                SpellMarker { caster: None },
            ))
            .id();

        let target = app
            .world_mut()
            .spawn((OscillatorySignature::new(450.0, 0.0),))
            .id();

        send_catalysis_request(&mut app, CatalysisRequest {
            spell,
            target,
            caster: None,
            on_contact_effect: None,
            despawn_on_contact: false,
        });

        app.update();

        let commits = drain_commits(&mut app);
        assert!(
            commits.is_empty(),
            "result below CATALYSIS_MIN_EFFECT_QE must not emit DeltaEnergyCommit"
        );
    }

    #[test]
    fn catalysis_faction_modifier_applied() {
        let mut app = strategy_test_app();

        // Caster = Red, Target = Blue -> enemy -> FACTION_ENEMY_MALUS = -0.2
        // Same freq+phase at t=0 -> raw interference = 1.0
        // Total interference = clamp(1.0 + (-0.2)) = 0.8 (still constructive)
        // With faction modifier the result changes vs no-faction.
        let caster = app
            .world_mut()
            .spawn(MobaIdentity {
                faction: Faction::Red,
                relational_tags: Vec::new(),
                critical_multiplier: 1.0,
            })
            .id();

        let spell = app
            .world_mut()
            .spawn((
                AlchemicalInjector::new(80.0, 450.0, 5.0),
                OscillatorySignature::new(450.0, 0.0),
                SpellMarker { caster: Some(caster) },
            ))
            .id();

        let target = app
            .world_mut()
            .spawn((
                OscillatorySignature::new(450.0, 0.0),
                MobaIdentity {
                    faction: Faction::Blue,
                    relational_tags: Vec::new(),
                    critical_multiplier: 1.0,
                },
            ))
            .id();

        send_catalysis_request(&mut app, CatalysisRequest {
            spell,
            target,
            caster: Some(caster),
            on_contact_effect: None,
            despawn_on_contact: false,
        });

        app.update();

        let commits = drain_commits(&mut app);
        assert_eq!(commits.len(), 1, "exactly one commit expected");
        let c = &commits[0];

        // Without faction modifier: interference = 1.0, result = 80.0
        // With enemy malus: interference = 0.8, result = 80.0 * 0.8 = 64.0 (not critical: 0.8 < 0.9)
        let no_faction_result = equations::catalysis_result(80.0, 1.0, 1.0);
        assert!(
            (c.result_qe - no_faction_result).abs() > 1.0,
            "faction modifier must influence the result: got {} vs no-faction {}",
            c.result_qe,
            no_faction_result,
        );
        // Verify the interference stored in the commit reflects the faction modifier
        assert!(
            (c.interference - 0.8).abs() < 1e-5,
            "interference must include faction malus: got {}",
            c.interference,
        );
    }
}
