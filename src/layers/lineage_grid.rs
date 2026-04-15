//! AP-6c (ADR-041): `LineageGrid` — tag per-celda del linaje dueño.
//! AP-6c (ADR-041): `LineageGrid` — per-cell lineage ownership tag.
//!
//! Capa de datos **paralela a `SpeciesGrid`** (`layers/species_grid.rs`): cada
//! celda guarda un `u64` que identifica al linaje cuya closure la marcó por
//! última vez.  `0` significa "sopa primordial, sin linaje asignado".
//!
//! Justificación (ADR-041 §2): para saber qué closure fisionó cuando
//! `pressure_ratio` cruza el umbral, el harness necesita una respuesta O(1)
//! a "¿de quién es esta celda?".  El membrane mask identifica qué especies
//! son de membrana, pero no qué closure es dueña de un blob — eso lo fija
//! el stamping per-celda en el snapshot inicial.
//!
//! Zero Bevy: `LineageGrid` es plain data.  Se instancia dentro del harness
//! (`SoupSim`) sin tocar `Resource`.  La viz Bevy de AP-6c.1+ la envolverá
//! con `#[derive(Resource)]` localmente si hace falta.
//!
//! Axiom 6: el tag emerge del tracking — no se declara identidad en el spawn.

/// Layout row-major, mismo orden que `SpeciesGrid::cells`.  `tags.len() == w*h`.
/// Max 3 campos (data, no component).
#[derive(Clone, Debug)]
pub struct LineageGrid {
    tags: Vec<u64>,
    width: usize,
    height: usize,
}

impl LineageGrid {
    /// Nueva grilla de `w × h` celdas, todas en linaje `0` ("sopa primordial").
    pub fn new(width: usize, height: usize) -> Self {
        Self { tags: vec![0; width.saturating_mul(height)], width, height }
    }

    #[inline] pub fn width(&self) -> usize { self.width }
    #[inline] pub fn height(&self) -> usize { self.height }
    #[inline] pub fn len(&self) -> usize { self.tags.len() }
    #[inline] pub fn is_empty(&self) -> bool { self.tags.is_empty() }

    /// Tag en `(x, y)`.  Fuera de rango ⇒ `0` (defensivo).
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u64 {
        if x >= self.width || y >= self.height { return 0; }
        self.tags[y * self.width + x]
    }

    /// Escribe `lineage` en `(x, y)` si está en rango; fuera ⇒ no-op.
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, lineage: u64) {
        if x >= self.width || y >= self.height { return; }
        self.tags[y * self.width + x] = lineage;
    }

    /// Tag un conjunto de celdas.  Política ADR-041 §4 "primero gana": sólo
    /// sobreescribe celdas cuyo tag actual es `0` — closures ya dueñas de
    /// una celda no son desplazadas por un mark posterior en el mismo tick.
    pub fn stamp_if_unowned(&mut self, cells: &[(u16, u16)], lineage: u64) {
        for &(x, y) in cells {
            let (xu, yu) = (x as usize, y as usize);
            if xu >= self.width || yu >= self.height { continue; }
            let idx = yu * self.width + xu;
            if self.tags[idx] == 0 { self.tags[idx] = lineage; }
        }
    }

    /// Tag incondicional — usado por `apply_fission` para sobrescribir los
    /// dos lados con sus `lineage_a`/`lineage_b` recién nacidos.
    pub fn stamp(&mut self, cells: &[(u16, u16)], lineage: u64) {
        for &(x, y) in cells {
            self.set(x as usize, y as usize, lineage);
        }
    }

    /// Linaje mayoritario sobre un conjunto de celdas.  Ignora el tag `0`
    /// (pre-linaje) salvo que **todas** sean `0` (retorna `0`).  Empates ⇒
    /// gana el linaje con menor `u64` (determinismo).  `cells` vacío ⇒ `0`.
    ///
    /// O(n²) sobre `cells` — aceptable: `blob.cells.len()` es típicamente
    /// < 256 (ADR-039 cost model: `blob_size` pequeño).
    pub fn dominant_lineage(&self, cells: &[(u16, u16)]) -> u64 {
        if cells.is_empty() { return 0; }
        let mut best: u64 = 0;
        let mut best_count: u32 = 0;
        for (i, &(x, y)) in cells.iter().enumerate() {
            let tag = self.get(x as usize, y as usize);
            if tag == 0 { continue; }
            let mut count: u32 = 1;
            for &(xj, yj) in &cells[i + 1..] {
                if self.get(xj as usize, yj as usize) == tag { count += 1; }
            }
            if count > best_count || (count == best_count && tag < best) {
                best = tag;
                best_count = count;
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_zero_initialized() {
        let g = LineageGrid::new(4, 3);
        assert_eq!(g.width(), 4);
        assert_eq!(g.height(), 3);
        assert_eq!(g.len(), 12);
        for y in 0..3 { for x in 0..4 { assert_eq!(g.get(x, y), 0); } }
    }

    #[test]
    fn get_out_of_range_returns_zero() {
        let mut g = LineageGrid::new(2, 2);
        g.set(0, 0, 42);
        assert_eq!(g.get(99, 99), 0);
        assert_eq!(g.get(0, 0), 42);
    }

    #[test]
    fn stamp_if_unowned_respects_first_wins() {
        let mut g = LineageGrid::new(4, 4);
        g.stamp_if_unowned(&[(1, 1), (2, 2)], 111);
        g.stamp_if_unowned(&[(1, 1), (3, 3)], 222);
        assert_eq!(g.get(1, 1), 111, "first writer keeps");
        assert_eq!(g.get(2, 2), 111);
        assert_eq!(g.get(3, 3), 222, "previously 0 → writable");
    }

    #[test]
    fn stamp_unconditional_overwrites() {
        let mut g = LineageGrid::new(4, 4);
        g.stamp(&[(1, 1)], 111);
        g.stamp(&[(1, 1)], 222);
        assert_eq!(g.get(1, 1), 222);
    }

    #[test]
    fn stamp_ignores_out_of_range_cells() {
        let mut g = LineageGrid::new(2, 2);
        g.stamp(&[(0, 0), (99, 99), (1, 1)], 7);
        assert_eq!(g.get(0, 0), 7);
        assert_eq!(g.get(1, 1), 7);
        // 99,99 silenciosamente ignorada — no panic.
    }

    #[test]
    fn dominant_lineage_empty_is_zero() {
        let g = LineageGrid::new(4, 4);
        assert_eq!(g.dominant_lineage(&[]), 0);
    }

    #[test]
    fn dominant_lineage_all_unowned_is_zero() {
        let g = LineageGrid::new(4, 4);
        assert_eq!(g.dominant_lineage(&[(1, 1), (2, 2)]), 0);
    }

    #[test]
    fn dominant_lineage_majority_wins() {
        let mut g = LineageGrid::new(4, 4);
        g.stamp(&[(0, 0), (1, 0), (2, 0)], 100);
        g.stamp(&[(0, 1)], 200);
        let cells = [(0, 0), (1, 0), (2, 0), (0, 1)];
        assert_eq!(g.dominant_lineage(&cells), 100);
    }

    #[test]
    fn dominant_lineage_ties_break_by_smaller_id() {
        let mut g = LineageGrid::new(4, 4);
        g.stamp(&[(0, 0)], 999);
        g.stamp(&[(1, 0)], 111);
        let cells = [(0, 0), (1, 0)];
        assert_eq!(g.dominant_lineage(&cells), 111, "tie → smaller lineage wins");
    }
}
