pub mod axiomatic;
pub use axiomatic::*;

use crate::blueprint::constants::*;

/// Factor de proximidad al pico en `[low, high]` (precondición: `hz` ya pasó el gating de banda).
#[inline]
fn abiogenesis_frequency_proximity(hz: f32, low: f32, high: f32, peak: f32) -> f32 {
    let span = (high - low).max(f32::EPSILON);
    let edge = ABIOGENESIS_FREQ_TRIANGLE_EDGE_EPS;
    if peak <= low + edge {
        ((high - hz) / span).clamp(0.0, 1.0)
    } else if peak >= high - edge {
        ((hz - low) / span).clamp(0.0, 1.0)
    } else if hz <= peak {
        ((hz - low) / (peak - low).max(f32::EPSILON)).clamp(0.0, 1.0)
    } else {
        ((high - hz) / (high - peak).max(f32::EPSILON)).clamp(0.0, 1.0)
    }
}

/// DEPRECATED: Flora-specific abiogenesis potential. Replaced by `axiomatic_abiogenesis_potential`
/// which derives spawn conditions from the 8 foundational axioms (frequency-agnostic).
/// Kept for reference and test regression. No system calls this function.
#[inline]
pub fn abiogenesis_potential(
    cell_qe: f32,
    cell_hz: f32,
    flora_band_low: f32,
    flora_band_high: f32,
    flora_peak_hz: f32,
    nutrient_water: f32,
    min_qe: f32,
) -> f32 {
    let qe = if cell_qe.is_finite() {
        cell_qe.max(0.0)
    } else {
        0.0
    };
    let hz = if cell_hz.is_finite() {
        cell_hz
    } else {
        return 0.0;
    };
    let low = if flora_band_low.is_finite() {
        flora_band_low
    } else {
        return 0.0;
    };
    let high = if flora_band_high.is_finite() {
        flora_band_high
    } else {
        return 0.0;
    };
    if low >= high {
        return 0.0;
    }
    let min_q = if min_qe.is_finite() {
        min_qe.max(0.0)
    } else {
        return 0.0;
    };
    if qe < min_q {
        return 0.0;
    }
    if hz < low || hz > high {
        return 0.0;
    }
    let water = if nutrient_water.is_finite() {
        nutrient_water.max(0.0)
    } else {
        0.0
    };
    if water <= 0.0 {
        return 0.0;
    }
    let peak = if flora_peak_hz.is_finite() {
        flora_peak_hz.clamp(low, high)
    } else {
        return 0.0;
    };
    let freq_proximity = abiogenesis_frequency_proximity(hz, low, high, peak);
    let energy_factor = (qe / min_q).clamp(0.0, ABIOGENESIS_POTENTIAL_QE_RATIO_CAP)
        * ABIOGENESIS_POTENTIAL_QE_RATIO_SCALE;
    let water_factor = water.clamp(0.0, 1.0);
    (freq_proximity * energy_factor * water_factor).clamp(0.0, 1.0)
}

/// Heurística de enlace desde qe acumulado en celda (perfil emergente, sin clamp de spawn).
#[inline]
pub fn abiogenesis_bond_heuristic_from_cell_qe(cell_qe: f32) -> f32 {
    let q = if cell_qe.is_finite() {
        cell_qe.max(0.0)
    } else {
        0.0
    };
    q * ABIOGENESIS_CELL_QE_TO_BOND_SCALE
}

/// `bond_energy_eb` para `MatterCoherence` al spawnear flora de campo.
#[inline]
pub fn abiogenesis_spawn_matter_bond(cell_qe: f32) -> f32 {
    abiogenesis_bond_heuristic_from_cell_qe(cell_qe)
        .clamp(ABIOGENESIS_SPAWN_BOND_MIN, ABIOGENESIS_SPAWN_BOND_MAX)
}

/// qe inicial del emergente (fracción de la celda).
#[inline]
pub fn abiogenesis_spawn_entity_qe(cell_qe: f32) -> f32 {
    let q = if cell_qe.is_finite() {
        cell_qe.max(0.0)
    } else {
        0.0
    };
    q * ABIOGENESIS_SPAWN_CELL_QE_FRACTION
}

// ── Fauna Abiogenesis (EA5-F) ────────────────────────────────────────────────

/// Score [0, 1] for fauna spawn potential.
///
/// Unlike flora (frequency-band gated), fauna is gated by energy density,
/// nutrient richness, and local flora/herbivore count (trophic succession).
///
/// Invariant: returns 0.0 if any precondition fails. Always in [0, 1].
pub fn fauna_abiogenesis_potential(
    cell_qe: f32,
    nutrient_density: f32,
    flora_neighbour_count: u32,
    min_flora: u32,
    water: f32,
    min_qe: f32,
    water_floor: f32,
) -> f32 {
    let qe = if cell_qe.is_finite() { cell_qe.max(0.0) } else { return 0.0 };
    let w = if water.is_finite() { water.max(0.0) } else { return 0.0 };
    let nd = if nutrient_density.is_finite() { nutrient_density.clamp(0.0, 1.0) } else { return 0.0 };
    let min_q = if min_qe.is_finite() { min_qe.max(0.0) } else { return 0.0 };
    if qe < min_q { return 0.0; }
    if w < water_floor.max(0.0) { return 0.0; }
    if flora_neighbour_count < min_flora { return 0.0; }

    let energy_factor = (qe / min_q.max(f32::EPSILON)).clamp(0.0, 2.0) * 0.5;
    let nutrient_factor = nd;
    let flora_density = (flora_neighbour_count as f32 / min_flora.max(1) as f32).clamp(0.0, 2.0) * 0.5;
    (energy_factor * nutrient_factor * flora_density).clamp(0.0, 1.0)
}

/// Infer trophic class from local herbivore density.
///
/// Returns `true` for Carnivore if enough herbivores nearby, `false` for Herbivore.
#[inline]
pub fn fauna_infer_is_carnivore(
    herbivore_neighbour_count: u32,
    min_herbivores_for_carnivore: u32,
) -> bool {
    herbivore_neighbour_count >= min_herbivores_for_carnivore
}

/// qe for newly spawned fauna (fraction of cell energy).
#[inline]
pub fn fauna_spawn_entity_qe(cell_qe: f32) -> f32 {
    let q = if cell_qe.is_finite() { cell_qe.max(0.0) } else { 0.0 };
    q * ABIOGENESIS_FAUNA_SPAWN_QE_FRACTION
}

/// Bond energy for fauna spawn, clamped to [BOND_MIN, BOND_MAX].
#[inline]
pub fn fauna_spawn_matter_bond(cell_qe: f32) -> f32 {
    let q = if cell_qe.is_finite() { cell_qe.max(0.0) } else { 0.0 };
    (q * ABIOGENESIS_FAUNA_CELL_QE_TO_BOND_SCALE)
        .clamp(ABIOGENESIS_FAUNA_SPAWN_BOND_MIN, ABIOGENESIS_FAUNA_SPAWN_BOND_MAX)
}

/// Determina el perfil de inferencia del organismo que emerge según condiciones locales.
/// Retorna `(growth_bias, branching_bias, resilience)`.
#[inline]
pub fn abiogenesis_profile_from_conditions(
    bond_energy_local: f32,
    water_saturation: f32,
) -> (f32, f32, f32) {
    let bond = if bond_energy_local.is_finite() {
        bond_energy_local
    } else {
        0.0
    };
    let water = if water_saturation.is_finite() {
        water_saturation.clamp(0.0, 1.0)
    } else {
        0.0
    };
    if bond > ABIOGENESIS_PROFILE_BOND_OAK_MIN {
        (
            ABIOGENESIS_OAK_GROWTH,
            ABIOGENESIS_OAK_BRANCHING,
            ABIOGENESIS_OAK_RESILIENCE,
        )
    } else if water > ABIOGENESIS_PROFILE_WATER_MOSS_MIN && bond < ABIOGENESIS_PROFILE_BOND_MOSS_MAX
    {
        (
            ABIOGENESIS_MOSS_GROWTH,
            ABIOGENESIS_MOSS_BRANCHING,
            ABIOGENESIS_MOSS_RESILIENCE,
        )
    } else {
        (
            ABIOGENESIS_ROSA_GROWTH,
            ABIOGENESIS_ROSA_BRANCHING,
            ABIOGENESIS_ROSA_RESILIENCE,
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    // ── fauna_abiogenesis_potential ──────────────────────────────────────────

    #[test]
    fn fauna_potential_below_min_qe_returns_zero() {
        let p = fauna_abiogenesis_potential(10.0, 0.5, 5, 3, 0.8, 60.0, 0.4);
        assert!(p.abs() < EPS, "got {p}");
    }

    #[test]
    fn fauna_potential_below_water_floor_returns_zero() {
        let p = fauna_abiogenesis_potential(100.0, 0.5, 5, 3, 0.2, 60.0, 0.4);
        assert!(p.abs() < EPS, "got {p}");
    }

    #[test]
    fn fauna_potential_insufficient_flora_returns_zero() {
        let p = fauna_abiogenesis_potential(100.0, 0.5, 1, 3, 0.8, 60.0, 0.4);
        assert!(p.abs() < EPS, "got {p}");
    }

    #[test]
    fn fauna_potential_all_conditions_met_returns_positive() {
        let p = fauna_abiogenesis_potential(120.0, 0.6, 5, 3, 0.8, 60.0, 0.4);
        assert!(p > 0.0, "should be positive: {p}");
    }

    #[test]
    fn fauna_potential_always_in_unit_range() {
        for qe in [60.0, 200.0, 1000.0] {
            let p = fauna_abiogenesis_potential(qe, 1.0, 8, 3, 1.0, 60.0, 0.4);
            assert!((0.0..=1.0).contains(&p), "out of range: {p} for qe={qe}");
        }
    }

    #[test]
    fn fauna_potential_nan_inputs_return_zero() {
        assert!(fauna_abiogenesis_potential(f32::NAN, 0.5, 5, 3, 0.8, 60.0, 0.4).abs() < EPS);
        assert!(fauna_abiogenesis_potential(100.0, f32::NAN, 5, 3, 0.8, 60.0, 0.4).abs() < EPS);
        assert!(fauna_abiogenesis_potential(100.0, 0.5, 5, 3, f32::NAN, 60.0, 0.4).abs() < EPS);
    }

    // ── fauna_infer_is_carnivore ────────────────────────────────────────────

    #[test]
    fn infer_herbivore_when_few_herbivores() {
        assert!(!fauna_infer_is_carnivore(1, 2));
    }

    #[test]
    fn infer_carnivore_when_enough_herbivores() {
        assert!(fauna_infer_is_carnivore(3, 2));
    }

    #[test]
    fn infer_carnivore_exact_threshold() {
        assert!(fauna_infer_is_carnivore(2, 2));
    }

    // ── fauna_spawn_entity_qe ───────────────────────────────────────────────

    #[test]
    fn fauna_spawn_qe_fraction_of_cell() {
        let qe = fauna_spawn_entity_qe(100.0);
        assert!((qe - 60.0).abs() < EPS, "got {qe}"); // 100 × 0.6
    }

    #[test]
    fn fauna_spawn_qe_zero_cell_returns_zero() {
        assert!(fauna_spawn_entity_qe(0.0).abs() < EPS);
    }

    #[test]
    fn fauna_spawn_qe_nan_returns_zero() {
        assert!(fauna_spawn_entity_qe(f32::NAN).abs() < EPS);
    }

    // ── fauna_spawn_matter_bond ─────────────────────────────────────────────

    #[test]
    fn fauna_bond_clamped_to_min() {
        let b = fauna_spawn_matter_bond(10.0); // 10 × 8 = 80 → clamp to 500
        assert!((b - ABIOGENESIS_FAUNA_SPAWN_BOND_MIN).abs() < EPS, "got {b}");
    }

    #[test]
    fn fauna_bond_clamped_to_max() {
        let b = fauna_spawn_matter_bond(1000.0); // 1000 × 8 = 8000 → clamp to 2500
        assert!((b - ABIOGENESIS_FAUNA_SPAWN_BOND_MAX).abs() < EPS, "got {b}");
    }

    #[test]
    fn fauna_bond_mid_range_scales() {
        let b = fauna_spawn_matter_bond(100.0); // 100 × 8 = 800
        assert!((b - 800.0).abs() < EPS, "got {b}");
    }

    // ── flora equations (existing, regression) ──────────────────────────────

    #[test]
    fn flora_potential_within_band_returns_positive() {
        let p = abiogenesis_potential(90.0, 90.0, 85.0, 110.0, 85.0, 0.8, 30.0);
        assert!(p > 0.0, "flora potential should be positive: {p}");
    }

    #[test]
    fn flora_potential_outside_band_returns_zero() {
        let p = abiogenesis_potential(90.0, 200.0, 85.0, 110.0, 85.0, 0.8, 30.0);
        assert!(p.abs() < EPS, "outside band: {p}");
    }
}
