// ── Transferencia térmica (equations::thermal_transfer) ──
/// Conductividad del host si no hay coherencia (medio “promedio” para conducción superficial).
pub const THERMAL_CONDUCTIVITY_FALLBACK: f32 = 0.5;

/// Coeficiente de convección calibrado (Sprint 03): que conducción gane en Surface frente a Immersed.
/// **No** confundir con `DEFAULT_CONVECTION_COEFF` (MG-1 balance radiativo de superficie; orden distinto).
pub const CONVECTIVE_COEFFICIENT: f32 = 0.15;

/// Visibilidad radiativa del host en plasma (emite fuerte hacia entidades cercanas).
pub const RADIATION_VISIBILITY_PLASMA: f32 = 1.0;

/// Visibilidad radiativa del host gaseoso caliente.
pub const RADIATION_VISIBILITY_GAS: f32 = 0.3;

/// Visibilidad radiativa para sólido/líquido (opacos, poca radiación directa).
pub const RADIATION_VISIBILITY_CONDENSED: f32 = 0.1;

/// Visibilidad si el host no tiene coherencia (mismo orden de magnitud que un gas opaco medio).
pub const RADIATION_VISIBILITY_FALLBACK: f32 = 0.5;

/// Distancia mínima en ley 1/r² para radiación (evita singularidad en d→0).
pub const RADIATION_MIN_DISTANCE: f32 = 1.0;
