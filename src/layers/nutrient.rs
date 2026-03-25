use bevy::prelude::*;

/// Capa 4: Perfil de nutrientes disponible para metabolismo.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, PartialEq)]
pub struct NutrientProfile {
    pub carbon_norm: f32,
    pub nitrogen_norm: f32,
    pub phosphorus_norm: f32,
    pub water_norm: f32,
}

impl NutrientProfile {
    pub fn new(
        carbon_norm: f32,
        nitrogen_norm: f32,
        phosphorus_norm: f32,
        water_norm: f32,
    ) -> Self {
        Self {
            carbon_norm: sanitize_norm(carbon_norm),
            nitrogen_norm: sanitize_norm(nitrogen_norm),
            phosphorus_norm: sanitize_norm(phosphorus_norm),
            water_norm: sanitize_norm(water_norm),
        }
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
    use super::NutrientProfile;

    #[test]
    fn nutrient_profile_new_clamps_norms() {
        let p = NutrientProfile::new(2.0, -1.0, 0.5, f32::NAN);
        assert_eq!(p.carbon_norm, 1.0);
        assert_eq!(p.nitrogen_norm, 0.0);
        assert_eq!(p.phosphorus_norm, 0.5);
        assert_eq!(p.water_norm, 0.0);
    }
}
