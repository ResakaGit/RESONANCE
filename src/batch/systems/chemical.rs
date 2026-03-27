//! Phase::ChemicalLayer batch systems — homeostasis, state transitions,
//! photosynthesis, nutrient uptake.

use crate::batch::arena::SimWorldFlat;
use crate::batch::constants::*;
use crate::batch::systems::thermodynamic::grid_cell;
use crate::blueprint::constants;
use crate::blueprint::equations;

/// L12 Homeostasis: frequency adaptation with energy cost.
///
/// Calls `equations::thermoregulation_cost` to compute metabolic drain.
pub fn homeostasis(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        if e.adapt_rate_hz <= 0.0 { continue; }
        let cost = equations::thermoregulation_cost(
            e.frequency_hz,                  // t_core proxy
            constants::ENDOTHERM_TARGET_TEMP, // t_env (simplified: use default)
            e.qe,                            // mass proxy
            e.conductivity,
            constants::INSULATION_BASE,
        );
        let drain = cost.min(e.qe);
        if drain > 0.0 { e.qe -= drain; }
    }
}

/// L4 MatterCoherence: phase transitions based on equivalent temperature.
///
/// Calls `equations::equivalent_temperature(density)` then threshold comparison.
pub fn state_transitions(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        if e.radius <= 0.0 { continue; }
        let density = equations::density(e.qe, e.radius);
        let temp = equations::equivalent_temperature(density);
        let new_state = temp_to_matter_state(temp, e.bond_energy);
        if e.matter_state != new_state { e.matter_state = new_state; }
    }
}

/// Threshold-based state classification matching `equations::state_from_temperature`.
///
/// Uses same constants as the Bevy system. Returns u8 encoding:
/// 0=Solid, 1=Liquid, 2=Gas, 3=Plasma.
fn temp_to_matter_state(temp: f32, bond_energy: f32) -> u8 {
    let threshold = bond_energy * constants::GAME_BOLTZMANN;
    if temp < threshold * constants::SOLID_TRANSITION { 0 }
    else if temp < threshold * constants::LIQUID_TRANSITION { 1 }
    else if temp < threshold * constants::GAS_TRANSITION { 2 }
    else { 3 }
}

/// Photosynthesis: entities absorb energy from irradiance grid.
///
/// Lux-band entities (frequency 400–700 Hz) get proportional intake.
pub fn photosynthesis(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        // Axiom 8: photosynthesis = resonance with solar frequency.
        // Axiom 7: attenuation with frequency distance.
        // resonance = exp(-Δf² / (2 × bandwidth²)) — Gaussian around SOLAR_FREQUENCY.
        let delta_f = (e.frequency_hz - SOLAR_FREQUENCY).abs();
        let solar_resonance = (-delta_f * delta_f / (2.0 * 200.0 * 200.0)).exp();
        if solar_resonance < SOLAR_RESONANCE_MIN { continue; }
        let cell = grid_cell(e.position);
        if cell >= GRID_CELLS { continue; }
        let irr = world.irradiance_grid[cell];
        if irr <= 0.0 { continue; }
        // Axiom 3: gain = base × interference_factor.
        let area = e.radius * e.radius;
        let gain = irr * area * PHOTOSYNTHESIS_EFFICIENCY * solar_resonance;
        e.qe += gain;
        // Axiom 5: producers enrich soil via nutrient cycling.
        world.nutrient_grid[cell] += gain * NUTRIENT_DEPOSIT_FRACTION;
    }
}

/// Nutrient uptake: entities absorb from nutrient grid, depleting it.
///
/// Entities with radius > 0 extract nutrients at their grid cell.
pub fn nutrient_uptake(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &world.entities[i];
        if e.radius <= 0.0 { continue; }
        let cell = grid_cell(e.position);
        if cell >= GRID_CELLS { continue; }
        let available = world.nutrient_grid[cell];
        if available <= 0.0 { continue; }
        let extract = (e.radius * NUTRIENT_UPTAKE_RATE).min(available);
        // Must re-borrow mutably in separate scope
        world.nutrient_grid[cell] -= extract;
        world.entities[i].qe += extract;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::EntitySlot;

    fn flora(w: &mut SimWorldFlat, qe: f32, pos: [f32; 2]) -> usize {
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.radius = 1.0;
        e.position = pos;
        e.archetype = 1; // flora
        e.trophic_class = 0; // primary producer
        e.bond_energy = 100.0;
        e.conductivity = 0.1;
        e.adapt_rate_hz = 0.1;
        e.frequency_hz = 500.0;
        w.spawn(e).unwrap()
    }

    // ── homeostasis ─────────────────────────────────────────────────────────

    #[test]
    fn homeostasis_drains_energy() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = flora(&mut w, 100.0, [0.0, 0.0]);
        let before = w.entities[idx].qe;
        homeostasis(&mut w);
        assert!(w.entities[idx].qe <= before);
    }

    #[test]
    fn homeostasis_skips_zero_adapt_rate() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = flora(&mut w, 100.0, [0.0, 0.0]);
        w.entities[idx].adapt_rate_hz = 0.0;
        homeostasis(&mut w);
        assert_eq!(w.entities[idx].qe, 100.0);
    }

    // ── state_transitions ───────────────────────────────────────────────────

    #[test]
    fn state_transitions_low_temp_is_solid() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = flora(&mut w, 0.01, [0.0, 0.0]);
        w.entities[idx].radius = 5.0;  // low qe + large radius → low density → low temp
        w.entities[idx].bond_energy = 5000.0; // high bond → high threshold
        state_transitions(&mut w);
        // temp = density / 1.0 ≈ tiny, threshold = 5000 * 0.3 = 1500 → solid
        assert_eq!(w.entities[idx].matter_state, 0, "low temp → solid");
    }

    #[test]
    fn state_transitions_high_temp_is_plasma() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = flora(&mut w, 1000.0, [0.0, 0.0]);
        w.entities[idx].radius = 0.1;  // high qe + small radius → high density → high temp
        w.entities[idx].bond_energy = 1.0; // low bond → low threshold
        state_transitions(&mut w);
        assert_eq!(w.entities[idx].matter_state, 3, "high temp → plasma");
    }

    #[test]
    fn temp_to_matter_state_thresholds() {
        assert_eq!(temp_to_matter_state(0.0, 100.0), 0);   // Solid
        assert_eq!(temp_to_matter_state(50.0, 100.0), 1);   // Liquid
        assert_eq!(temp_to_matter_state(200.0, 100.0), 2);  // Gas
        assert_eq!(temp_to_matter_state(500.0, 100.0), 3);  // Plasma
    }

    // ── photosynthesis ──────────────────────────────────────────────────────

    #[test]
    fn photosynthesis_increases_flora_energy() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = flora(&mut w, 50.0, [3.0, 3.0]);
        let cell = grid_cell([3.0, 3.0]);
        w.irradiance_grid[cell] = 10.0;
        let before = w.entities[idx].qe;
        photosynthesis(&mut w);
        assert!(w.entities[idx].qe > before, "flora should gain energy");
    }

    #[test]
    fn photosynthesis_low_resonance_minimal_gain() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 50.0;
        e.radius = 1.0;
        e.position = [3.0, 3.0];
        e.frequency_hz = 1500.0; // very far from SOLAR_FREQUENCY (400) → near-zero resonance
        let idx = w.spawn(e).unwrap();
        let cell = grid_cell([3.0, 3.0]);
        w.irradiance_grid[cell] = 10.0;
        let before = w.entities[idx].qe;
        photosynthesis(&mut w);
        assert!(w.entities[idx].qe - before < 0.5, "low resonance → minimal gain");
    }

    // ── nutrient_uptake ─────────────────────────────────────────────────────

    #[test]
    fn nutrient_uptake_transfers_from_grid() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = flora(&mut w, 50.0, [5.0, 5.0]);
        let cell = grid_cell([5.0, 5.0]);
        w.nutrient_grid[cell] = 20.0;
        let grid_before = w.nutrient_grid[cell];
        let qe_before = w.entities[idx].qe;
        nutrient_uptake(&mut w);
        assert!(w.entities[idx].qe > qe_before, "entity gains nutrients");
        assert!(w.nutrient_grid[cell] < grid_before, "grid depletes");
        // Conservation: gain = grid depletion
        let gain = w.entities[idx].qe - qe_before;
        let depl = grid_before - w.nutrient_grid[cell];
        assert!((gain - depl).abs() < 1e-5, "gain={gain} depl={depl}");
    }

    #[test]
    fn nutrient_uptake_empty_grid_no_change() {
        let mut w = SimWorldFlat::new(0, 0.05);
        flora(&mut w, 50.0, [5.0, 5.0]);
        nutrient_uptake(&mut w);
        assert_eq!(w.entities[0].qe, 50.0);
    }
}
