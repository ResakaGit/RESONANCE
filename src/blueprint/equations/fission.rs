//! AP-4: Fission — criterio y geometría de división emergente (ADR-039).
//! AP-4: Fission — emergent division criterion and geometry.
//!
//! Una vesícula se divide cuando la **presión interna** (producción química
//! acumulada) supera la **capacidad de cohesión** (perímetro × fuerza media
//! de membrana).  Ningún threshold mágico: `FISSION_PRESSURE_RATIO` se deriva
//! de las 4 constantes fundamentales (`DISSIPATION_PLASMA / DISSIPATION_SOLID`).
//!
//! Pure fns + Copy structs.  Sin mutación más allá de `apply_fission`, que
//! toma `&mut SpeciesGrid` explícitamente (llegará en Cycle 3).
//!
//! Axiom 3: selección emergente — closures fuertes se replican, débiles mueren.
//! Axiom 6: criterio emergente, no decretado.
//! Axiom 7: `pinch_axis` = eje principal de la covarianza espacial (PCA 2D).

use crate::blueprint::constants::chemistry::{KINETIC_STABILITY_EPSILON, MAX_SPECIES};
use crate::blueprint::equations::blob_topology::{BlobIndex, centroid, perimeter};
use crate::blueprint::equations::derived_thresholds::DISSIPATION_PLASMA;
use crate::blueprint::equations::reaction_kinetics::mass_action_rate;
use crate::layers::reaction_network::ReactionNetwork;
use crate::layers::species_grid::SpeciesGrid;
use crate::math_types::Vec2;

// ── Producción interna (Axiom 3) ────────────────────────────────────────────

/// Suma de `mass_action_rate` sobre toda reacción y toda celda del blob.
/// Unidad: qe × tick⁻¹ (agregado).  Celdas fuera del grid se ignoran.
pub fn internal_production(
    blob: &BlobIndex,
    grid: &SpeciesGrid,
    network: &ReactionNetwork,
    bandwidth: f32,
) -> f32 {
    let (w, h) = (grid.width(), grid.height());
    let mut total = 0.0_f32;
    for &(x, y) in &blob.cells {
        let (xu, yu) = (x as usize, y as usize);
        if xu >= w || yu >= h { continue; }
        let cell = grid.cell(xu, yu);
        for r in network.reactions() {
            total += mass_action_rate(&cell.species, r, cell.freq, bandwidth);
        }
    }
    total
}

// ── Cohesión (perímetro × fuerza media) ─────────────────────────────────────

/// Media de `strength_field` sobre celdas del blob.  Vacío ⇒ 0.
pub fn mean_membrane_strength(
    blob: &BlobIndex,
    strength_field: &[f32],
    width: usize,
    height: usize,
) -> f32 {
    if blob.is_empty() || width == 0 || height == 0 { return 0.0; }
    let n_expected = width * height;
    if strength_field.len() != n_expected { return 0.0; }
    let mut sum = 0.0_f32;
    let mut count = 0_u32;
    for &(x, y) in &blob.cells {
        let (xu, yu) = (x as usize, y as usize);
        if xu >= width || yu >= height { continue; }
        let v = strength_field[yu * width + xu];
        if v.is_finite() {
            sum += v;
            count += 1;
        }
    }
    if count == 0 { 0.0 } else { sum / count as f32 }
}

/// Capacidad de contención: `perímetro × fuerza_media_de_membrana`.
#[inline]
pub fn cohesion_capacity(
    blob: &BlobIndex,
    strength_field: &[f32],
    width: usize,
    height: usize,
) -> f32 {
    let p = perimeter(blob, width, height) as f32;
    p * mean_membrane_strength(blob, strength_field, width, height)
}

// ── Pressure ratio (criterio de fisión) ─────────────────────────────────────

/// Razón de presión interna sobre cohesión de membrana.
/// `ratio > FISSION_PRESSURE_RATIO` ⇒ el blob debe dividirse (trigger en AP-6).
/// Cohesión nula ⇒ retorna `0.0` (sin membrana no hay tensión que vencer,
/// el blob se disuelve por difusión; no es fisión — es muerte).
pub fn pressure_ratio(
    blob: &BlobIndex,
    grid: &SpeciesGrid,
    network: &ReactionNetwork,
    strength_field: &[f32],
    bandwidth: f32,
) -> f32 {
    let cohesion = cohesion_capacity(blob, strength_field, grid.width(), grid.height());
    if cohesion <= KINETIC_STABILITY_EPSILON { return 0.0; }
    internal_production(blob, grid, network, bandwidth) / cohesion
}

// ── Pinch axis (PCA 2D, closed-form) ────────────────────────────────────────

/// Eje principal de la covarianza espacial del blob — dirección de pinch.
/// Blob vacío o singleton ⇒ fallback `Vec2::X` (división horizontal arbitraria).
/// Resultado siempre normalizado (‖v‖ ≈ 1).  Signo arbitrario (propiedad PCA).
pub fn pinch_axis(blob: &BlobIndex) -> Vec2 {
    if blob.cells.len() < 2 { return Vec2::X; }
    let c = centroid(blob);

    // Matriz de covarianza 2×2 simétrica.
    let (mut sxx, mut syy, mut sxy) = (0.0_f32, 0.0_f32, 0.0_f32);
    for &(x, y) in &blob.cells {
        let dx = x as f32 - c.x;
        let dy = y as f32 - c.y;
        sxx += dx * dx;
        syy += dy * dy;
        sxy += dx * dy;
    }
    let n = blob.cells.len() as f32;
    let cxx = sxx / n;
    let cyy = syy / n;
    let cxy = sxy / n;

    // Autovalores: λ = (trace ± √(trace² − 4·det)) / 2.
    let trace = cxx + cyy;
    let det = cxx * cyy - cxy * cxy;
    let disc = (trace * trace - 4.0 * det).max(0.0);
    let lambda_max = 0.5 * (trace + disc.sqrt());

    // Autovector asociado: si cxy ≠ 0, (λ − cyy, cxy). Si cxy = 0, eje canónico.
    let v = if cxy.abs() > f32::EPSILON {
        Vec2::new(lambda_max - cyy, cxy)
    } else if cxx >= cyy {
        Vec2::X
    } else {
        Vec2::Y
    };

    v.try_normalize().unwrap_or(Vec2::X)
}

// ── Child lineage hash (FNV-1a, determinístico) ─────────────────────────────

/// Hash determinístico `(parent, tick, side)` → u64, estilo FNV-1a (mismo
/// esquema que `closure_hash`).  `side ∈ {0, 1}` separa los dos hijos.
pub fn child_lineage(parent: u64, tick: u64, side: u8) -> u64 {
    fnv1a_mix3(parent, tick, side as u64)
}

/// Mapea un `closure.hash` a un `lineage_id` no-cero determinístico
/// (ADR-041).  Sirve como identidad inicial del linaje antes de cualquier
/// fisión: todas las celdas mask-marcadas por la closure reciben este tag.
///
/// Garantía: `hash_to_lineage(h) != 0` — el cero queda reservado como "sopa
/// primordial sin linaje asignado".  Si la mezcla FNV-1a produjera 0 (colisión
/// teórica), retornamos el valor degenerado `1`; el sesgo es numéricamente
/// indetectable (2^-64) y preserva la regla `0 ⇔ pre-linaje`.
pub fn hash_to_lineage(closure_hash: u64) -> u64 {
    // `tick`/`side` artificiales para re-usar el mismo avalanche FNV que
    // `child_lineage`, pero con un dominio disjunto (side=2).
    let h = fnv1a_mix3(closure_hash, 0, 2);
    if h == 0 { 1 } else { h }
}

#[inline]
fn fnv1a_mix3(a: u64, b: u64, c: u64) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    #[inline]
    fn mix(mut h: u64, x: u64) -> u64 {
        for shift in (0..64).step_by(8) {
            h ^= (x >> shift) & 0xff;
            h = h.wrapping_mul(FNV_PRIME);
        }
        h
    }
    mix(mix(mix(FNV_OFFSET, a), b), c)
}

// ── apply_fission (partición + tax) ─────────────────────────────────────────

/// Resultado de un evento de fisión.  Max 5 campos (data struct, no component).
#[derive(Clone, Debug)]
pub struct FissionOutcome {
    /// Celdas asignadas al hijo A (lado positivo del eje perpendicular).
    pub cells_a: Vec<(u16, u16)>,
    /// Celdas asignadas al hijo B (lado negativo).
    pub cells_b: Vec<(u16, u16)>,
    /// Linaje heredado por el hijo A.
    pub lineage_a: u64,
    /// Linaje heredado por el hijo B.
    pub lineage_b: u64,
    /// qe disipado por el tax de fisión (Axiom 4): `DISSIPATION_PLASMA × pre_qe`.
    pub dissipated_qe: f32,
}

impl FissionOutcome {
    /// Outcome trivial — blob vacío o axis degenerado; ningún mutado aplicado.
    pub fn empty(parent: u64, tick: u64) -> Self {
        Self {
            cells_a: Vec::new(),
            cells_b: Vec::new(),
            lineage_a: child_lineage(parent, tick, 0),
            lineage_b: child_lineage(parent, tick, 1),
            dissipated_qe: 0.0,
        }
    }
}

/// Ejecuta la fisión del blob: particiona sus celdas según el eje perpendicular
/// a `axis`, aplica el tax `DISSIPATION_PLASMA` a cada celda (escalando todas
/// las especies), y retorna los dos conjuntos con sus linajes derivados.
///
/// Conservación (ADR-039, Axioms 2 + 4):
///   Σ species pre-fission = Σ species post-fission + dissipated_qe.
///
/// Grid se muta **sólo en las celdas del blob**.  Celdas fuera del grid se
/// ignoran silenciosamente (pueden venir de un blob pre-computado frente a
/// un grid re-dimensionado).
pub fn apply_fission(
    grid: &mut SpeciesGrid,
    blob: &BlobIndex,
    axis: Vec2,
    parent_lineage: u64,
    tick: u64,
) -> FissionOutcome {
    if blob.is_empty() {
        return FissionOutcome::empty(parent_lineage, tick);
    }
    let axis = axis.try_normalize().unwrap_or(Vec2::X);
    // Perpendicular ccw — define el lado positivo del pinch.
    let perp = Vec2::new(-axis.y, axis.x);
    let c = centroid(blob);

    let (w, h) = (grid.width(), grid.height());
    let mut cells_a: Vec<(u16, u16)> = Vec::with_capacity(blob.cells.len() / 2 + 1);
    let mut cells_b: Vec<(u16, u16)> = Vec::with_capacity(blob.cells.len() / 2 + 1);
    let mut pre_qe = 0.0_f32;

    for &(x, y) in &blob.cells {
        // Partición determinística (== 0 ⇒ A).
        let d = Vec2::new(x as f32 - c.x, y as f32 - c.y).dot(perp);
        if d >= 0.0 { cells_a.push((x, y)); } else { cells_b.push((x, y)); }

        let (xu, yu) = (x as usize, y as usize);
        if xu < w && yu < h {
            let cell = grid.cell_mut(xu, yu);
            let mut cell_pre = 0.0_f32;
            for s in 0..MAX_SPECIES {
                cell_pre += cell.species[s];
                cell.species[s] *= 1.0 - DISSIPATION_PLASMA;
            }
            pre_qe += cell_pre;
        }
    }

    FissionOutcome {
        cells_a,
        cells_b,
        lineage_a: child_lineage(parent_lineage, tick, 0),
        lineage_b: child_lineage(parent_lineage, tick, 1),
        dissipated_qe: pre_qe * DISSIPATION_PLASMA,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::chemistry::REACTION_FREQ_BANDWIDTH_DEFAULT as BW;
    use crate::layers::reaction::SpeciesId;

    fn raf_net() -> ReactionNetwork {
        let spec = r#"(reactions: [
            (reactants: [(0,1),(1,1)], products: [(2,1)],       k: 1.0, freq: 50.0),
            (reactants: [(2,1)],       products: [(0,1),(3,1)], k: 0.5, freq: 50.0),
            (reactants: [(3,1),(1,1)], products: [(1,1),(2,1)], k: 0.8, freq: 50.0),
        ])"#;
        ReactionNetwork::from_ron_str(spec).unwrap()
    }

    fn blob_rect(x0: u16, y0: u16, x1: u16, y1: u16) -> BlobIndex {
        let mut cells = Vec::new();
        for y in y0..=y1 { for x in x0..=x1 { cells.push((x, y)); } }
        BlobIndex { id: 0, cells, lineage: 0 }
    }

    // ── internal_production ────────────────────────────────────────────────

    #[test]
    fn production_zero_without_reactants() {
        let net = raf_net();
        let grid = SpeciesGrid::new(4, 4, 50.0); // vacía
        let b = blob_rect(1, 1, 2, 2);
        assert_eq!(internal_production(&b, &grid, &net, BW), 0.0);
    }

    #[test]
    fn production_accumulates_across_cells() {
        let net = raf_net();
        let mut grid = SpeciesGrid::new(4, 4, 50.0);
        for y in 1..=2 {
            for x in 1..=2 {
                grid.seed(x, y, SpeciesId::new(0).unwrap(), 2.0);
                grid.seed(x, y, SpeciesId::new(1).unwrap(), 2.0);
            }
        }
        let small = blob_rect(1, 1, 1, 1);
        let big   = blob_rect(1, 1, 2, 2);
        let p_small = internal_production(&small, &grid, &net, BW);
        let p_big   = internal_production(&big,   &grid, &net, BW);
        assert!(p_big > p_small);
        assert!((p_big - 4.0 * p_small).abs() < 1e-4, "linear in cell count");
    }

    // ── cohesion_capacity ──────────────────────────────────────────────────

    #[test]
    fn cohesion_is_zero_on_zero_strength_field() {
        let grid = SpeciesGrid::new(4, 4, 50.0);
        let b = blob_rect(1, 1, 2, 2);
        let field = vec![0.0_f32; grid.len()];
        assert_eq!(cohesion_capacity(&b, &field, grid.width(), grid.height()), 0.0);
    }

    #[test]
    fn cohesion_uses_perimeter_and_mean_strength() {
        let grid = SpeciesGrid::new(4, 4, 50.0);
        let b = blob_rect(1, 1, 2, 2); // 2×2 ⇒ perimeter = 8
        let field = vec![0.5_f32; grid.len()];
        let c = cohesion_capacity(&b, &field, grid.width(), grid.height());
        assert!((c - 8.0 * 0.5).abs() < 1e-5);
    }

    #[test]
    fn cohesion_guards_field_length_mismatch() {
        let b = blob_rect(0, 0, 1, 1);
        let field = vec![1.0_f32; 3]; // demasiado corto para 4×4
        assert_eq!(cohesion_capacity(&b, &field, 4, 4), 0.0);
    }

    // ── pressure_ratio ─────────────────────────────────────────────────────

    #[test]
    fn pressure_ratio_is_zero_when_cohesion_zero() {
        let net = raf_net();
        let mut grid = SpeciesGrid::new(4, 4, 50.0);
        grid.seed(1, 1, SpeciesId::new(0).unwrap(), 10.0);
        grid.seed(1, 1, SpeciesId::new(1).unwrap(), 10.0);
        let b = blob_rect(1, 1, 2, 2);
        let zero_field = vec![0.0_f32; grid.len()];
        assert_eq!(pressure_ratio(&b, &grid, &net, &zero_field, BW), 0.0);
    }

    #[test]
    fn pressure_ratio_increases_with_production() {
        let net = raf_net();
        let mut grid_lo = SpeciesGrid::new(4, 4, 50.0);
        let mut grid_hi = SpeciesGrid::new(4, 4, 50.0);
        for y in 1..=2 { for x in 1..=2 {
            grid_lo.seed(x, y, SpeciesId::new(0).unwrap(), 1.0);
            grid_lo.seed(x, y, SpeciesId::new(1).unwrap(), 1.0);
            grid_hi.seed(x, y, SpeciesId::new(0).unwrap(), 10.0);
            grid_hi.seed(x, y, SpeciesId::new(1).unwrap(), 10.0);
        }}
        let b = blob_rect(1, 1, 2, 2);
        let field = vec![0.3_f32; grid_lo.len()];
        let r_lo = pressure_ratio(&b, &grid_lo, &net, &field, BW);
        let r_hi = pressure_ratio(&b, &grid_hi, &net, &field, BW);
        assert!(r_hi > r_lo);
    }

    // ── pinch_axis (PCA) ───────────────────────────────────────────────────

    #[test]
    fn pinch_axis_horizontal_on_elongated_x() {
        // Blob 5×1 a lo ancho ⇒ eje principal ≈ (1, 0).
        let cells: Vec<(u16, u16)> = (0..5).map(|x| (x, 3)).collect();
        let b = BlobIndex { id: 0, cells, lineage: 0 };
        let a = pinch_axis(&b);
        assert!(a.x.abs() > 0.99, "expected |ax| ≈ 1, got {a:?}");
        assert!(a.y.abs() < 0.01);
    }

    #[test]
    fn pinch_axis_vertical_on_elongated_y() {
        let cells: Vec<(u16, u16)> = (0..5).map(|y| (3, y)).collect();
        let b = BlobIndex { id: 0, cells, lineage: 0 };
        let a = pinch_axis(&b);
        assert!(a.y.abs() > 0.99, "expected |ay| ≈ 1, got {a:?}");
        assert!(a.x.abs() < 0.01);
    }

    #[test]
    fn pinch_axis_falls_back_on_trivial_input() {
        let empty = BlobIndex { id: 0, cells: vec![], lineage: 0 };
        assert_eq!(pinch_axis(&empty), Vec2::X);
        let single = BlobIndex { id: 0, cells: vec![(1, 1)], lineage: 0 };
        assert_eq!(pinch_axis(&single), Vec2::X);
    }

    #[test]
    fn pinch_axis_is_normalized() {
        // Blob diagonal — axis debe ser unit vector a ±(1,1)/√2.
        let b = BlobIndex {
            id: 0,
            cells: vec![(0, 0), (1, 1), (2, 2), (3, 3)],
            lineage: 0,
        };
        let a = pinch_axis(&b);
        assert!((a.length() - 1.0).abs() < 1e-4, "len={}", a.length());
        assert!((a.x.abs() - a.y.abs()).abs() < 0.05);
    }

    // ── child_lineage + apply_fission ──────────────────────────────────────

    #[test]
    fn child_lineage_is_deterministic_and_distinguishes_sides() {
        let a = child_lineage(42, 100, 0);
        let b = child_lineage(42, 100, 1);
        assert_ne!(a, b, "two sides must differ");
        assert_eq!(a, child_lineage(42, 100, 0), "deterministic");
        assert_ne!(a, child_lineage(42, 101, 0), "different tick ⇒ different hash");
        assert_ne!(a, child_lineage(43, 100, 0), "different parent ⇒ different hash");
    }

    #[test]
    fn hash_to_lineage_is_deterministic_and_nonzero() {
        let a = hash_to_lineage(0xABCD_1234);
        let b = hash_to_lineage(0xABCD_1234);
        let c = hash_to_lineage(0xABCE_1234);
        assert_eq!(a, b, "deterministic");
        assert_ne!(a, c, "distinct inputs ⇒ distinct outputs");
        assert_ne!(a, 0, "0 reserved for pre-lineage sopa");
    }

    #[test]
    fn hash_to_lineage_disjoint_from_child_lineage_domain() {
        // child_lineage usa side ∈ {0,1}; hash_to_lineage usa side=2.
        // Basta con muestrear 8 pares (parent=hash) para convencerse.
        for h in [1u64, 2, 100, 0xDEAD, 0xBEEF, u64::MAX, 0xCAFEBABE, 0x1F2E3D4C] {
            let lin = hash_to_lineage(h);
            assert_ne!(lin, child_lineage(h, 0, 0));
            assert_ne!(lin, child_lineage(h, 0, 1));
        }
    }

    fn seed_blob_uniform(grid: &mut SpeciesGrid, blob: &BlobIndex, s: SpeciesId, qe: f32) {
        for &(x, y) in &blob.cells {
            grid.seed(x as usize, y as usize, s, qe);
        }
    }

    #[test]
    fn apply_fission_conserves_mass_modulo_tax() {
        let mut grid = SpeciesGrid::new(6, 6, 50.0);
        let blob = blob_rect(1, 1, 4, 4); // 16 celdas
        seed_blob_uniform(&mut grid, &blob, SpeciesId::new(0).unwrap(), 5.0);
        let pre = grid.total_qe();
        let outcome = apply_fission(&mut grid, &blob, Vec2::X, 42, 7);
        let post = grid.total_qe();
        assert!(
            (pre - post - outcome.dissipated_qe).abs() < 1e-3,
            "pre={pre} post={post} diss={}",
            outcome.dissipated_qe,
        );
        assert!((outcome.dissipated_qe - pre * DISSIPATION_PLASMA).abs() < 1e-3);
    }

    #[test]
    fn apply_fission_partitions_cells_disjointly() {
        let mut grid = SpeciesGrid::new(6, 6, 50.0);
        let blob = blob_rect(1, 1, 4, 4);
        let outcome = apply_fission(&mut grid, &blob, Vec2::X, 0, 0);
        let total = outcome.cells_a.len() + outcome.cells_b.len();
        assert_eq!(total, blob.cells.len());
        let mut all: Vec<_> = outcome.cells_a.iter().chain(&outcome.cells_b).cloned().collect();
        all.sort();
        let before_len = all.len();
        all.dedup();
        assert_eq!(all.len(), before_len, "no cell appears in both halves");
    }

    #[test]
    fn apply_fission_horizontal_axis_splits_by_y() {
        // axis=(1,0) ⇒ perp=(0,1) ⇒ d = (y - cy). Centroide en y=2.5,
        // filas y<2.5 ⇒ cells_b, y≥2.5 ⇒ cells_a.
        let mut grid = SpeciesGrid::new(6, 6, 50.0);
        let blob = blob_rect(1, 1, 4, 4); // centroide (2.5, 2.5)
        let outcome = apply_fission(&mut grid, &blob, Vec2::X, 0, 0);
        assert!(outcome.cells_a.iter().all(|&(_, y)| y as f32 >= 2.5));
        assert!(outcome.cells_b.iter().all(|&(_, y)| (y as f32) < 2.5));
    }

    #[test]
    fn apply_fission_is_deterministic() {
        let mut g1 = SpeciesGrid::new(6, 6, 50.0);
        let mut g2 = SpeciesGrid::new(6, 6, 50.0);
        let blob = blob_rect(1, 1, 4, 4);
        seed_blob_uniform(&mut g1, &blob, SpeciesId::new(0).unwrap(), 3.0);
        seed_blob_uniform(&mut g2, &blob, SpeciesId::new(0).unwrap(), 3.0);
        let o1 = apply_fission(&mut g1, &blob, Vec2::X, 99, 5);
        let o2 = apply_fission(&mut g2, &blob, Vec2::X, 99, 5);
        assert_eq!(o1.cells_a, o2.cells_a);
        assert_eq!(o1.cells_b, o2.cells_b);
        assert_eq!(o1.lineage_a, o2.lineage_a);
        assert_eq!(o1.lineage_b, o2.lineage_b);
        assert!((o1.dissipated_qe - o2.dissipated_qe).abs() < 1e-5);
    }

    #[test]
    fn apply_fission_empty_blob_is_noop_on_grid() {
        let mut grid = SpeciesGrid::new(4, 4, 50.0);
        grid.seed(1, 1, SpeciesId::new(0).unwrap(), 7.0);
        let pre = grid.total_qe();
        let empty = BlobIndex { id: 0, cells: vec![], lineage: 0 };
        let outcome = apply_fission(&mut grid, &empty, Vec2::X, 1, 1);
        assert_eq!(grid.total_qe(), pre, "no mutation on empty blob");
        assert_eq!(outcome.dissipated_qe, 0.0);
        assert!(outcome.cells_a.is_empty() && outcome.cells_b.is_empty());
        assert_ne!(outcome.lineage_a, outcome.lineage_b);
    }

    #[test]
    fn apply_fission_scales_species_by_one_minus_dplasma() {
        let mut grid = SpeciesGrid::new(4, 4, 50.0);
        let blob = blob_rect(1, 1, 2, 2);
        let s = SpeciesId::new(0).unwrap();
        seed_blob_uniform(&mut grid, &blob, s, 10.0);
        let _ = apply_fission(&mut grid, &blob, Vec2::X, 0, 0);
        for &(x, y) in &blob.cells {
            let v = grid.cell(x as usize, y as usize).species[0];
            let expected = 10.0 * (1.0 - DISSIPATION_PLASMA);
            assert!((v - expected).abs() < 1e-4, "v={v} expected={expected}");
        }
    }
}
