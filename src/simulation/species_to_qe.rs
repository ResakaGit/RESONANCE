//! AI-1 (ADR-043): bridge `SpeciesGrid` → `EnergyFieldGrid`.
//! AI-1 (ADR-043): `SpeciesGrid` → `EnergyFieldGrid` bridge.
//!
//! Una vez por tick, lee concentraciones del `SpeciesGrid` (química AP-*
//! mass-action) y proyecta qe al `EnergyFieldGrid` (campo qe-based del
//! simulador principal) modulado por alineación de frecuencia (Ax 8).
//!
//! Direccional: species → qe.  El inverso (qe → species) queda fuera de
//! scope para AI-1 (ver ADR-043 §3 "Out of scope").
//!
//! `Option<Res<...>>` para que el sistema sea no-op cuando la sopa AP-*
//! no está cargada — preserva ADR-040 byte-equivalence del harness AP.

use bevy::prelude::*;

use crate::blueprint::constants::chemistry::{
    REACTION_FREQ_BANDWIDTH_DEFAULT, SPECIES_TO_QE_COUPLING,
};
use crate::blueprint::equations::reaction_kinetics::frequency_alignment;
use crate::layers::reaction_network::ReactionNetwork;
use crate::layers::species_grid::SpeciesGrid;
use crate::worldgen::EnergyFieldGrid;

/// Pure fn — `(species, network, &mut field, dt) → ()`.  Determinista.
///
/// Por cada celda del `SpeciesGrid` con `Σ species > 0`:
///   1. Calcula `mean_freq = network.mean_product_frequency(cell.species)`
///   2. Calcula `alignment = frequency_alignment(mean_freq, field_freq, BW)`
///   3. Inyecta `dqe = Σ species × alignment × COUPLING × dt` al `EnergyFieldGrid`
///
/// Mapeo de coordenadas: 1-a-1 con clamping al menor de los dos grids.
/// Si `species` es más grande que `field`, las celdas extra se ignoran;
/// si `field` es más grande, sólo se modifica la región cubierta.
pub fn inject_species_to_field(
    species: &SpeciesGrid,
    network: &ReactionNetwork,
    field: &mut EnergyFieldGrid,
    dt: f32,
) {
    if dt <= 0.0 || network.is_empty() { return; }
    let max_x = species.width().min(field.width as usize);
    let max_y = species.height().min(field.height as usize);
    for y in 0..max_y {
        for x in 0..max_x {
            let cell = species.cell(x, y);
            let cell_qe: f32 = cell.species.iter().sum();
            if !cell_qe.is_finite() || cell_qe <= 0.0 { continue; }
            let mean_freq = network.mean_product_frequency(&cell.species);
            // Field freq se lee primero (no necesita mut).
            let field_freq = field
                .cell_xy(x as u32, y as u32)
                .map(|c| c.dominant_frequency_hz)
                .unwrap_or(0.0);
            let alignment = frequency_alignment(
                mean_freq, field_freq, REACTION_FREQ_BANDWIDTH_DEFAULT,
            );
            let dqe = cell_qe * alignment * SPECIES_TO_QE_COUPLING * dt;
            if !dqe.is_finite() || dqe <= 0.0 { continue; }
            if let Some(field_cell) = field.cell_xy_mut(x as u32, y as u32) {
                field_cell.accumulated_qe += dqe;
            }
        }
    }
}

/// Sistema Bevy thin wrapper.  Se registra en `Phase::ChemicalLayer`.
/// Sin sopa AP-* (resources ausentes) ⇒ no-op.
pub fn species_to_qe_injection_system(
    species: Option<Res<SpeciesGrid>>,
    network: Option<Res<ReactionNetwork>>,
    field: Option<ResMut<EnergyFieldGrid>>,
    fixed_time: Res<Time<Fixed>>,
) {
    let (Some(species), Some(network), Some(mut field)) = (species, network, field)
        else { return; };
    inject_species_to_field(&species, &network, &mut field, fixed_time.delta_secs());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::reaction::SpeciesId;
    use crate::math_types::Vec2;

    fn formose_net() -> ReactionNetwork {
        let text = std::fs::read_to_string("assets/reactions/formose.ron").unwrap();
        ReactionNetwork::from_ron_str(&text).unwrap()
    }

    fn empty_field(w: u32, h: u32, freq: f32) -> EnergyFieldGrid {
        let mut field = EnergyFieldGrid::new(w, h, 1.0, Vec2::ZERO);
        // Sembrar freq dominante para que alignment no sea 0
        for y in 0..h {
            for x in 0..w {
                if let Some(c) = field.cell_xy_mut(x, y) {
                    c.dominant_frequency_hz = freq;
                }
            }
        }
        field
    }

    fn species_with_spot(w: usize, h: usize, sp: u8, qe: f32) -> SpeciesGrid {
        let mut g = SpeciesGrid::new(w, h, 50.0);
        let cx = w / 2;
        let cy = h / 2;
        let s = SpeciesId::new(sp).unwrap();
        for dy in -2_i32..=2 {
            for dx in -2_i32..=2 {
                let x = (cx as i32 + dx) as usize;
                let y = (cy as i32 + dy) as usize;
                if x < w && y < h { g.seed(x, y, s, qe); }
            }
        }
        g
    }

    // ── Pure fn: comportamiento ────────────────────────────────────────────

    #[test]
    fn no_op_on_empty_grid() {
        let species = SpeciesGrid::new(8, 8, 50.0);
        let net = formose_net();
        let mut field = empty_field(8, 8, 50.0);
        let pre = field.total_qe();
        inject_species_to_field(&species, &net, &mut field, 0.1);
        assert_eq!(field.total_qe(), pre, "grid vacío ⇒ cero injection");
    }

    #[test]
    fn no_op_on_zero_dt() {
        let species = species_with_spot(8, 8, 1, 50.0); // C2
        let net = formose_net();
        let mut field = empty_field(8, 8, 50.0);
        let pre = field.total_qe();
        inject_species_to_field(&species, &net, &mut field, 0.0);
        assert_eq!(field.total_qe(), pre, "dt=0 ⇒ no-op");
    }

    #[test]
    fn injects_qe_when_freq_aligned() {
        // formose tiene reacciones a freq=50; campo freq=50 ⇒ alignment ≈ 1
        let species = species_with_spot(8, 8, 1, 100.0);
        let net = formose_net();
        let mut field = empty_field(8, 8, 50.0);
        inject_species_to_field(&species, &net, &mut field, 1.0);
        let post_qe = field.total_qe();
        assert!(post_qe > 0.0, "spot+aligned freq ⇒ qe inyectado, got {post_qe}");
        // Cota superior: spot 5×5 = 25 cells × 100 qe × 1.0 align × 0.02 coupling × 1 dt = 50
        // No estricto porque alignment puede ser <1 con freq=50.
        assert!(post_qe <= 25.0 * 100.0 * SPECIES_TO_QE_COUPLING * 1.0 + 1e-3,
                "no excede cota teórica, got {post_qe}");
    }

    #[test]
    fn no_injection_when_freq_misaligned() {
        // formose freq=50; campo freq=300 ⇒ alignment ≈ 0
        let species = species_with_spot(8, 8, 1, 100.0);
        let net = formose_net();
        let mut field = empty_field(8, 8, 300.0);
        inject_species_to_field(&species, &net, &mut field, 1.0);
        let post_qe = field.total_qe();
        // Bandwidth 50 Hz, |Δf|=250 → alignment cae a casi cero
        assert!(post_qe < 0.1,
                "freq desalineada ⇒ injection ≈ 0, got {post_qe}");
    }

    #[test]
    fn injection_is_deterministic() {
        let net = formose_net();
        let mut field_a = empty_field(8, 8, 50.0);
        let mut field_b = empty_field(8, 8, 50.0);
        let species = species_with_spot(8, 8, 1, 50.0);
        for _ in 0..10 {
            inject_species_to_field(&species, &net, &mut field_a, 0.1);
            inject_species_to_field(&species, &net, &mut field_b, 0.1);
        }
        assert_eq!(field_a.total_qe(), field_b.total_qe(),
                   "mismo input ⇒ mismo output (10 iter)");
    }

    #[test]
    fn handles_grid_size_mismatch() {
        // species 16×16, field 8×8 — sólo región común se procesa, sin panic
        let species = species_with_spot(16, 16, 1, 50.0);
        let net = formose_net();
        let mut field = empty_field(8, 8, 50.0);
        inject_species_to_field(&species, &net, &mut field, 1.0);
        // No panic + algún qe (spot 5×5 centrado en (8,8) cae fuera del field 8×8;
        // sí cae el cuadrante superior-izquierdo del spot → cells 6,7,_)
        // Si el centro está a (8,8), esquina inferior-derecha del spot está en (10,10)
        // que está fuera del field 8×8.  Pero cells (6,6),(6,7),(7,6),(7,7) sí están dentro.
        assert!(field.total_qe() >= 0.0); // contrato mínimo: no panic
    }

    // ── Empty network edge ─────────────────────────────────────────────────

    #[test]
    fn no_op_on_empty_network() {
        let species = species_with_spot(8, 8, 1, 50.0);
        let net = ReactionNetwork::default();
        let mut field = empty_field(8, 8, 50.0);
        inject_species_to_field(&species, &net, &mut field, 1.0);
        assert_eq!(field.total_qe(), 0.0, "red vacía ⇒ no injection");
    }
}
