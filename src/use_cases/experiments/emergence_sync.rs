//! Experimento EM-1.4 — Prueba de emergencia del Axioma 6 vía sincronización.
//!
//! Corre N osciladores de fase acoplados (mean-field de Kuramoto) desde un
//! arranque desordenado y mide el order parameter `R(t)`. Ejecuta DOS condiciones
//! sobre el **mismo estado inicial**: la regla local activa (`coupling = K`) y la
//! regla ablada (`coupling = 0`). La emergencia queda probada si
//! `r_final ≫ r_final_ablated` — el orden global es consecuencia de la regla local
//! sobre N-1 vecinos, no de programación top-down (Axioma 6). Ver ADR-046.
//!
//! Config → Report, funciones puras, cero Bevy, determinista (PCG interno).

use serde::{Deserialize, Serialize};

use crate::blueprint::constants::{SYNC_DT, SYNC_SUPERCRITICAL_RATIO, SYNC_TAIL_WINDOW_FRAC};
use crate::blueprint::equations::determinism::{gaussian_f32, next_u64, unit_f32};
use crate::blueprint::equations::emergence::synchronization::{
    emergence_sync_verdict, kuramoto_meanfield_phase_step, kuramoto_order_parameter_complex,
};

use core::f32::consts::TAU;

/// Configuración del experimento de sincronización.
#[derive(Clone, Debug)]
pub struct SyncConfig {
    /// Semilla determinista (PCG interno).
    pub seed: u64,
    /// Número de osciladores (clamp `[1, 4096]`).
    pub n_oscillators: usize,
    /// Desviación estándar de la distribución de frecuencias `ω` (rompe simetría).
    pub freq_spread: f32,
    /// Acoplamiento `K` de la condición ACOPLADA (la ablada usa `0`).
    pub coupling: f32,
    /// Cantidad de pasos temporales.
    pub ticks: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        let freq_spread = 0.10;
        Self {
            seed: 0,
            n_oscillators: 256,
            freq_spread,
            coupling: SYNC_SUPERCRITICAL_RATIO * freq_spread,
            ticks: 2000,
        }
    }
}

/// Reporte serializable del experimento.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SyncReport {
    pub seed: u64,
    pub n_oscillators: u32,
    pub n_ticks: u64,
    pub coupling: f32,
    pub freq_spread: f32,
    /// Serie `R(t)` de la condición acoplada.
    pub r_series: Vec<f32>,
    /// Serie `R(t)` de la condición ablada (`coupling = 0`).
    pub r_series_ablated: Vec<f32>,
    /// Media de la cola de `r_series` (régimen estacionario).
    pub r_final: f32,
    /// Media de la cola de `r_series_ablated`.
    pub r_final_ablated: f32,
    /// `true` si `r_final` sincroniza y `r_final_ablated` permanece bajo (Ax6 PASS).
    pub synchronized: bool,
}

impl SyncReport {
    /// Serializa a JSON (patrón de `SoupReport::to_json`).
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

/// Estado inicial determinista: fases uniformes en `[0, 2π)`, `ω ~ N(0, spread)`.
/// El marco es rotante (frecuencia base irrelevante para `R`), por eso `ω` centra en 0.
///
/// El stream avanza **tres** estados por oscilador: uno para la fase y **dos** para
/// la gaussiana. `gaussian_f32` consume dos estados internos (Box-Muller: `u1 = state`,
/// `u2 = next_u64(state)`); sin el avance extra el `u2` del draw `k` sería el `u1` de
/// la fase `k+1` → correlación intra-stream ω↔θ. Ver ADR-047 §2.2.
fn init_state(seed: u64, n: usize, spread: f32) -> (Vec<f32>, Vec<f32>) {
    let mut state = next_u64(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut next = || {
        state = next_u64(state);
        state
    };
    let mut phases = Vec::with_capacity(n);
    let mut omegas = Vec::with_capacity(n);
    for _ in 0..n {
        phases.push(unit_f32(next()) * TAU);
        omegas.push(gaussian_f32(next(), spread));
        let _ = next();
    }
    (phases, omegas)
}

/// Media de la cola final (`SYNC_TAIL_WINDOW_FRAC`) de una serie.
fn tail_mean(series: &[f32]) -> f32 {
    if series.is_empty() {
        return 0.0;
    }
    let tail = ((series.len() as f32) * SYNC_TAIL_WINDOW_FRAC).ceil() as usize;
    let tail = tail.clamp(1, series.len());
    let start = series.len() - tail;
    series[start..].iter().sum::<f32>() / tail as f32
}

/// Integra `ticks` pasos del mean-field y devuelve `(serie R(t), r_final)`.
fn simulate(phases0: &[f32], omegas: &[f32], coupling: f32, ticks: u64) -> (Vec<f32>, f32) {
    let mut cur = phases0.to_vec();
    let mut nxt = cur.clone();
    let mut r_series = Vec::with_capacity(ticks as usize);
    for _ in 0..ticks {
        let (r, psi) = kuramoto_order_parameter_complex(&cur);
        r_series.push(r);
        for i in 0..cur.len() {
            nxt[i] = kuramoto_meanfield_phase_step(cur[i], omegas[i], r, psi, coupling, SYNC_DT);
        }
        std::mem::swap(&mut cur, &mut nxt);
    }
    let r_final = tail_mean(&r_series);
    (r_series, r_final)
}

/// Corre el experimento completo: condición acoplada + ablación causal (`K = 0`)
/// sobre el mismo estado inicial. Pura y determinista.
pub fn run_sync_experiment(config: &SyncConfig) -> SyncReport {
    let n = config.n_oscillators.clamp(1, 4096);
    let ticks = config.ticks.max(1);
    let (phases0, omegas) = init_state(config.seed, n, config.freq_spread);

    let (r_series, r_final) = simulate(&phases0, &omegas, config.coupling, ticks);
    let (r_series_ablated, r_final_ablated) = simulate(&phases0, &omegas, 0.0, ticks);

    SyncReport {
        seed: config.seed,
        n_oscillators: n as u32,
        n_ticks: ticks,
        coupling: config.coupling,
        freq_spread: config.freq_spread,
        r_series,
        r_series_ablated,
        r_final,
        r_final_ablated,
        synchronized: emergence_sync_verdict(r_final, r_final_ablated),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fast_config(seed: u64) -> SyncConfig {
        // N y ticks reducidos para tests rápidos, manteniendo el régimen supercrítico.
        let freq_spread = 0.10;
        SyncConfig {
            seed,
            n_oscillators: 128,
            freq_spread,
            coupling: SYNC_SUPERCRITICAL_RATIO * freq_spread,
            ticks: 1500,
        }
    }

    #[test]
    fn coupled_condition_synchronizes() {
        let r = run_sync_experiment(&fast_config(1));
        assert!(r.r_final > 0.8, "acoplado supercrítico debe sincronizar, R={}", r.r_final);
    }

    #[test]
    fn ablated_condition_stays_incoherent() {
        let r = run_sync_experiment(&fast_config(1));
        assert!(
            r.r_final_ablated < 0.3,
            "sin la regla local no hay orden global, R_ablated={}",
            r.r_final_ablated
        );
    }

    #[test]
    fn emergence_gap_is_decisive() {
        let r = run_sync_experiment(&fast_config(7));
        assert!(r.synchronized, "gap acoplado-ablado debe cruzar el umbral de veredicto");
        assert!(r.r_final - r.r_final_ablated > 0.4);
    }

    #[test]
    fn subcritical_coupling_does_not_synchronize() {
        // Control: acoplamiento muy por debajo del crítico (K_c ≈ 1.6·spread ≈ 0.16)
        // no produce orden global → sin emergencia. Confirma que la sincronización
        // exige el régimen supercrítico, no cualquier acoplamiento no nulo.
        let cfg = SyncConfig { coupling: 0.02, ..fast_config(3) };
        let r = run_sync_experiment(&cfg);
        assert!(
            !r.synchronized,
            "subcrítico no debe sincronizar, R_final={}",
            r.r_final
        );
    }

    #[test]
    fn single_oscillator_never_synchronizes_as_emergence() {
        let cfg = SyncConfig { n_oscillators: 1, ..fast_config(2) };
        let r = run_sync_experiment(&cfg);
        // Un oscilador es R=1 en ambas condiciones → sin gap → no es emergencia.
        assert!(!r.synchronized);
    }

    #[test]
    fn is_deterministic() {
        let a = run_sync_experiment(&fast_config(42));
        let b = run_sync_experiment(&fast_config(42));
        assert_eq!(a.to_json().unwrap(), b.to_json().unwrap());
    }

    #[test]
    fn series_length_matches_ticks() {
        let cfg = fast_config(5);
        let r = run_sync_experiment(&cfg);
        assert_eq!(r.r_series.len() as u64, cfg.ticks);
        assert_eq!(r.r_series_ablated.len() as u64, cfg.ticks);
    }
}
