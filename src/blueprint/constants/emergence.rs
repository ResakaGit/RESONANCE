//! Constantes de los sistemas de emergencia (ET-2 a ET-14, AC-2 a AC-5).
//! Centralizadas desde sistemas individuales para auditabilidad axiomática.
//!
//! Emergence system constants (ET-2 through ET-14, AC-2 through AC-5).
//! Centralized from individual systems for axiomatic auditability.

// ─── ET-2: Theory of Mind ────────────────────────────────────────────────────

/// Radio de escaneo para detectar vecinos observables (unidades mundo).
/// Scan radius for detecting observable neighbors (world units).
pub const MODEL_SCAN_RADIUS: f32 = 10.0;

/// Tasa de aprendizaje del modelo predictivo (EMA blend).
/// Learning rate for predictive model updates (EMA blend).
pub const MODEL_LEARNING_RATE: f32 = 0.1;

/// Desviación máxima de frecuencia para precisión del modelo (Hz).
/// Maximum frequency deviation for model accuracy (Hz).
pub const MODEL_MAX_FREQ_DEVIATION: f32 = 500.0;

// ─── ET-9: Niche Adaptation ─────────────────────────────────────────────────

/// Radio de escaneo para detectar competidores de nicho (unidades mundo).
/// Scan radius for detecting niche competitors (world units).
pub const NICHE_SCAN_RADIUS: f32 = 12.0;

/// Umbral de solapamiento para activar desplazamiento de carácter.
/// Overlap threshold to trigger character displacement.
pub const NICHE_OVERLAP_DISPLACEMENT_THRESHOLD: f32 = 0.3;
