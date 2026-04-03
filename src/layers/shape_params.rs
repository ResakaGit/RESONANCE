use bevy::prelude::*;

use crate::blueprint::constants::morphogenesis as mg;

/// Parámetros de forma inferidos por el optimizer (MG-4).
/// Traducidos a `GeometryInfluence` por el mesh builder de GF1.
#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct MorphogenesisShapeParams {
    /// Ratio largo/diámetro. 1.0 = esfera, >3.0 = fusiforme, >6.0 = torpedo.
    fineness_ratio: f32,
    /// Escala longitudinal (metros). Derivada de SpatialVolume.
    length_scale: f32,
    /// C_shape del tick actual (diagnóstico).
    current_shape_cost: f32,
}

impl Default for MorphogenesisShapeParams {
    fn default() -> Self {
        Self {
            fineness_ratio: mg::FINENESS_DEFAULT,
            length_scale: 0.0,
            current_shape_cost: 0.0,
        }
    }
}

impl MorphogenesisShapeParams {
    pub fn new(fineness_ratio: f32) -> Self {
        Self {
            fineness_ratio: fineness_ratio.clamp(mg::FINENESS_MIN, mg::FINENESS_MAX),
            length_scale: 0.0,
            current_shape_cost: 0.0,
        }
    }

    #[inline]
    pub fn fineness_ratio(&self) -> f32 {
        self.fineness_ratio
    }
    #[inline]
    pub fn length_scale(&self) -> f32 {
        self.length_scale
    }
    #[inline]
    pub fn current_shape_cost(&self) -> f32 {
        self.current_shape_cost
    }

    /// Actualiza los tres campos con guard change detection.
    pub fn update(&mut self, fineness: f32, length_scale: f32, cost: f32) {
        let f = fineness.clamp(mg::FINENESS_MIN, mg::FINENESS_MAX);
        if (self.fineness_ratio - f).abs() > mg::SHAPE_OPTIMIZER_EPSILON
            || (self.length_scale - length_scale).abs() > mg::SHAPE_OPTIMIZER_EPSILON
        {
            self.fineness_ratio = f;
            self.length_scale = length_scale;
            self.current_shape_cost = cost;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::morphogenesis as mg;

    #[test]
    fn default_fineness_matches_constant() {
        let p = MorphogenesisShapeParams::default();
        assert!((p.fineness_ratio() - mg::FINENESS_DEFAULT).abs() < 1e-6);
    }

    #[test]
    fn new_clamps_fineness_to_bounds() {
        let lo = MorphogenesisShapeParams::new(0.0);
        assert!((lo.fineness_ratio() - mg::FINENESS_MIN).abs() < 1e-6);
        let hi = MorphogenesisShapeParams::new(100.0);
        assert!((hi.fineness_ratio() - mg::FINENESS_MAX).abs() < 1e-6);
    }

    #[test]
    fn update_guards_tiny_change() {
        let mut p = MorphogenesisShapeParams::new(3.0);
        // Primero establecer baseline con length_scale=1.0 para aislar el guard de fineness.
        p.update(3.0, 1.0, 10.0);
        p.update(3.0 + mg::SHAPE_OPTIMIZER_EPSILON * 0.5, 1.0, 10.0);
        assert!(
            (p.fineness_ratio() - 3.0).abs() < 1e-6,
            "tiny change should be guarded"
        );
    }

    #[test]
    fn update_applies_significant_change() {
        let mut p = MorphogenesisShapeParams::new(3.0);
        p.update(5.0, 2.0, 15.0);
        assert!((p.fineness_ratio() - 5.0).abs() < 1e-6);
        assert!((p.length_scale() - 2.0).abs() < 1e-6);
        assert!((p.current_shape_cost() - 15.0).abs() < 1e-6);
    }

    #[test]
    fn mapping_invariant_fineness_to_geometry() {
        // fineness=4, radius=1 → length_budget=8, radius_base=0.5 → ratio=16 ≈ fineness²
        let f = 4.0_f32;
        let r = 1.0_f32;
        let length_budget = (r * 2.0) * f;
        let radius_base = (r * 2.0) / f;
        let ratio = length_budget / radius_base;
        assert!(
            (ratio - f * f).abs() < 1e-4,
            "ratio {} ≈ fineness² {}",
            ratio,
            f * f
        );
    }
}
