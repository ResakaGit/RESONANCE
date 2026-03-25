//! Clasificación geométrica por celda: altitude + slope + acumulación → [`TerrainType`](crate::topology::contracts::TerrainType).
//!
//! Stateless, determinista, sin vecinos (T5 / docs/design/TOPOLOGY.md §5.2).

use serde::{Deserialize, Serialize};

use crate::topology::constants::{CLIFF_SLOPE_THRESHOLD, RIVER_THRESHOLD};
use crate::topology::contracts::TerrainType;

/// Umbrales configurables (RON / `TerrainConfig::classification`).
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct ClassificationThresholds {
    pub peak_altitude: f32,
    pub ridge_altitude: f32,
    pub plateau_altitude: f32,
    pub cliff_slope: f32,
    pub slope_threshold: f32,
    pub river_accumulation: f32,
    pub basin_max_slope: f32,
    pub basin_max_altitude: f32,
    pub valley_max_altitude: f32,
}

impl Default for ClassificationThresholds {
    fn default() -> Self {
        Self {
            peak_altitude: 140.0,
            ridge_altitude: 95.0,
            plateau_altitude: 110.0,
            cliff_slope: CLIFF_SLOPE_THRESHOLD,
            slope_threshold: 8.0,
            river_accumulation: RIVER_THRESHOLD,
            basin_max_slope: 4.0,
            basin_max_altitude: 35.0,
            valley_max_altitude: 55.0,
        }
    }
}

/// Clasifica una celda; orden de reglas fijo (primera coincidencia gana).
///
/// Si algún escalar no es finito (NaN/±∞), devuelve [`TerrainType::Plain`] — contrato explícito T5.
#[inline]
pub fn classify_terrain(
    altitude: f32,
    slope: f32,
    drainage_accumulation: f32,
    thresholds: &ClassificationThresholds,
) -> TerrainType {
    if !altitude.is_finite() || !slope.is_finite() || !drainage_accumulation.is_finite() {
        return TerrainType::Plain;
    }

    let st = thresholds.slope_threshold;
    if slope >= thresholds.cliff_slope {
        return TerrainType::Cliff;
    }
    if drainage_accumulation >= thresholds.river_accumulation {
        return TerrainType::Riverbed;
    }
    if altitude >= thresholds.peak_altitude && slope >= st {
        return TerrainType::Peak;
    }
    if altitude >= thresholds.ridge_altitude && slope >= st * 0.5 {
        return TerrainType::Ridge;
    }
    if altitude >= thresholds.plateau_altitude && slope < st * 0.3 {
        return TerrainType::Plateau;
    }
    if altitude <= thresholds.valley_max_altitude && drainage_accumulation > 0.0 && slope < st {
        return TerrainType::Valley;
    }
    if altitude <= thresholds.basin_max_altitude
        && slope < thresholds.basin_max_slope
        && drainage_accumulation < 1.0
    {
        return TerrainType::Basin;
    }
    if slope >= st {
        return TerrainType::Slope;
    }
    TerrainType::Plain
}

/// Clasifica el grid completo.
///
/// **Contrato:** `altitude`, `slope` y `drainage_accumulation` deben tener la misma longitud;
/// si no, panic (fallo de caller: grid SoA corrupto). No trunca como `Iterator::zip`.
pub fn classify_all(
    altitude: &[f32],
    slope: &[f32],
    drainage_accumulation: &[f32],
    thresholds: &ClassificationThresholds,
) -> Vec<TerrainType> {
    let n = altitude.len();
    assert_eq!(
        n,
        slope.len(),
        "classify_all: altitude and slope must have the same length"
    );
    assert_eq!(
        n,
        drainage_accumulation.len(),
        "classify_all: altitude and drainage_accumulation must have the same length"
    );

    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push(classify_terrain(
            altitude[i],
            slope[i],
            drainage_accumulation[i],
            thresholds,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Umbrales que satisfacen el criterio de aceptación del sprint T5 (tabla de ejemplos).
    fn acceptance_thresholds() -> ClassificationThresholds {
        ClassificationThresholds {
            peak_altitude: 150.0,
            ridge_altitude: 100.0,
            plateau_altitude: 130.0,
            cliff_slope: CLIFF_SLOPE_THRESHOLD,
            slope_threshold: 25.0,
            river_accumulation: RIVER_THRESHOLD,
            basin_max_slope: 5.0,
            basin_max_altitude: 15.0,
            valley_max_altitude: 25.0,
        }
    }

    #[test]
    fn slope_70_is_cliff() {
        let t = acceptance_thresholds();
        assert_eq!(classify_terrain(0.0, 70.0, 0.0, &t), TerrainType::Cliff);
    }

    #[test]
    fn high_accumulation_moderate_slope_is_riverbed() {
        let t = acceptance_thresholds();
        assert_eq!(
            classify_terrain(0.0, 10.0, 150.0, &t),
            TerrainType::Riverbed
        );
    }

    #[test]
    fn high_alt_significant_slope_is_peak() {
        let t = acceptance_thresholds();
        assert_eq!(classify_terrain(180.0, 30.0, 0.0, &t), TerrainType::Peak);
    }

    #[test]
    fn ridge_band() {
        let t = acceptance_thresholds();
        assert_eq!(classify_terrain(120.0, 15.0, 0.0, &t), TerrainType::Ridge);
    }

    #[test]
    fn high_flat_is_plateau() {
        let t = acceptance_thresholds();
        assert_eq!(classify_terrain(150.0, 2.0, 0.0, &t), TerrainType::Plateau);
    }

    #[test]
    fn low_alt_with_flow_is_valley() {
        let t = acceptance_thresholds();
        assert_eq!(classify_terrain(20.0, 5.0, 30.0, &t), TerrainType::Valley);
    }

    #[test]
    fn low_flat_no_accumulation_is_basin() {
        let t = acceptance_thresholds();
        assert_eq!(classify_terrain(10.0, 2.0, 0.0, &t), TerrainType::Basin);
    }

    #[test]
    fn steep_not_cliff_is_slope() {
        let t = acceptance_thresholds();
        assert_eq!(classify_terrain(60.0, 25.0, 0.0, &t), TerrainType::Slope);
    }

    #[test]
    fn mid_alt_gentle_is_plain() {
        let t = acceptance_thresholds();
        assert_eq!(classify_terrain(50.0, 5.0, 0.0, &t), TerrainType::Plain);
    }

    #[test]
    fn cliff_wins_over_river() {
        let t = acceptance_thresholds();
        assert_eq!(classify_terrain(0.0, 70.0, 200.0, &t), TerrainType::Cliff);
    }

    #[test]
    fn classify_all_matches_per_cell() {
        let t = acceptance_thresholds();
        let alt = [180.0, 10.0, 50.0, 0.0];
        let sl = [30.0, 10.0, 5.0, 70.0];
        let acc = [0.0, 150.0, 0.0, 5.0];
        let all = classify_all(&alt, &sl, &acc, &t);
        assert_eq!(all.len(), 4);
        for i in 0..4 {
            assert_eq!(
                all[i],
                classify_terrain(alt[i], sl[i], acc[i], &t),
                "cell {}",
                i
            );
        }
        assert_eq!(all[0], TerrainType::Peak);
        assert_eq!(all[1], TerrainType::Riverbed);
        assert_eq!(all[3], TerrainType::Cliff);
    }

    #[test]
    fn determinism_same_inputs_same_output() {
        let t = acceptance_thresholds();
        let a = classify_terrain(77.7, 12.3, 44.4, &t);
        let b = classify_terrain(77.7, 12.3, 44.4, &t);
        assert_eq!(a, b);
    }

    #[test]
    fn non_finite_inputs_are_plain() {
        let t = acceptance_thresholds();
        assert_eq!(classify_terrain(f32::NAN, 5.0, 0.0, &t), TerrainType::Plain);
        assert_eq!(
            classify_terrain(0.0, f32::INFINITY, 0.0, &t),
            TerrainType::Plain
        );
        assert_eq!(
            classify_terrain(0.0, 5.0, f32::NEG_INFINITY, &t),
            TerrainType::Plain
        );
    }

    #[test]
    #[should_panic(expected = "classify_all")]
    fn classify_all_panics_when_slice_lengths_differ() {
        let t = acceptance_thresholds();
        let _ = classify_all(&[1.0, 2.0], &[1.0], &[1.0, 2.0], &t);
    }
}
