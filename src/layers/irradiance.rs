use bevy::prelude::*;

/// Capa 1 (extensión): irradiancia recibida por una entidad materializada.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, PartialEq)]
#[component(storage = "SparseSet")]
pub struct IrradianceReceiver {
    /// Densidad de fotones acumulada [0, +inf).
    pub photon_density: f32,
    /// Fracción absorbida según visibilidad elemental [0, 1].
    pub absorbed_fraction: f32,
}

impl IrradianceReceiver {
    pub fn new(photon_density: f32, absorbed_fraction: f32) -> Self {
        Self {
            photon_density: sanitize_non_negative(photon_density),
            absorbed_fraction: sanitize_norm(absorbed_fraction),
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
    use super::IrradianceReceiver;

    #[test]
    fn irradiance_receiver_new_sanitizes_values() {
        let r = IrradianceReceiver::new(-2.0, f32::INFINITY);
        assert_eq!(r.photon_density, 0.0);
        assert_eq!(r.absorbed_fraction, 0.0);
    }
}
