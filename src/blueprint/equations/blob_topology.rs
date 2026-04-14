//! AP-4: Blob topology — componentes conexas de alta `membrane_strength`.
//! AP-4: Blob topology — connected components of high membrane strength.
//!
//! El "blob" es la representación discreta de una vesícula emergente (ADR-038):
//! un conjunto de celdas adyacentes cuya fuerza de membrana supera un umbral.
//! Este módulo provee estructura + detección + métricas geométricas puras.
//!
//! Todo es data-en/data-fuera.  Sin Resource, sin system, sin RNG.  El `id` es
//! un contador efímero (regenerable cada tick de detección); el `lineage` es
//! identidad persistente y lo asigna el caller (AP-6 wiring + `LineageRegistry`).
//!
//! Axiom 6: blob emerge del campo — no se declara.
//! Axiom 7: 4-conectividad refleja vecindad discreta sobre grid.

use crate::math_types::Vec2;

/// Componente conexa sobre grid 2D.  Max 4 campos (regla repo).
/// `cells` es row-major-friendly pero no ordenado: flood-fill escribe en DFS.
#[derive(Clone, Debug)]
pub struct BlobIndex {
    /// Contador efímero ∈ [0, n_blobs) dentro de la detección actual.
    pub id: u32,
    /// Coordenadas `(x, y)` de cada celda integrada al blob.
    pub cells: Vec<(u16, u16)>,
    /// Linaje heredado.  `0` indica "sin padre conocido" (pre-poblado por caller).
    pub lineage: u64,
}

impl BlobIndex {
    #[inline] pub fn len(&self) -> usize { self.cells.len() }
    #[inline] pub fn is_empty(&self) -> bool { self.cells.is_empty() }
}

// ── Flood-fill (Axiom 7, 4-conectividad) ────────────────────────────────────

/// Encuentra todas las componentes conexas de `field > threshold` sobre un grid
/// `width × height` row-major.  `lineage` de cada blob queda en `0` — el caller
/// lo sobrescribe al cruzar con su registro de linajes.
///
/// Field no-finito en alguna celda ⇒ esa celda no pertenece a ningún blob
/// (guard Axiom 2).  Field vacío ⇒ retorna `Vec::new()`.
pub fn find_blobs(field: &[f32], width: usize, height: usize, threshold: f32) -> Vec<BlobIndex> {
    let n = width.saturating_mul(height);
    if field.len() != n || n == 0 { return Vec::new(); }

    let mut visited = vec![false; n];
    let mut blobs: Vec<BlobIndex> = Vec::new();
    let mut stack: Vec<(u16, u16)> = Vec::new();

    let is_member = |idx: usize| -> bool {
        let v = field[idx];
        v.is_finite() && v > threshold
    };

    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            if visited[i] || !is_member(i) { continue; }

            // Nuevo blob: DFS iterativo.
            let mut cells: Vec<(u16, u16)> = Vec::new();
            stack.clear();
            stack.push((x as u16, y as u16));
            visited[i] = true;
            while let Some((cx, cy)) = stack.pop() {
                cells.push((cx, cy));
                let cx_u = cx as usize;
                let cy_u = cy as usize;
                // 4-conectividad: izq, der, arriba, abajo.
                let mut push = |nx: usize, ny: usize| {
                    let ni = ny * width + nx;
                    if !visited[ni] && is_member(ni) {
                        visited[ni] = true;
                        stack.push((nx as u16, ny as u16));
                    }
                };
                if cx_u > 0              { push(cx_u - 1, cy_u); }
                if cx_u + 1 < width      { push(cx_u + 1, cy_u); }
                if cy_u > 0              { push(cx_u, cy_u - 1); }
                if cy_u + 1 < height     { push(cx_u, cy_u + 1); }
            }

            blobs.push(BlobIndex {
                id: blobs.len() as u32,
                cells,
                lineage: 0,
            });
        }
    }
    blobs
}

// ── Perimeter (conteo de aristas expuestas) ────────────────────────────────

/// Número de aristas del blob adyacentes a "no-blob" o al borde del grid.
/// Ej.: celda aislada ⇒ 4; cuadrado 2×2 ⇒ 8; blob vacío ⇒ 0.
pub fn perimeter(blob: &BlobIndex, width: usize, height: usize) -> u32 {
    if blob.is_empty() || width == 0 || height == 0 { return 0; }
    let n = width * height;
    let mut in_blob = vec![false; n];
    for &(x, y) in &blob.cells {
        let (xu, yu) = (x as usize, y as usize);
        if xu < width && yu < height { in_blob[yu * width + xu] = true; }
    }
    let mut p = 0_u32;
    for &(x, y) in &blob.cells {
        let xu = x as usize;
        let yu = y as usize;
        // Cada vecino fuera-del-blob (incluyendo off-grid) suma 1 arista.
        let left  = xu == 0           || !in_blob[yu * width + (xu - 1)];
        let right = xu + 1 >= width   || !in_blob[yu * width + (xu + 1)];
        let up    = yu == 0           || !in_blob[(yu - 1) * width + xu];
        let down  = yu + 1 >= height  || !in_blob[(yu + 1) * width + xu];
        p += left as u32 + right as u32 + up as u32 + down as u32;
    }
    p
}

// ── Centroid (media aritmética) ────────────────────────────────────────────

/// Centroide geométrico del blob.  Blob vacío ⇒ `Vec2::ZERO`.
pub fn centroid(blob: &BlobIndex) -> Vec2 {
    if blob.is_empty() { return Vec2::ZERO; }
    let (mut sx, mut sy) = (0.0_f32, 0.0_f32);
    for &(x, y) in &blob.cells {
        sx += x as f32;
        sy += y as f32;
    }
    let n = blob.cells.len() as f32;
    Vec2::new(sx / n, sy / n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field_from(grid: &[&[f32]]) -> (Vec<f32>, usize, usize) {
        let h = grid.len();
        let w = grid.first().map(|r| r.len()).unwrap_or(0);
        let mut out = Vec::with_capacity(w * h);
        for row in grid { out.extend_from_slice(row); }
        (out, w, h)
    }

    // ── find_blobs ─────────────────────────────────────────────────────────

    #[test]
    fn find_blobs_empty_on_empty_grid() {
        assert!(find_blobs(&[], 0, 0, 0.5).is_empty());
    }

    #[test]
    fn find_blobs_ignores_cells_below_threshold() {
        let (f, w, h) = field_from(&[&[0.1, 0.2, 0.3]]);
        assert!(find_blobs(&f, w, h, 0.5).is_empty());
    }

    #[test]
    fn find_blobs_detects_single_component() {
        let (f, w, h) = field_from(&[
            &[0.0, 1.0, 1.0, 0.0],
            &[0.0, 1.0, 1.0, 0.0],
            &[0.0, 0.0, 0.0, 0.0],
        ]);
        let blobs = find_blobs(&f, w, h, 0.5);
        assert_eq!(blobs.len(), 1);
        assert_eq!(blobs[0].len(), 4);
        assert_eq!(blobs[0].lineage, 0, "lineage default 0");
    }

    #[test]
    fn find_blobs_separates_disconnected_components() {
        let (f, w, h) = field_from(&[
            &[1.0, 0.0, 1.0],
            &[1.0, 0.0, 0.0],
            &[0.0, 0.0, 1.0],
        ]);
        let blobs = find_blobs(&f, w, h, 0.5);
        assert_eq!(blobs.len(), 3);
        let sizes: Vec<usize> = blobs.iter().map(|b| b.len()).collect();
        assert!(sizes.contains(&2)); // par vertical izquierda
        assert!(sizes.contains(&1)); // singleton derecha-arriba
    }

    #[test]
    fn find_blobs_excludes_non_finite_cells() {
        let (f, w, h) = field_from(&[&[1.0, f32::NAN, 1.0]]);
        let blobs = find_blobs(&f, w, h, 0.5);
        // NaN rompe conectividad ⇒ dos blobs singletons.
        assert_eq!(blobs.len(), 2);
    }

    // ── perimeter ──────────────────────────────────────────────────────────

    #[test]
    fn perimeter_of_empty_blob_is_zero() {
        let b = BlobIndex { id: 0, cells: vec![], lineage: 0 };
        assert_eq!(perimeter(&b, 4, 4), 0);
    }

    #[test]
    fn perimeter_of_single_cell_is_four() {
        let b = BlobIndex { id: 0, cells: vec![(1, 1)], lineage: 0 };
        assert_eq!(perimeter(&b, 3, 3), 4);
    }

    #[test]
    fn perimeter_of_two_by_two_square_is_eight() {
        let b = BlobIndex {
            id: 0,
            cells: vec![(1, 1), (2, 1), (1, 2), (2, 2)],
            lineage: 0,
        };
        assert_eq!(perimeter(&b, 4, 4), 8);
    }

    #[test]
    fn perimeter_counts_grid_boundary_as_exposed() {
        // Celda en esquina (0,0) de grid 1×1 — los 4 lados son borde.
        let b = BlobIndex { id: 0, cells: vec![(0, 0)], lineage: 0 };
        assert_eq!(perimeter(&b, 1, 1), 4);
    }

    // ── centroid ───────────────────────────────────────────────────────────

    #[test]
    fn centroid_of_empty_blob_is_zero() {
        let b = BlobIndex { id: 0, cells: vec![], lineage: 0 };
        assert_eq!(centroid(&b), Vec2::ZERO);
    }

    #[test]
    fn centroid_of_symmetric_blob_is_at_center() {
        let b = BlobIndex {
            id: 0,
            cells: vec![(1, 1), (3, 1), (1, 3), (3, 3)],
            lineage: 0,
        };
        let c = centroid(&b);
        assert!((c.x - 2.0).abs() < 1e-5);
        assert!((c.y - 2.0).abs() < 1e-5);
    }
}
