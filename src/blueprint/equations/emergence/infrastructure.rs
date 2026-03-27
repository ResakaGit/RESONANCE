//! ET-4: Infrastructure / Field Modification — ecuaciones puras. Sin deps de Bevy.

pub const MAX_INFRASTRUCTURE_AMPLIFIER: f32 = 2.0;

/// Delta de campo producido por inversión energética en una celda.
pub fn field_modification_delta(invested_qe: f32, modification_rate: f32) -> f32 {
    invested_qe * modification_rate
}

/// Decay de la modificación acumulada en una celda (entropía de infraestructura).
pub fn field_modification_decay(current_delta: f32, decay_rate: f32) -> f32 {
    current_delta * (1.0 - decay_rate).max(0.0)
}

/// Amplificación de intake para una entidad en una celda con infraestructura.
pub fn infrastructure_intake_amplifier(field_delta: f32, amplification_factor: f32) -> f32 {
    1.0 + (field_delta * amplification_factor).clamp(0.0, MAX_INFRASTRUCTURE_AMPLIFIER - 1.0)
}

/// ROI de invertir en infraestructura.
/// expected_uses: cuántas veces se usará. use_benefit: qe ganado por uso.
pub fn infrastructure_roi(
    investment_cost: f32,
    expected_uses: f32,
    use_benefit: f32,
    maintenance_per_tick: f32,
    horizon_ticks: u32,
) -> f32 {
    let total_benefit = expected_uses * use_benefit;
    let total_cost = investment_cost + maintenance_per_tick * horizon_ticks as f32;
    total_benefit - total_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_modification_delta_proportional() {
        assert!((field_modification_delta(10.0, 0.05) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn field_modification_decay_slow() {
        assert!((field_modification_decay(100.0, 0.001) - 99.9).abs() < 1e-3);
    }

    #[test]
    fn field_modification_decay_full_removes_all() {
        assert!((field_modification_decay(100.0, 1.0)).abs() < 1e-5);
    }

    #[test]
    fn infrastructure_intake_amplifier_min_one() {
        assert!(infrastructure_intake_amplifier(0.0, 0.002) >= 1.0);
    }

    #[test]
    fn infrastructure_intake_amplifier_capped() {
        assert!(infrastructure_intake_amplifier(1e9, 1.0) <= MAX_INFRASTRUCTURE_AMPLIFIER);
    }

    #[test]
    fn infrastructure_roi_breakeven() {
        // 20 uses × 1.0 = 20; cost = 10 + 0.1×100 = 20; ROI = 0
        assert!((infrastructure_roi(10.0, 20.0, 1.0, 0.1, 100) - 0.0).abs() < 1e-4);
    }
}
