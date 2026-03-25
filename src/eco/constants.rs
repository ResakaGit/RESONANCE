//! Umbrales y tuning de Eco-Boundaries.
//! Las transiciones de fase y qe mínimo **no se duplican**: se reexportan desde `blueprint::constants`.

pub use crate::blueprint::constants::{
    GAS_TRANSITION, LIQUID_TRANSITION, QE_MIN_EXISTENCE, SOLID_TRANSITION,
};

// ── Clasificación zonal (densidad relativa al pipeline V7 / coherencia) ──
/// Densidad por encima de este valor sugiere medio acuático profundo (Subaquatic).
pub const SUBAQUATIC_DENSITY_THRESHOLD: f32 = 2.0;

/// Densidad por encima de este valor en sólido → Subterranean (más denso que subacuático típico).
pub const SUBTERRANEAN_DENSITY_THRESHOLD: f32 = 5.0;

/// Gradiente térmico (escala con `GAS_TRANSITION` en el clasificador E2) para marcar ThermalShock.
pub const THERMAL_SHOCK_GRADIENT: f32 = GAS_TRANSITION * 0.25;

/// Umbral de qe por debajo del cual la celda se trata como Void (alineado a existencia mínima).
pub const VOID_QE_THRESHOLD: f32 = QE_MIN_EXISTENCE;

/// Altura normalizada por encima de la cual prima HighAtmosphere (coordenadas de mundo / grid).
pub const ATMOSPHERE_CEILING_HEIGHT: f32 = 120.0;

/// Banda de frecuencia dominante asociada a Ignis (alineada al almanaque ~400–450 Hz; margen 400–500).
pub const IGNIS_DOMINANT_MIN_HZ: f32 = 400.0;
pub const IGNIS_DOMINANT_MAX_HZ: f32 = 500.0;

/// Por debajo de esta densidad (ρ de `cell_density`) una celda gaseosa se trata como atmósfera alta.
pub const THIN_ATMOSPHERE_DENSITY_MAX: f32 = 0.05;

/// Escala de temperatura para normalizar gradientes en `PhaseBoundary` (evita números mágicos sueltos).
pub const PHASE_GRADIENT_TEMP_SPAN: f32 = crate::blueprint::constants::GAS_TRANSITION * 2.0;

/// Escala de ΔHz para `ElementFrontier` en `compute_gradient_factor`.
pub const ELEMENT_GRADIENT_HZ_SPAN: f32 = 600.0;

/// Umbral relativo mínimo de diferencia de densidad para marcar `DensityGradient` en `infer_transition_type`.
pub const DENSITY_JUMP_RELATIVE_MIN: f32 = 0.25;

/// ΔHz mínimo para preferir `ElementFrontier` cuando fase y ρ ya son similares pero las zonas difieren.
pub const ELEMENT_ZONE_HZ_BREAK: f32 = 50.0;

// ── Recomputación del campo derivado ──
/// Ticks mínimos entre recomputos completos del `EcoBoundaryField` (mitiga thrash si el grid oscila).
pub const BOUNDARY_RECOMPUTE_COOLDOWN: u32 = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn umbrales_densidad_ordenados() {
        assert!(
            SUBTERRANEAN_DENSITY_THRESHOLD > SUBAQUATIC_DENSITY_THRESHOLD,
            "subterranean must require higher density than subaquatic"
        );
    }

    #[test]
    fn void_qe_alineado_a_blueprint() {
        assert_eq!(
            VOID_QE_THRESHOLD,
            crate::blueprint::constants::QE_MIN_EXISTENCE
        );
    }

    #[test]
    fn transiciones_fase_no_duplicadas() {
        assert_eq!(
            SOLID_TRANSITION,
            crate::blueprint::constants::SOLID_TRANSITION
        );
        assert_eq!(
            LIQUID_TRANSITION,
            crate::blueprint::constants::LIQUID_TRANSITION
        );
        assert_eq!(GAS_TRANSITION, crate::blueprint::constants::GAS_TRANSITION);
    }

    #[test]
    fn thermal_shock_usa_gas_transition() {
        assert_eq!(THERMAL_SHOCK_GRADIENT, GAS_TRANSITION * 0.25);
    }
}
