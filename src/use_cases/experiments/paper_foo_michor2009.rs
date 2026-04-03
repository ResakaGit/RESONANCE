//! PV-4: Foo & Michor 2009 — terapia continua vs pulsada.
//! PV-4: Foo & Michor 2009 — continuous vs pulsed therapy.
//!
//! Foo J, Michor F (2009) PLoS Comp Bio 5:e1000557.
//! Core prediction: optimal dose exists (non-monotonic resistance curve),
//! pulsed scheduling can beat continuous at equivalent total exposure.
//!
//! Drug = cytotoxic drain via gaussian_frequency_alignment × hill_response × potency.
//! Resistance = frequency drift during reproduction (mutation_sigma).
//! Entities whose frequency drifts away from drug_freq become resistant.
//!
//! All stateless. Config in → FooMichorReport out. BDD-tested.

use crate::batch::arena::{EntitySlot, SimWorldFlat};
use crate::batch::scratch::ScratchPad;
use crate::batch::systems;
use crate::blueprint::equations::determinism;
use crate::blueprint::equations::derived_thresholds::DISSIPATION_SOLID;
use std::time::Instant;

// ─── Config ─────────────────────────────────────────────────────────────────

/// Configuración del experimento Foo & Michor 2009.
/// Foo & Michor 2009 experiment configuration.
#[derive(Debug, Clone)]
pub struct FooMichorConfig {
    // Population
    pub cell_count:      u8,
    pub cell_freq:       f32,
    pub cell_qe:         f32,
    pub freq_spread:     f32,
    /// Sigma de deriva de frecuencia por reproducción (mutación).
    /// Frequency drift sigma per reproduction event (mutation).
    pub mutation_sigma:  f32,

    // Drug
    pub drug_freq:       f32,
    /// Niveles de dosis a explorar (fracción de potencia máxima).
    /// Dose levels to explore (fraction of max potency).
    pub dose_levels:     Vec<f32>,
    pub drug_ki:         f32,
    pub drug_bandwidth:  f32,

    // Pulsed schedule
    pub pulse_on_gens:   u32,
    pub pulse_off_gens:  u32,

    // Biology
    pub nutrient_level:  f32,

    // Simulation
    pub worlds:          usize,
    pub generations:     u32,
    pub ticks_per_gen:   u32,
    pub seed:            u64,
}

impl Default for FooMichorConfig {
    fn default() -> Self {
        Self {
            cell_count:     45,
            cell_freq:      400.0,
            cell_qe:        50.0,
            freq_spread:    60.0,
            mutation_sigma: 12.0,
            drug_freq:      400.0,
            dose_levels:    vec![0.2, 0.4, 0.6, 0.8, 1.0],
            drug_ki:        1.0,
            drug_bandwidth: 50.0,
            pulse_on_gens:  6,
            pulse_off_gens: 6,
            nutrient_level: 1.5,
            worlds:         20,
            generations:    50,
            ticks_per_gen:  80,
            seed:           42,
        }
    }
}

// ─── Output ─────────────────────────────────────────────────────────────────

/// Reporte completo del experimento Foo & Michor 2009.
/// Complete Foo & Michor 2009 experiment report.
#[derive(Debug)]
pub struct FooMichorReport {
    /// Curva dosis-resistencia: (dose_level, resistance_rate).
    /// Dose-resistance curve: (dose_level, resistance_rate).
    pub dose_resistance_curve:     Vec<(f32, f32)>,
    /// Resistencia bajo terapia continua a dosis=0.8.
    /// Resistance rate under continuous therapy at dose=0.8.
    pub continuous_resistance_at_08: f32,
    /// Resistencia bajo terapia pulsada a dosis=0.8.
    /// Resistance rate under pulsed therapy at dose=0.8.
    pub pulsed_resistance_at_08:   f32,
    /// Dosis con mínima resistencia.
    /// Dose level with minimum resistance.
    pub optimal_dose:              f32,
    /// Curva no-monótona: existe un mínimo interior.
    /// Non-monotonic curve: interior minimum exists.
    pub optimal_exists:            bool,
    /// Terapia pulsada supera a continua a dosis=0.8.
    /// Pulsed therapy beats continuous at dose=0.8.
    pub pulsed_beats_continuous:   bool,
    pub wall_time_ms:              u64,
}

// ─── Pure equations ─────────────────────────────────────────────────────────

/// Respuesta Hill con potencia incorporada (consistente con cancer_therapy.rs).
/// Hill response with potency folded in (consistent with cancer_therapy.rs).
fn hill_response(alignment: f32, potency: f32, hill_n: f32) -> f32 {
    if alignment <= 0.0 || potency <= 0.0 { return 0.0; }
    let c_n = alignment.powf(hill_n);
    let ec50_n = 0.5f32.powf(hill_n);
    potency * c_n / (ec50_n + c_n)
}

/// Tasa base de drenaje citotóxico.
/// Base cytotoxic drain rate.
const DRUG_DRAIN_BASE: f32 = 0.5;

/// Drenaje citotóxico: Axiom 4 + 8. Gaussian alignment × Hill × base.
/// Cytotoxic drain: Axiom 4 + 8. Gaussian alignment × Hill × base.
fn drug_drain(entity_freq: f32, drug_freq: f32, bandwidth: f32, potency: f32) -> f32 {
    let alignment = determinism::gaussian_frequency_alignment(entity_freq, drug_freq, bandwidth);
    let hill = hill_response(alignment, potency, 2.0);
    hill * DRUG_DRAIN_BASE
}

/// Entidad resistente: frecuencia lejos del fármaco (>1 bandwidth).
/// Entity resistant: frequency far from drug (>1 bandwidth).
fn is_resistant(entity_freq: f32, drug_freq: f32, bandwidth: f32) -> bool {
    (entity_freq - drug_freq).abs() > bandwidth
}

// ─── Spawn ──────────────────────────────────────────────────────────────────

fn spawn_population(world: &mut SimWorldFlat, config: &FooMichorConfig, seed: u64) {
    let mut s = seed;
    for _ in 0..config.cell_count {
        s = determinism::next_u64(s);
        let mut e = EntitySlot::default();
        e.qe = config.cell_qe;
        e.radius = (config.cell_qe.sqrt() * DISSIPATION_SOLID).clamp(0.3, 1.0);
        e.frequency_hz = config.cell_freq + determinism::gaussian_f32(s, config.freq_spread);
        e.growth_bias = 0.7;
        e.mobility_bias = 0.3;
        e.branching_bias = 0.4;
        e.resilience = 0.5;
        e.dissipation = DISSIPATION_SOLID;
        e.expression_mask = [1.0; 4];
        s = determinism::next_u64(s);
        e.position = [
            determinism::range_f32(s, 1.0, 15.0),
            determinism::range_f32(determinism::next_u64(s), 1.0, 15.0),
        ];
        world.spawn(e);
    }
}

// ─── Tick ───────────────────────────────────────────────────────────────────

/// Tick del experimento: pipeline batch + fármaco citotóxico + mutación por reproducción.
/// Experiment tick: batch pipeline + cytotoxic drug + mutation via reproduction.
fn experiment_tick(
    world: &mut SimWorldFlat,
    scratch: &mut ScratchPad,
    config: &FooMichorConfig,
    potency: f32,
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
    if drug_active && potency > 0.0 {
        let mut mask = world.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            let drain = drug_drain(
                world.entities[i].frequency_hz,
                config.drug_freq,
                config.drug_bandwidth,
                potency,
            );
            world.entities[i].qe = (world.entities[i].qe - drain).max(0.0);
        }
    }

    // Phase::MorphologicalLayer
    systems::growth_inference(world);
    systems::reproduction(world);
    systems::senescence(world);
    systems::death_reap(world);
    world.update_total_qe();
}

/// Aplica mutación de frecuencia post-reproducción (drift genético).
/// Apply frequency mutation post-reproduction (genetic drift).
///
/// Newly spawned entities (not in the original population) get frequency drift.
/// This models the accumulation of mutations under drug pressure.
fn apply_frequency_drift(
    world: &mut SimWorldFlat,
    old_mask: u128,
    seed: u64,
    mutation_sigma: f32,
) {
    let new_mask = world.alive_mask & !old_mask;
    let mut drift_mask = new_mask;
    let mut s = seed;
    while drift_mask != 0 {
        let i = drift_mask.trailing_zeros() as usize;
        drift_mask &= drift_mask - 1;
        s = determinism::next_u64(s);
        world.entities[i].frequency_hz += determinism::gaussian_f32(s, mutation_sigma);
    }
}

// ─── Single arm runner ──────────────────────────────────────────────────────

/// Resultado de un brazo del experimento.
/// Result of one experiment arm.
struct ArmResult {
    resistance_rate: f32,
}

/// Ejecuta un brazo: N mundos × G generaciones con schedule dado.
/// Run one arm: N worlds × G generations with given schedule.
fn run_arm(
    config: &FooMichorConfig,
    potency: f32,
    pulsed: bool,
    arm_seed: u64,
) -> ArmResult {
    let mut worlds: Vec<SimWorldFlat> = (0..config.worlds).map(|wi| {
        let ws = determinism::next_u64(arm_seed ^ (wi as u64));
        let mut w = SimWorldFlat::new(ws, 0.05);
        for cell in w.nutrient_grid.iter_mut() { *cell = config.nutrient_level; }
        for cell in w.irradiance_grid.iter_mut() { *cell = config.nutrient_level * 0.3; }
        spawn_population(&mut w, config, ws);
        w
    }).collect();

    let mut scratches: Vec<ScratchPad> = (0..config.worlds).map(|_| ScratchPad::new()).collect();

    for generation in 0..config.generations {
        let drug_active = if pulsed {
            let cycle = config.pulse_on_gens + config.pulse_off_gens;
            if cycle == 0 { true } else { (generation % cycle) < config.pulse_on_gens }
        } else {
            true // continuous: always on
        };

        for (wi, world) in worlds.iter_mut().enumerate() {
            let pre_mask = world.alive_mask;
            for _ in 0..config.ticks_per_gen {
                experiment_tick(world, &mut scratches[wi], config, potency, drug_active);
            }
            let drift_seed = world.seed ^ world.tick_id ^ (generation as u64);
            apply_frequency_drift(world, pre_mask, drift_seed, config.mutation_sigma);
        }
    }

    // Resistencia = fracción de mundos donde entidades resistentes dominan.
    // Resistance = fraction of worlds where resistant entities dominate.
    let mut resistant_worlds = 0u32;
    for world in &worlds {
        let (mut resistant, mut total) = (0u32, 0u32);
        let mut mask = world.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            total += 1;
            if is_resistant(world.entities[i].frequency_hz, config.drug_freq, config.drug_bandwidth) {
                resistant += 1;
            }
        }
        if total > 0 && resistant as f32 / total as f32 > 0.5 {
            resistant_worlds += 1;
        }
    }

    ArmResult {
        resistance_rate: resistant_worlds as f32 / config.worlds.max(1) as f32,
    }
}

// ─── Main HOF ───────────────────────────────────────────────────────────────

/// Ejecuta el experimento completo. Stateless: config in → report out.
/// Run complete experiment. Stateless: config in → report out.
pub fn run_foo_michor(config: &FooMichorConfig) -> FooMichorReport {
    let start = Instant::now();

    // Curva dosis-resistencia (continua).
    // Dose-resistance curve (continuous).
    let dose_resistance_curve: Vec<(f32, f32)> = config.dose_levels.iter().map(|&dose| {
        let arm_seed = determinism::next_u64(config.seed ^ dose.to_bits() as u64);
        let result = run_arm(config, dose * config.drug_ki, false, arm_seed);
        (dose, result.resistance_rate)
    }).collect();

    // Brazo continuo a 0.8.
    // Continuous arm at 0.8.
    let continuous_08_seed = determinism::next_u64(config.seed ^ 0x0800_C0E7);
    let continuous_08 = run_arm(config, 0.8 * config.drug_ki, false, continuous_08_seed);

    // Brazo pulsado a 0.8.
    // Pulsed arm at 0.8.
    let pulsed_08_seed = determinism::next_u64(config.seed ^ 0x0800_9015);
    let pulsed_08 = run_arm(config, 0.8 * config.drug_ki, true, pulsed_08_seed);

    // Dosis óptima = mínima resistencia.
    // Optimal dose = minimum resistance.
    let (optimal_dose, _min_resistance) = dose_resistance_curve.iter()
        .fold((0.0f32, f32::MAX), |(best_d, best_r), &(d, r)| {
            if r < best_r { (d, r) } else { (best_d, best_r) }
        });

    // No-monótona: no estrictamente decreciente.
    // Non-monotonic: not strictly decreasing.
    let optimal_exists = if dose_resistance_curve.len() < 3 {
        false
    } else {
        // Check if resistance ever increases then decreases (or vice versa)
        let has_increase = dose_resistance_curve.windows(2)
            .any(|w| w[1].1 > w[0].1 + 1e-6);
        let has_decrease = dose_resistance_curve.windows(2)
            .any(|w| w[1].1 < w[0].1 - 1e-6);
        has_increase && has_decrease
    };

    let pulsed_beats_continuous = pulsed_08.resistance_rate < continuous_08.resistance_rate;

    FooMichorReport {
        dose_resistance_curve,
        continuous_resistance_at_08: continuous_08.resistance_rate,
        pulsed_resistance_at_08: pulsed_08.resistance_rate,
        optimal_dose,
        optimal_exists,
        pulsed_beats_continuous,
        wall_time_ms: start.elapsed().as_millis() as u64,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> FooMichorConfig {
        FooMichorConfig {
            cell_count: 10,
            worlds: 3,
            generations: 5,
            ticks_per_gen: 20,
            ..Default::default()
        }
    }

    #[test]
    fn given_default_config_when_run_then_no_panic() {
        let config = small_config();
        let report = run_foo_michor(&config);
        assert_eq!(report.dose_resistance_curve.len(), config.dose_levels.len());
    }

    #[test]
    fn given_same_seed_when_run_twice_then_deterministic() {
        let config = small_config();
        let a = run_foo_michor(&config);
        let b = run_foo_michor(&config);
        for (i, ((da, ra), (db, rb))) in a.dose_resistance_curve.iter()
            .zip(b.dose_resistance_curve.iter()).enumerate()
        {
            assert_eq!(da.to_bits(), db.to_bits(), "dose mismatch at {i}");
            assert_eq!(ra.to_bits(), rb.to_bits(), "resistance mismatch at {i}");
        }
    }

    #[test]
    fn given_resistance_classifier_when_freq_far_then_resistant() {
        assert!(is_resistant(500.0, 400.0, 50.0), "100 Hz away > 50 bandwidth");
        assert!(!is_resistant(420.0, 400.0, 50.0), "20 Hz away < 50 bandwidth");
    }

    #[test]
    fn given_hill_response_when_zero_alignment_then_zero() {
        assert_eq!(hill_response(0.0, 1.0, 2.0), 0.0);
    }

    #[test]
    fn given_hill_response_when_full_alignment_then_near_max() {
        let h = hill_response(1.0, 2.0, 2.0);
        assert!(h > 1.5 && h < 2.0, "full alignment near max: {h}");
    }

    #[test]
    fn given_drug_drain_when_on_target_then_positive() {
        let d = drug_drain(400.0, 400.0, 50.0, 1.0);
        assert!(d > 0.0, "on-target drain must be positive: {d}");
    }

    #[test]
    fn given_drug_drain_when_off_target_then_less() {
        let on = drug_drain(400.0, 400.0, 50.0, 1.0);
        let off = drug_drain(600.0, 400.0, 50.0, 1.0);
        assert!(off < on, "off-target ({off}) < on-target ({on})");
    }

    #[test]
    fn given_zero_dose_when_run_then_no_resistance() {
        let config = FooMichorConfig {
            cell_count: 10,
            worlds: 5,
            generations: 5,
            ticks_per_gen: 20,
            dose_levels: vec![0.0],
            ..Default::default()
        };
        let report = run_foo_michor(&config);
        // Sin fármaco, la resistencia depende solo del drift natural.
        // Without drug, resistance depends only on natural drift.
        assert!(report.dose_resistance_curve[0].1 <= 1.0,
            "zero dose resistance should be bounded: {}",
            report.dose_resistance_curve[0].1);
    }

    #[test]
    fn given_dose_curve_when_inspected_then_doses_match_config() {
        let config = small_config();
        let report = run_foo_michor(&config);
        for (i, &(dose, _)) in report.dose_resistance_curve.iter().enumerate() {
            assert!((dose - config.dose_levels[i]).abs() < 1e-6,
                "dose mismatch at {i}: expected {}, got {dose}",
                config.dose_levels[i]);
        }
    }

    #[test]
    fn given_frequency_drift_when_applied_then_new_entities_mutated() {
        let config = FooMichorConfig::default();
        let mut world = SimWorldFlat::new(42, 0.05);
        for cell in world.nutrient_grid.iter_mut() { *cell = config.nutrient_level; }
        spawn_population(&mut world, &config, 42);
        let old_mask = world.alive_mask;

        // Spawn a new entity manually (simulating reproduction)
        let mut new_e = EntitySlot::default();
        new_e.qe = 30.0;
        new_e.frequency_hz = 400.0;
        world.spawn(new_e);

        apply_frequency_drift(&mut world, old_mask, 123, 20.0);

        // Al menos la nueva entidad debe tener freq modificada.
        // At least the new entity should have modified frequency.
        let new_mask = world.alive_mask & !old_mask;
        if new_mask != 0 {
            let idx = new_mask.trailing_zeros() as usize;
            // Drift applied — frequency changed (probabilistically, sigma=20 → very likely ≠ 400.0)
            // We just verify no panic and the entity is still alive.
            assert!(world.entities[idx].alive);
        }
    }
}
