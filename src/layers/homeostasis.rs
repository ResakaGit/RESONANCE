use bevy::prelude::*;

/// Capa 12: Adaptación frecuencial con costo energético.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Homeostasis {
    pub adapt_rate_hz: f32,
    pub qe_cost_per_hz: f32,
    pub stability_band_hz: f32,
    pub enabled: bool,
}

impl Homeostasis {
    pub fn new(
        adapt_rate_hz: f32,
        qe_cost_per_hz: f32,
        stability_band_hz: f32,
        enabled: bool,
    ) -> Self {
        Self {
            adapt_rate_hz: adapt_rate_hz.max(0.0),
            qe_cost_per_hz: qe_cost_per_hz.max(0.0),
            stability_band_hz: stability_band_hz.max(0.0),
            enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_negative_rates() {
        let h = Homeostasis::new(-5.0, -1.0, -2.0, true);
        assert_eq!(h.adapt_rate_hz, 0.0);
        assert_eq!(h.qe_cost_per_hz, 0.0);
        assert_eq!(h.stability_band_hz, 0.0);
        assert!(h.enabled);
    }

    #[test]
    fn new_preserves_non_negative_inputs() {
        let h = Homeostasis::new(10.0, 0.5, 2.0, false);
        assert!((h.adapt_rate_hz - 10.0).abs() < 1e-5);
        assert!((h.qe_cost_per_hz - 0.5).abs() < 1e-5);
        assert!((h.stability_band_hz - 2.0).abs() < 1e-5);
        assert!(!h.enabled);
    }

    #[test]
    fn enabled_flag_roundtrips() {
        let on = Homeostasis::new(1.0, 1.0, 1.0, true);
        let off = Homeostasis::new(1.0, 1.0, 1.0, false);
        assert!(on.enabled);
        assert!(!off.enabled);
    }
}
