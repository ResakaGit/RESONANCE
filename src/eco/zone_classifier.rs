//! Clasificación pura `EnergyCell` → `ZoneClass` (sin ECS).
//! El tamaño de celda entra como parámetro porque la densidad ρ depende del volumen implícito.
//! `HighAtmosphere` usa gas + ρ baja (sin altura mundial): cuando exista terreno alineado al grid,
//! se puede extender la firma con coordenada Y/`ATMOSPHERE_CEILING_HEIGHT` (ver blueprint §5.1).

use crate::blueprint::constants::{GAS_TRANSITION, SOLID_TRANSITION};
use crate::eco::constants::{
    IGNIS_DOMINANT_MAX_HZ, IGNIS_DOMINANT_MIN_HZ, SUBAQUATIC_DENSITY_THRESHOLD,
    SUBTERRANEAN_DENSITY_THRESHOLD, THIN_ATMOSPHERE_DENSITY_MAX, VOID_QE_THRESHOLD,
};
use crate::eco::contracts::ZoneClass;
use crate::layers::MatterState;
use crate::worldgen::EnergyCell;
use crate::worldgen::propagation::cell_density;

#[inline]
fn ignis_dominant(cell: &EnergyCell) -> bool {
    let hz = cell.dominant_frequency_hz;
    hz.is_finite() && (IGNIS_DOMINANT_MIN_HZ..=IGNIS_DOMINANT_MAX_HZ).contains(&hz)
}

/// Árbol de decisión del sprint E2: una sola zona por celda, sin ambigüedad.
pub fn classify_cell(cell: &EnergyCell, cell_size_m: f32) -> ZoneClass {
    let qe = cell.accumulated_qe;
    if !qe.is_finite() || qe < VOID_QE_THRESHOLD {
        return ZoneClass::Void;
    }

    let density = cell_density(qe, cell_size_m);
    let temp = cell.temperature;
    let matter = cell.matter_state;

    // Orden explícito sprint E2: materia/densidad antes que Volcanic/Frozen.
    if matter == MatterState::Liquid && density > SUBAQUATIC_DENSITY_THRESHOLD {
        return ZoneClass::Subaquatic;
    }
    if matter == MatterState::Solid && density > SUBTERRANEAN_DENSITY_THRESHOLD {
        return ZoneClass::Subterranean;
    }
    if temp > GAS_TRANSITION && ignis_dominant(cell) {
        return ZoneClass::Volcanic;
    }
    if temp < SOLID_TRANSITION {
        return ZoneClass::Frozen;
    }
    if matter == MatterState::Gas && density <= THIN_ATMOSPHERE_DENSITY_MAX {
        return ZoneClass::HighAtmosphere;
    }

    ZoneClass::Surface
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{GAS_TRANSITION, SOLID_TRANSITION};
    use crate::worldgen::propagation::{cell_density, cell_temperature};

    fn cell_liquid_dense() -> EnergyCell {
        let cell_size = 1.0_f32;
        // ρ alta ⇒ T alta; forzamos líquido elevando eb implícito no existe en EnergyCell —
        // usamos qe alto en celda pequeña y estado ya líquido.
        let qe = (SUBAQUATIC_DENSITY_THRESHOLD * 2.0 + 0.5) * cell_size * cell_size * cell_size;
        let rho = cell_density(qe, cell_size);
        EnergyCell {
            accumulated_qe: qe,
            temperature: cell_temperature(rho),
            matter_state: MatterState::Liquid,
            dominant_frequency_hz: 250.0,
            ..Default::default()
        }
    }

    #[test]
    fn classify_qe_zero_es_void() {
        let c = EnergyCell {
            accumulated_qe: 0.0,
            ..Default::default()
        };
        assert_eq!(classify_cell(&c, 1.0), ZoneClass::Void);
    }

    #[test]
    fn classify_liquido_denso_es_subaquatic() {
        let c = cell_liquid_dense();
        assert_eq!(c.matter_state, MatterState::Liquid);
        assert!(
            cell_density(c.accumulated_qe, 1.0) > SUBAQUATIC_DENSITY_THRESHOLD,
            "density precondition"
        );
        assert_eq!(classify_cell(&c, 1.0), ZoneClass::Subaquatic);
    }

    #[test]
    fn classify_normal_es_surface() {
        let c = EnergyCell {
            accumulated_qe: 2.0,
            matter_state: MatterState::Liquid,
            temperature: SOLID_TRANSITION + 0.2,
            dominant_frequency_hz: 250.0,
            ..Default::default()
        };
        assert_eq!(classify_cell(&c, 1.0), ZoneClass::Surface);
    }

    #[test]
    fn classify_volcanic_requiere_ignis_y_calor() {
        let c = EnergyCell {
            accumulated_qe: 5.0,
            temperature: GAS_TRANSITION + 0.5,
            matter_state: MatterState::Gas,
            dominant_frequency_hz: 420.0,
            ..Default::default()
        };
        assert_eq!(classify_cell(&c, 1.0), ZoneClass::Volcanic);
    }

    /// Regresión orden E2: líquido denso gana sobre rama Ignis/calor.
    #[test]
    fn classify_liquido_denso_no_cae_en_volcanic_aunque_ignis_caliente() {
        let mut c = cell_liquid_dense();
        c.temperature = GAS_TRANSITION + 1.0;
        c.dominant_frequency_hz = 430.0;
        assert_eq!(classify_cell(&c, 1.0), ZoneClass::Subaquatic);
    }
}
