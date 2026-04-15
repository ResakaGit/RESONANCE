//! AP-5: Property test — operacionalización del cap. 10 del paper.
//! AP-5: Persistence property test — paper §10 operationalization.
//!
//! > "Lo que persiste es aquello que encontró una forma de copiarse antes de disiparse."
//!
//! Tres bloques de aserciones sobre sopas aleatorias generadas por `run_soup`:
//!
//! 1. **Determinismo** — misma seed ⇒ mismo reporte (base para reproducibilidad).
//! 2. **Invariantes numéricos** — fates finitos, acotados, no NaN.
//! 3. **Contrato de persistencia** — toda closure superviviente satisface
//!    `k_stability_mean_last ≥ 1.0` OR `pressure_events ≥ 1` — es decir,
//!    encontró sostenerse o replicar antes de disiparse.
//!
//! **Tuning:** `cases=32` por defecto (PR, <60 s).  Sobre-escribir con
//! `PROPTEST_CASES=1000` en CI nightly para validación extensiva.

use proptest::prelude::*;
use resonance::use_cases::experiments::autopoiesis::{SoupConfig, run_soup};

/// Config rápida para proptest — grid pequeño, pocos ticks.
fn fast_soup(seed: u64) -> SoupConfig {
    SoupConfig {
        seed,
        n_species: 6,
        n_reactions: 12,
        food_size: 2,
        grid: (6, 6),
        ticks: 400,
        equilibration_ticks: 50,
        detection_every: 50,
        last_window_ticks: 200,
        initial_food_qe: 2.0,
        dt: 0.1,
        food_spot_radius: None,
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        .. ProptestConfig::default()
    })]

    /// Determinismo: misma seed ⇒ mismo reporte (invariante base).
    #[test]
    fn run_soup_is_deterministic_for_any_seed(seed in 0u64..100) {
        let a = run_soup(&fast_soup(seed));
        let b = run_soup(&fast_soup(seed));
        prop_assert_eq!(&a.fates, &b.fates);
        prop_assert_eq!(a.n_closures_initial, b.n_closures_initial);
        prop_assert!((a.total_dissipated - b.total_dissipated).abs() < 1e-3);
    }

    /// Invariantes numéricos sobre `SoupReport`.  Toda seed debe producir
    /// valores finitos, no-negativos y con `pressure_events` acotado.
    #[test]
    fn report_fates_are_bounded_and_finite(seed in 0u64..100) {
        let c = fast_soup(seed);
        let r = run_soup(&c);
        prop_assert!(r.total_dissipated.is_finite());
        prop_assert!(r.total_dissipated >= 0.0);
        let max_events = (c.ticks / c.detection_every) as u32;
        for f in &r.fates {
            prop_assert!(f.k_stability_mean_last.is_finite(),
                "k={} for hash {:#x}", f.k_stability_mean_last, f.hash);
            prop_assert!(f.k_stability_mean_last >= 0.0);
            prop_assert!(f.pressure_events <= max_events);
        }
    }

    /// **Contrato AP-5** — cap. 10 del paper.  Toda closure superviviente
    /// debe demostrar al menos una de las dos formas de persistencia:
    ///   - `k_stability_mean_last ≥ 1.0` — reconstrucción ≥ decay (Pross).
    ///   - `pressure_events ≥ 1` — cruzó el umbral de fisión (proxy de réplica).
    ///
    /// Si esto falla para alguna seed, el simulador no operacionaliza el
    /// invariante: una closure sobrevive sin evidencia termodinámica de
    /// cómo lo logra — bug en AP-2 (k_stab) o AP-4 (pressure).
    #[test]
    fn surviving_closures_satisfy_persistence_contract(seed in 0u64..64) {
        let r = run_soup(&fast_soup(seed));
        for f in &r.fates {
            if f.survived {
                prop_assert!(
                    f.k_stability_mean_last >= 1.0 || f.pressure_events >= 1,
                    "surviving closure {:#x} lacks persistence evidence: \
                     k_mean={:.4} pressure_events={}",
                    f.hash, f.k_stability_mean_last, f.pressure_events,
                );
            }
        }
    }
}
