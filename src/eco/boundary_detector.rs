//! Detección de fronteras entre zonas adyacentes (8-vecinos). Funciones puras.

use crate::eco::constants::{
    DENSITY_JUMP_RELATIVE_MIN, ELEMENT_GRADIENT_HZ_SPAN, ELEMENT_ZONE_HZ_BREAK,
    PHASE_GRADIENT_TEMP_SPAN, THERMAL_SHOCK_GRADIENT,
};
use crate::eco::contracts::{BoundaryMarker, TransitionType, ZoneClass};
use crate::eco::zone_classifier::classify_cell;
use crate::worldgen::EnergyCell;
use crate::worldgen::propagation::cell_density;

/// Orden de los 8 vecinos: NW, N, NE, W, E, SW, S, SE (deltas x,y).
pub const NEIGHBOR_OFFSETS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// Factor de interpolación [0,1] hacia el vecino según el tipo de transición dominante.
pub fn compute_gradient_factor(
    cell: &EnergyCell,
    neighbor: &EnergyCell,
    transition: TransitionType,
    cell_size_m: f32,
) -> f32 {
    let rho_a = cell_density(cell.accumulated_qe, cell_size_m);
    let rho_b = cell_density(neighbor.accumulated_qe, cell_size_m);
    let t_a = cell.temperature;
    let t_b = neighbor.temperature;

    let g = match transition {
        TransitionType::PhaseBoundary => {
            let span = PHASE_GRADIENT_TEMP_SPAN.max(1e-4);
            (t_b - t_a).abs() / span
        }
        TransitionType::DensityGradient => {
            let span = (rho_a.abs().max(rho_b.abs())).max(1e-4);
            (rho_b - rho_a).abs() / span
        }
        TransitionType::ElementFrontier => {
            let span = ELEMENT_GRADIENT_HZ_SPAN.max(1e-4);
            (neighbor.dominant_frequency_hz - cell.dominant_frequency_hz).abs() / span
        }
        TransitionType::ThermalShock => {
            let span = THERMAL_SHOCK_GRADIENT.max(1e-4);
            (t_b - t_a).abs() / span
        }
    };

    if !g.is_finite() {
        return 0.5;
    }
    g.clamp(0.0, 1.0)
}

/// Infiere el tipo de frontera a partir de las dos celdas y sus clases zonales.
pub fn infer_transition_type(
    zone_a: ZoneClass,
    zone_b: ZoneClass,
    cell: &EnergyCell,
    neighbor: &EnergyCell,
    cell_size_m: f32,
) -> TransitionType {
    if cell.matter_state != neighbor.matter_state {
        return TransitionType::PhaseBoundary;
    }
    let dt = (cell.temperature - neighbor.temperature).abs();
    if dt > THERMAL_SHOCK_GRADIENT {
        return TransitionType::ThermalShock;
    }
    let rho_a = cell_density(cell.accumulated_qe, cell_size_m);
    let rho_b = cell_density(neighbor.accumulated_qe, cell_size_m);
    let rho_scale = rho_a.abs().max(rho_b.abs()).max(1e-3);
    if (rho_a - rho_b).abs() > DENSITY_JUMP_RELATIVE_MIN * rho_scale {
        return TransitionType::DensityGradient;
    }
    if zone_a != zone_b {
        let hz = (cell.dominant_frequency_hz - neighbor.dominant_frequency_hz).abs();
        if hz > ELEMENT_ZONE_HZ_BREAK {
            return TransitionType::ElementFrontier;
        }
    }
    TransitionType::ElementFrontier
}

/// Compara la celda central con 8 vecinos ya resueltos (en bordes del grid se repite la celda central).
/// Si varios vecinos difieren, se elige el de mayor `gradient_factor` (transición más marcada).
pub fn detect_boundary(
    cell: &EnergyCell,
    neighbors: &[EnergyCell; 8],
    cell_zone: ZoneClass,
    cell_zone_id: u16,
    cell_size_m: f32,
) -> BoundaryMarker {
    let mut best: Option<(f32, ZoneClass, TransitionType)> = None;
    for n in neighbors {
        let nz = classify_cell(n, cell_size_m);
        if nz == cell_zone {
            continue;
        }
        let transition = infer_transition_type(cell_zone, nz, cell, n, cell_size_m);
        let gf = compute_gradient_factor(cell, n, transition, cell_size_m).max(1e-4);
        let replace = best
            .as_ref()
            .map(|(best_g, _, _)| gf > *best_g)
            .unwrap_or(true);
        if replace {
            best = Some((gf, nz, transition));
        }
    }
    if let Some((gf, nz, transition)) = best {
        return BoundaryMarker::Boundary {
            zone_a: cell_zone,
            zone_b: nz,
            gradient_factor: gf,
            transition_type: transition,
        };
    }
    BoundaryMarker::Interior {
        zone_id: cell_zone_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::MatterState;

    fn base_cell() -> EnergyCell {
        EnergyCell {
            accumulated_qe: 2.0,
            temperature: 1.0,
            matter_state: MatterState::Liquid,
            dominant_frequency_hz: 200.0,
            ..Default::default()
        }
    }

    #[test]
    fn detect_todos_vecinos_misma_zona_es_interior() {
        let c = base_cell();
        let n = [
            c.clone(),
            c.clone(),
            c.clone(),
            c.clone(),
            c.clone(),
            c.clone(),
            c.clone(),
            c.clone(),
        ];
        let z = classify_cell(&c, 1.0);
        let m = detect_boundary(&c, &n, z, 7, 1.0);
        assert!(matches!(m, BoundaryMarker::Interior { zone_id: 7 }));
    }

    #[test]
    fn detect_un_vecino_distinto_es_boundary_con_gradiente_positivo() {
        let c = base_cell();
        let mut other = c.clone();
        other.accumulated_qe = 0.0;
        other.temperature = 0.0;
        other.matter_state = MatterState::Solid;
        let n = [
            other.clone(),
            c.clone(),
            c.clone(),
            c.clone(),
            c.clone(),
            c.clone(),
            c.clone(),
            c.clone(),
        ];
        let z = classify_cell(&c, 1.0);
        let m = detect_boundary(&c, &n, z, 1, 1.0);
        match m {
            BoundaryMarker::Boundary {
                gradient_factor, ..
            } => {
                assert!(gradient_factor > 0.0);
            }
            BoundaryMarker::Interior { .. } => panic!("expected Boundary"),
        }
    }

    #[test]
    fn infer_solid_vs_liquid_es_phase_boundary() {
        let solid_cell = EnergyCell {
            accumulated_qe: 3.0,
            temperature: 0.1,
            matter_state: MatterState::Solid,
            dominant_frequency_hz: 80.0,
            ..Default::default()
        };

        let mut liq = solid_cell.clone();
        liq.matter_state = MatterState::Liquid;
        liq.temperature = 1.5;

        let t = infer_transition_type(
            ZoneClass::Frozen,
            ZoneClass::Surface,
            &solid_cell,
            &liq,
            1.0,
        );
        assert_eq!(t, TransitionType::PhaseBoundary);
    }
}
