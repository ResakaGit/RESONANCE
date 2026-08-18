// ── Capa 3: flujo ──
use crate::blueprint::equations::derived_thresholds::{DENSITY_SCALE, DISSIPATION_PLASMA};

/// Tasa base de disipación entrópica en vacío, sin overlays (qe/s).
///
/// Axioma 4 (2ª Ley), derivada de fundamentales: en vacío el `qe` fluye
/// desligado —régimen no-ligado tipo plasma, sin coherencia de materia que lo
/// retenga— así que su pérdida entrópica usa el coeficiente del estado plasma
/// `DISSIPATION_PLASMA` a escala de grid `DENSITY_SCALE`.
/// `20.0 × 0.25 = 5.0`. Derivada, no calibrada suelta.
pub const DEFAULT_DISSIPATION_RATE: f32 = DENSITY_SCALE * DISSIPATION_PLASMA;
