//! Observer — API compartida para seed del universo y zoom-in vía bridges.
//! Observer — shared API for universe seeding and bridge-driven zoom-in.
//!
//! Capa de abstracción entre los binarios `cosmic_telescope` (3D viewer) y
//! `cosmic_telescope_headless` (CI validation). Ambos necesitan:
//!   - Big Bang determinista en S0 (`seed_universe`).
//!   - Zoom-in que prefiere los bridges específicos por escala (más ricos que
//!     el `collapse_parent` genérico de CT-1).
//!   - Localizar la entidad dominante de una escala (`largest_entity_in`).
//!
//! Mantiene `ScaleManager` como única fuente de verdad; no introduce ni
//! duplica estado. No toca el contrato público del módulo (ADR-036 §D2/§D5).

use crate::blueprint::domain_enums::MatterState;
use crate::blueprint::equations::derived_thresholds::{
    COHERENCE_BANDWIDTH, DISSIPATION_LIQUID, DISSIPATION_SOLID,
};
use crate::blueprint::equations::proteome_inference::infer_proteome;

use super::bridges::cosmo_to_stellar::expand_cluster;
use super::bridges::planetary_to_ecological::planet_to_map_config;
use super::bridges::stellar_to_planetary::{expand_stellar_system, PlanetSpec};
use super::constants::{
    ECOLOGICAL_CAPTURE_FRACTION, ECOLOGICAL_PROBE_TEMPERATURE, INTERACTIVE_BIG_BANG_CLUSTERS,
    INTERACTIVE_BIG_BANG_TOTAL_QE, INTERACTIVE_BIG_BANG_WARMUP_TICKS, MAX_PLANETS_PER_STAR,
    MAX_STARS_PER_CLUSTER, MIN_PLANETS_PER_STAR, MIN_STARS_PER_CLUSTER, STELLAR_DISK_FRACTION,
};
use super::scale_manager::{CosmicEntity, ScaleInstance, ScaleManager};
use super::scales::cosmological::{cosmo_tick, init_big_bang, CosmoConfig};
use super::ScaleLevel;

// ─── Big Bang seeding ──────────────────────────────────────────────────────

/// Parámetros de inicialización del universo (S0 Cosmológico).
/// Universe initialization parameters (S0 Cosmological).
#[derive(Clone, Copy, Debug)]
pub struct BigBangParams {
    pub seed: u64,
    pub n_clusters: usize,
    pub total_qe: f64,
    pub warmup_ticks: usize,
}

impl BigBangParams {
    /// Preset balanceado para visualización interactiva. Parámetros viven en
    /// `cosmic::constants` — editables sin tocar código de lógica.
    /// Balanced preset for interactive viz.
    pub const fn interactive(seed: u64) -> Self {
        Self {
            seed,
            n_clusters: INTERACTIVE_BIG_BANG_CLUSTERS,
            total_qe: INTERACTIVE_BIG_BANG_TOTAL_QE,
            warmup_ticks: INTERACTIVE_BIG_BANG_WARMUP_TICKS,
        }
    }
}

/// Siembra una instancia S0 Cosmológica en el `ScaleManager`, reemplazando
/// cualquier estado previo. Deja `observed = Cosmological` y `universe_seed`.
/// Seeds S0 Cosmological into the manager, resetting prior state.
pub fn seed_universe(mgr: &mut ScaleManager, params: &BigBangParams) {
    let mut cfg = CosmoConfig::default_with_seed(params.seed);
    cfg.n_initial_clusters = params.n_clusters;
    cfg.total_qe = params.total_qe;

    let mut world = init_big_bang(&cfg);
    for _ in 0..params.warmup_ticks { cosmo_tick(&mut world, &cfg); }

    let mut instance = ScaleInstance::new(ScaleLevel::Cosmological, world.entities.len(), params.seed);
    instance.world = world;
    instance.world.total_qe_initial = instance.world.total_qe();

    mgr.instances.clear();
    mgr.universe_seed = params.seed;
    mgr.observed = ScaleLevel::Cosmological;
    mgr.insert(instance);
}

// ─── Queries ───────────────────────────────────────────────────────────────

/// Devuelve el `entity_id` de la entidad viva con mayor `qe` en `level`.
/// Returns the entity_id of the highest-qe live entity at `level`.
pub fn largest_entity_in(mgr: &ScaleManager, level: ScaleLevel) -> Option<u32> {
    let inst = mgr.get(level)?;
    inst.world
        .entities
        .iter()
        .filter(|e| e.alive)
        .max_by(|a, b| a.qe.total_cmp(&b.qe))
        .map(|e| e.entity_id)
}

// ─── Bridge-driven zoom-in ─────────────────────────────────────────────────

/// Helper: crea una `CosmicEntity` inmóvil (`velocity=0`, `age=0`,
/// `alive=true`) lista para `CosmicWorld::spawn`. El `entity_id` lo asigna
/// el `spawn` — se deja en 0 y es ignorado.
#[inline]
fn stationary_entity(
    qe: f64,
    radius: f64,
    frequency_hz: f64,
    position: [f64; 3],
    dissipation: f64,
) -> CosmicEntity {
    CosmicEntity {
        qe,
        radius,
        frequency_hz,
        phase: 0.0,
        position,
        velocity: [0.0; 3],
        dissipation,
        age_ticks: 0,
        entity_id: 0,
        alive: true,
    }
}

/// Zoom-in usando el bridge específico de la escala `from`. Inserta la instancia
/// hija en `mgr`, congela al padre y mueve `observed` al nivel hijo.
///
/// Zoom-in via scale-specific bridge. Inserts the child, freezes the parent,
/// and moves `observed` to the child level.
///
/// Devuelve el `ScaleLevel` hijo si el bridge produjo entidades, o `None` si
/// `from == Molecular`, el padre no existe, o el bridge no generó hijos.
///
/// Los bridges son más ricos que `collapse_parent` genérico: producen
/// poblaciones realistas (IMF Salpeter, Titius-Bode, Hill/Kleiber).
pub fn zoom_via_bridge(
    mgr: &mut ScaleManager,
    parent_id: u32,
    from: ScaleLevel,
) -> Option<ScaleLevel> {
    zoom_via_bridge_with_seed(mgr, parent_id, from, mgr.universe_seed)
}

/// Variante de `zoom_via_bridge` con seed explícito — base del multiverso
/// (CT-9): distintas `seed` sobre el mismo padre producen branches paralelos
/// determinísticos. El `zoom_seed` del hijo queda igual a `seed`.
///
/// Explicit-seed variant: different `seed` on the same parent yields a
/// parallel, deterministic multiverse branch.
pub fn zoom_via_bridge_with_seed(
    mgr: &mut ScaleManager,
    parent_id: u32,
    from: ScaleLevel,
    seed: u64,
) -> Option<ScaleLevel> {
    let parent = clone_parent(mgr, from, parent_id)?;
    let bandwidth = COHERENCE_BANDWIDTH as f64;

    let child = match from {
        ScaleLevel::Cosmological => build_stellar(&parent, seed, bandwidth, parent_id),
        ScaleLevel::Stellar => build_planetary(&parent, seed, bandwidth, parent_id),
        ScaleLevel::Planetary => build_ecological(&parent, seed, parent_id),
        ScaleLevel::Ecological => build_molecular(&parent, seed, parent_id),
        ScaleLevel::Molecular => None,
    }?;

    let child_level = child.level;
    mgr.insert(child);
    if let Some(p) = mgr.get_mut(from) { p.freeze(); }
    mgr.observed = child_level;
    Some(child_level)
}

/// Re-ejecuta el bridge de la escala observada con `new_seed`, conservando el
/// mismo padre (y por tanto el breadcrumb). Base de `Tab → cycle seed` (CT-9).
/// Devuelve `None` si observed es S0 (sin padre) o el estado es inconsistente.
///
/// Re-runs the observed scale's bridge with `new_seed`, keeping the same
/// parent. Foundation for the `Tab → cycle seed` multiverse interaction.
pub fn rebranch_observed(mgr: &mut ScaleManager, new_seed: u64) -> Option<ScaleLevel> {
    let observed = mgr.observed;
    let parent_level = observed.parent()?;
    let parent_id = mgr.get(observed)?.parent_entity_id?;
    mgr.remove(observed);
    if let Some(p) = mgr.get_mut(parent_level) { p.unfreeze(); }
    zoom_via_bridge_with_seed(mgr, parent_id, parent_level, new_seed)
}

fn clone_parent(mgr: &ScaleManager, from: ScaleLevel, parent_id: u32) -> Option<CosmicEntity> {
    mgr.get(from)?
        .world
        .entities
        .iter()
        .find(|e| e.entity_id == parent_id && e.alive)
        .copied()
}

fn build_stellar(parent: &CosmicEntity, seed: u64, bandwidth: f64, parent_id: u32) -> Option<ScaleInstance> {
    let exp = expand_cluster(
        parent,
        seed,
        bandwidth,
        MIN_STARS_PER_CLUSTER,
        MAX_STARS_PER_CLUSTER,
    )?;
    let mut child = ScaleInstance::new(ScaleLevel::Stellar, exp.world.entities.len(), seed)
        .with_parent(parent_id);
    child.world = exp.world;
    child.world.total_qe_initial = child.world.total_qe();
    Some(child)
}

fn build_planetary(parent: &CosmicEntity, seed: u64, bandwidth: f64, parent_id: u32) -> Option<ScaleInstance> {
    let disk_qe = parent.qe * STELLAR_DISK_FRACTION;
    let planets = expand_stellar_system(
        parent,
        disk_qe,
        seed,
        bandwidth,
        MIN_PLANETS_PER_STAR,
        MAX_PLANETS_PER_STAR,
    );
    if planets.is_empty() { return None; }

    let mut child = ScaleInstance::new(ScaleLevel::Planetary, planets.len(), seed)
        .with_parent(parent_id);
    for p in &planets {
        child.world.spawn(planet_spec_to_entity(p));
    }
    child.world.total_qe_initial = child.world.total_qe();
    Some(child)
}

fn planet_spec_to_entity(p: &PlanetSpec) -> CosmicEntity {
    stationary_entity(
        p.qe,
        p.qe.powf(1.0 / 3.0).max(1e-3),
        p.frequency_hz,
        [p.orbital_radius, 0.0, 0.0],
        DISSIPATION_LIQUID as f64,
    )
}

fn build_ecological(parent: &CosmicEntity, seed: u64, parent_id: u32) -> Option<ScaleInstance> {
    // Validamos que el planeta produce un MapConfig coherente. El worldgen
    // real vive en otro plugin; aquí mantenemos un placeholder agregado.
    let spec = PlanetSpec {
        qe: parent.qe,
        frequency_hz: parent.frequency_hz,
        orbital_radius: parent.position[0],
        temperature: ECOLOGICAL_PROBE_TEMPERATURE,
        matter_state: MatterState::Liquid,
    };
    let _map = planet_to_map_config(&spec, seed);

    let mut child = ScaleInstance::new(ScaleLevel::Ecological, 1, seed)
        .with_parent(parent_id);
    child.world.spawn(stationary_entity(
        parent.qe * ECOLOGICAL_CAPTURE_FRACTION,
        1.0,
        parent.frequency_hz,
        [0.0; 3],
        DISSIPATION_LIQUID as f64,
    ));
    child.world.total_qe_initial = child.world.total_qe();
    Some(child)
}

fn build_molecular(parent: &CosmicEntity, seed: u64, parent_id: u32) -> Option<ScaleInstance> {
    let proteome = infer_proteome(parent.qe, parent.frequency_hz, parent.age_ticks, seed);
    if proteome.is_empty() { return None; }

    let mut child = ScaleInstance::new(ScaleLevel::Molecular, proteome.len(), seed)
        .with_parent(parent_id);
    for p in &proteome {
        child.world.spawn(stationary_entity(
            p.qe_budget,
            (p.n_residues as f64).powf(1.0 / 3.0),
            parent.frequency_hz,
            [0.0; 3],
            DISSIPATION_SOLID as f64,
        ));
    }
    child.world.total_qe_initial = child.world.total_qe();
    Some(child)
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_universe_populates_cosmological_and_clears_others() {
        let mut mgr = ScaleManager::default();
        mgr.insert(ScaleInstance::new(ScaleLevel::Stellar, 4, 1));
        seed_universe(&mut mgr, &BigBangParams::interactive(7));

        assert_eq!(mgr.universe_seed, 7);
        assert_eq!(mgr.observed, ScaleLevel::Cosmological);
        assert!(mgr.has(ScaleLevel::Cosmological));
        assert!(!mgr.has(ScaleLevel::Stellar), "prior state must be cleared");
    }

    #[test]
    fn seed_universe_is_deterministic() {
        let mut a = ScaleManager::default();
        let mut b = ScaleManager::default();
        seed_universe(&mut a, &BigBangParams::interactive(42));
        seed_universe(&mut b, &BigBangParams::interactive(42));
        let qa = a.get(ScaleLevel::Cosmological).unwrap().world.total_qe();
        let qb = b.get(ScaleLevel::Cosmological).unwrap().world.total_qe();
        assert!((qa - qb).abs() < 1e-9);
    }

    #[test]
    fn largest_entity_picks_max_qe() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(1));
        let id = largest_entity_in(&mgr, ScaleLevel::Cosmological).expect("has clusters");
        let inst = mgr.get(ScaleLevel::Cosmological).unwrap();
        let target = inst.world.entities.iter().find(|e| e.entity_id == id).unwrap();
        let max_qe = inst.world.entities.iter().filter(|e| e.alive).map(|e| e.qe).fold(f64::NEG_INFINITY, f64::max);
        assert_eq!(target.qe, max_qe);
    }

    #[test]
    fn zoom_via_bridge_descends_one_scale_and_freezes_parent() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(3));
        let cluster_id = largest_entity_in(&mgr, ScaleLevel::Cosmological).unwrap();

        let child = zoom_via_bridge(&mut mgr, cluster_id, ScaleLevel::Cosmological);
        assert_eq!(child, Some(ScaleLevel::Stellar));
        assert_eq!(mgr.observed, ScaleLevel::Stellar);
        assert!(mgr.get(ScaleLevel::Cosmological).unwrap().frozen);
    }

    #[test]
    fn zoom_via_bridge_full_descent_reaches_molecular() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(9));

        let mut from = ScaleLevel::Cosmological;
        for _ in 0..4 {
            let pid = largest_entity_in(&mgr, from).expect("parent alive");
            let next = zoom_via_bridge(&mut mgr, pid, from).expect("bridge produced child");
            from = next;
        }
        assert_eq!(from, ScaleLevel::Molecular);
        assert!(mgr.has(ScaleLevel::Molecular));
    }

    #[test]
    fn zoom_via_bridge_at_molecular_returns_none() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(11));
        let mut from = ScaleLevel::Cosmological;
        for _ in 0..4 {
            let pid = largest_entity_in(&mgr, from).unwrap();
            from = zoom_via_bridge(&mut mgr, pid, from).unwrap();
        }
        let pid = largest_entity_in(&mgr, ScaleLevel::Molecular).unwrap();
        assert_eq!(zoom_via_bridge(&mut mgr, pid, ScaleLevel::Molecular), None);
    }

    #[test]
    fn rebranch_observed_at_s0_is_noop() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(4));
        assert_eq!(rebranch_observed(&mut mgr, 99), None);
        assert_eq!(mgr.observed, ScaleLevel::Cosmological);
    }

    #[test]
    fn rebranch_observed_produces_divergent_branch_same_parent() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(5));
        let pid = largest_entity_in(&mgr, ScaleLevel::Cosmological).unwrap();
        zoom_via_bridge(&mut mgr, pid, ScaleLevel::Cosmological).unwrap();

        let parent_id_a = mgr.get(ScaleLevel::Stellar).unwrap().parent_entity_id;
        let qe_a = mgr.get(ScaleLevel::Stellar).unwrap().world.total_qe();

        rebranch_observed(&mut mgr, 7777).expect("rebranch");
        let parent_id_b = mgr.get(ScaleLevel::Stellar).unwrap().parent_entity_id;
        let qe_b = mgr.get(ScaleLevel::Stellar).unwrap().world.total_qe();

        assert_eq!(parent_id_a, parent_id_b, "same parent across branches");
        assert_ne!(qe_a, qe_b, "different seed must diverge");
        assert_eq!(mgr.get(ScaleLevel::Stellar).unwrap().zoom_seed, 7777);
    }

    #[test]
    fn rebranch_observed_same_seed_reproduces_branch() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(6));
        let pid = largest_entity_in(&mgr, ScaleLevel::Cosmological).unwrap();
        zoom_via_bridge_with_seed(&mut mgr, pid, ScaleLevel::Cosmological, 1234).unwrap();
        let qe_a = mgr.get(ScaleLevel::Stellar).unwrap().world.total_qe();

        rebranch_observed(&mut mgr, 9999).expect("rebranch 1");
        rebranch_observed(&mut mgr, 1234).expect("rebranch 2 — replay");
        let qe_b = mgr.get(ScaleLevel::Stellar).unwrap().world.total_qe();

        assert!((qe_a - qe_b).abs() < 1e-9, "same seed must reproduce branch exactly");
    }

    #[test]
    fn zoom_via_bridge_with_missing_parent_is_noop() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(2));
        assert_eq!(zoom_via_bridge(&mut mgr, u32::MAX, ScaleLevel::Cosmological), None);
        assert_eq!(mgr.observed, ScaleLevel::Cosmological);
    }
}
