//! Cancer Therapy Simulation — resistance dynamics under selective pressure.
//!
//! Drug = frequency-selective dissipation (Axiom 4 + 8).
//! Resistance = mutation shifts frequency away from target (NOT ecological drift).
//! Relapse = quiescent stem cells survive, reactivate when niche empties (Axiom 6).
//! Pharmacokinetics = Hill equation dose-response + ramp up/down (Axiom 4).
//! Microenvironment = nutrient-dependent growth modulation (Axiom 7).
//!
//! All stateless. Config in → TherapyReport out.

use crate::batch::arena::{EntitySlot, SimWorldFlat};
use crate::batch::constants::MAX_ENTITIES;
use crate::batch::scratch::ScratchPad;
use crate::blueprint::equations::determinism;
use std::time::Instant;

// ─── Config ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TherapyConfig {
    // Population
    pub normal_count: u8,
    pub cancer_count: u8,
    pub normal_freq: f32,
    pub cancer_freq: f32,

    // Drug
    pub drug_target_freq: f32,
    pub drug_potency: f32,
    pub drug_bandwidth: f32,
    pub treatment_start_gen: u32,
    pub treatment_pause_gens: u32,
    /// Hill coefficient (1=hyperbolic, 2=sigmoidal). Standard pharmacology.
    pub hill_coefficient: f32,

    // Biology
    pub quiescent_fraction: f32,
    pub quiescent_drug_sensitivity: f32,
    /// Stem cells reactivate growth when cancer_count < this × initial.
    pub stem_reactivation_threshold: f32,
    pub normal_regen_rate: f32,
    pub relapse_threshold: f32,
    /// Immune cells per world (attack cancer by frequency proximity).
    pub immune_count: u8,
    /// Scaling: 1 entity = this many real cells.
    pub cells_per_entity: f32,
    /// Pharmacokinetic ramp: generations to reach full potency.
    pub pk_ramp_gens: u32,

    // Cell properties (injectable)
    pub normal_qe: f32,
    pub normal_growth: f32,
    pub normal_resilience: f32,
    pub normal_dissipation: f32,
    /// Normal cell trophic class. 0=producer (photosynthesis). Healthy tissue.
    pub normal_trophic: u8,
    pub cancer_qe: f32,
    pub cancer_growth: f32,
    pub cancer_resilience: f32,
    pub cancer_dissipation: f32,
    /// Cancer cell trophic class. 3=carnivore (Warburg: consumes host glucose, no photosynthesis).
    pub cancer_trophic: u8,

    // Simulation
    pub worlds: usize,
    pub generations: u32,
    pub ticks_per_gen: u32,
    pub seed: u64,
}

impl Default for TherapyConfig {
    fn default() -> Self {
        Self {
            normal_count: 30, cancer_count: 15,
            normal_freq: 250.0, cancer_freq: 400.0,
            drug_target_freq: 400.0, drug_potency: 2.0,
            drug_bandwidth: 50.0, treatment_start_gen: 5,
            treatment_pause_gens: 0, hill_coefficient: 2.0,
            quiescent_fraction: 0.05, quiescent_drug_sensitivity: 0.1,
            stem_reactivation_threshold: 0.2,
            normal_regen_rate: 0.15, relapse_threshold: 3.0,
            immune_count: 5, cells_per_entity: 1e7,
            pk_ramp_gens: 3,
            normal_qe: 30.0, normal_growth: 0.3,
            normal_resilience: 0.8, normal_dissipation: 0.01,
            normal_trophic: 0,  // producer: healthy tissue does photosynthesis
            cancer_qe: 40.0, cancer_growth: 0.9,
            cancer_resilience: 0.2, cancer_dissipation: 0.005,
            cancer_trophic: 3,  // carnivore: Warburg effect, consumes host glucose
            worlds: 100, generations: 100,
            ticks_per_gen: 300, seed: 42,
        }
    }
}

// ─── Output ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TherapySnapshot {
    pub generation: u32,
    pub cancer_alive_mean: f32,
    pub normal_alive_mean: f32,
    pub cancer_freq_mean: f32,
    pub cancer_freq_std: f32,
    pub resistance_index: f32,
    pub clonal_diversity: f32,
    pub drug_active: bool,
    pub total_drug_drain: f32,
    pub effective_potency: f32,
}

#[derive(Debug)]
pub struct TherapyReport {
    pub config: TherapyConfig,
    pub timeline: Vec<TherapySnapshot>,
    pub generations_to_resistance: Option<u32>,
    pub tumor_eliminated: bool,
    pub relapse_gen: Option<u32>,
    pub wall_time_ms: u64,
}

// ─── Pure equations ─────────────────────────────────────────────────────────

/// Hill equation dose-response. Standard pharmacology: `E = Emax × C^n / (EC50^n + C^n)`.
/// Axiom 4: energy dissipation follows thermodynamic response curve.
fn hill_response(alignment: f32, potency: f32, hill_n: f32) -> f32 {
    if alignment <= 0.0 || potency <= 0.0 { return 0.0; }
    let c_n = alignment.powf(hill_n);
    let ec50_n = 0.5f32.powf(hill_n); // EC50 at 50% alignment
    potency * c_n / (ec50_n + c_n)
}

/// Drug drain per entity. Hill equation × frequency alignment. Axiom 4+8.
fn drug_drain(entity_freq: f32, target_freq: f32, potency: f32, bandwidth: f32, hill_n: f32) -> f32 {
    let alignment = determinism::gaussian_frequency_alignment(entity_freq, target_freq, bandwidth);
    hill_response(alignment, potency, hill_n)
}

/// Pharmacokinetic ramp: potency builds up over pk_ramp_gens. Axiom 4.
fn effective_potency(generation: u32, config: &TherapyConfig) -> f32 {
    if !is_drug_active(generation, config) { return 0.0; }
    let gens_on = gens_since_drug_start(generation, config);
    if config.pk_ramp_gens == 0 { return config.drug_potency; }
    let ramp = (gens_on as f32 / config.pk_ramp_gens as f32).min(1.0);
    config.drug_potency * ramp
}

fn is_drug_active(generation: u32, config: &TherapyConfig) -> bool {
    if generation < config.treatment_start_gen { return false; }
    if config.treatment_pause_gens == 0 { return true; }
    let cycle = config.treatment_pause_gens * 2;
    ((generation - config.treatment_start_gen) % cycle) < config.treatment_pause_gens
}

fn gens_since_drug_start(generation: u32, config: &TherapyConfig) -> u32 {
    if generation < config.treatment_start_gen { return 0; }
    if config.treatment_pause_gens == 0 { return generation - config.treatment_start_gen; }
    let cycle = config.treatment_pause_gens * 2;
    (generation - config.treatment_start_gen) % cycle
}

fn is_cancer(freq: f32, config: &TherapyConfig) -> bool {
    (freq - config.cancer_freq).abs() < (freq - config.normal_freq).abs()
}

fn is_quiescent(entity: &EntitySlot) -> bool { entity.growth_bias < 0.05 }

// ─── Per-generation world operations ────────────────────────────────────────

/// Apply drug with Hill pharmacology. Returns total drain. Axiom 4+5.
fn apply_drug(world: &mut SimWorldFlat, config: &TherapyConfig, eff_potency: f32) -> f32 {
    let mut total = 0.0f32;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;

        let mut drain = drug_drain(
            world.entities[i].frequency_hz, config.drug_target_freq,
            eff_potency, config.drug_bandwidth, config.hill_coefficient,
        );

        if is_quiescent(&world.entities[i]) {
            drain *= config.quiescent_drug_sensitivity;
        }

        let loss = drain.min(world.entities[i].qe);
        world.entities[i].qe -= loss;
        total += loss;

        // Axiom 5: drained energy → nutrient grid (conservation)
        let gx = (world.entities[i].position[0] as usize).min(15);
        let gy = (world.entities[i].position[1] as usize).min(15);
        let idx = (gy * 16 + gx).min(world.nutrient_grid.len() - 1);
        world.nutrient_grid[idx] += loss;
    }
    total
}

/// Anchor cancer cell frequencies: only changes via reproduction mutation, not drift.
/// Fix #1: resistance timescale becomes biologically correct.
fn anchor_cancer_frequencies(world: &mut SimWorldFlat, config: &TherapyConfig, spawn_freqs: &[f32; MAX_ENTITIES]) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        if is_cancer(world.entities[i].frequency_hz, config) && spawn_freqs[i] != 0.0 {
            world.entities[i].frequency_hz = spawn_freqs[i];
        }
    }
}

/// Stem cell reactivation: when tumor is mostly killed, stem cells "wake up".
/// Fix #2: enables relapse. Axiom 6: niche-dependent behavior.
fn reactivate_stem_cells(world: &mut SimWorldFlat, config: &TherapyConfig) {
    let cancer_count = count_type(world, config, true);
    let threshold = (config.cancer_count as f32 * config.stem_reactivation_threshold) as u32;

    if cancer_count > threshold { return; } // bulk tumor still present, no reactivation

    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        if is_cancer(world.entities[i].frequency_hz, config) && is_quiescent(&world.entities[i]) {
            // Gradually increase growth (waking up)
            // Stem reactivation rate = normal_regen_rate (same homeostatic speed)
            let reactivation_step = config.normal_regen_rate;
            world.entities[i].growth_bias = (world.entities[i].growth_bias + reactivation_step).min(config.cancer_growth);
        }
    }
}

/// Microenvironment: nutrient modulates cancer growth. Axiom 7 (local).
fn apply_microenvironment(world: &mut SimWorldFlat, config: &TherapyConfig) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        if !is_cancer(world.entities[i].frequency_hz, config) { continue; }
        if is_quiescent(&world.entities[i]) { continue; }

        let gx = (world.entities[i].position[0] as usize).min(15);
        let gy = (world.entities[i].position[1] as usize).min(15);
        let idx = (gy * 16 + gx).min(world.nutrient_grid.len() - 1);
        let nutrient = world.nutrient_grid[idx];

        // High nutrient → high growth. Low nutrient → quiescent-like (hypoxia).
        // Nutrient baseline = normal_qe (healthy tissue energy level)
        let nutrient_factor = (nutrient / config.normal_qe.max(1.0)).clamp(0.1, 1.0);
        world.entities[i].growth_bias = config.cancer_growth * nutrient_factor;
    }
}

fn regenerate_normals(world: &mut SimWorldFlat, config: &TherapyConfig, seed: u64) {
    let current = count_type(world, config, false);
    let deficit = (config.normal_count as u32).saturating_sub(current);
    let mut s = seed;
    for _ in 0..deficit {
        s = determinism::next_u64(s);
        if determinism::unit_f32(s) > config.normal_regen_rate { continue; }
        if world.first_free_slot().is_none() { break; }
        s = determinism::next_u64(s);
        let mut slot = EntitySlot::default();
        // New regenerated cells start at half energy (recovering)
        slot.qe = config.normal_qe * config.normal_regen_rate.clamp(0.3, 0.8);
        slot.radius = 0.4;
        slot.frequency_hz = config.normal_freq + determinism::gaussian_f32(s, 8.0);
        slot.growth_bias = config.normal_growth;
        slot.resilience = config.normal_resilience;
        slot.dissipation = config.normal_dissipation;
        slot.expression_mask = [1.0; 4];
        s = determinism::next_u64(s);
        slot.position = [determinism::range_f32(s, 1.0, 15.0), determinism::range_f32(determinism::next_u64(s), 1.0, 15.0)];
        world.spawn(slot);
    }
}

fn count_type(world: &SimWorldFlat, config: &TherapyConfig, cancer: bool) -> u32 {
    let mut n = 0u32;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        if is_cancer(world.entities[i].frequency_hz, config) == cancer { n += 1; }
    }
    n
}

// ─── Snapshot ───────────────────────────────────────────────────────────────

fn compute_snapshot(worlds: &[SimWorldFlat], generation: u32, config: &TherapyConfig, drain: f32, eff_pot: f32) -> TherapySnapshot {
    let nw = worlds.len().max(1) as f32;
    let (mut tc, mut tn, mut fsum, mut fsq, mut fcount, mut div) = (0u32, 0u32, 0.0f32, 0.0f32, 0u32, 0.0f32);

    for w in worlds {
        let mut mask = w.alive_mask;
        let mut wf = [0.0f32; MAX_ENTITIES];
        let mut wc = 0usize;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            let f = w.entities[i].frequency_hz;
            if is_cancer(f, config) {
                tc += 1; fsum += f; fsq += f * f; fcount += 1;
                if wc < MAX_ENTITIES { wf[wc] = f; wc += 1; }
            } else { tn += 1; }
        }
        if wc >= 2 {
            let (ps, pc) = (0..wc).flat_map(|a| ((a+1)..wc).map(move |b| (a,b)))
                .fold((0.0f32, 0u32), |(s,n),(a,b)| (s + (wf[a]-wf[b]).abs(), n+1));
            div += if pc > 0 { ps / pc as f32 } else { 0.0 };
        }
    }

    let cf = fcount.max(1) as f32;
    let fm = fsum / cf;
    let fs = ((fsq / cf) - fm * fm).max(0.0).sqrt();
    let ri = (fm - config.drug_target_freq).abs() / config.drug_bandwidth.max(1.0);
    let has = fcount > 0;

    TherapySnapshot {
        generation, drug_active: is_drug_active(generation, config),
        cancer_alive_mean: tc as f32 / nw, normal_alive_mean: tn as f32 / nw,
        cancer_freq_mean: if has { fm } else { 0.0 }, cancer_freq_std: if has { fs } else { 0.0 },
        resistance_index: if has { ri } else { 0.0 },
        clonal_diversity: div / nw, total_drug_drain: drain, effective_potency: eff_pot,
    }
}

// ─── Main HOF ───────────────────────────────────────────────────────────────

pub fn run(config: &TherapyConfig) -> TherapyReport {
    let start = Instant::now();

    let mut worlds: Vec<SimWorldFlat> = (0..config.worlds).map(|wi| {
        let ws = determinism::next_u64(config.seed ^ (wi as u64));
        let mut w = SimWorldFlat::new(ws, 0.05);
        spawn_population(&mut w, config, ws);
        w
    }).collect();

    // Record spawn frequencies for anchoring (Fix #1)
    let spawn_freqs: Vec<[f32; MAX_ENTITIES]> = worlds.iter().map(|w| {
        let mut sf = [0.0f32; MAX_ENTITIES];
        let mut mask = w.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            if is_cancer(w.entities[i].frequency_hz, config) {
                sf[i] = w.entities[i].frequency_hz;
            }
        }
        sf
    }).collect();

    let mut scratches: Vec<ScratchPad> = (0..config.worlds).map(|_| ScratchPad::new()).collect();
    let mut timeline = Vec::with_capacity(config.generations as usize);

    for g in 0..config.generations {
        let eff_pot = effective_potency(g, config);
        let mut gen_drain = 0.0f32;

        for (wi, world) in worlds.iter_mut().enumerate() {
            // Drug applied every tick at full potency (continuous infusion model).
            // Potency = qe drain per tick per entity at full alignment.
            for _ in 0..config.ticks_per_gen {
                if eff_pot > 0.0 { gen_drain += apply_drug(world, config, eff_pot); }
                world.tick(&mut scratches[wi]);
            }
            apply_microenvironment(world, config);
            reactivate_stem_cells(world, config);
            regenerate_normals(world, config, world.seed ^ world.tick_id ^ (g as u64));
            anchor_cancer_frequencies(world, config, &spawn_freqs[wi]);
        }

        timeline.push(compute_snapshot(&worlds, g, config, gen_drain, eff_pot));
    }

    let generations_to_resistance = timeline.iter()
        .find(|s| s.resistance_index > 1.0 && s.cancer_alive_mean > 1.0)
        .map(|s| s.generation);
    let min_cancer = timeline.iter().map(|s| s.cancer_alive_mean).fold(f32::MAX, f32::min);
    let tumor_eliminated = min_cancer < 1.0;
    let relapse_gen = if tumor_eliminated {
        let min_gen = timeline.iter()
            .min_by(|a,b| a.cancer_alive_mean.partial_cmp(&b.cancer_alive_mean).unwrap_or(std::cmp::Ordering::Equal))
            .map(|s| s.generation).unwrap_or(0);
        timeline.iter().find(|s| s.generation > min_gen + 5 && s.cancer_alive_mean > config.relapse_threshold)
            .map(|s| s.generation)
    } else { None };

    TherapyReport {
        config: config.clone(), timeline, generations_to_resistance,
        tumor_eliminated, relapse_gen,
        wall_time_ms: start.elapsed().as_millis() as u64,
    }
}

// ─── Spawn ──────────────────────────────────────────────────────────────────

fn spawn_population(world: &mut SimWorldFlat, config: &TherapyConfig, seed: u64) {
    let mut s = seed;
    /// Spawn a single cell. Pure factory: config → EntitySlot. No side effects.
    let spawn_cell = |s: &mut u64, freq: f32, freq_sigma: f32, qe: f32, growth: f32,
                      resilience: f32, dissipation: f32, trophic: u8, pos_range: (f32, f32)| -> EntitySlot {
        *s = determinism::next_u64(*s);
        let mut slot = EntitySlot::default();
        slot.qe = qe;
        slot.radius = 0.5;
        slot.frequency_hz = freq + determinism::gaussian_f32(*s, freq_sigma);
        slot.growth_bias = growth;
        slot.resilience = resilience;
        slot.dissipation = dissipation;
        slot.trophic_class = trophic;
        slot.expression_mask = [1.0; 4];
        *s = determinism::next_u64(*s);
        slot.position = [determinism::range_f32(*s, pos_range.0, pos_range.1),
                         determinism::range_f32(determinism::next_u64(*s), pos_range.0, pos_range.1)];
        slot
    };

    // Healthy cells: trophic from config (default: producer=photosynthesis)
    for _ in 0..config.normal_count {
        let slot = spawn_cell(&mut s, config.normal_freq, 10.0, config.normal_qe,
            config.normal_growth, config.normal_resilience, config.normal_dissipation,
            config.normal_trophic, (1.0, 15.0));
        world.spawn(slot);
    }

    // Active cancer cells: trophic from config (default: carnivore=Warburg, no photosynthesis)
    let quiescent_n = (config.cancer_count as f32 * config.quiescent_fraction) as u8;
    for _ in 0..(config.cancer_count - quiescent_n) {
        let slot = spawn_cell(&mut s, config.cancer_freq, 15.0, config.cancer_qe,
            config.cancer_growth, config.cancer_resilience, config.cancer_dissipation,
            config.cancer_trophic, (5.0, 11.0));
        world.spawn(slot);
    }

    // Quiescent stem cells: same trophic as active cancer (Warburg), but dormant.
    for _ in 0..quiescent_n {
        let slot = spawn_cell(&mut s, config.cancer_freq, 5.0,
            config.cancer_qe * config.quiescent_drug_sensitivity,
            0.01, config.normal_resilience, config.cancer_dissipation * 0.2,
            config.cancer_trophic, (6.0, 10.0));
        world.spawn(slot);
    }

    // Immune cells: carnivore (attack by predation). Not photosynthetic.
    for _ in 0..config.immune_count {
        let mut slot = spawn_cell(&mut s, config.cancer_freq, 20.0,
            config.normal_qe * 0.8, config.normal_growth * 0.3,
            config.normal_resilience * 0.9, config.normal_dissipation * 2.0,
            3, (1.0, 15.0)); // trophic=3=carnivore (immune killer)
        slot.mobility_bias = config.cancer_growth * 0.9;
        world.spawn(slot);
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn hill_response_zero_alignment() { assert_eq!(hill_response(0.0, 2.0, 2.0), 0.0); }
    #[test] fn hill_response_full_alignment() {
        let h = hill_response(1.0, 2.0, 2.0);
        // Hill: 1^2 / (0.5^2 + 1^2) = 0.8 → potency × 0.8 = 1.6
        assert!(h > 1.5 && h < 2.0, "full alignment near max: {h}");
    }
    #[test] fn hill_response_half_alignment() {
        let h = hill_response(0.5, 2.0, 1.0);
        assert!(h > 0.5 && h < 1.8, "half alignment → partial effect: {h}");
    }
    #[test] fn hill_sigmoidal_steeper() {
        let h1 = hill_response(0.3, 2.0, 1.0);
        let h2 = hill_response(0.3, 2.0, 4.0);
        assert!(h2 < h1, "steeper Hill → less effect at low alignment");
    }

    #[test] fn drain_on_target() { assert!(drug_drain(400.0, 400.0, 2.0, 50.0, 2.0) > 1.5); }
    #[test] fn drain_off_target() { assert!(drug_drain(600.0, 400.0, 2.0, 50.0, 2.0) < drug_drain(400.0, 400.0, 2.0, 50.0, 2.0)); }
    #[test] fn drain_nan_safe() { assert_eq!(drug_drain(f32::NAN, 400.0, 2.0, 50.0, 2.0), 0.0); }

    #[test] fn pk_ramp_gradual() {
        let c = TherapyConfig { treatment_start_gen: 0, pk_ramp_gens: 10, ..Default::default() };
        let p0 = effective_potency(0, &c);
        let p5 = effective_potency(5, &c);
        let p10 = effective_potency(10, &c);
        assert!(p0 < p5, "potency increases: {p0} < {p5}");
        assert!(p5 < p10, "potency increases: {p5} < {p10}");
        assert!((p10 - c.drug_potency).abs() < 0.01, "full potency at ramp end");
    }

    #[test] fn drug_before_start() { assert!(!is_drug_active(4, &TherapyConfig { treatment_start_gen: 5, ..Default::default() })); }
    #[test] fn drug_continuous() { assert!(is_drug_active(50, &TherapyConfig { treatment_start_gen: 0, treatment_pause_gens: 0, ..Default::default() })); }
    #[test] fn drug_intermittent() {
        let c = TherapyConfig { treatment_start_gen: 0, treatment_pause_gens: 3, ..Default::default() };
        assert!(is_drug_active(0, &c)); assert!(!is_drug_active(3, &c)); assert!(is_drug_active(6, &c));
    }

    #[test] fn quiescent_low_growth() { let mut e = EntitySlot::default(); e.growth_bias = 0.01; assert!(is_quiescent(&e)); }
    #[test] fn active_high_growth() { let mut e = EntitySlot::default(); e.growth_bias = 0.9; assert!(!is_quiescent(&e)); }

    #[test] fn run_no_panic() { assert_eq!(run(&TherapyConfig { worlds: 2, generations: 3, ticks_per_gen: 20, ..Default::default() }).timeline.len(), 3); }
    #[test] fn run_deterministic() {
        let c = TherapyConfig { worlds: 2, generations: 5, ticks_per_gen: 20, ..Default::default() };
        let (a, b) = (run(&c), run(&c));
        for i in 0..5 { assert_eq!(a.timeline[i].cancer_alive_mean.to_bits(), b.timeline[i].cancer_alive_mean.to_bits()); }
    }

    #[test] fn drug_reduces_cancer() {
        let no = run(&TherapyConfig { worlds: 10, generations: 15, ticks_per_gen: 50, treatment_start_gen: 999, ..Default::default() });
        let yes = run(&TherapyConfig { worlds: 10, generations: 15, ticks_per_gen: 50, treatment_start_gen: 0, drug_potency: 5.0, ..Default::default() });
        assert!(yes.timeline.last().unwrap().cancer_alive_mean <= no.timeline.last().unwrap().cancer_alive_mean);
    }

    #[test] fn conservation_tracked() {
        let r = run(&TherapyConfig { worlds: 3, generations: 10, ticks_per_gen: 50, treatment_start_gen: 0, ..Default::default() });
        assert!(r.timeline.iter().any(|s| s.total_drug_drain > 0.0));
    }

    #[test] fn effective_potency_tracked() {
        let r = run(&TherapyConfig { worlds: 2, generations: 10, ticks_per_gen: 20, pk_ramp_gens: 5, ..Default::default() });
        let pot_early = r.timeline[5].effective_potency;
        let pot_late = r.timeline[9].effective_potency;
        assert!(pot_late >= pot_early);
    }

    #[test] fn cancer_trophic_not_producer() {
        // Cancer cells must NOT be trophic=0 (producer) to avoid photosynthesis.
        // Warburg effect: tumors consume glucose, they don't photosynthesize.
        let config = TherapyConfig::default();
        assert_eq!(config.cancer_trophic, 3, "cancer must be carnivore (Warburg)");
        assert_eq!(config.normal_trophic, 0, "normals are producers (healthy tissue)");
    }

    #[test] fn cancer_cells_spawned_as_carnivore() {
        let config = TherapyConfig::default();
        let mut w = SimWorldFlat::new(42, 0.05);
        spawn_population(&mut w, &config, 42);
        let mut mask = w.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            if is_cancer(w.entities[i].frequency_hz, &config) {
                assert_eq!(w.entities[i].trophic_class, 3,
                    "cancer cell at slot {i} must be carnivore, got {}", w.entities[i].trophic_class);
            }
        }
    }

    #[test] fn snapshot_empty() {
        let s = compute_snapshot(&[SimWorldFlat::new(42, 0.05)], 0, &TherapyConfig::default(), 0.0, 0.0);
        assert_eq!(s.cancer_alive_mean, 0.0);
    }
}
