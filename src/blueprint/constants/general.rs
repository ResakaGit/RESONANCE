// ── General ──
/// Energía mínima para que una entidad siga existiendo.
pub const QE_MIN_EXISTENCE: f32 = 0.01;

/// Umbral base de inanición (L0): por debajo del umbral adaptativo (`starvation_threshold`) la entidad
/// muere por estrés metabólico si `0 < qe < umbral` (EA4). Calibración en `blueprint/constants`.
pub const METABOLIC_STARVATION_BASE_THRESHOLD_QE: f32 = 5.0;

/// EA7 — `competition_energy_drain`: el drain se escala por `1.0 - resilience * este factor` (resilience ∈ \[0,1\]).
pub const COMPETITION_RESILIENCE_DRAIN_ATTENUATION: f32 = 0.6;

