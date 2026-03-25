// ── Fog of War (G12): proveedor de visión en grid ──
/// Radio máximo world-space del stamp circular en el grid (cache; la señal real está en `equations`).
pub const FOG_DEFAULT_PROVIDER_RADIUS: f32 = 14.0;
/// Umbral de `perception_signal` reservado para tuning futuro (sensibilidad del perceptor).
pub const FOG_DEFAULT_SENSITIVITY: f32 = 0.05;
