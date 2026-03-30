//! Phase::MorphologicalLayer batch systems — senescence, growth, reproduction,
//! abiogenesis, morphological adaptation.

use crate::batch::arena::{EntitySlot, SimWorldFlat};
use crate::batch::constants::*;
use crate::batch::genome::GenomeBlob;
use crate::batch::systems::thermodynamic::grid_cell;
use crate::blueprint::{constants, equations};
use crate::blueprint::equations::determinism;
use crate::blueprint::equations::emergence::senescence as senescence_eq;
use crate::blueprint::equations::codon_genome;
use crate::blueprint::equations::variable_genome;

/// Age-dependent dissipation: older entities lose energy faster.
///
/// Calls `equations::age_dependent_dissipation(base, tick_age, coeff)`.
/// Age is approximated as `world.tick_id - entity_id` (lower id = older).
pub fn senescence(world: &mut SimWorldFlat) {
    let tick = world.tick_id;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        let age = tick.saturating_sub(e.entity_id as u64);
        let rate = senescence_eq::age_dependent_dissipation(
            e.dissipation, age, SENESCENCE_COEFF,
        );
        let loss = (e.qe * rate).min(e.qe);
        if loss > 0.0 { e.qe -= loss; }
    }
}

/// Growth inference: radius grows toward max via logistic curve.
///
/// Calls `equations::allometric_radius(r0, r_max, k, 1)`.
/// `r_max` = `growth_bias * MAX_ALLOMETRIC_RADIUS`.
pub fn growth_inference(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        if e.growth_bias <= 0.0 || e.qe <= 0.0 { continue; }
        let r_max = e.growth_bias * MAX_ALLOMETRIC_RADIUS;
        if e.radius >= r_max { continue; }
        let new_r = equations::allometric_radius(e.radius, r_max, GROWTH_RATE_K, 1);
        if e.radius != new_r { e.radius = new_r; }
    }
}

/// Asteroid impact: periodic catastrophic dissipation in a localized area.
///
/// Axiom 4: extreme dissipation event. Axiom 7: localized by distance.
/// Opens ecological niches for new species (Axiom 6: emergence from extinction).
pub fn asteroid_impact(world: &mut SimWorldFlat) {
    if ASTEROID_INTERVAL == 0 || world.tick_id % ASTEROID_INTERVAL != 0 { return; }
    let rng = determinism::next_u64(world.seed ^ world.tick_id ^ 0xA57E);
    let impact = [
        determinism::range_f32(rng, 0.0, GRID_SIDE as f32),
        determinism::range_f32(determinism::next_u64(rng), 0.0, GRID_SIDE as f32),
    ];
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let dx = world.entities[i].position[0] - impact[0];
        let dy = world.entities[i].position[1] - impact[1];
        if dx * dx + dy * dy < ASTEROID_RADIUS_SQ {
            world.entities[i].qe *= ASTEROID_SURVIVAL_FRACTION;
        }
    }
    // Devastate nutrient grid near impact
    for cell in 0..GRID_CELLS {
        let cx = (cell % GRID_SIDE) as f32;
        let cy = (cell / GRID_SIDE) as f32;
        let dx = cx - impact[0];
        let dy = cy - impact[1];
        if dx * dx + dy * dy < ASTEROID_RADIUS_SQ {
            world.nutrient_grid[cell] *= ASTEROID_SURVIVAL_FRACTION;
        }
    }
}

/// Death reap: mark and kill entities below QE_MIN_EXISTENCE.
///
/// Returns nutrients to grid (DEATH_NUTRIENT_RETURN fraction).
pub fn death_reap(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        if world.entities[i].qe >= QE_MIN_EXISTENCE { continue; }
        // Return fraction of remaining qe to nutrient grid
        let cell = grid_cell(world.entities[i].position);
        if cell < GRID_CELLS {
            world.nutrient_grid[cell] += world.entities[i].qe * DEATH_NUTRIENT_RETURN;
        }
        world.events.record_death(i as u8);
        world.kill(i);
    }
}

/// Reproduction: entities above threshold spawn offspring with mutated genome.
///
/// Energy transfer: parent loses REPRODUCTION_TRANSFER_FRACTION, child receives it.
/// Genome: inherited with gaussian mutation (DEFAULT_MUTATION_SIGMA).
/// Position: near parent (within 2× radius).
pub fn reproduction(world: &mut SimWorldFlat) {
    // Collect reproducers first (avoid borrowing issues during spawn)
    let mut repro_list = [(0u8, 0u64); MAX_ENTITIES];
    let mut repro_count = 0usize;

    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        if world.entities[i].qe >= REPRODUCTION_THRESHOLD && repro_count < MAX_ENTITIES {
            let rng = determinism::next_u64(world.seed ^ world.tick_id ^ (i as u64));
            repro_list[repro_count] = (i as u8, rng);
            repro_count += 1;
        }
    }

    for r in 0..repro_count {
        let (pi, rng) = repro_list[r];
        let parent_idx = pi as usize;
        if world.alive_mask & (1 << parent_idx) == 0 { continue; }

        let Some(child_idx) = world.first_free_slot() else { continue; };

        // Mutate via VariableGenome (supports gene duplication/deletion)
        let parent_vg = &world.genomes[parent_idx];
        let child_vg = variable_genome::mutate_variable(parent_vg, rng);

        // Also produce classic GenomeBlob for EntitySlot (backward compatible)
        let (child_biases, _sigma) = variable_genome::to_genome_blob_biases(&child_vg);
        let parent_genome = GenomeBlob::from_slot(&world.entities[parent_idx]);
        let child_genome = GenomeBlob {
            growth_bias: child_biases[0],
            mobility_bias: child_biases[1],
            branching_bias: child_biases[2],
            resilience: child_biases[3],
            ..parent_genome
        };

        let transfer = world.entities[parent_idx].qe * REPRODUCTION_TRANSFER_FRACTION;
        world.entities[parent_idx].qe -= transfer;

        let parent_pos = world.entities[parent_idx].position;
        let parent_radius = world.entities[parent_idx].radius;

        let mut child = EntitySlot::default();
        child.alive = true;
        child.entity_id = world.next_id;
        world.next_id += 1;
        child_genome.apply(&mut child);
        child.qe = transfer;
        child.radius = ABIOGENESIS_INITIAL_RADIUS;
        child.dissipation = world.entities[parent_idx].dissipation;
        child.frequency_hz = world.entities[parent_idx].frequency_hz;
        child.position = [
            parent_pos[0] + determinism::unit_f32(rng) * parent_radius * 2.0 - parent_radius,
            parent_pos[1] + determinism::unit_f32(determinism::next_u64(rng)) * parent_radius * 2.0 - parent_radius,
        ];

        world.entities[child_idx] = child;
        world.genomes[child_idx] = child_vg;
        // PD-5: Propagate codon genome + code table with mutation
        let parent_cg = world.codon_genomes[parent_idx];
        let parent_ct = world.codon_tables[parent_idx];
        world.codon_genomes[child_idx] = codon_genome::mutate_codon(&parent_cg, rng);
        world.codon_tables[child_idx] = codon_genome::mutate_table(&parent_ct, determinism::next_u64(rng));
        world.alive_mask |= 1 << child_idx;
        world.entity_count += 1;
        world.events.record_reproduction(pi, child_idx as u8);
    }
}

/// Abiogenesis: spontaneous cell generation when population is low and energy is high.
pub fn abiogenesis(world: &mut SimWorldFlat) {
    if world.entity_count >= ABIOGENESIS_POP_CAP { return; }
    let grid_energy: f32 = world.irradiance_grid.iter().sum();
    if grid_energy < ABIOGENESIS_ENERGY_THRESHOLD { return; }

    let Some(idx) = world.first_free_slot() else { return; };
    let rng = determinism::next_u64(world.seed ^ world.tick_id ^ 0xAB10);

    let mut cell = EntitySlot::default();
    cell.alive = true;
    cell.archetype = 3; // cell
    cell.entity_id = world.next_id;
    world.next_id += 1;
    cell.qe = ABIOGENESIS_INITIAL_QE;
    cell.radius = ABIOGENESIS_INITIAL_RADIUS;
    cell.frequency_hz = determinism::range_f32(rng, ABIOGENESIS_FREQ_MIN, ABIOGENESIS_FREQ_MAX);
    cell.growth_bias = determinism::unit_f32(determinism::next_u64(rng));
    cell.resilience = ABIOGENESIS_DEFAULT_RESILIENCE;
    cell.trophic_class = 0; // primary producer
    cell.dissipation = ABIOGENESIS_DEFAULT_DISSIPATION;
    let s1 = determinism::next_u64(determinism::next_u64(rng));
    let s2 = determinism::next_u64(s1);
    cell.position = [
        determinism::range_f32(s1, 0.0, GRID_SIDE as f32),
        determinism::range_f32(s2, 0.0, GRID_SIDE as f32),
    ];

    world.entities[idx] = cell;
    world.genomes[idx] = variable_genome::VariableGenome::from_biases(
        cell.growth_bias, cell.mobility_bias, cell.branching_bias, cell.resilience,
    );
    world.codon_genomes[idx] = codon_genome::CodonGenome::from_seed(
        determinism::next_u64(rng ^ 0xCD),
    );
    world.codon_tables[idx] = codon_genome::CodonTable::default();
    world.alive_mask |= 1 << idx;
    world.entity_count += 1;
}

/// Morpho adaptation: Bergmann (cold → grow) + Wolff (movement → strengthen bonds).
///
/// Calls `equations::bergmann_radius_pressure` and `equations::use_driven_bone_density`.
pub fn morpho_adaptation(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        if e.radius <= 0.0 { continue; }

        // Bergmann: compute equivalent temperature, apply growth pressure
        let density = equations::density(e.qe, e.radius);
        let temp = equations::equivalent_temperature(density);
        let bergmann = equations::bergmann_radius_pressure(
            temp, constants::MORPHO_TARGET_TEMPERATURE,
        );
        let new_growth = (e.growth_bias + bergmann * constants::MORPHO_ADAPTATION_RATE).clamp(0.0, 1.0);
        if e.growth_bias != new_growth { e.growth_bias = new_growth; }

        // Wolff: use-driven bone density from movement speed
        let speed_sq = e.velocity[0] * e.velocity[0] + e.velocity[1] * e.velocity[1];
        let load = speed_sq.sqrt(); // speed as load proxy
        let new_bond = equations::use_driven_bone_density(load, e.bond_energy);
        if e.bond_energy != new_bond { e.bond_energy = new_bond; }
    }
}

/// Senescence coefficient — rate of age-dependent drain increase.
const SENESCENCE_COEFF: f32 = 0.0001;

/// Maximum radius an entity can grow to (scaled by growth_bias).
const MAX_ALLOMETRIC_RADIUS: f32 = 3.0;

/// Growth rate constant for logistic curve (per tick).
const GROWTH_RATE_K: f32 = 0.01;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::EntitySlot;

    fn spawn(w: &mut SimWorldFlat, qe: f32, growth: f32) -> usize {
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.radius = 0.5;
        e.dissipation = 0.01;
        e.growth_bias = growth;
        w.spawn(e).unwrap()
    }

    // ── senescence ──────────────────────────────────────────────────────────

    #[test]
    fn senescence_drains_old_entities_more() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let young = spawn(&mut w, 100.0, 0.0);
        let old = spawn(&mut w, 100.0, 0.0);
        // Simulate age by advancing tick_id
        w.tick_id = 1000;
        w.entities[old].entity_id = 0;    // spawned at tick 0 → age 1000
        w.entities[young].entity_id = 999; // spawned at tick 999 → age 1
        senescence(&mut w);
        assert!(
            w.entities[old].qe < w.entities[young].qe,
            "old entity should lose more: old={} young={}",
            w.entities[old].qe, w.entities[young].qe,
        );
    }

    #[test]
    fn senescence_never_negative() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 0.02, 0.0);
        w.tick_id = 100_000;
        w.entities[idx].entity_id = 0;
        senescence(&mut w);
        assert!(w.entities[idx].qe >= 0.0);
    }

    // ── growth_inference ────────────────────────────────────────────────────

    #[test]
    fn growth_increases_radius() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0, 0.8);
        let before = w.entities[idx].radius;
        growth_inference(&mut w);
        assert!(w.entities[idx].radius > before, "radius should grow");
    }

    #[test]
    fn growth_approaches_max() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0, 1.0);
        for _ in 0..10_000 {
            growth_inference(&mut w);
        }
        let r_max = 1.0 * MAX_ALLOMETRIC_RADIUS;
        assert!(
            (w.entities[idx].radius - r_max).abs() < 0.1,
            "radius={} should approach r_max={r_max}",
            w.entities[idx].radius,
        );
    }

    #[test]
    fn growth_zero_bias_no_change() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0, 0.0);
        growth_inference(&mut w);
        assert_eq!(w.entities[idx].radius, 0.5);
    }

    #[test]
    fn growth_does_not_exceed_max() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0, 0.5);
        w.entities[idx].radius = 0.5 * MAX_ALLOMETRIC_RADIUS; // already at max
        let before = w.entities[idx].radius;
        growth_inference(&mut w);
        assert_eq!(w.entities[idx].radius, before, "should not grow past max");
    }

    // ── death_reap ──────────────────────────────────────────────────────────

    #[test]
    fn death_reap_kills_starved() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 0.001, 0.0); // below QE_MIN_EXISTENCE
        assert_eq!(w.entity_count, 1);
        death_reap(&mut w);
        assert_eq!(w.entity_count, 0);
        assert!(w.alive_mask & (1 << idx) == 0);
    }

    #[test]
    fn death_reap_returns_nutrients() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 0.005, 0.0);
        w.entities[idx].position = [3.0, 3.0];
        let cell = grid_cell([3.0, 3.0]);
        let grid_before = w.nutrient_grid[cell];
        death_reap(&mut w);
        assert!(w.nutrient_grid[cell] > grid_before, "nutrients should return to grid");
    }

    #[test]
    fn death_reap_spares_healthy() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn(&mut w, 100.0, 0.0);
        death_reap(&mut w);
        assert_eq!(w.entity_count, 1, "healthy entity should survive");
    }

    // ── reproduction ────────────────────────────────────────────────────────

    #[test]
    fn reproduction_spawns_child() {
        let mut w = SimWorldFlat::new(42, 0.05);
        let parent = spawn(&mut w, 100.0, 0.5);
        w.entities[parent].archetype = 2;
        w.entities[parent].trophic_class = 1;
        w.entities[parent].frequency_hz = 440.0;
        w.entities[parent].dissipation = 0.01;
        assert_eq!(w.entity_count, 1);
        reproduction(&mut w);
        assert_eq!(w.entity_count, 2, "child should be spawned");
    }

    #[test]
    fn reproduction_transfers_energy() {
        let mut w = SimWorldFlat::new(42, 0.05);
        let parent = spawn(&mut w, 100.0, 0.5);
        w.entities[parent].archetype = 2;
        w.entities[parent].frequency_hz = 440.0;
        w.entities[parent].dissipation = 0.01;
        let total_before = w.entities[parent].qe;
        reproduction(&mut w);
        let parent_after = w.entities[parent].qe;
        let child_qe = w.entities[1].qe;
        assert!(parent_after < total_before, "parent should lose energy");
        assert!(child_qe > 0.0, "child should have energy");
        assert!((parent_after + child_qe - total_before).abs() < 1e-4, "energy conserved");
    }

    #[test]
    fn reproduction_inherits_genome_with_mutation() {
        let mut w = SimWorldFlat::new(42, 0.05);
        let parent = spawn(&mut w, 100.0, 0.8);
        w.entities[parent].archetype = 2;
        w.entities[parent].mobility_bias = 0.6;
        w.entities[parent].frequency_hz = 440.0;
        w.entities[parent].dissipation = 0.01;
        reproduction(&mut w);
        // Child genome should be close to parent but not identical
        let child = &w.entities[1];
        // With VariableGenome mutation, effective biases may shift more than classic mutation
        // due to gene duplication + modulation. Allow wider tolerance.
        assert!((child.growth_bias - 0.8).abs() < 0.5, "growth_bias in range of parent");
        assert_eq!(child.archetype, 2, "archetype inherited");
    }

    #[test]
    fn reproduction_skips_low_energy() {
        let mut w = SimWorldFlat::new(42, 0.05);
        spawn(&mut w, 10.0, 0.5); // below REPRODUCTION_THRESHOLD
        reproduction(&mut w);
        assert_eq!(w.entity_count, 1, "should not reproduce");
    }

    // ── abiogenesis ─────────────────────────────────────────────────────────

    #[test]
    fn abiogenesis_spawns_with_high_irradiance() {
        let mut w = SimWorldFlat::new(42, 0.05);
        // Fill irradiance grid above threshold
        for cell in &mut w.irradiance_grid { *cell = 10.0; }
        assert_eq!(w.entity_count, 0);
        abiogenesis(&mut w);
        assert_eq!(w.entity_count, 1, "should spawn a cell");
        assert_eq!(w.entities[0].archetype, 3, "should be a cell");
        assert!(w.entities[0].qe > 0.0);
    }

    #[test]
    fn abiogenesis_suppressed_at_pop_cap() {
        let mut w = SimWorldFlat::new(42, 0.05);
        for cell in &mut w.irradiance_grid { *cell = 10.0; }
        // Fill to cap
        for _ in 0..ABIOGENESIS_POP_CAP {
            let mut e = EntitySlot::default();
            e.qe = 10.0;
            w.spawn(e);
        }
        let count_before = w.entity_count;
        abiogenesis(&mut w);
        assert_eq!(w.entity_count, count_before, "should not spawn at cap");
    }

    #[test]
    fn abiogenesis_suppressed_with_low_irradiance() {
        let mut w = SimWorldFlat::new(42, 0.05);
        // Grid empty (default 0.0) → below threshold
        abiogenesis(&mut w);
        assert_eq!(w.entity_count, 0, "should not spawn without energy");
    }

    // ── morpho_adaptation ───────────────────────────────────────────────────

    #[test]
    fn morpho_adaptation_cold_increases_growth() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 0.1, 0.5); // very low qe → low density → low temp
        w.entities[idx].radius = 5.0;
        w.entities[idx].bond_energy = 100.0;
        let before = w.entities[idx].growth_bias;
        morpho_adaptation(&mut w);
        // Low temp → Bergmann → growth_bias should increase
        assert!(
            w.entities[idx].growth_bias >= before,
            "cold → growth_bias should not decrease: {} → {}",
            before, w.entities[idx].growth_bias,
        );
    }

    #[test]
    fn morpho_adaptation_movement_affects_bonds() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0, 0.5);
        w.entities[idx].velocity = [5.0, 5.0]; // moving
        w.entities[idx].bond_energy = 100.0;
        let before = w.entities[idx].bond_energy;
        morpho_adaptation(&mut w);
        // Movement → Wolff → bond_energy changes
        assert_ne!(w.entities[idx].bond_energy, before, "movement should affect bonds");
    }
}
