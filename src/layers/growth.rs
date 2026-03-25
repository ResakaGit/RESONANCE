use bevy::prelude::*;

/// Capa 4: presupuesto metabólico disponible para crecimiento.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, PartialEq)]
#[component(storage = "SparseSet")]
pub struct GrowthBudget {
    pub biomass_available: f32,
    pub limiting_factor: u8,
    pub efficiency: f32,
}

/// Ancla de radio base para crecimiento alométrico (TL6).
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, PartialEq)]
pub struct AllometricRadiusAnchor {
    pub base_radius: f32,
}

impl AllometricRadiusAnchor {
    pub fn new(base_radius: f32) -> Self {
        Self {
            base_radius: sanitize_non_negative(base_radius),
        }
    }
}

impl GrowthBudget {
    pub fn new(biomass_available: f32, limiting_factor: u8, efficiency: f32) -> Self {
        Self {
            biomass_available: sanitize_non_negative(biomass_available),
            limiting_factor: limiting_factor.min(3),
            efficiency: sanitize_norm(efficiency),
        }
    }
}

#[inline]
fn sanitize_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[inline]
fn sanitize_norm(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{AllometricRadiusAnchor, GrowthBudget};

    #[test]
    fn growth_budget_new_clamps_fields() {
        let b = GrowthBudget::new(-1.0, 9, f32::INFINITY);
        assert_eq!(b.biomass_available, 0.0);
        assert_eq!(b.limiting_factor, 3);
        assert_eq!(b.efficiency, 0.0);
    }

    #[test]
    fn allometric_anchor_new_sanitizes_radius() {
        let a = AllometricRadiusAnchor::new(f32::NAN);
        assert_eq!(a.base_radius, 0.0);
    }
}
