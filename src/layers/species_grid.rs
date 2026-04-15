//! AP-0: SpeciesGrid — concentraciones por celda (SoA), Resource opt-in.
//! AP-0: SpeciesGrid — per-cell concentrations (SoA), opt-in Resource.
//!
//! No toca el `NutrientFieldGrid` existente: este grid se instala sólo cuando
//! un track (p. ej. AUTOPOIESIS) lo necesita.  Grid ≠ Component — es un recurso
//! plano 2D cuya única responsabilidad es representar `[f32; MAX_SPECIES]` por
//! celda más una frecuencia ambiental (para catálisis Axiom 8).
//!
//! Se modifica con `diffuse_species` y `apply_reaction` (ver `reaction_kinetics`).

use bevy::prelude::*;

use crate::blueprint::constants::chemistry::MAX_SPECIES;
use crate::layers::reaction::SpeciesId;

/// Concentraciones + frecuencia de una celda. Alineación natural a 4 bytes.
/// Por-celda: 128 B de especies + 4 B de `freq` = 132 B.
#[derive(Clone, Debug)]
pub struct SpeciesCell {
    pub species: [f32; MAX_SPECIES],
    pub freq: f32,
}

impl Default for SpeciesCell {
    fn default() -> Self {
        Self { species: [0.0; MAX_SPECIES], freq: 0.0 }
    }
}

impl SpeciesCell {
    /// Total de `qe` acumulado en la celda (Σ de todas las especies).
    #[inline]
    pub fn total_qe(&self) -> f32 {
        self.species.iter().copied().sum()
    }
}

/// Grid 2D row-major de celdas químicas. Dimensiones fijas por construcción.
#[derive(Resource, Clone, Debug)]
pub struct SpeciesGrid {
    width: usize,
    height: usize,
    cells: Vec<SpeciesCell>,
}

impl SpeciesGrid {
    /// Crea un grid homogéneo (todas las concentraciones 0, freq uniforme).
    pub fn new(width: usize, height: usize, freq: f32) -> Self {
        let n = width.checked_mul(height).expect("grid size overflow");
        let mut cells = vec![SpeciesCell::default(); n];
        for c in &mut cells { c.freq = freq; }
        Self { width, height, cells }
    }

    #[inline] pub fn width(&self) -> usize { self.width }
    #[inline] pub fn height(&self) -> usize { self.height }
    #[inline] pub fn len(&self) -> usize { self.cells.len() }
    #[inline] pub fn is_empty(&self) -> bool { self.cells.is_empty() }

    /// Índice row-major. **Panic** (debug) si fuera de rango.
    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        debug_assert!(x < self.width && y < self.height, "cell ({x},{y}) out of range");
        y * self.width + x
    }

    #[inline]
    pub fn cell(&self, x: usize, y: usize) -> &SpeciesCell {
        &self.cells[self.idx(x, y)]
    }

    #[inline]
    pub fn cell_mut(&mut self, x: usize, y: usize) -> &mut SpeciesCell {
        let i = self.idx(x, y);
        &mut self.cells[i]
    }

    #[inline] pub fn cells(&self) -> &[SpeciesCell] { &self.cells }
    #[inline] pub fn cells_mut(&mut self) -> &mut [SpeciesCell] { &mut self.cells }

    /// Acceso a celda con coords flotantes (centroides PCA, etc.).
    /// Devuelve `None` si las coords (truncadas a `usize`) caen fuera del grid.
    /// AI-2 (ADR-044): usado por `build_fission_event` con centroide PCA del blob.
    #[inline]
    pub fn cell_xy_clamped(&self, x: f32, y: f32) -> Option<&SpeciesCell> {
        if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 { return None; }
        let xu = x as usize;
        let yu = y as usize;
        if xu >= self.width || yu >= self.height { return None; }
        Some(&self.cells[yu * self.width + xu])
    }

    /// Suma global de una especie en el grid.  `NONE` → `0.0`.
    pub fn total_for_species(&self, s: SpeciesId) -> f32 {
        if s.is_none() { return 0.0; }
        let i = s.index();
        self.cells.iter().map(|c| c.species[i]).sum()
    }

    /// Vector de totales por especie (útil para AP-1 `food_set_from_grid`).
    pub fn totals_per_species(&self) -> [f32; MAX_SPECIES] {
        let mut out = [0.0_f32; MAX_SPECIES];
        for c in &self.cells {
            for (acc, &v) in out.iter_mut().zip(c.species.iter()) { *acc += v; }
        }
        out
    }

    /// Qe total en todo el grid (Σ todas las celdas, Σ todas las especies).
    pub fn total_qe(&self) -> f32 {
        self.cells.iter().map(SpeciesCell::total_qe).sum()
    }

    /// Siembra una celda con una concentración dada. No-op si `species.is_none()`.
    pub fn seed(&mut self, x: usize, y: usize, s: SpeciesId, amount: f32) {
        if s.is_none() || !amount.is_finite() { return; }
        let i = s.index();
        self.cell_mut(x, y).species[i] = amount.max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(w: usize, h: usize) -> SpeciesGrid { SpeciesGrid::new(w, h, 50.0) }

    #[test]
    fn new_initializes_zero_concentrations() {
        let g = mk(4, 3);
        assert_eq!(g.width(), 4);
        assert_eq!(g.height(), 3);
        assert_eq!(g.len(), 12);
        assert_eq!(g.total_qe(), 0.0);
    }

    #[test]
    fn freq_propagates_to_all_cells() {
        let g = mk(2, 2);
        for c in g.cells() { assert_eq!(c.freq, 50.0); }
    }

    #[test]
    fn seed_and_read_back() {
        let mut g = mk(2, 2);
        let s = SpeciesId::new(3).unwrap();
        g.seed(1, 0, s, 4.5);
        assert_eq!(g.cell(1, 0).species[3], 4.5);
        assert_eq!(g.total_for_species(s), 4.5);
    }

    #[test]
    fn seed_ignores_none_species() {
        let mut g = mk(1, 1);
        g.seed(0, 0, SpeciesId::NONE, 1.0);
        assert_eq!(g.total_qe(), 0.0);
    }

    #[test]
    fn seed_clamps_negative_to_zero() {
        let mut g = mk(1, 1);
        g.seed(0, 0, SpeciesId::new(0).unwrap(), -3.0);
        assert_eq!(g.cell(0, 0).species[0], 0.0);
    }

    #[test]
    fn totals_per_species_sums_all_cells() {
        let mut g = mk(3, 1);
        let s = SpeciesId::new(2).unwrap();
        g.seed(0, 0, s, 1.0);
        g.seed(1, 0, s, 2.0);
        g.seed(2, 0, s, 3.0);
        let totals = g.totals_per_species();
        assert_eq!(totals[2], 6.0);
        // other slots remain zero
        assert_eq!(totals.iter().sum::<f32>(), 6.0);
    }

    #[test]
    fn index_is_row_major() {
        let g = mk(4, 2);
        assert_eq!(g.idx(0, 0), 0);
        assert_eq!(g.idx(3, 0), 3);
        assert_eq!(g.idx(0, 1), 4);
        assert_eq!(g.idx(3, 1), 7);
    }
}
