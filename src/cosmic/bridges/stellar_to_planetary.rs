//! Bridge S1 → S2 — estrella + disco expanden en sistema planetario.
//! Bridge S1 → S2 — star + disk expand into planetary system.
//!
//! CT-5 / ADR-036 §D4. Produce `PlanetSpec` con órbita, temperatura y estado
//! de materia emergentes. La zona habitable emerge de T ∈ [T_solid, T_liquid].

use crate::blueprint::domain_enums::MatterState;
use crate::blueprint::equations::derived_thresholds::DISSIPATION_LIQUID;
use crate::blueprint::equations::determinism;
use crate::blueprint::equations::planetary_system as ps;
use crate::blueprint::equations::scale_inference as inf;

use crate::cosmic::scale_manager::CosmicEntity;

/// Espec. de un planeta inferido de una estrella.
/// Inferred planet spec from a parent star.
#[derive(Clone, Debug)]
pub struct PlanetSpec {
    pub qe: f64,
    pub frequency_hz: f64,
    pub orbital_radius: f64,
    pub temperature: f64,
    pub matter_state: MatterState,
}

/// Expande una estrella con disco en un vector de planetas.
/// Expands a star with disk into a planet vector.
///
/// Retorna `Vec::new()` si `disk_qe <= 0` o no hay espacio para al menos 1 planeta.
/// `disk_qe` es la qe disponible en el disco protoplanetario (budget).
pub fn expand_stellar_system(
    star: &CosmicEntity,
    disk_qe: f64,
    seed: u64,
    bandwidth: f64,
    min_planets: usize,
    max_planets: usize,
) -> Vec<PlanetSpec> {
    if disk_qe <= 0.0 { return Vec::new(); }

    // Kleiber: N ∝ disk_qe^0.75. scale_factor=0.5 (CT-1 ZoomConfig::Planetary).
    let n = inf::kleiber_child_count(disk_qe, 0.5, min_planets.max(1), max_planets);
    if n == 0 { return Vec::new(); }

    // Radio orbital interno: proporcional al radio de la estrella. Evita colisión.
    let r_inner = star.radius.max(1e-3) * 2.0;
    let radii = ps::titius_bode_radii(n, r_inner);

    // qe por planeta: Pool Invariant con decaimiento 1/r² (Ax 7).
    // Asignar pesos ∝ 1/r² y normalizar para que sum(qe) = disk_qe·(1-dissipation).
    let budget = disk_qe * (1.0 - DISSIPATION_LIQUID as f64);
    let weights: Vec<f64> = radii.iter().map(|r| 1.0 / (r * r)).collect();
    let weight_sum: f64 = weights.iter().sum();
    let freqs = inf::distribute_frequencies(star.frequency_hz, n, bandwidth, seed);

    let mut rng = seed;
    (0..n)
        .map(|i| {
            rng = determinism::next_u64(rng);
            let qe = budget * weights[i] / weight_sum.max(1e-18);
            let temperature = ps::planet_temperature(star.qe, radii[i]);
            let matter_state = ps::matter_state_from_temperature(temperature);
            PlanetSpec {
                qe,
                frequency_hz: freqs[i],
                orbital_radius: radii[i],
                temperature,
                matter_state,
            }
        })
        .collect()
}

/// Agrega planetas de vuelta a un estado-estrella (qe, freq dominante, órbita media).
/// Aggregates planets back into star-level observables.
pub fn aggregate_planetary_to_star(planets: &[PlanetSpec]) -> inf::AggregateState {
    let qes: Vec<f64> = planets.iter().map(|p| p.qe).collect();
    let freqs: Vec<f64> = planets.iter().map(|p| p.frequency_hz).collect();
    let positions: Vec<[f64; 3]> = planets.iter().map(|p| [p.orbital_radius, 0.0, 0.0]).collect();
    inf::aggregate_to_parent(&qes, &freqs, &positions)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::equations::derived_thresholds::COHERENCE_BANDWIDTH;

    fn sample_star(qe: f64) -> CosmicEntity {
        CosmicEntity {
            qe,
            radius: qe.powf(1.0 / 3.0).max(1.0),
            frequency_hz: 80.0,
            phase: 0.0,
            position: [0.0; 3],
            velocity: [0.0; 3],
            dissipation: 0.25,
            age_ticks: 0,
            entity_id: 42,
            alive: true,
        }
    }

    #[test]
    fn planet_count_respects_kleiber_range() {
        let star = sample_star(100.0);
        let planets = expand_stellar_system(&star, 10.0, 7, COHERENCE_BANDWIDTH as f64, 3, 12);
        assert!((3..=12).contains(&planets.len()));
    }

    #[test]
    fn pool_invariant_on_planet_budget() {
        let star = sample_star(100.0);
        let disk_qe = 20.0;
        let planets = expand_stellar_system(&star, disk_qe, 7, 50.0, 3, 12);
        let sum: f64 = planets.iter().map(|p| p.qe).sum();
        assert!(sum < disk_qe, "sum {sum} >= disk {disk_qe}");
        // Budget reserves `(1 - DISSIPATION_LIQUID)` of disk_qe for the planets.
        let expected = disk_qe * (1.0 - DISSIPATION_LIQUID as f64);
        assert!((sum - expected).abs() < 1e-6);
    }

    #[test]
    fn orbital_radii_geometric() {
        let star = sample_star(500.0);
        let planets = expand_stellar_system(&star, 50.0, 3, 50.0, 4, 12);
        if planets.len() < 2 { return; }
        for i in 0..planets.len() - 1 {
            let ratio = planets[i + 1].orbital_radius / planets[i].orbital_radius;
            assert!((ratio - ps::TITIUS_BODE_RATIO).abs() < 1e-6);
        }
    }

    #[test]
    fn inner_planets_hotter_than_outer() {
        let star = sample_star(2000.0);
        let planets = expand_stellar_system(&star, 200.0, 1, 50.0, 5, 12);
        if planets.len() < 2 { return; }
        for i in 0..planets.len() - 1 {
            assert!(
                planets[i].temperature >= planets[i + 1].temperature,
                "T[{i}]={} not ≥ T[{}]={}",
                planets[i].temperature, i + 1, planets[i + 1].temperature,
            );
        }
    }

    #[test]
    fn some_planet_in_habitable_zone_for_reasonable_star() {
        // Star qe tuned so Titius-Bode radii sweep through habitable zone.
        let star = sample_star(50.0);
        let planets = expand_stellar_system(&star, 50.0, 1, 50.0, 3, 20);
        let any_liquid = planets.iter().any(|p| p.matter_state == MatterState::Liquid);
        assert!(any_liquid, "no liquid planet in {} planets", planets.len());
    }

    #[test]
    fn qe_weights_follow_inverse_square() {
        let star = sample_star(100.0);
        let planets = expand_stellar_system(&star, 50.0, 7, 50.0, 4, 12);
        if planets.len() < 2 { return; }
        for i in 0..planets.len() - 1 {
            assert!(
                planets[i].qe > planets[i + 1].qe,
                "inner planet qe should exceed outer (Ax 7 decay)",
            );
        }
    }

    #[test]
    fn expand_empty_disk_returns_empty() {
        let star = sample_star(100.0);
        assert!(expand_stellar_system(&star, 0.0, 1, 50.0, 3, 12).is_empty());
    }

    #[test]
    fn expand_deterministic_with_seed() {
        let star = sample_star(200.0);
        let a = expand_stellar_system(&star, 30.0, 11, 50.0, 3, 12);
        let b = expand_stellar_system(&star, 30.0, 11, 50.0, 3, 12);
        assert_eq!(a.len(), b.len());
        for (pa, pb) in a.iter().zip(&b) {
            assert_eq!(pa.qe, pb.qe);
            assert_eq!(pa.orbital_radius, pb.orbital_radius);
            assert_eq!(pa.frequency_hz, pb.frequency_hz);
        }
    }

    #[test]
    fn aggregate_preserves_qe_sum() {
        let star = sample_star(300.0);
        let planets = expand_stellar_system(&star, 60.0, 5, 50.0, 4, 12);
        let agg = aggregate_planetary_to_star(&planets);
        let direct: f64 = planets.iter().map(|p| p.qe).sum();
        assert!((agg.qe - direct).abs() < 1e-9);
    }
}
