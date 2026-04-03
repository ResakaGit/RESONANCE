use crate::blueprint::constants::FREQ_BAND_MAX_HZ;

/// Resonance factor between two frequencies: 1.0 when identical, ~0.0 at opposite bands.
/// cos²(Δf/BAND * π/2) — same formula used in the energy-competition blueprint.
pub fn resonance_factor(freq_a_hz: f32, freq_b_hz: f32) -> f32 {
    let delta = (freq_a_hz - freq_b_hz).abs();
    let normalized = (delta / FREQ_BAND_MAX_HZ).min(1.0);
    (normalized * std::f32::consts::FRAC_PI_2).cos().powi(2)
}

/// Effective extraction of `a` over `b`, weighted by resonance overlap.
pub fn effective_extraction(extraction_capacity: f32, freq_a_hz: f32, freq_b_hz: f32) -> f32 {
    extraction_capacity * resonance_factor(freq_a_hz, freq_b_hz)
}

/// Resistance to extraction: lower = easier to eliminate. Returns `f32::MAX` when `effective_ext` ≤ 0.
pub fn extraction_resistance(qe: f32, effective_ext: f32) -> f32 {
    if effective_ext <= 0.0 {
        return f32::MAX;
    }
    qe / effective_ext
}

/// Total threat magnitude from enemies within `sensory_radius` of `self_pos`.
pub fn threat_magnitude(
    self_pos: [f32; 2],
    enemies: &[([f32; 2], f32)],
    sensory_radius: f32,
) -> f32 {
    enemies
        .iter()
        .filter(|(pos, _)| {
            let dx = pos[0] - self_pos[0];
            let dy = pos[1] - self_pos[1];
            (dx * dx + dy * dy).sqrt() <= sensory_radius
        })
        .map(|(_, cap)| cap)
        .sum()
}

/// Normalised direction of greatest threat (weighted sum of enemy vectors). `[0,0]` if no enemies.
pub fn threat_gradient(self_pos: [f32; 2], enemies: &[([f32; 2], f32)]) -> [f32; 2] {
    let mut gx = 0.0_f32;
    let mut gz = 0.0_f32;
    for (pos, cap) in enemies {
        let dx = pos[0] - self_pos[0];
        let dz = pos[1] - self_pos[1];
        let dist = (dx * dx + dz * dz).sqrt().max(0.001);
        gx += (dx / dist) * cap;
        gz += (dz / dist) * cap;
    }
    let mag = (gx * gx + gz * gz).sqrt();
    if mag < 0.001 {
        [0.0, 0.0]
    } else {
        [gx / mag, gz / mag]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resonance_same_freq() {
        assert!((resonance_factor(440.0, 440.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn resonance_opposite_bands() {
        assert!(resonance_factor(0.0, 1100.0) < 0.01);
    }

    #[test]
    fn resonance_partial_match() {
        assert!(resonance_factor(440.0, 880.0) < resonance_factor(440.0, 450.0));
    }

    #[test]
    fn effective_extraction_same_freq() {
        assert!((effective_extraction(10.0, 440.0, 440.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn resistance_zero_ext_returns_max() {
        assert_eq!(extraction_resistance(100.0, 0.0), f32::MAX);
    }

    #[test]
    fn resistance_normal() {
        assert!((extraction_resistance(100.0, 10.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn threat_gradient_no_enemies() {
        assert_eq!(threat_gradient([0.0, 0.0], &[]), [0.0, 0.0]);
    }

    #[test]
    fn threat_magnitude_empty() {
        assert_eq!(threat_magnitude([0.0, 0.0], &[], 100.0), 0.0);
    }
}
