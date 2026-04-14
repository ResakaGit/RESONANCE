//! Motor de zoom — colapsa entidades padre en hijos y agrega hijos en padre.
//! Zoom engine — collapses parent entities into children and aggregates back.
//!
//! CT-1. ADR-036 §D2, §D5. Axiom compliance:
//! - Pool Invariant (Ax 2): `sum(children.qe) <= parent.qe × (1 - dissipation)`
//! - Dissipation (Ax 4): zoom siempre pierde energía
//! - Oscillatory (Ax 8): frecuencias hijas en banda `parent.freq ± bandwidth`
//!
//! Matemática pura en `blueprint/equations/scale_inference.rs`.

use bevy::prelude::*;

use crate::blueprint::equations::derived_thresholds::COHERENCE_BANDWIDTH;
use crate::blueprint::equations::determinism;
use crate::blueprint::equations::scale_inference::{
    self as inf, AggregateState, InferenceMatterState,
};

use super::ScaleLevel;
use super::scale_manager::{CosmicEntity, ScaleInstance, ScaleManager};

// ─── Events ─────────────────────────────────────────────────────────────────

/// Solicitud de colapso observacional en una entidad del nivel indicado.
/// Request to collapse into an entity at the specified level.
#[derive(Event, Debug, Clone, Copy)]
pub struct ZoomInEvent {
    pub target_scale: ScaleLevel,
    pub parent_entity_id: u32,
    pub observer_seed: u64,
}

/// Solicitud de volver al nivel padre del observado actualmente.
/// Request to return from the observed level to its parent.
#[derive(Event, Debug, Clone, Copy)]
pub struct ZoomOutEvent;

// ─── Zoom configuration per child scale ─────────────────────────────────────

/// Parámetros de inferencia para colapsar hacia una escala hija.
/// Inference parameters to collapse into a child scale.
#[derive(Clone, Copy, Debug)]
pub struct ZoomConfig {
    pub child_scale_factor: f64,
    pub min_children: usize,
    pub max_children: usize,
    pub matter_state: InferenceMatterState,
    pub bandwidth: f64,
    pub capacity_hint: usize,
}

impl ZoomConfig {
    /// Config por defecto para una escala hija dada.
    /// Default config per child scale.
    pub const fn for_child(child: ScaleLevel) -> Self {
        let bw = COHERENCE_BANDWIDTH as f64;
        match child {
            ScaleLevel::Cosmological => Self {
                child_scale_factor: 1.0,
                min_children: 1,
                max_children: 1,
                matter_state: InferenceMatterState::Plasma,
                bandwidth: bw,
                capacity_hint: 1,
            },
            ScaleLevel::Stellar => Self {
                child_scale_factor: 0.3,
                min_children: 20,
                max_children: 500,
                matter_state: InferenceMatterState::Plasma,
                bandwidth: bw,
                capacity_hint: 512,
            },
            ScaleLevel::Planetary => Self {
                child_scale_factor: 0.5,
                min_children: 3,
                max_children: 20,
                matter_state: InferenceMatterState::Gas,
                bandwidth: bw,
                capacity_hint: 32,
            },
            ScaleLevel::Ecological => Self {
                child_scale_factor: 2.0,
                min_children: 10,
                max_children: 200,
                matter_state: InferenceMatterState::Liquid,
                bandwidth: bw,
                capacity_hint: 256,
            },
            ScaleLevel::Molecular => Self {
                child_scale_factor: 5.0,
                min_children: 20,
                max_children: 500,
                matter_state: InferenceMatterState::Solid,
                bandwidth: bw,
                capacity_hint: 512,
            },
        }
    }
}

// ─── Deterministic seed derivation ──────────────────────────────────────────

/// Deriva `zoom_seed` desde universo + padre + observador. FNV-like mixing.
/// Derives `zoom_seed` from universe + parent + observer. FNV-like mixing.
#[inline]
pub const fn derive_zoom_seed(universe_seed: u64, parent_entity_id: u32, observer_seed: u64) -> u64 {
    let mut h = universe_seed;
    h ^= (parent_entity_id as u64).wrapping_mul(0x100000001B3);
    h ^= observer_seed.wrapping_mul(0xCBF29CE484222325);
    h = h.wrapping_mul(0x100000001B3);
    h ^ (h >> 33)
}

// ─── Collapse (zoom-in) ─────────────────────────────────────────────────────

/// Colapsa una entidad padre en una `ScaleInstance` hija vía inferencia axiomática.
/// Collapses a parent entity into a child `ScaleInstance` via axiom-constrained inference.
///
/// Retorna `None` si `parent_level` no tiene hijo (Molecular) o si la energía es nula.
/// Returns `None` when no child level exists (Molecular) or energy is null.
pub fn collapse_parent(
    parent: &CosmicEntity,
    parent_level: ScaleLevel,
    universe_seed: u64,
    observer_seed: u64,
) -> Option<ScaleInstance> {
    let child_level = parent_level.child()?;
    let cfg = ZoomConfig::for_child(child_level);
    let zoom_seed = derive_zoom_seed(universe_seed, parent.entity_id, observer_seed);

    let n = inf::kleiber_child_count(
        parent.qe,
        cfg.child_scale_factor,
        cfg.min_children,
        cfg.max_children,
    );
    if n == 0 { return None; }

    let qes = inf::distribute_energy(parent.qe, n, cfg.matter_state, zoom_seed);
    let freqs = inf::distribute_frequencies(parent.frequency_hz, n, cfg.bandwidth, zoom_seed);
    let positions = inf::distribute_positions_3d(parent.position, parent.radius, n, zoom_seed);

    let mut instance = ScaleInstance::new(child_level, cfg.capacity_hint, zoom_seed)
        .with_parent(parent.entity_id);

    for i in 0..n {
        let qe = qes[i];
        let radius = qe.max(1e-9).powf(1.0 / 3.0);
        let velocity = thermal_velocity(parent.velocity, qe, zoom_seed.wrapping_add(i as u64));
        let child = CosmicEntity {
            qe,
            radius,
            frequency_hz: freqs[i],
            phase: 0.0,
            position: positions[i],
            velocity,
            dissipation: cfg.matter_state.dissipation(),
            age_ticks: 0,
            entity_id: 0,
            alive: true,
        };
        instance.world.spawn(child);
    }
    instance.world.total_qe_initial = instance.world.total_qe();
    Some(instance)
}

/// Velocidad inicial = velocidad del padre + perturbación térmica ~ 1/√qe (equipartición).
/// Initial velocity = parent velocity + thermal noise ~ 1/√qe (equipartition).
fn thermal_velocity(parent_vel: [f64; 3], child_qe: f64, seed: u64) -> [f64; 3] {
    let sigma = 1.0 / child_qe.max(1.0).sqrt();
    let mut v = parent_vel;
    let mut rng = seed;
    for component in v.iter_mut() {
        rng = determinism::next_u64(rng);
        let z = determinism::gaussian_f32(rng, sigma as f32) as f64;
        *component += z;
    }
    v
}

// ─── Aggregate (zoom-out) ───────────────────────────────────────────────────

/// Agrega el estado de una instancia hija (qe suma, freq media ponderada, centroide).
/// Aggregates a child instance (qe sum, qe-weighted mean freq, centroid).
pub fn aggregate_child(instance: &ScaleInstance) -> AggregateState {
    let alive = instance.world.entities.iter().filter(|e| e.alive);
    let qes: Vec<f64> = alive.clone().map(|e| e.qe).collect();
    let freqs: Vec<f64> = alive.clone().map(|e| e.frequency_hz).collect();
    let positions: Vec<[f64; 3]> = alive.map(|e| e.position).collect();
    inf::aggregate_to_parent(&qes, &freqs, &positions)
}

// ─── Systems ────────────────────────────────────────────────────────────────

/// Procesa `ZoomInEvent`: colapsa al padre, inserta hija, congela padre.
/// Handles `ZoomInEvent`: collapses parent, inserts child, freezes parent.
pub fn zoom_in_system(
    mut events: EventReader<ZoomInEvent>,
    mut mgr: ResMut<ScaleManager>,
) {
    for ev in events.read() {
        let Some(parent_instance) = mgr.get(ev.target_scale) else { continue; };
        let Some(parent) = parent_instance
            .world
            .entities
            .iter()
            .find(|e| e.entity_id == ev.parent_entity_id && e.alive)
            .copied()
        else {
            continue;
        };

        let universe_seed = mgr.universe_seed;
        let Some(child) = collapse_parent(&parent, ev.target_scale, universe_seed, ev.observer_seed)
        else {
            continue;
        };

        let child_level = child.level;
        mgr.insert(child);
        if let Some(parent_inst) = mgr.get_mut(ev.target_scale) {
            parent_inst.freeze();
        }
        if mgr.observed != child_level { mgr.observed = child_level; }
    }
}

/// Procesa `ZoomOutEvent`: agrega la instancia observada en su padre, descongela.
/// Handles `ZoomOutEvent`: aggregates observed into its parent, unfreezes.
pub fn zoom_out_system(
    mut events: EventReader<ZoomOutEvent>,
    mut mgr: ResMut<ScaleManager>,
) {
    for _ in events.read() {
        let observed = mgr.observed;
        let Some(parent_level) = observed.parent() else { continue; };
        let Some(instance) = mgr.get(observed) else { continue; };
        let Some(parent_entity_id) = instance.parent_entity_id else { continue; };

        let agg = aggregate_child(instance);

        if let Some(parent_inst) = mgr.get_mut(parent_level) {
            if let Some(parent_entity) = parent_inst
                .world
                .entities
                .iter_mut()
                .find(|e| e.entity_id == parent_entity_id)
            {
                if parent_entity.qe != agg.qe { parent_entity.qe = agg.qe; }
                if parent_entity.frequency_hz != agg.frequency_hz {
                    parent_entity.frequency_hz = agg.frequency_hz;
                }
                if parent_entity.position != agg.position { parent_entity.position = agg.position; }
            }
            parent_inst.unfreeze();
        }

        mgr.remove(observed);
        if mgr.observed != parent_level { mgr.observed = parent_level; }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_parent() -> CosmicEntity {
        CosmicEntity {
            qe: 1000.0,
            radius: 10.0,
            frequency_hz: 100.0,
            phase: 0.0,
            position: [0.0, 0.0, 0.0],
            velocity: [1.0, 0.0, 0.0],
            dissipation: 0.02,
            age_ticks: 100,
            entity_id: 7,
            alive: true,
        }
    }

    #[test]
    fn collapse_produces_children() {
        let p = sample_parent();
        let inst = collapse_parent(&p, ScaleLevel::Ecological, 42, 1).unwrap();
        assert_eq!(inst.level, ScaleLevel::Molecular);
        assert!(inst.world.n_alive() > 0);
    }

    #[test]
    fn collapse_at_molecular_returns_none() {
        let p = sample_parent();
        assert!(collapse_parent(&p, ScaleLevel::Molecular, 42, 1).is_none());
    }

    #[test]
    fn collapse_applies_dissipation() {
        let p = sample_parent();
        let inst = collapse_parent(&p, ScaleLevel::Stellar, 42, 1).unwrap();
        let sum = inst.world.total_qe();
        assert!(sum < p.qe, "sum {sum} >= parent {}", p.qe);
        assert!(sum > 0.0);
    }

    #[test]
    fn collapse_deterministic_same_seed() {
        let p = sample_parent();
        let a = collapse_parent(&p, ScaleLevel::Cosmological, 42, 1).unwrap();
        let b = collapse_parent(&p, ScaleLevel::Cosmological, 42, 1).unwrap();
        assert_eq!(a.world.n_alive(), b.world.n_alive());
        for (ea, eb) in a.world.entities.iter().zip(&b.world.entities) {
            assert_eq!(ea.qe, eb.qe);
            assert_eq!(ea.frequency_hz, eb.frequency_hz);
            assert_eq!(ea.position, eb.position);
            assert_eq!(ea.velocity, eb.velocity);
        }
    }

    #[test]
    fn collapse_different_observer_seed_diverges() {
        let p = sample_parent();
        let a = collapse_parent(&p, ScaleLevel::Cosmological, 42, 1).unwrap();
        let b = collapse_parent(&p, ScaleLevel::Cosmological, 42, 2).unwrap();
        let any_diff = a
            .world
            .entities
            .iter()
            .zip(&b.world.entities)
            .any(|(ea, eb)| ea.qe != eb.qe || ea.position != eb.position);
        assert!(any_diff, "different observer_seed must diverge (multiverse)");
    }

    #[test]
    fn collapse_frequencies_within_bandwidth_band() {
        let p = sample_parent();
        let inst = collapse_parent(&p, ScaleLevel::Ecological, 42, 1).unwrap();
        let bw = COHERENCE_BANDWIDTH as f64;
        for e in &inst.world.entities {
            let diff = (e.frequency_hz - p.frequency_hz).abs();
            assert!(
                diff < 6.0 * bw,
                "freq diff {diff} exceeds ±6σ of bandwidth {bw}",
            );
        }
    }

    #[test]
    fn round_trip_preserves_pool_invariant() {
        let p = sample_parent();
        let inst = collapse_parent(&p, ScaleLevel::Stellar, 42, 1).unwrap();
        let sum_children = inst.world.total_qe();
        let agg = aggregate_child(&inst);
        assert!(agg.qe <= p.qe, "agg {} > parent {}", agg.qe, p.qe);
        assert!((agg.qe - sum_children).abs() < 1e-9);
    }

    #[test]
    fn zoom_seed_deterministic_and_sensitive() {
        let base = derive_zoom_seed(42, 7, 1);
        assert_eq!(base, derive_zoom_seed(42, 7, 1));
        assert_ne!(base, derive_zoom_seed(43, 7, 1));
        assert_ne!(base, derive_zoom_seed(42, 8, 1));
        assert_ne!(base, derive_zoom_seed(42, 7, 2));
    }

    // ─── System-level tests ─────────────────────────────────────────────────

    fn app_with_cosmic() -> App {
        let mut app = App::new();
        app.add_plugins(crate::cosmic::CosmicPlugin);
        app.add_systems(Update, (zoom_in_system, zoom_out_system).chain());
        app
    }

    fn seed_parent_at(app: &mut App, level: ScaleLevel, parent: CosmicEntity) -> u32 {
        let mut mgr = app.world_mut().resource_mut::<ScaleManager>();
        let mut instance = ScaleInstance::new(level, 4, 1);
        let id = instance.world.spawn(parent);
        instance.world.total_qe_initial = instance.world.total_qe();
        mgr.insert(instance);
        mgr.observed = level;
        id
    }

    #[test]
    fn system_zoom_in_creates_child_and_freezes_parent() {
        let mut app = app_with_cosmic();
        let parent = sample_parent();
        let parent_id = seed_parent_at(&mut app, ScaleLevel::Cosmological, parent);

        app.world_mut().send_event(ZoomInEvent {
            target_scale: ScaleLevel::Cosmological,
            parent_entity_id: parent_id,
            observer_seed: 11,
        });
        app.update();

        let mgr = app.world().resource::<ScaleManager>();
        assert_eq!(mgr.observed, ScaleLevel::Stellar);
        assert!(mgr.has(ScaleLevel::Stellar));
        assert!(mgr.get(ScaleLevel::Cosmological).unwrap().frozen);
    }

    #[test]
    fn system_zoom_out_removes_child_and_unfreezes_parent() {
        let mut app = app_with_cosmic();
        let parent = sample_parent();
        let parent_id = seed_parent_at(&mut app, ScaleLevel::Cosmological, parent);

        app.world_mut().send_event(ZoomInEvent {
            target_scale: ScaleLevel::Cosmological,
            parent_entity_id: parent_id,
            observer_seed: 11,
        });
        app.update();
        app.world_mut().send_event(ZoomOutEvent);
        app.update();

        let mgr = app.world().resource::<ScaleManager>();
        assert_eq!(mgr.observed, ScaleLevel::Cosmological);
        assert!(!mgr.has(ScaleLevel::Stellar));
        assert!(!mgr.get(ScaleLevel::Cosmological).unwrap().frozen);
    }

    #[test]
    fn system_round_trip_non_increasing_parent_qe() {
        let mut app = app_with_cosmic();
        let parent = sample_parent();
        let parent_id = seed_parent_at(&mut app, ScaleLevel::Cosmological, parent);
        let qe_before = parent.qe;

        app.world_mut().send_event(ZoomInEvent {
            target_scale: ScaleLevel::Cosmological,
            parent_entity_id: parent_id,
            observer_seed: 11,
        });
        app.update();
        app.world_mut().send_event(ZoomOutEvent);
        app.update();

        let mgr = app.world().resource::<ScaleManager>();
        let restored = mgr
            .get(ScaleLevel::Cosmological)
            .unwrap()
            .world
            .entities
            .iter()
            .find(|e| e.entity_id == parent_id)
            .unwrap();
        assert!(
            restored.qe <= qe_before,
            "qe must not grow across zoom: {} > {}",
            restored.qe,
            qe_before,
        );
        assert!(restored.qe > 0.0);
    }
}
