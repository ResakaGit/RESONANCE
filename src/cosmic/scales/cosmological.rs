//! Escala S0 — N-body gravitacional, Big Bang, formación de clusters.
//! Scale S0 — N-body gravitational, Big Bang, cluster formation.
//!
//! Transforma `CosmicWorld` tick a tick. Matemática pura en
//! `blueprint/equations/cosmic_gravity.rs` y `verlet.rs`.
//!
//! Axiomas:
//! - Ax 5 (Conservation): `total_qe` monotone decreasing via dissipation.
//! - Ax 7 (Distance Attenuation): InverseSquare.
//! - Ax 8 (Oscillatory): gravity modulada por alineamiento de frecuencia.

use crate::blueprint::equations::cosmic_gravity as grav;
use crate::blueprint::equations::derived_thresholds::{COHERENCE_BANDWIDTH, DISSIPATION_SOLID};
use crate::blueprint::equations::determinism;
use crate::blueprint::equations::verlet::{position_step_3d, velocity_step_3d};
use crate::cosmic::scale_manager::{CosmicEntity, CosmicWorld};

// ─── Configuration ──────────────────────────────────────────────────────────

/// Parámetros del universo S0. Derivables de fundamentales salvo N y seed.
/// S0 universe parameters. All derivable except count and seed.
#[derive(Clone, Copy, Debug)]
pub struct CosmoConfig {
    pub n_initial_clusters: usize,
    pub total_qe: f64,
    pub expansion_rate: f64,
    pub dissipation_rate: f64,
    pub dt: f64,
    pub seed: u64,
}

impl CosmoConfig {
    /// Config por defecto: N=128, total_qe=1e6, dt pequeño (escala cosmológica).
    /// Default: N=128, total_qe=1e6, small dt (cosmological steps are gentle).
    ///
    /// `dt=0.01` mantiene dissipation por paso ≈ 5e-5 (ratio 10^-4 vs Hubble),
    /// consistente con tiempo de relajación gravitacional >> tiempo de disipación.
    pub fn default_with_seed(seed: u64) -> Self {
        Self {
            n_initial_clusters: 128,
            total_qe: 1.0e6,
            expansion_rate: grav::expansion_rate_default(),
            dissipation_rate: DISSIPATION_SOLID as f64,
            dt: 0.01,
            seed,
        }
    }
}

// ─── Big Bang initialization ────────────────────────────────────────────────

/// Distribuye `total_qe` entre N partículas en una esfera pequeña con velocidades
/// radiales (expansión inicial). Frecuencias uniformes en `(0, 5·bandwidth]`.
/// Distributes `total_qe` over N particles in a small sphere with radial velocities.
pub fn init_big_bang(config: &CosmoConfig) -> CosmicWorld {
    let n = config.n_initial_clusters;
    let mut world = CosmicWorld::new(n);
    if n == 0 || config.total_qe <= 0.0 { return world; }

    let qe_per = config.total_qe / n as f64;
    let initial_radius = grav::plummer_softening() * 4.0;
    let bw = COHERENCE_BANDWIDTH as f64;
    let mut rng = config.seed.wrapping_add(0xA5A5_5A5A_A5A5_5A5A);

    for _ in 0..n {
        let (position, direction) = sample_point_on_sphere(&mut rng, initial_radius);
        rng = determinism::next_u64(rng);
        let speed = determinism::unit_f32(rng) as f64 * initial_radius * 0.5;
        rng = determinism::next_u64(rng);
        let freq = determinism::unit_f32(rng) as f64 * (5.0 * bw);

        let entity = CosmicEntity {
            qe: qe_per,
            radius: qe_per.powf(1.0 / 3.0).max(1e-6),
            frequency_hz: freq,
            phase: 0.0,
            position,
            velocity: [direction[0] * speed, direction[1] * speed, direction[2] * speed],
            dissipation: config.dissipation_rate,
            age_ticks: 0,
            entity_id: 0,
            alive: true,
        };
        world.spawn(entity);
    }
    world.total_qe_initial = world.total_qe();
    world
}

/// Muestrea un punto uniforme dentro de una esfera, con vector unitario radial.
/// Uniformly samples a point inside a sphere plus its unit radial vector.
fn sample_point_on_sphere(rng: &mut u64, max_radius: f64) -> ([f64; 3], [f64; 3]) {
    for _ in 0..64 {
        let mut p = [0.0_f64; 3];
        let mut r2 = 0.0;
        for component in p.iter_mut() {
            *rng = determinism::next_u64(*rng);
            *component = (determinism::unit_f32(*rng) as f64 * 2.0 - 1.0) * max_radius;
            r2 += *component * *component;
        }
        if r2 <= max_radius * max_radius && r2 > 0.0 {
            let r = r2.sqrt();
            let dir = [p[0] / r, p[1] / r, p[2] / r];
            return (p, dir);
        }
    }
    ([max_radius * 0.5, 0.0, 0.0], [1.0, 0.0, 0.0])
}

// ─── Tick ───────────────────────────────────────────────────────────────────

/// Un paso cosmológico: gravedad + expansión + integración Verlet + dissipation.
/// One cosmological step: gravity + expansion + Verlet + dissipation.
///
/// Protocolo Velocity-Verlet de un paso sin cachear aceleración antigua
/// (simplificación: Euler-symplectic variante). Para producción se puede
/// promover a Verlet completo manteniendo acc_prev en un Vec<[f64;3]>.
pub fn cosmo_tick(world: &mut CosmicWorld, config: &CosmoConfig) {
    let alive_indices: Vec<usize> = world
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| e.alive)
        .map(|(i, _)| i)
        .collect();
    let n = alive_indices.len();
    if n < 2 { return; }

    let positions: Vec<[f64; 3]> = alive_indices.iter().map(|&i| world.entities[i].position).collect();
    let masses: Vec<f64> = alive_indices.iter().map(|&i| world.entities[i].qe).collect();
    let freqs: Vec<f64> = alive_indices.iter().map(|&i| world.entities[i].frequency_hz).collect();

    let acc = grav::gravity_accelerations(&positions, &masses, &freqs);

    let dt = config.dt;
    let zero = [0.0_f64; 3];
    for (k, &idx) in alive_indices.iter().enumerate() {
        let e = &mut world.entities[idx];
        let new_pos = position_step_3d(e.position, e.velocity, acc[k], dt);
        let mut new_vel = velocity_step_3d(e.velocity, zero, acc[k], dt);
        grav::apply_expansion(&mut new_vel, e.position, config.expansion_rate, dt);

        e.position = new_pos;
        e.velocity = new_vel;
        e.qe = grav::apply_dissipation_qe(e.qe, config.dissipation_rate, dt);
        e.age_ticks = e.age_ticks.saturating_add(1);
        if e.qe <= 0.0 { e.alive = false; }
    }
    world.tick_id = world.tick_id.saturating_add(1);
}

// ─── Cluster detection ──────────────────────────────────────────────────────

/// Un cluster detectado por union-find sobre vecindad espacial.
/// A cluster detected via union-find over spatial neighborhood.
#[derive(Clone, Debug, Default)]
pub struct ClusterStats {
    pub n_members: usize,
    pub total_qe: f64,
    pub centroid: [f64; 3],
    pub freq_variance: f64,
}

/// Detecta clusters agrupando entidades vivas cuyos centros estén a distancia
/// menor que `link_distance`. Retorna lista de estadísticas por cluster.
/// Detects clusters by grouping live entities within `link_distance` of each other.
pub fn detect_clusters(world: &CosmicWorld, link_distance: f64) -> Vec<ClusterStats> {
    let idx: Vec<usize> = world
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| e.alive)
        .map(|(i, _)| i)
        .collect();
    let n = idx.len();
    if n == 0 { return Vec::new(); }

    let mut parent: Vec<usize> = (0..n).collect();

    fn find(p: &mut [usize], x: usize) -> usize {
        let mut r = x;
        while p[r] != r { r = p[r]; }
        let mut cur = x;
        while p[cur] != r {
            let next = p[cur];
            p[cur] = r;
            cur = next;
        }
        r
    }
    fn union(p: &mut [usize], a: usize, b: usize) {
        let ra = find(p, a);
        let rb = find(p, b);
        if ra != rb { p[ra] = rb; }
    }

    let link2 = link_distance * link_distance;
    for i in 0..n {
        for j in (i + 1)..n {
            let pi = world.entities[idx[i]].position;
            let pj = world.entities[idx[j]].position;
            let dx = pi[0] - pj[0];
            let dy = pi[1] - pj[1];
            let dz = pi[2] - pj[2];
            if dx * dx + dy * dy + dz * dz <= link2 {
                union(&mut parent, i, j);
            }
        }
    }

    let mut groups: std::collections::BTreeMap<usize, Vec<usize>> = std::collections::BTreeMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(idx[i]);
    }

    groups
        .into_values()
        .map(|members| stats_for(world, &members))
        .collect()
}

fn stats_for(world: &CosmicWorld, members: &[usize]) -> ClusterStats {
    let total_qe: f64 = members.iter().map(|&i| world.entities[i].qe).sum();
    let mut centroid = [0.0_f64; 3];
    if total_qe > 0.0 {
        for &i in members {
            let e = &world.entities[i];
            for d in 0..3 { centroid[d] += e.qe * e.position[d]; }
        }
        for d in 0..3 { centroid[d] /= total_qe; }
    }
    let mean_freq: f64 = if members.is_empty() {
        0.0
    } else {
        members.iter().map(|&i| world.entities[i].frequency_hz).sum::<f64>() / members.len() as f64
    };
    let freq_variance = if members.is_empty() {
        0.0
    } else {
        members
            .iter()
            .map(|&i| (world.entities[i].frequency_hz - mean_freq).powi(2))
            .sum::<f64>()
            / members.len() as f64
    };
    ClusterStats {
        n_members: members.len(),
        total_qe,
        centroid,
        freq_variance,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn big_bang_allocates_n_entities() {
        let cfg = CosmoConfig::default_with_seed(42);
        let world = init_big_bang(&cfg);
        assert_eq!(world.n_alive(), cfg.n_initial_clusters);
    }

    #[test]
    fn big_bang_conserves_qe_at_init() {
        let cfg = CosmoConfig::default_with_seed(7);
        let world = init_big_bang(&cfg);
        let total = world.total_qe();
        assert!(
            (total - cfg.total_qe).abs() / cfg.total_qe < 1e-9,
            "total {total} vs expected {}",
            cfg.total_qe,
        );
    }

    #[test]
    fn big_bang_deterministic_with_seed() {
        let a = init_big_bang(&CosmoConfig::default_with_seed(11));
        let b = init_big_bang(&CosmoConfig::default_with_seed(11));
        for (ea, eb) in a.entities.iter().zip(&b.entities) {
            assert_eq!(ea.position, eb.position);
            assert_eq!(ea.velocity, eb.velocity);
            assert_eq!(ea.frequency_hz, eb.frequency_hz);
        }
    }

    #[test]
    fn tick_conserves_qe_monotone_decreasing() {
        let cfg = CosmoConfig::default_with_seed(3);
        let mut world = init_big_bang(&cfg);
        let mut prev = world.total_qe();
        for _ in 0..50 {
            cosmo_tick(&mut world, &cfg);
            let now = world.total_qe();
            assert!(now <= prev + 1e-9, "qe grew: {prev} -> {now}");
            prev = now;
        }
    }

    #[test]
    fn tick_advances_tick_id() {
        let cfg = CosmoConfig::default_with_seed(5);
        let mut world = init_big_bang(&cfg);
        let before = world.tick_id;
        cosmo_tick(&mut world, &cfg);
        assert_eq!(world.tick_id, before + 1);
    }

    #[test]
    fn expansion_increases_mean_distance() {
        let mut cfg = CosmoConfig::default_with_seed(9);
        cfg.expansion_rate = 1.0; // fuerte para test rápido
        cfg.dissipation_rate = 0.0; // aislar el efecto
        let mut world = init_big_bang(&cfg);
        let mean0 = mean_distance_from_origin(&world);
        for _ in 0..20 { cosmo_tick(&mut world, &cfg); }
        let mean1 = mean_distance_from_origin(&world);
        assert!(mean1 > mean0, "expansion failed to spread: {mean0} -> {mean1}");
    }

    fn mean_distance_from_origin(w: &CosmicWorld) -> f64 {
        let alive: Vec<_> = w.entities.iter().filter(|e| e.alive).collect();
        if alive.is_empty() { return 0.0; }
        alive
            .iter()
            .map(|e| (e.position[0].powi(2) + e.position[1].powi(2) + e.position[2].powi(2)).sqrt())
            .sum::<f64>()
            / alive.len() as f64
    }

    #[test]
    fn detect_clusters_trivial_single_group() {
        let cfg = CosmoConfig {
            n_initial_clusters: 10,
            total_qe: 100.0,
            expansion_rate: 0.0,
            dissipation_rate: 0.0,
            dt: 0.0,
            seed: 1,
        };
        let world = init_big_bang(&cfg);
        // link_distance grande: todos en un solo cluster
        let clusters = detect_clusters(&world, 1.0e9);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].n_members, 10);
    }

    #[test]
    fn detect_clusters_isolates_when_link_zero() {
        let cfg = CosmoConfig {
            n_initial_clusters: 5,
            total_qe: 50.0,
            expansion_rate: 0.0,
            dissipation_rate: 0.0,
            dt: 0.0,
            seed: 1,
        };
        let world = init_big_bang(&cfg);
        let clusters = detect_clusters(&world, 0.0);
        assert_eq!(clusters.len(), 5);
    }

    #[test]
    fn big_bang_zero_qe_yields_empty_world() {
        let cfg = CosmoConfig {
            n_initial_clusters: 10,
            total_qe: 0.0,
            expansion_rate: 0.0,
            dissipation_rate: 0.0,
            dt: 0.0,
            seed: 1,
        };
        let world = init_big_bang(&cfg);
        assert_eq!(world.n_alive(), 0);
    }
}
