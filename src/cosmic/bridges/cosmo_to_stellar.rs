//! Bridge S0 → S1 — cluster cosmológico expande en estrellas + discos.
//! Bridge S0 → S1 — cosmological cluster expands into stars + disks.
//!
//! CT-4 / ADR-036 §D4. Orquesta:
//! 1) Cuenta estelar via Kleiber (scale_inference).
//! 2) Masas por Salpeter IMF (stellar_dynamics).
//! 3) Frecuencias heredadas con bandwidth (scale_inference).
//! 4) Posiciones uniformes dentro del cluster (scale_inference).
//! 5) Disco protoplanetario para las top-K estrellas más masivas.

use crate::blueprint::equations::cosmic_gravity;
use crate::blueprint::equations::derived_thresholds::{COHERENCE_BANDWIDTH, DISSIPATION_PLASMA};
use crate::blueprint::equations::determinism;
use crate::blueprint::equations::scale_inference as inf;
use crate::blueprint::equations::stellar_dynamics as sd;

use crate::cosmic::scale_manager::{CosmicEntity, CosmicWorld};

/// Fracción de las estrellas más masivas que reciben disco protoplanetario.
/// Fraction of most massive stars that gain a protoplanetary disk.
pub const DISK_TOP_FRACTION: f64 = 0.1;

/// Partículas por disco protoplanetario (gas/polvo alrededor de la estrella).
/// Particles per protoplanetary disk (gas/dust around the host).
pub const DISK_PARTICLES: usize = 6;

/// Resultado de expansión: mundo estelar + índice de cuántas son estrellas (no disco).
/// Expansion result: stellar world + how many of the first entities are stars.
#[derive(Debug)]
pub struct StellarExpansion {
    pub world: CosmicWorld,
    pub n_stars: usize,
}

/// Expande un cluster en un `StellarExpansion` determinista.
/// Entidades [0, n_stars) son estrellas; [n_stars, end) son partículas de disco.
/// Expands a cluster deterministically. Entities [0, n_stars) are stars.
pub fn expand_cluster(
    cluster: &CosmicEntity,
    seed: u64,
    bandwidth: f64,
    min_stars: usize,
    max_stars: usize,
) -> Option<StellarExpansion> {
    if cluster.qe <= 0.0 { return None; }
    // Kleiber: N ∝ qe^0.75. scale_factor=0.3 viene de CT-1 ZoomConfig::Stellar.
    let n_stars = inf::kleiber_child_count(cluster.qe, 0.3, min_stars.max(1), max_stars);
    if n_stars == 0 { return None; }

    // Pool Invariant: reservar `cluster.qe × (1 - dissipation_plasma)` para las estrellas.
    let star_budget = cluster.qe * (1.0 - DISSIPATION_PLASMA as f64);
    let masses = sd::salpeter_mass_distribution(n_stars, star_budget, seed);
    let freqs = inf::distribute_frequencies(cluster.frequency_hz, n_stars, bandwidth, seed);
    let positions = inf::distribute_positions_3d(cluster.position, cluster.radius, n_stars, seed);

    // Índices ordenados por masa descendente para seleccionar los top-K con disco.
    let mut ranked: Vec<usize> = (0..n_stars).collect();
    ranked.sort_by(|&a, &b| masses[b].partial_cmp(&masses[a]).unwrap_or(std::cmp::Ordering::Equal));
    let k_disks = ((n_stars as f64 * DISK_TOP_FRACTION).ceil() as usize).max(if n_stars > 10 { 1 } else { 0 });
    let disk_hosts: std::collections::BTreeSet<usize> = ranked.into_iter().take(k_disks).collect();

    let mut world = CosmicWorld::new(n_stars + k_disks * DISK_PARTICLES);

    // Pass 1: todas las estrellas.
    for i in 0..n_stars {
        let mass = masses[i];
        let velocity = thermal_velocity(cluster.velocity, mass, seed.wrapping_add(i as u64));
        world.spawn(CosmicEntity {
            qe: mass,
            radius: mass.powf(1.0 / 3.0).max(1e-6),
            frequency_hz: freqs[i],
            phase: 0.0,
            position: positions[i],
            velocity,
            dissipation: DISSIPATION_PLASMA as f64,
            age_ticks: 0,
            entity_id: 0,
            alive: true,
        });
    }

    // Pass 2: discos protoplanetarios solo para hosts seleccionados.
    for &i in &disk_hosts {
        let host_mass = masses[i];
        let host_pos = positions[i];
        let host_vel = world.entities[i].velocity;
        spawn_disk(&mut world, host_pos, host_vel, host_mass, seed.wrapping_add(i as u64 ^ 0xBEEF));
    }

    world.total_qe_initial = world.total_qe();
    Some(StellarExpansion { world, n_stars })
}

fn thermal_velocity(parent_vel: [f64; 3], star_mass: f64, seed: u64) -> [f64; 3] {
    let sigma = 1.0 / star_mass.max(1.0).sqrt();
    let mut v = parent_vel;
    let mut rng = seed;
    for component in v.iter_mut() {
        rng = determinism::next_u64(rng);
        *component += determinism::gaussian_f32(rng, sigma as f32) as f64;
    }
    v
}

/// Inserta `DISK_PARTICLES` alrededor del host con momento angular coherente.
/// Inserts `DISK_PARTICLES` orbiting the host with coherent angular momentum.
fn spawn_disk(
    world: &mut CosmicWorld,
    host_pos: [f64; 3],
    host_vel: [f64; 3],
    host_mass: f64,
    seed: u64,
) {
    let g = cosmic_gravity::gravitational_constant();
    let bw = COHERENCE_BANDWIDTH as f64;
    // Partículas de gas/polvo: qe pequeña (~1% de la estrella).
    let disk_qe = host_mass * 0.01 / DISK_PARTICLES as f64;
    let inner_r = host_mass.powf(1.0 / 3.0) * 2.0;
    let mut rng = seed;
    for k in 0..DISK_PARTICLES {
        rng = determinism::next_u64(rng);
        let angle = k as f64 * std::f64::consts::TAU / DISK_PARTICLES as f64
            + determinism::unit_f32(rng) as f64 * 0.1;
        let radius = inner_r * (1.0 + k as f64 * 0.5);
        let offset = [radius * angle.cos(), radius * angle.sin(), 0.0];
        let speed = sd::keplerian_speed(g, host_mass, radius);
        let tangential = sd::tangential_velocity_xy(offset, speed);

        world.spawn(CosmicEntity {
            qe: disk_qe,
            radius: disk_qe.powf(1.0 / 3.0).max(1e-6),
            frequency_hz: bw, // gas frío — freq baja, independiente del host
            phase: 0.0,
            position: [host_pos[0] + offset[0], host_pos[1] + offset[1], host_pos[2] + offset[2]],
            velocity: [
                host_vel[0] + tangential[0],
                host_vel[1] + tangential[1],
                host_vel[2] + tangential[2],
            ],
            dissipation: DISSIPATION_PLASMA as f64,
            age_ticks: 0,
            entity_id: 0,
            alive: true,
        });
    }
}

/// Agrega un `CosmicWorld` estelar de vuelta a estado de cluster (para zoom-out).
/// Aggregates a stellar `CosmicWorld` back into cluster state (for zoom-out).
pub fn aggregate_stellar_to_cluster(world: &CosmicWorld) -> inf::AggregateState {
    let alive = world.entities.iter().filter(|e| e.alive);
    let qes: Vec<f64> = alive.clone().map(|e| e.qe).collect();
    let freqs: Vec<f64> = alive.clone().map(|e| e.frequency_hz).collect();
    let positions: Vec<[f64; 3]> = alive.map(|e| e.position).collect();
    inf::aggregate_to_parent(&qes, &freqs, &positions)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cluster(qe: f64) -> CosmicEntity {
        CosmicEntity {
            qe,
            radius: 50.0,
            frequency_hz: 40.0,
            phase: 0.0,
            position: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            dissipation: 0.25,
            age_ticks: 0,
            entity_id: 1,
            alive: true,
        }
    }

    #[test]
    fn expand_cluster_respects_kleiber_range() {
        let c = sample_cluster(10_000.0);
        let exp = expand_cluster(&c, 42, COHERENCE_BANDWIDTH as f64, 20, 100).unwrap();
        assert!(
            (20..=100).contains(&exp.n_stars),
            "star count {} out of Kleiber bounds",
            exp.n_stars,
        );
    }

    #[test]
    fn expand_cluster_preserves_pool_invariant() {
        let c = sample_cluster(5_000.0);
        let exp = expand_cluster(&c, 42, COHERENCE_BANDWIDTH as f64, 20, 100).unwrap();
        let total = exp.world.total_qe();
        assert!(total < c.qe, "total {total} >= parent {}", c.qe);
        // Most qe goes to stars (disk particles carry ~1% per host).
        assert!(total > c.qe * 0.5, "too much dissipation: {total} < 50%");
    }

    #[test]
    fn expand_cluster_deterministic() {
        let c = sample_cluster(3_000.0);
        let a = expand_cluster(&c, 7, COHERENCE_BANDWIDTH as f64, 20, 100).unwrap();
        let b = expand_cluster(&c, 7, COHERENCE_BANDWIDTH as f64, 20, 100).unwrap();
        assert_eq!(a.n_stars, b.n_stars);
        assert_eq!(a.world.entities.len(), b.world.entities.len());
        for (ea, eb) in a.world.entities.iter().zip(&b.world.entities) {
            assert_eq!(ea.qe, eb.qe);
            assert_eq!(ea.position, eb.position);
            assert_eq!(ea.velocity, eb.velocity);
        }
    }

    #[test]
    fn expand_cluster_zero_qe_returns_none() {
        let c = sample_cluster(0.0);
        assert!(expand_cluster(&c, 42, 50.0, 1, 100).is_none());
    }

    #[test]
    fn expand_cluster_creates_at_least_one_disk_when_many_stars() {
        let c = sample_cluster(50_000.0);
        let exp = expand_cluster(&c, 42, COHERENCE_BANDWIDTH as f64, 20, 200).unwrap();
        assert!(exp.n_stars > 10, "precondition: need >10 stars, got {}", exp.n_stars);
        let disk_count = exp.world.entities.len() - exp.n_stars;
        assert!(disk_count >= DISK_PARTICLES, "no disk particles spawned");
    }

    #[test]
    fn disk_particles_have_nonzero_angular_momentum() {
        let c = sample_cluster(20_000.0);
        let exp = expand_cluster(&c, 42, COHERENCE_BANDWIDTH as f64, 20, 100).unwrap();
        let mut any_l = false;
        // Disk particles start at index n_stars.
        for e in &exp.world.entities[exp.n_stars..] {
            let l = sd::angular_momentum(e.position, e.velocity, e.qe);
            let mag = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
            if mag > 0.0 { any_l = true; break; }
        }
        assert!(any_l, "no disk particle has angular momentum");
    }

    #[test]
    fn aggregate_matches_inf_contract() {
        let c = sample_cluster(4_000.0);
        let exp = expand_cluster(&c, 1, COHERENCE_BANDWIDTH as f64, 20, 100).unwrap();
        let agg = aggregate_stellar_to_cluster(&exp.world);
        assert!((agg.qe - exp.world.total_qe()).abs() < 1e-9);
    }
}
