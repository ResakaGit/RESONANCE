//! ET-12: Continental Drift / Tectonics — ecuaciones puras. Sin deps de Bevy.

/// Estrés acumulado en el borde entre dos placas.
pub fn boundary_stress(relative_velocity: f32, contact_length: f32, friction_coeff: f32) -> f32 {
    relative_velocity * contact_length * friction_coeff
}

/// Amplitud del evento sísmico al liberar estrés acumulado.
pub fn seismic_amplitude(stress_released: f32, depth_factor: f32) -> f32 {
    (stress_released * depth_factor).sqrt()
}

/// Delta de qe en una celda tras un evento sísmico.
/// Ley de potencias: atenúa con la distancia al cuadrado.
pub fn seismic_qe_delta(amplitude: f32, distance: f32, is_constructive: bool) -> f32 {
    let base = amplitude / (1.0 + distance.powi(2));
    if is_constructive { base } else { -base }
}

/// Uplift geológico: incremento de qe base por actividad volcánica.
pub fn volcanic_qe_uplift(magma_flux: f32, eruption_efficiency: f32) -> f32 {
    magma_flux * eruption_efficiency
}

/// Erosión: reducción de qe base por estrés tectónico.
pub fn tectonic_erosion(cell_qe: f32, erosion_rate: f32) -> f32 {
    cell_qe * erosion_rate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_stress_proportional() {
        assert!((boundary_stress(1.0, 10.0, 0.5) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn seismic_amplitude_sqrt_of_stress() {
        assert!((seismic_amplitude(4.0, 1.0) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn seismic_qe_delta_destructive_is_negative() {
        assert!(seismic_qe_delta(10.0, 0.0, false) < 0.0);
    }

    #[test]
    fn seismic_qe_delta_constructive_is_positive() {
        assert!(seismic_qe_delta(10.0, 0.0, true) > 0.0);
    }

    #[test]
    fn seismic_qe_delta_attenuates_with_distance() {
        let near = seismic_qe_delta(10.0, 0.0, true);
        let far = seismic_qe_delta(10.0, 10.0, true);
        assert!(near > far);
    }

    #[test]
    fn volcanic_qe_uplift_proportional() {
        assert!((volcanic_qe_uplift(5.0, 0.8) - 4.0).abs() < 1e-5);
    }

    #[test]
    fn tectonic_erosion_proportional() {
        assert!((tectonic_erosion(100.0, 0.1) - 10.0).abs() < 1e-5);
    }
}
