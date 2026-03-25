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

/// Evalúa si las condiciones energéticas permiten abiogénesis en una celda.
/// Retorna un score \[0, 1\]: por encima del umbral del sistema → spawn posible.
///
/// Factor de frecuencia: triángulo lineal con vértice en `flora_peak_hz` (clampeado a `[low,high]`),
/// valor **0** en `low` y `high`, **1** en el pico (incluye pico en borde, p. ej. Flora en `flora.ron`).
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
