// ── Capa 4→5: alometría (TL6) ──
/// Tasa logística de crecimiento radial por tick (escalada por `GrowthBudget`).
pub const ALLOMETRIC_GROWTH_RATE: f32 = 0.002;
/// Factor de radio máximo relativo al radio de spawn/base.
pub const ALLOMETRIC_MAX_RADIUS_FACTOR: f32 = 3.0;
/// Piso mínimo de intake efectivo en alometría para evitar starvation instantánea.
pub const ALLOMETRIC_INTAKE_FLOOR: f32 = 0.1;
/// Factor geométrico de esfera para proxy de superficie: `((4/3)π)^(2/3)`.
pub const ALLOMETRIC_SURFACE_FACTOR: f32 = 2.598_076;
/// Epsilon de escritura para evitar `Changed<SpatialVolume>` espurio.
pub const VOLUME_WRITE_EPS: f32 = 1e-4;
