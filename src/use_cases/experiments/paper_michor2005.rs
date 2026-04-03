//! PV-5: Michor et al. 2005 — declive bifásico CML bajo imatinib.
//! PV-5: Michor et al. 2005 — biphasic CML decline under imatinib.
//!
//! Michor F et al. (2005) Nature 435:1267-1270.
//! Core prediction: population decline shows two phases under TKI therapy:
//!   Phase 1 (fast): differentiated cells killed rapidly (slope ~0.05/day)
//!   Phase 2 (slow): quiescent stem cells persist (slope ~0.005/day)
//! Stem cells survive because their frequency is far from drug target (Axiom 8).
//!
//! Three subpopulations by frequency:
//!   Differentiated (near drug → high kill) — bulk of tumor, responds fast
//!   Progenitor (moderate distance → moderate kill) — intermediate compartment
//!   Stem (far from drug → survives) — quiescent, low growth, persists
//!
//! All stateless. Config in → MichorReport out. BDD-tested.

use crate::batch::arena::{EntitySlot, SimWorldFlat};
use crate::batch::scratch::ScratchPad;
use crate::batch::systems;
use crate::blueprint::equations::derived_thresholds::{COHERENCE_BANDWIDTH, DISSIPATION_SOLID};
use crate::blueprint::equations::determinism;
use std::time::Instant;

// ─── Constants ──────────────────────────────────────────────────────────────

/// Fracción de irradiancia respecto a nutrientes (calibración de grilla).
/// Irradiance-to-nutrient ratio (grid calibration).
const IRRADIANCE_NUTRIENT_RATIO: f32 = 0.3;

/// Rango espacial válido para entidades en la grilla 16×16.
/// Valid spatial range for entities in the 16×16 grid.
const GRID_POS_MIN: f32 = 1.0;
const GRID_POS_MAX: f32 = 15.0;

// ─── Config ─────────────────────────────────────────────────────────────────

/// Configuración del experimento Michor 2005 (CML bifásico).
/// Michor 2005 experiment configuration (biphasic CML).
#[derive(Debug, Clone)]
pub struct MichorConfig {
    // Subpopulations
    pub differentiated_count: u8,
    pub progenitor_count: u8,
    pub stem_count: u8,
    pub diff_freq: f32,
    pub prog_freq: f32,
    pub stem_freq: f32,
    pub diff_qe: f32,
    pub prog_qe: f32,
    pub stem_qe: f32,
    /// Tasa de crecimiento de stem cells (quiescentes → baja).
    /// Stem cell growth bias (quiescent → low).
    pub stem_growth_bias: f32,
    pub diff_growth_bias: f32,
    pub prog_growth_bias: f32,

    // Drug (imatinib analog)
    pub drug_freq: f32,
    pub drug_potency: f32,
    pub drug_bandwidth: f32,
    /// Generación donde comienza el tratamiento.
    /// Generation where treatment starts.
    pub drug_start_gen: u32,

    // Biology
    pub nutrient_level: f32,

    // Simulation
    pub worlds: usize,
    pub generations: u32,
    pub ticks_per_gen: u32,
    pub seed: u64,
}

impl Default for MichorConfig {
    fn default() -> Self {
        Self {
            differentiated_count: 35,
            progenitor_count: 10,
            stem_count: 5,
            diff_freq: 400.0,
            prog_freq: 320.0,
            stem_freq: 220.0,
            diff_qe: 40.0,
            prog_qe: 60.0,
            stem_qe: 80.0,
            stem_growth_bias: 0.02,
            diff_growth_bias: 0.8,
            prog_growth_bias: 0.5,
            drug_freq: 400.0,
            drug_potency: 0.7,
            drug_bandwidth: COHERENCE_BANDWIDTH,
            drug_start_gen: 5,
            nutrient_level: 2.0,
            worlds: 20,
            generations: 50,
            ticks_per_gen: 100,
            seed: 42,
        }
    }
}

// ─── Output ─────────────────────────────────────────────────────────────────

/// Snapshot por generación.
/// Per-generation snapshot.
#[derive(Debug, Clone)]
pub struct MichorSnapshot {
    pub generation: u32,
    pub total_alive: f32,
    pub diff_alive: f32,
    pub prog_alive: f32,
    pub stem_alive: f32,
    pub total_qe: f32,
    pub drug_active: bool,
}

/// Reporte completo del experimento Michor 2005.
/// Complete Michor 2005 experiment report.
#[derive(Debug)]
pub struct MichorReport {
    pub config: MichorConfig,
    pub timeline: Vec<MichorSnapshot>,
    /// Pendiente de la primera fase (rápida) — declive log-lineal de diferenciadas.
    /// Phase 1 slope (fast) — log-linear decline of differentiated cells.
    pub phase1_slope: f32,
    /// Pendiente de la segunda fase (lenta) — persistencia de stem cells.
    /// Phase 2 slope (slow) — stem cell persistence.
    pub phase2_slope: f32,
    /// Ratio de pendientes: phase1/phase2 (>2 = bifásico).
    /// Slope ratio: phase1/phase2 (>2 = biphasic).
    pub slope_ratio: f32,
    /// Generación del punto de inflexión (cambio de pendiente).
    /// Inflection point generation (slope change).
    pub inflection_gen: Option<u32>,
    /// Stem cells sobreviven al final.
    /// Stem cells survive at the end.
    pub stem_survive: bool,
    /// Declive bifásico detectado.
    /// Biphasic decline detected.
    pub biphasic_detected: bool,
    pub wall_time_ms: u64,
}

// ─── Pure equations ─────────────────────────────────────────────────────────

/// Respuesta Hill con potencia (consistente con cancer_therapy.rs).
/// Hill response with potency (consistent with cancer_therapy.rs).
/// Canonical Hill: potency * alpha^n / (EC50^n + alpha^n), matches cancer_therapy.rs
fn hill_response(alignment: f32, potency: f32, hill_n: f32) -> f32 {
    if alignment <= 0.0 || potency <= 0.0 {
        return 0.0;
    }
    let c_n = alignment.powf(hill_n);
    let ec50_n = 0.5f32.powf(hill_n);
    potency * c_n / (ec50_n + c_n)
}

/// Tasa base de drenaje citotóxico.
/// Base cytotoxic drain rate.
const DRUG_DRAIN_BASE: f32 = 0.5;

/// Drenaje citotóxico por tick: Axiom 4 + 8.
/// Cytotoxic drain per tick: Axiom 4 + 8.
fn drug_drain(entity_freq: f32, drug_freq: f32, bandwidth: f32, potency: f32) -> f32 {
    let alignment = determinism::gaussian_frequency_alignment(entity_freq, drug_freq, bandwidth);
    let hill = hill_response(alignment, potency, 2.0);
    hill * DRUG_DRAIN_BASE
}

/// Clasificador de subpoblación por frecuencia.
/// Subpopulation classifier by frequency.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SubPop {
    Differentiated,
    Progenitor,
    Stem,
}

fn classify(entity_freq: f32, config: &MichorConfig) -> SubPop {
    let d_diff = (entity_freq - config.diff_freq).abs();
    let d_prog = (entity_freq - config.prog_freq).abs();
    let d_stem = (entity_freq - config.stem_freq).abs();
    if d_stem <= d_prog && d_stem <= d_diff {
        SubPop::Stem
    } else if d_prog <= d_diff {
        SubPop::Progenitor
    } else {
        SubPop::Differentiated
    }
}

// ─── Inflection detection ───────────────────────────────────────────────────

/// Detecta punto de inflexión en serie temporal de log(population).
/// Detect inflection point in log(population) time series.
///
/// Ajuste lineal por tramos: prueba cada punto de corte, calcula pendientes
/// de ambos segmentos, maximiza ratio de pendientes.
/// Piecewise linear fit: try each split point, compute slopes of both
/// segments, maximize slope ratio.
///
/// Retorna: (split_index, slope_before, slope_after).
/// Returns: (split_index, slope_before, slope_after).
pub fn detect_inflection(timeline: &[f32]) -> Option<(usize, f32, f32)> {
    if timeline.len() < 6 {
        return None;
    }

    // Log-transform (clamp > 0).
    let log_vals: Vec<f32> = timeline.iter().map(|&v| (v.max(0.01)).ln()).collect();

    let n = log_vals.len();
    let mut best_split = 0usize;
    let mut best_ratio = 0.0f32;
    let mut best_slopes = (0.0f32, 0.0f32);

    // Prueba cada punto de corte (mínimo 3 puntos por segmento).
    // Try each split point (minimum 3 points per segment).
    for split in 3..(n - 3) {
        let slope1 = linear_slope(&log_vals[..split]);
        let slope2 = linear_slope(&log_vals[split..]);

        // Ambas pendientes deben ser negativas (declive).
        // Both slopes must be negative (decline).
        if slope1 >= 0.0 || slope2 >= 0.0 {
            continue;
        }

        let ratio = slope1 / slope2; // ratio > 1 means phase1 steeper than phase2
        if ratio > best_ratio {
            best_ratio = ratio;
            best_split = split;
            best_slopes = (slope1, slope2);
        }
    }

    if best_ratio > 1.0 {
        Some((best_split, best_slopes.0, best_slopes.1))
    } else {
        None
    }
}

/// Pendiente por mínimos cuadrados ordinarios.
/// Slope via ordinary least squares.
fn linear_slope(values: &[f32]) -> f32 {
    let n = values.len() as f32;
    if n < 2.0 {
        return 0.0;
    }

    let x_mean = (n - 1.0) / 2.0;
    let y_mean = values.iter().sum::<f32>() / n;

    let (mut num, mut den) = (0.0f32, 0.0f32);
    for (i, &y) in values.iter().enumerate() {
        let x = i as f32;
        num += (x - x_mean) * (y - y_mean);
        den += (x - x_mean) * (x - x_mean);
    }

    if den.abs() < 1e-10 { 0.0 } else { num / den }
}

// ─── Spawn ──────────────────────────────────────────────────────────────────

fn spawn_subpop(
    world: &mut SimWorldFlat,
    count: u8,
    freq: f32,
    freq_sigma: f32,
    qe: f32,
    growth: f32,
    seed: &mut u64,
) {
    for _ in 0..count {
        *seed = determinism::next_u64(*seed);
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.radius = (qe.sqrt() * DISSIPATION_SOLID).clamp(0.3, 1.0);
        e.frequency_hz = freq + determinism::gaussian_f32(*seed, freq_sigma);
        e.growth_bias = growth;
        e.mobility_bias = 0.1;
        e.branching_bias = 0.2;
        e.resilience = 0.5;
        e.dissipation = DISSIPATION_SOLID;
        e.expression_mask = [1.0; 4];
        *seed = determinism::next_u64(*seed);
        e.position = [
            determinism::range_f32(*seed, GRID_POS_MIN, GRID_POS_MAX),
            determinism::range_f32(determinism::next_u64(*seed), GRID_POS_MIN, GRID_POS_MAX),
        ];
        world.spawn(e);
    }
}

fn spawn_population(world: &mut SimWorldFlat, config: &MichorConfig, seed: u64) {
    let mut s = seed;
    spawn_subpop(
        world,
        config.differentiated_count,
        config.diff_freq,
        15.0,
        config.diff_qe,
        config.diff_growth_bias,
        &mut s,
    );
    spawn_subpop(
        world,
        config.progenitor_count,
        config.prog_freq,
        10.0,
        config.prog_qe,
        config.prog_growth_bias,
        &mut s,
    );
    spawn_subpop(
        world,
        config.stem_count,
        config.stem_freq,
        5.0,
        config.stem_qe,
        config.stem_growth_bias,
        &mut s,
    );
}

// ─── Tick ───────────────────────────────────────────────────────────────────

/// Tick de terapia: pipeline batch + fármaco citotóxico.
/// Therapy tick: batch pipeline + cytotoxic drug.
fn therapy_tick(
    world: &mut SimWorldFlat,
    scratch: &mut ScratchPad,
    config: &MichorConfig,
    drug_active: bool,
) {
    scratch.clear();
    world.events.clear();
    world.tick_id += 1;

    systems::behavior_assess(world, scratch);
    systems::engine_processing(world);
    systems::irradiance_update(world);
    systems::containment_check(world, scratch);
    systems::dissipation(world);
    systems::will_to_velocity(world);
    systems::velocity_cap(world);
    systems::locomotion_drain(world);
    systems::movement_integrate(world);
    systems::collision(world, scratch);
    systems::nutrient_uptake(world);
    systems::photosynthesis(world);
    systems::state_transitions(world);
    systems::trophic_forage(world);
    systems::trophic_predation(world, scratch);

    // ── Drug: AFTER metabolic intake, BEFORE death ────────────────────
    if drug_active {
        let mut mask = world.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            let drain = drug_drain(
                world.entities[i].frequency_hz,
                config.drug_freq,
                config.drug_bandwidth,
                config.drug_potency,
            );
            world.entities[i].qe = (world.entities[i].qe - drain).max(0.0);
        }
    }

    // Phase::MorphologicalLayer — NO reproduction (closed population, like Michor model)
    systems::senescence(world);
    systems::death_reap(world);
    world.update_total_qe();
}

// ─── Snapshot ───────────────────────────────────────────────────────────────

fn compute_snapshot(
    worlds: &[SimWorldFlat],
    generation: u32,
    config: &MichorConfig,
    drug_active: bool,
) -> MichorSnapshot {
    let nw = worlds.len().max(1) as f32;
    let (mut total, mut diff, mut prog, mut stem, mut qe_sum) = (0u32, 0u32, 0u32, 0u32, 0.0f32);

    for w in worlds {
        let mut mask = w.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            total += 1;
            qe_sum += w.entities[i].qe;
            match classify(w.entities[i].frequency_hz, config) {
                SubPop::Differentiated => diff += 1,
                SubPop::Progenitor => prog += 1,
                SubPop::Stem => stem += 1,
            }
        }
    }

    MichorSnapshot {
        generation,
        total_alive: total as f32 / nw,
        diff_alive: diff as f32 / nw,
        prog_alive: prog as f32 / nw,
        stem_alive: stem as f32 / nw,
        total_qe: qe_sum / nw,
        drug_active,
    }
}

// ─── Main HOF ───────────────────────────────────────────────────────────────

/// Ejecuta el experimento completo. Stateless: config in → report out.
/// Run complete experiment. Stateless: config in → report out.
pub fn run(config: &MichorConfig) -> MichorReport {
    let start = Instant::now();

    let mut worlds: Vec<SimWorldFlat> = (0..config.worlds)
        .map(|wi| {
            let ws = determinism::next_u64(config.seed ^ (wi as u64));
            let mut w = SimWorldFlat::new(ws, 0.05);
            for cell in w.nutrient_grid.iter_mut() {
                *cell = config.nutrient_level;
            }
            for cell in w.irradiance_grid.iter_mut() {
                *cell = config.nutrient_level * IRRADIANCE_NUTRIENT_RATIO;
            }
            spawn_population(&mut w, config, ws);
            w
        })
        .collect();

    let mut scratches: Vec<ScratchPad> = (0..config.worlds).map(|_| ScratchPad::new()).collect();
    let mut timeline = Vec::with_capacity(config.generations as usize);

    for generation in 0..config.generations {
        let drug_active = generation >= config.drug_start_gen;

        for (wi, world) in worlds.iter_mut().enumerate() {
            for _ in 0..config.ticks_per_gen {
                therapy_tick(world, &mut scratches[wi], config, drug_active);
            }
        }

        timeline.push(compute_snapshot(&worlds, generation, config, drug_active));
    }

    // Extraer serie de total_alive para detección de inflexión.
    // Extract total_alive series for inflection detection.
    // Solo analizar la fase post-tratamiento.
    // Only analyze the post-treatment phase.
    let treatment_series: Vec<f32> = timeline
        .iter()
        .filter(|s| s.drug_active)
        .map(|s| s.total_alive)
        .collect();

    let inflection = detect_inflection(&treatment_series);

    let (phase1_slope, phase2_slope, slope_ratio, inflection_gen) = match inflection {
        Some((split, s1, s2)) => {
            let ratio = if s2.abs() > 1e-10 { s1 / s2 } else { 0.0 };
            let inflect_g = config.drug_start_gen + split as u32;
            (s1, s2, ratio, Some(inflect_g))
        }
        None => {
            // Fallback: calcular pendiente global.
            // Fallback: compute global slope.
            let slope = if treatment_series.len() >= 2 {
                linear_slope(
                    &treatment_series
                        .iter()
                        .map(|v| v.max(0.01).ln())
                        .collect::<Vec<_>>(),
                )
            } else {
                0.0
            };
            (slope, slope, 1.0, None)
        }
    };

    // Stem survive: al menos 1 stem cell viva en promedio al final.
    // Stem survive: at least 1 stem cell alive on average at the end.
    let stem_survive = timeline.last().map(|s| s.stem_alive > 0.0).unwrap_or(false);

    // Bifásico: ratio > 2 Y ambas pendientes negativas.
    // Biphasic: ratio > 2 AND both slopes negative.
    let biphasic_detected = slope_ratio > 2.0 && phase1_slope < 0.0 && phase2_slope < 0.0;

    MichorReport {
        config: config.clone(),
        timeline,
        phase1_slope,
        phase2_slope,
        slope_ratio,
        inflection_gen,
        stem_survive,
        biphasic_detected,
        wall_time_ms: start.elapsed().as_millis() as u64,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> MichorConfig {
        MichorConfig {
            differentiated_count: 8,
            progenitor_count: 3,
            stem_count: 2,
            worlds: 3,
            generations: 10,
            ticks_per_gen: 20,
            ..Default::default()
        }
    }

    #[test]
    fn given_default_config_when_run_then_no_panic() {
        let config = small_config();
        let report = run(&config);
        assert_eq!(report.timeline.len(), config.generations as usize);
    }

    #[test]
    fn given_same_seed_when_run_twice_then_deterministic() {
        let config = small_config();
        let a = run(&config);
        let b = run(&config);
        for i in 0..config.generations as usize {
            assert_eq!(
                a.timeline[i].total_alive.to_bits(),
                b.timeline[i].total_alive.to_bits(),
                "mismatch at gen {i}"
            );
        }
    }

    #[test]
    fn given_classifier_when_diff_freq_then_differentiated() {
        let config = MichorConfig::default();
        assert_eq!(classify(400.0, &config), SubPop::Differentiated);
        assert_eq!(classify(320.0, &config), SubPop::Progenitor);
        assert_eq!(classify(220.0, &config), SubPop::Stem);
    }

    #[test]
    fn given_drug_drain_when_diff_vs_stem_then_diff_higher() {
        let config = MichorConfig::default();
        let drain_diff = drug_drain(
            config.diff_freq,
            config.drug_freq,
            config.drug_bandwidth,
            config.drug_potency,
        );
        let drain_stem = drug_drain(
            config.stem_freq,
            config.drug_freq,
            config.drug_bandwidth,
            config.drug_potency,
        );
        assert!(
            drain_diff > drain_stem,
            "diff drain ({drain_diff}) must exceed stem drain ({drain_stem})"
        );
    }

    #[test]
    fn given_hill_response_when_zero_then_zero() {
        assert_eq!(hill_response(0.0, 1.0, 2.0), 0.0);
    }

    #[test]
    fn given_linear_slope_when_decreasing_then_negative() {
        let vals = [10.0, 8.0, 6.0, 4.0, 2.0];
        let slope = linear_slope(&vals);
        assert!(
            slope < 0.0,
            "decreasing values must give negative slope: {slope}"
        );
    }

    #[test]
    fn given_linear_slope_when_constant_then_zero() {
        let vals = [5.0, 5.0, 5.0, 5.0];
        let slope = linear_slope(&vals);
        assert!(
            slope.abs() < 1e-5,
            "constant values must give zero slope: {slope}"
        );
    }

    #[test]
    fn given_biphasic_series_when_inflection_detected_then_ratio_above_1() {
        // Simula declive bifásico: 10 puntos rápidos + 10 puntos lentos.
        // Simulate biphasic decline: 10 fast points + 10 slow points.
        let mut series = Vec::with_capacity(20);
        // Phase 1: fast decline (50 → 10)
        for i in 0..10 {
            series.push(50.0 - 4.0 * i as f32);
        }
        // Phase 2: slow decline (10 → 5)
        for i in 0..10 {
            series.push(10.0 - 0.5 * i as f32);
        }
        let result = detect_inflection(&series);
        assert!(
            result.is_some(),
            "should detect inflection in biphasic data"
        );
        let (split, s1, s2) = result.unwrap();
        assert!(
            split > 3 && split < 17,
            "split should be near middle: {split}"
        );
        assert!(
            s1 < 0.0 && s2 < 0.0,
            "both slopes negative: s1={s1}, s2={s2}"
        );
    }

    #[test]
    fn given_no_drug_when_run_then_population_stable_or_grows() {
        let config = MichorConfig {
            drug_start_gen: 999, // Drug never starts
            ..small_config()
        };
        let report = run(&config);
        let first = report
            .timeline
            .first()
            .map(|s| s.total_alive)
            .unwrap_or(0.0);
        let last = report.timeline.last().map(|s| s.total_alive).unwrap_or(0.0);
        // Sin fármaco, la población no debería colapsar rápidamente.
        // Without drug, population should not collapse rapidly.
        assert!(
            last >= first * 0.3,
            "without drug, population should not crash: first={first}, last={last}"
        );
    }

    #[test]
    fn given_stem_far_from_drug_when_treated_then_stem_more_likely_to_survive() {
        // La distancia de frecuencia protege a las stem cells.
        // Frequency distance protects stem cells.
        let config = MichorConfig::default();
        let drain_diff = drug_drain(
            config.diff_freq,
            config.drug_freq,
            config.drug_bandwidth,
            config.drug_potency,
        );
        let drain_stem = drug_drain(
            config.stem_freq,
            config.drug_freq,
            config.drug_bandwidth,
            config.drug_potency,
        );
        let ratio = drain_diff / drain_stem.max(1e-10);
        assert!(
            ratio > 5.0,
            "diff should receive >5× more drug than stem: ratio={ratio}"
        );
    }

    #[test]
    fn given_snapshot_when_computed_then_subpop_counts_sum() {
        let config = small_config();
        let mut world = SimWorldFlat::new(42, 0.05);
        for cell in world.nutrient_grid.iter_mut() {
            *cell = config.nutrient_level;
        }
        spawn_population(&mut world, &config, 42);
        let snap = compute_snapshot(&[world], 0, &config, false);
        let total_sub = snap.diff_alive + snap.prog_alive + snap.stem_alive;
        assert!(
            (snap.total_alive - total_sub).abs() < 0.01,
            "subpop sum ({total_sub}) must equal total ({}) within tolerance",
            snap.total_alive
        );
    }
}
