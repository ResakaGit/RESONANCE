/// Constantes del track **Morfogénesis inferida** (MG-1, reservas MG-4).
/// Usar `constants::morphogenesis::*` en código nuevo del track; el `pub use` en `mod.rs`
/// mantiene compatibilidad con `use crate::blueprint::constants::*`.
pub mod morphogenesis {
    /// σ de Stefan-Boltzmann (escalado al modelo de temperatura equivalente).
    pub const STEFAN_BOLTZMANN: f32 = 5.67e-8;
    /// ε por defecto (cuerpo gris) en balance radiativo inferido.
    pub const DEFAULT_EMISSIVITY: f32 = 0.9;
    /// h por defecto (convección natural) en balance de superficie inferida (MG-1).
    /// Contexto distinto de `CONVECTIVE_COEFFICIENT` en `thermal_transfer` (conducción superficial).
    pub const DEFAULT_CONVECTION_COEFF: f32 = 10.0;
    /// Factor de capacidad calorífica por unidad de qe: `C_v = qe * SPECIFIC_HEAT_FACTOR`.
    pub const SPECIFIC_HEAT_FACTOR: f32 = 0.01;

    /// C_D de referencia (cuerpo poco alargado, p. ej. esfera).
    pub const DRAG_COEFF_BASE: f32 = 0.47;
    /// C_D mínimo (cuerpo muy fusiforme).
    pub const DRAG_COEFF_MIN: f32 = 0.04;
    /// Sensibilidad del arrastre al cuadrado del fineness ratio.
    pub const DRAG_FINENESS_SCALE: f32 = 0.15;

    /// Amortiguación del optimizador de forma (MG-4).
    pub const SHAPE_OPTIMIZER_DAMPING: f32 = 0.3;
    /// Iteraciones máximas por frame del optimizador de forma (MG-4).
    pub const SHAPE_OPTIMIZER_MAX_ITER: u32 = 3;

    /// Albedo mínimo inferido (superficie muy absorbente).
    pub const ALBEDO_MIN: f32 = 0.05;
    /// Albedo máximo inferido (superficie muy reflectante).
    pub const ALBEDO_MAX: f32 = 0.95;
    /// Sin flujo solar efectivo (`I * A_proj ≈ 0`): albedo neutro.
    pub const ALBEDO_FALLBACK: f32 = 0.5;
    /// Piso para `I * A_proj`. Más estricto que `DIVISION_GUARD_EPSILON` en `numeric_math` (régimen físico distinto).
    pub const ALBEDO_IRRADIANCE_FLUX_EPS: f32 = 1e-6;

    // ── MG-4: Shape Optimizer ──

    /// Fineness ratio mínimo (esfera: forma más compacta).
    pub const FINENESS_MIN: f32 = 1.0;
    /// Fineness ratio máximo (torpedo extremo).
    pub const FINENESS_MAX: f32 = 8.0;
    /// Fineness ratio por defecto (ligeramente alargado).
    pub const FINENESS_DEFAULT: f32 = 1.5;
    /// Guard change detection del optimizer.
    pub const SHAPE_OPTIMIZER_EPSILON: f32 = 0.01;
    /// Paso finite-difference para gradiente numérico.
    pub const SHAPE_FD_DELTA: f32 = 0.1;

    /// Rugosidad mínima: superficie equivalente a esfera lisa.
    pub const RUGOSITY_MIN: f32 = 1.0;
    /// Rugosidad máxima: hasta ~4× superficie de esfera equivalente.
    pub const RUGOSITY_MAX: f32 = 4.0;

    // ── MG-5: Albedo Inference System ──

    /// Guard change detection para α inferido.
    pub const ALBEDO_EPSILON: f32 = 0.005;
    /// Peso base de luminosidad en blend albedo→visual (piso sin reflejo).
    pub const ALBEDO_LUMINOSITY_BASE_WEIGHT: f32 = 0.3;
    /// Peso de albedo en blend albedo→visual (rango reflejante).
    pub const ALBEDO_LUMINOSITY_ALBEDO_WEIGHT: f32 = 0.7;

    // ── MG-7: Surface Rugosity System ──

    /// Guard change detection para rugosity inferida.
    pub const RUGOSITY_EPSILON: f32 = 0.02;
    /// Tope de subdivisión GF1 por rugosidad (2× geometría base máximo).
    pub const RUGOSITY_MAX_DETAIL_MULTIPLIER: f32 = 2.0;
    /// Techo de segmentos por entidad para presupuesto geométrico.
    pub const MAX_SEGMENTS_PER_ENTITY: u32 = 64;
}
