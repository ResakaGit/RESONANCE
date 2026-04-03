//! ET-9: Multidimensional Niche (Hutchinson hypervolume) — ecuaciones puras.

/// Solapamiento de nicho entre dos entidades en 4D: [0,1].
/// niche_a/b: centro del nicho. width_a/b: radio por dimensión.
pub fn niche_overlap(
    niche_a: [f32; 4],
    width_a: [f32; 4],
    niche_b: [f32; 4],
    width_b: [f32; 4],
) -> f32 {
    let mut product = 1.0f32;
    for d in 0..4 {
        let dist = (niche_a[d] - niche_b[d]).abs();
        let combined = width_a[d] + width_b[d];
        if combined <= 0.0 {
            return 0.0;
        }
        product *= (1.0 - dist / combined).clamp(0.0, 1.0);
    }
    product
}

/// Presión competitiva: solapamiento × demanda de recursos compartidos.
pub fn competitive_pressure(overlap: f32, resource_demand_a: f32, resource_demand_b: f32) -> f32 {
    overlap * (resource_demand_a * resource_demand_b).sqrt()
}

/// Desplazamiento de carácter: delta de centro para alejarse del competidor.
pub fn character_displacement(
    own_center: f32,
    competitor_center: f32,
    displacement_rate: f32,
) -> f32 {
    let direction = if own_center >= competitor_center {
        1.0
    } else {
        -1.0
    };
    direction * displacement_rate
}

/// Amplitud del nicho: media geométrica de los radios en 4D.
pub fn niche_breadth(width: [f32; 4]) -> f32 {
    (width[0] * width[1] * width[2] * width[3]).powf(0.25)
}

/// Especialización óptima: nicho estrecho en entornos estables, amplio en variables.
pub fn optimal_niche_width(env_variance: f32, resource_density: f32) -> f32 {
    (env_variance / (resource_density + f32::EPSILON))
        .sqrt()
        .clamp(0.1, 5.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn niche_overlap_identical_niches() {
        assert!((niche_overlap([0.0; 4], [1.0; 4], [0.0; 4], [1.0; 4]) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn niche_overlap_no_contact() {
        assert!((niche_overlap([0.0; 4], [1.0; 4], [3.0; 4], [1.0; 4])).abs() < 1e-5);
    }

    #[test]
    fn niche_breadth_uniform() {
        assert!((niche_breadth([2.0; 4]) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn character_displacement_moves_away_from_competitor() {
        assert!((character_displacement(0.0, 1.0, 0.01) - (-0.01)).abs() < 1e-6);
        assert!((character_displacement(1.0, 0.0, 0.01) - 0.01).abs() < 1e-6);
    }

    #[test]
    fn competitive_pressure_zero_no_overlap() {
        assert_eq!(competitive_pressure(0.0, 5.0, 5.0), 0.0);
    }

    #[test]
    fn optimal_niche_width_clamped() {
        let w = optimal_niche_width(1e9, 0.001);
        assert!(w <= 5.0);
        let w2 = optimal_niche_width(0.0, 100.0);
        assert!(w2 >= 0.1);
    }
}
