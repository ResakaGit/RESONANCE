//! Sharma et al. 2010 (Cell 141:69-80) — Persistentes tolerantes al fármaco.
//! Sharma et al. 2010 (Cell 141:69-80) — Drug-tolerant persisters.
//!
//! Core prediction: ~0.3% of cells survive high-dose cytotoxic, show >10× reduced
//! sensitivity, and recover sensitivity in ~9 doublings after drug removal.
//!
//! Three phases: pre-treatment (baseline), treatment (selection), recovery (drug holiday).
//! Persisters = quiescent cells with low growth_bias AND freq far from drug target.
//! After drug removal, persister offspring shift freq back toward sensitive range.
//!
//! All stateless. Config in → SharmaReport out. BDD-tested.

use crate::batch::arena::{EntitySlot, SimWorldFlat};
use crate::batch::scratch::ScratchPad;
use crate::batch::systems;
use crate::blueprint::equations::derived_thresholds::{COHERENCE_BANDWIDTH, DISSIPATION_SOLID};
use crate::blueprint::equations::determinism;
use std::time::Instant;

// ─── Constants ──────────────────────────────────────────────────────────────

/// Hill coefficient for dose-response (sigmoidal). Standard pharmacology.
const HILL_COEFF: f32 = 2.0;

/// Drenaje citotóxico base por tick.
/// Base cytotoxic drain per tick.
const DRUG_DRAIN_BASE: f32 = 0.6;

/// Fracción de irradiancia respecto a nutrientes (calibración de grilla).
/// Irradiance-to-nutrient ratio (grid calibration).
const IRRADIANCE_NUTRIENT_RATIO: f32 = 0.3;

/// Rango espacial válido para entidades en la grilla 16×16.
/// Valid spatial range for entities in the 16×16 grid.
const GRID_POS_MIN: f32 = 1.0;
const GRID_POS_MAX: f32 = 15.0;

// ─── Config ─────────────────────────────────────────────────────────────────

/// Fase del experimento: pre-tratamiento, tratamiento o recuperación.
/// Experiment phase: pre-treatment, treatment, or recovery.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SharmaPhase {
    PreTreatment,
    Treatment,
    Recovery,
}

/// Configuración del experimento Sharma 2010 (persistentes).
/// Sharma 2010 experiment configuration (drug-tolerant persisters).
#[derive(Debug, Clone)]
pub struct SharmaConfig {
    // Population
    pub total_cells: u8,
    pub quiescent_fraction: f32,
    pub quiescent_growth_bias: f32,
    pub quiescent_freq_offset: f32,
    pub sensitive_freq: f32,

    // Drug
    pub drug_freq: f32,
    pub drug_potency: f32,
    pub drug_bandwidth: f32,
    pub drug_start_gen: u32,
    pub drug_stop_gen: u32,

    // Biology
    pub nutrient_level: f32,

    // Simulation
    pub worlds: usize,
    pub generations: u32,
    pub ticks_per_gen: u32,
    pub seed: u64,
}

impl Default for SharmaConfig {
    fn default() -> Self {
        Self {
            total_cells: 50,
            quiescent_fraction: 0.06, // 3 out of 50, scaled from 0.3%
            quiescent_growth_bias: 0.03,
            quiescent_freq_offset: 200.0,
            sensitive_freq: 400.0,
            drug_freq: 400.0,
            drug_potency: 0.8,
            drug_bandwidth: COHERENCE_BANDWIDTH,
            drug_start_gen: 5,
            drug_stop_gen: 40,
            nutrient_level: 2.0,
            worlds: 20,
            generations: 60,
            ticks_per_gen: 100,
            seed: 42,
        }
    }
}

// ─── Output ─────────────────────────────────────────────────────────────────

/// Snapshot por generación del experimento Sharma.
/// Per-generation snapshot for the Sharma experiment.
#[derive(Debug, Clone)]
pub struct SharmaSnapshot {
    pub generation: u32,
    pub alive_mean: f32,
    pub qe_mean: f32,
    pub persister_frac: f32,
    pub phase: SharmaPhase,
}

/// Reporte completo del experimento Sharma 2010.
/// Complete report for the Sharma 2010 experiment.
#[derive(Debug)]
pub struct SharmaReport {
    pub config: SharmaConfig,
    pub timeline: Vec<SharmaSnapshot>,
    pub peak_population: f32,
    pub post_drug_survivors: f32,
    pub persister_fraction: f32,
    pub recovery_detected: bool,
    pub recovery_gen: Option<u32>,
    pub wall_time_ms: u64,
}

// ─── Pure equations ─────────────────────────────────────────────────────────

/// Hill dose-response para drenaje citotóxico.
/// Hill dose-response for cytotoxic drain.
/// Canonical Hill: potency * alpha^n / (EC50^n + alpha^n), matches cancer_therapy.rs
fn hill_response(alignment: f32, potency: f32, hill_n: f32) -> f32 {
    if alignment <= 0.0 || potency <= 0.0 {
        return 0.0;
    }
    let c_n = alignment.powf(hill_n);
    let ec50_n = 0.5f32.powf(hill_n);
    potency * c_n / (ec50_n + c_n)
}

/// Drenaje citotóxico por tick para una entidad. Axioma 4+8.
/// Cytotoxic drain per tick for one entity. Axiom 4+8.
fn cytotoxic_drain(entity_freq: f32, config: &SharmaConfig) -> f32 {
    let alignment = determinism::gaussian_frequency_alignment(
        entity_freq,
        config.drug_freq,
        config.drug_bandwidth,
    );
    let hill = hill_response(alignment, config.drug_potency, HILL_COEFF);
    hill * DRUG_DRAIN_BASE
}

/// Determina la fase del experimento según la generación.
/// Determine experiment phase from generation number.
fn phase_for_generation(generation: u32, config: &SharmaConfig) -> SharmaPhase {
    if generation < config.drug_start_gen {
        SharmaPhase::PreTreatment
    } else if generation < config.drug_stop_gen {
        SharmaPhase::Treatment
    } else {
        SharmaPhase::Recovery
    }
}

/// Clasifica entidad como persister: growth_bias bajo Y frecuencia lejos del fármaco.
/// Classify entity as persister: low growth_bias AND frequency far from drug.
fn is_persister(entity: &EntitySlot, config: &SharmaConfig) -> bool {
    let growth_low = entity.growth_bias < 0.05;
    let alignment = determinism::gaussian_frequency_alignment(
        entity.frequency_hz,
        config.drug_freq,
        COHERENCE_BANDWIDTH,
    );
    growth_low && alignment < 0.3
}

// ─── Per-tick drug application ──────────────────────────────────────────────

/// Aplica fármaco citotóxico a todas las entidades vivas.
/// Apply cytotoxic drug to all alive entities.
fn apply_drug(world: &mut SimWorldFlat, config: &SharmaConfig) -> f32 {
    let mut total_drain = 0.0f32;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let drain = cytotoxic_drain(world.entities[i].frequency_hz, config);
        world.entities[i].qe = (world.entities[i].qe - drain).max(0.0);
        total_drain += drain;
    }
    total_drain
}

// ─── Pipeline tick ──────────────────────────────────────────────────────────

/// Tick del experimento: pipeline batch estándar + fármaco condicional por fase.
/// Experiment tick: standard batch pipeline + phase-conditional drug.
fn sharma_tick(
    world: &mut SimWorldFlat,
    scratch: &mut ScratchPad,
    config: &SharmaConfig,
    phase: SharmaPhase,
) {
    scratch.clear();
    world.events.clear();
    world.tick_id += 1;

    // Phase::Input
    systems::behavior_assess(world, scratch);

    // Phase::ThermodynamicLayer
    systems::engine_processing(world);
    systems::irradiance_update(world);
    systems::containment_check(world, scratch);

    // Phase::AtomicLayer
    systems::dissipation(world);
    systems::will_to_velocity(world);
    systems::velocity_cap(world);
    systems::locomotion_drain(world);
    systems::movement_integrate(world);
    systems::collision(world, scratch);

    // Phase::ChemicalLayer
    systems::nutrient_uptake(world);
    systems::photosynthesis(world);
    systems::state_transitions(world);

    // Phase::MetabolicLayer
    systems::trophic_forage(world);
    systems::trophic_predation(world, scratch);

    // ── Drug: only during Treatment phase ──
    if phase == SharmaPhase::Treatment {
        apply_drug(world, config);
    }

    // Phase::MorphologicalLayer
    systems::growth_inference(world);
    systems::reproduction(world);
    systems::senescence(world);
    systems::death_reap(world);
    world.update_total_qe();
}

// ─── Snapshot ───────────────────────────────────────────────────────────────

fn compute_snapshot(
    worlds: &[SimWorldFlat],
    generation: u32,
    config: &SharmaConfig,
    phase: SharmaPhase,
) -> SharmaSnapshot {
    let nw = worlds.len().max(1) as f32;
    let (mut alive, mut qe_sum, mut persisters) = (0u32, 0.0f32, 0u32);

    for w in worlds {
        let mut mask = w.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            alive += 1;
            qe_sum += w.entities[i].qe;
            if is_persister(&w.entities[i], config) {
                persisters += 1;
            }
        }
    }

    let n = alive.max(1) as f32;
    SharmaSnapshot {
        generation,
        alive_mean: alive as f32 / nw,
        qe_mean: qe_sum / n,
        persister_frac: persisters as f32 / n,
        phase,
    }
}

// ─── Spawn ──────────────────────────────────────────────────────────────────

fn spawn_population(world: &mut SimWorldFlat, config: &SharmaConfig, seed: u64) {
    let mut s = seed;
    let quiescent_n = (config.total_cells as f32 * config.quiescent_fraction) as u8;
    let sensitive_n = config.total_cells - quiescent_n;

    // Sensitive cells: high growth, frequency near drug target.
    for _ in 0..sensitive_n {
        s = determinism::next_u64(s);
        let mut e = EntitySlot::default();
        e.qe = 60.0;
        e.radius = 0.5;
        e.frequency_hz = config.sensitive_freq + determinism::gaussian_f32(s, 10.0);
        e.growth_bias = 0.7;
        e.mobility_bias = 0.2;
        e.branching_bias = 0.3;
        e.resilience = 0.5;
        e.dissipation = DISSIPATION_SOLID;
        e.expression_mask = [1.0; 4];
        s = determinism::next_u64(s);
        e.position = [
            determinism::range_f32(s, GRID_POS_MIN, GRID_POS_MAX),
            determinism::range_f32(determinism::next_u64(s), GRID_POS_MIN, GRID_POS_MAX),
        ];
        world.spawn(e);
    }

    // Quiescent persisters: low growth, frequency shifted away from drug target.
    for _ in 0..quiescent_n {
        s = determinism::next_u64(s);
        let mut e = EntitySlot::default();
        e.qe = 40.0; // lower starting energy (dormant)
        e.radius = 0.3;
        e.frequency_hz = config.sensitive_freq
            + config.quiescent_freq_offset
            + determinism::gaussian_f32(s, 15.0);
        e.growth_bias = config.quiescent_growth_bias;
        e.mobility_bias = 0.1;
        e.branching_bias = 0.2;
        e.resilience = 0.8; // persisters are more resilient
        e.dissipation = DISSIPATION_SOLID * 0.6; // Quiescent: 60% of solid-state metabolism
        e.expression_mask = [1.0; 4];
        s = determinism::next_u64(s);
        e.position = [
            determinism::range_f32(s, GRID_POS_MIN, GRID_POS_MAX),
            determinism::range_f32(determinism::next_u64(s), GRID_POS_MIN, GRID_POS_MAX),
        ];
        world.spawn(e);
    }
}

// ─── Main HOF ───────────────────────────────────────────────────────────────

/// Ejecuta el experimento completo Sharma 2010. Stateless: config in → report out.
/// Run complete Sharma 2010 experiment. Stateless: config in → report out.
pub fn run(config: &SharmaConfig) -> SharmaReport {
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
        let phase = phase_for_generation(generation, config);

        for (wi, world) in worlds.iter_mut().enumerate() {
            for _ in 0..config.ticks_per_gen {
                sharma_tick(world, &mut scratches[wi], config, phase);
            }
        }

        timeline.push(compute_snapshot(&worlds, generation, config, phase));
    }

    // Peak population (pre-treatment or early treatment).
    let peak_population = timeline.iter().map(|s| s.alive_mean).fold(0.0f32, f32::max);

    // Survivors right after treatment ends.
    let post_drug_survivors = timeline
        .iter()
        .find(|s| s.phase == SharmaPhase::Recovery)
        .map(|s| s.alive_mean)
        .unwrap_or(0.0);

    // Persister fraction at end of treatment.
    let persister_fraction = timeline
        .iter()
        .rfind(|s| s.phase == SharmaPhase::Treatment)
        .map(|s| s.persister_frac)
        .unwrap_or(0.0);

    // Recovery detection: population recovers to >50% of peak after drug removal.
    let recovery_threshold = peak_population * 0.5;
    let recovery_gen = timeline
        .iter()
        .find(|s| s.phase == SharmaPhase::Recovery && s.alive_mean > recovery_threshold)
        .map(|s| s.generation);
    let recovery_detected = recovery_gen.is_some();

    SharmaReport {
        config: config.clone(),
        timeline,
        peak_population,
        post_drug_survivors,
        persister_fraction,
        recovery_detected,
        recovery_gen,
        wall_time_ms: start.elapsed().as_millis() as u64,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> SharmaConfig {
        SharmaConfig {
            worlds: 3,
            generations: 15,
            ticks_per_gen: 30,
            drug_start_gen: 3,
            drug_stop_gen: 10,
            ..Default::default()
        }
    }

    #[test]
    fn hill_response_zero_alignment_returns_zero() {
        assert_eq!(hill_response(0.0, 0.8, 2.0), 0.0);
    }

    #[test]
    fn hill_response_full_alignment_near_max() {
        let h = hill_response(1.0, 0.8, 2.0);
        // c_n=1, ec50_n=0.25, h = 0.8 * 1/(0.25+1) = 0.64
        assert!(h > 0.5 && h < 1.0, "full alignment → strong effect: {h}");
    }

    #[test]
    fn cytotoxic_drain_on_target_greater_than_off_target() {
        let c = SharmaConfig::default();
        let on = cytotoxic_drain(400.0, &c);
        let off = cytotoxic_drain(700.0, &c);
        assert!(
            on > off,
            "on-target drain ({on}) must exceed off-target ({off})"
        );
    }

    #[test]
    fn phase_classification_correct() {
        let c = SharmaConfig::default();
        assert_eq!(phase_for_generation(0, &c), SharmaPhase::PreTreatment);
        assert_eq!(phase_for_generation(4, &c), SharmaPhase::PreTreatment);
        assert_eq!(phase_for_generation(5, &c), SharmaPhase::Treatment);
        assert_eq!(phase_for_generation(39, &c), SharmaPhase::Treatment);
        assert_eq!(phase_for_generation(40, &c), SharmaPhase::Recovery);
        assert_eq!(phase_for_generation(59, &c), SharmaPhase::Recovery);
    }

    #[test]
    fn is_persister_classifies_correctly() {
        let c = SharmaConfig::default();
        let mut e = EntitySlot::default();
        // Low growth + far from drug → persister
        e.growth_bias = 0.02;
        e.frequency_hz = 650.0; // far from 400 Hz drug
        assert!(is_persister(&e, &c));
        // High growth → not persister
        e.growth_bias = 0.7;
        assert!(!is_persister(&e, &c));
        // Low growth but near drug freq → not persister (alignment > 0.3)
        e.growth_bias = 0.02;
        e.frequency_hz = 400.0;
        assert!(!is_persister(&e, &c));
    }

    #[test]
    fn run_no_panic() {
        let r = run(&small_config());
        assert_eq!(r.timeline.len(), 15);
    }

    #[test]
    fn run_deterministic() {
        let c = small_config();
        let a = run(&c);
        let b = run(&c);
        for i in 0..c.generations as usize {
            assert_eq!(
                a.timeline[i].alive_mean.to_bits(),
                b.timeline[i].alive_mean.to_bits(),
                "non-deterministic at gen {i}"
            );
        }
    }
}
