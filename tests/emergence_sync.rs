//! Prueba de emergencia del Axioma 6 (Emergence at Scale) — integración headless.
//!
//! Corre el experimento de sincronización de Kuramoto y verifica que el order
//! parameter global emerge de la regla local (acoplado) y colapsa sin ella
//! (ablado). El caso ablado es el CONTROL NEGATIVO CAUSAL: misma población, misma
//! semilla, sólo se apaga la regla → sin orden global. Análogo al control negativo
//! de `tests/r2_determinism.rs`.
//!
//! cargo test --test emergence_sync

use resonance::blueprint::constants::{
    SYNC_ABLATED_R_MAX, SYNC_GAP_MIN, SYNC_R_PASS_MIN, SYNC_SUPERCRITICAL_RATIO,
};
use resonance::use_cases::experiments::emergence_sync::{run_sync_experiment, SyncConfig};

fn supercritical(seed: u64) -> SyncConfig {
    let freq_spread = 0.10;
    SyncConfig {
        seed,
        n_oscillators: 256,
        freq_spread,
        coupling: SYNC_SUPERCRITICAL_RATIO * freq_spread,
        ticks: 2000,
    }
}

/// Ax6-1: con la regla local activa, el orden global emerge (R alto).
#[test]
fn coupled_population_synchronizes() {
    let r = run_sync_experiment(&supercritical(11));
    assert!(
        r.r_final > SYNC_R_PASS_MIN,
        "orden global debe emerger con la regla local: R_final={} (umbral {})",
        r.r_final,
        SYNC_R_PASS_MIN
    );
}

/// Ax6-2 (CONTROL NEGATIVO): sin la regla local, no hay orden global.
#[test]
fn ablated_population_stays_disordered() {
    let r = run_sync_experiment(&supercritical(11));
    assert!(
        r.r_final_ablated < SYNC_ABLATED_R_MAX,
        "sin la regla local el orden global no aparece: R_ablated={} (umbral {})",
        r.r_final_ablated,
        SYNC_ABLATED_R_MAX
    );
}

/// Ax6-3: el gap acoplado-ablado es decisivo — la regla es causa suficiente.
#[test]
fn emergence_gap_proves_causality() {
    let r = run_sync_experiment(&supercritical(11));
    let gap = r.r_final - r.r_final_ablated;
    assert!(
        gap > SYNC_GAP_MIN,
        "el orden global es consecuencia de la regla N-1: gap={} (umbral {})",
        gap,
        SYNC_GAP_MIN
    );
    assert!(r.synchronized, "veredicto de emergencia debe ser PASS");
}

/// Ax6-4: reproducible a través de semillas (no es un artefacto de una seed).
#[test]
fn emergence_holds_across_seeds() {
    for seed in [1u64, 2, 3, 100, 777] {
        let r = run_sync_experiment(&supercritical(seed));
        assert!(
            r.synchronized,
            "seed {}: R_final={}, R_ablated={}",
            seed, r.r_final, r.r_final_ablated
        );
    }
}
