use std::f32::consts::PI;
use crate::blueprint::constants::*;
use crate::layers::{AmbientPressure, ContactType, MatterCoherence, MatterState};

// ═══════════════════════════════════════════════
// Colisiones: Transferencia de Energía
// ═══════════════════════════════════════════════

/// Energía transferida durante una colisión.
/// qe_transfer = min(qe_a, qe_b) * |I| * conductividad * dt
pub fn collision_transfer(
    qe_a: f32,
    qe_b: f32,
    interference: f32,
    conductivity: f32,
    dt: f32,
) -> f32 {
    qe_a.min(qe_b) * interference.abs() * conductivity * dt
}

/// Área de intersección (2D) entre dos círculos.
/// Retorna 0 si no hay solapamiento.
pub fn circle_intersection_area(dist: f32, r1: f32, r2: f32) -> f32 {
    let d = dist.max(0.0);
    let r1 = r1.max(0.0);
    let r2 = r2.max(0.0);

    if r1 <= 0.0 || r2 <= 0.0 {
        return 0.0;
    }

    // Separados o tangentes externamente.
    if d >= r1 + r2 {
        return 0.0;
    }

    // Uno dentro del otro sin solapar borde.
    let r_small = r1.min(r2);
    if d <= (r1 - r2).abs() {
        return PI * r_small * r_small;
    }

    // Caso general: fórmula estándar de intersección de círculos.
    if d <= DISTANCE_EPSILON {
        return PI * r_small * r_small;
    }

    let x1 = ((d * d + r1 * r1 - r2 * r2) / (2.0 * d * r1)).clamp(-1.0, 1.0);
    let x2 = ((d * d + r2 * r2 - r1 * r1) / (2.0 * d * r2)).clamp(-1.0, 1.0);

    let part1 = r1 * r1 * x1.acos();
    let part2 = r2 * r2 * x2.acos();
    let part3 = 0.5 * ((-d + r1 + r2) * (d + r1 - r2) * (d - r1 + r2) * (d + r1 + r2)).sqrt();

    part1 + part2 - part3
}

/// Cálculo de transferencia de energía por canal de contención.
/// Valor positivo = inyecta energía en la entidad, negativo = drena.
pub fn thermal_transfer(
    contact: ContactType,
    host_pressure: &AmbientPressure,
    host_coherence: Option<&MatterCoherence>,
    entity_coherence: &MatterCoherence,
    distance: f32,
    overlap_area: f32,
    dt: f32,
) -> f32 {
    let base = host_pressure.delta_qe_constant;
    if dt <= 0.0 {
        return 0.0;
    }

    match contact {
        ContactType::Surface => {
            // CONDUCCIÓN: k_host × k_entity × área_contacto × base × dt
            let k_host =
                host_coherence.map_or(THERMAL_CONDUCTIVITY_FALLBACK, |c| c.thermal_conductivity());
            let k_entity = entity_coherence.thermal_conductivity();
            base * k_host * k_entity * overlap_area * dt
        }
        ContactType::Immersed => {
            // CONVECCIÓN: depende del “medio” (viscosidad) y del “contacto volumétrico”.
            // Calibración Sprint 03: reduce la convección para que conducción gane en Surface.
            let viscosity = host_pressure.terrain_viscosity;
            let convective_coefficient = CONVECTIVE_COEFFICIENT;
            base * viscosity * convective_coefficient * overlap_area * dt
        }
        ContactType::Radiated => {
            // RADIACIÓN: visibilidad (estado del host) y decaimiento 1/dist^2
            let vis = host_coherence.map_or(RADIATION_VISIBILITY_FALLBACK, |c| match c.state() {
                MatterState::Plasma => RADIATION_VISIBILITY_PLASMA,
                MatterState::Gas => RADIATION_VISIBILITY_GAS,
                _ => RADIATION_VISIBILITY_CONDENSED,
            });
            let inv_sq = 1.0 / distance.max(RADIATION_MIN_DISTANCE).powi(2);
            base * vis * inv_sq * dt
        }
    }
}
