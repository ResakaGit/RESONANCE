//! AP-3: Emergent membrane — cohesión por gradiente de productos (ADR-038).
//! AP-3: Emergent membrane — product-density gradient cohesion.
//!
//! Zero componente `Membrane`.  Zero resource `Membrane`.  Todo es lectura
//! pura del gradiente de densidad de especies-producto sobre el `SpeciesGrid`.
//! Lo que un humano ve como "vesícula" es el iso-contour del campo escalar
//! producido aquí — el simulador ignora la palabra "membrana".
//!
//! Axiom 2: damping reduce flux pero nunca destruye qe (pool invariant).
//! Axiom 4: `(1 − DISSIPATION_LIQUID)` + `MEMBRANE_MIN_FLUX_RATIO` ⇒ siempre escapa algo.
//! Axiom 6: membrana no se declara; emerge del campo.
//! Axiom 7: damping exponencial ≡ atenuación efectiva con distancia.

use crate::blueprint::constants::chemistry::{
    MAX_SPECIES, MEMBRANE_DAMPING, MEMBRANE_MIN_FLUX_RATIO,
};
use crate::blueprint::equations::derived_thresholds::DISSIPATION_LIQUID;
use crate::layers::species_grid::SpeciesGrid;
use crate::math_types::Vec2;

// ── Density helper (privado) ────────────────────────────────────────────────

/// Densidad escalar de especies-producto en una celda: `Σ_{s ∈ mask} species[s]`.
#[inline]
fn cell_density(species: &[f32; MAX_SPECIES], mask: &[bool; MAX_SPECIES]) -> f32 {
    let mut d = 0.0_f32;
    for s in 0..MAX_SPECIES {
        if mask[s] { d += species[s]; }
    }
    d
}

// ── Gradient (Axiom 7, reflectivo en bordes) ────────────────────────────────

/// Gradiente discreto 2D de la densidad de productos sobre el grid.
/// Central-difference en interior; forward/backward en bordes (ghost celda = interior,
/// coherente con la condición de `diffuse_species`).
pub fn local_gradient(
    grid: &SpeciesGrid,
    x: usize,
    y: usize,
    mask: &[bool; MAX_SPECIES],
) -> Vec2 {
    let w = grid.width();
    let h = grid.height();
    if w == 0 || h == 0 { return Vec2::ZERO; }
    debug_assert!(x < w && y < h, "cell ({x},{y}) out of range");

    let d = |cx: usize, cy: usize| cell_density(&grid.cell(cx, cy).species, mask);

    let dx = if w == 1                  { 0.0 }
             else if x == 0             { d(1, y)     - d(0, y) }
             else if x == w - 1         { d(w - 1, y) - d(w - 2, y) }
             else                       { 0.5 * (d(x + 1, y) - d(x - 1, y)) };

    let dy = if h == 1                  { 0.0 }
             else if y == 0             { d(x, 1)     - d(x, 0) }
             else if y == h - 1         { d(x, h - 1) - d(x, h - 2) }
             else                       { 0.5 * (d(x, y + 1) - d(x, y - 1)) };

    Vec2::new(dx, dy)
}

// ── Strength (Axiom 4) ──────────────────────────────────────────────────────

/// Fuerza de membrana local.  `= ‖∇ρ‖ · bond_energy_avg · (1 − DISSIPATION_LIQUID)`.
/// Inputs no-finitos o negativos ⇒ `0.0` (Axiom 2: no inventa presión).
#[inline]
pub fn membrane_strength(gradient_norm: f32, bond_energy_avg: f32) -> f32 {
    if !gradient_norm.is_finite() || !bond_energy_avg.is_finite() { return 0.0; }
    gradient_norm.max(0.0) * bond_energy_avg.max(0.0) * (1.0 - DISSIPATION_LIQUID)
}

// ── Flux damping factor (∈ [MIN_RATIO, 1]) ──────────────────────────────────

/// Factor multiplicativo por-flux para atenuación por membrana.
/// `strength == 0` ⇒ `1.0` (sin membrana).  `strength → ∞` ⇒ `MEMBRANE_MIN_FLUX_RATIO`
/// (jamás sella perfectamente — Axiom 4).  Inputs no-finitos ⇒ `1.0` (sin damping).
#[inline]
pub fn damped_flux_factor(strength: f32) -> f32 {
    if !strength.is_finite() || strength <= 0.0 { return 1.0; }
    (-strength * MEMBRANE_DAMPING).exp().max(MEMBRANE_MIN_FLUX_RATIO)
}

// ── Field builder (scratch, caller-owned) ───────────────────────────────────

/// Rellena `out` con el damping factor por celda (longitud = `grid.len()`).
/// Patrón `ScratchPad`: caller reusa el `Vec<f32>` entre ticks.  Mask todo-`false`
/// ⇒ todo el campo `1.0` (sin membrana en ningún lado).
pub fn compute_membrane_field(
    grid: &SpeciesGrid,
    mask: &[bool; MAX_SPECIES],
    bond_energy_avg: f32,
    out: &mut Vec<f32>,
) {
    out.clear();
    out.reserve(grid.len());
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let g = local_gradient(grid, x, y, mask).length();
            let s = membrane_strength(g, bond_energy_avg);
            out.push(damped_flux_factor(s));
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::reaction::SpeciesId;

    fn s0() -> SpeciesId { SpeciesId::new(0).unwrap() }

    fn mask_only_s0() -> [bool; MAX_SPECIES] {
        let mut m = [false; MAX_SPECIES];
        m[0] = true;
        m
    }

    // ── local_gradient ─────────────────────────────────────────────────────

    #[test]
    fn gradient_is_zero_on_homogeneous_field() {
        let mut g = SpeciesGrid::new(4, 4, 50.0);
        for y in 0..4 { for x in 0..4 { g.seed(x, y, s0(), 2.0); } }
        let m = mask_only_s0();
        let grad = local_gradient(&g, 2, 2, &m);
        assert!(grad.length() < 1e-6, "grad={grad:?}");
    }

    #[test]
    fn gradient_tracks_x_ramp() {
        // ρ(x,y) = x → ∂ρ/∂x = 1, ∂ρ/∂y = 0 (central-difference exacto).
        let mut g = SpeciesGrid::new(5, 3, 50.0);
        for y in 0..3 {
            for x in 0..5 {
                g.seed(x, y, s0(), x as f32);
            }
        }
        let grad = local_gradient(&g, 2, 1, &mask_only_s0());
        assert!((grad.x - 1.0).abs() < 1e-5, "dx={}", grad.x);
        assert!(grad.y.abs() < 1e-5, "dy={}", grad.y);
    }

    #[test]
    fn gradient_uses_forward_difference_at_boundary() {
        // En x=0, central no es válido; forward debe dar d(1,y) - d(0,y) = 3 - 1 = 2.
        let mut g = SpeciesGrid::new(3, 1, 50.0);
        g.seed(0, 0, s0(), 1.0);
        g.seed(1, 0, s0(), 3.0);
        g.seed(2, 0, s0(), 5.0);
        let grad = local_gradient(&g, 0, 0, &mask_only_s0());
        assert!((grad.x - 2.0).abs() < 1e-5, "dx={}", grad.x);
    }

    #[test]
    fn gradient_ignores_species_outside_mask() {
        let mut g = SpeciesGrid::new(3, 1, 50.0);
        // Especie 1 con gradiente fuerte, pero mask sólo sigue especie 0.
        let s1 = SpeciesId::new(1).unwrap();
        g.seed(0, 0, s1, 0.0);
        g.seed(1, 0, s1, 10.0);
        g.seed(2, 0, s1, 20.0);
        let grad = local_gradient(&g, 1, 0, &mask_only_s0());
        assert!(grad.length() < 1e-6);
    }

    // ── membrane_strength ──────────────────────────────────────────────────

    #[test]
    fn strength_is_zero_on_zero_gradient() {
        assert_eq!(membrane_strength(0.0, 5.0), 0.0);
    }

    #[test]
    fn strength_is_linear_in_gradient() {
        let a = membrane_strength(1.0, 2.0);
        let b = membrane_strength(2.0, 2.0);
        assert!((b - 2.0 * a).abs() < 1e-6);
    }

    #[test]
    fn strength_applies_liquid_dissipation_factor() {
        // s = g · e · (1 - DISSIPATION_LIQUID). Con g=10, e=1 ⇒ 10·0.98 = 9.8.
        let s = membrane_strength(10.0, 1.0);
        let expected = 10.0 * (1.0 - DISSIPATION_LIQUID);
        assert!((s - expected).abs() < 1e-5);
    }

    #[test]
    fn strength_rejects_non_finite_and_negative() {
        assert_eq!(membrane_strength(f32::NAN, 1.0), 0.0);
        assert_eq!(membrane_strength(1.0, f32::INFINITY), 0.0);
        assert_eq!(membrane_strength(-5.0, 1.0), 0.0);
        assert_eq!(membrane_strength(1.0, -3.0), 0.0);
    }

    // ── damped_flux_factor ─────────────────────────────────────────────────

    #[test]
    fn damping_is_one_at_zero_strength() {
        assert_eq!(damped_flux_factor(0.0), 1.0);
    }

    #[test]
    fn damping_clamps_above_min_flux_ratio() {
        let f = damped_flux_factor(1e6);
        assert!(f >= MEMBRANE_MIN_FLUX_RATIO);
        assert!(f <= MEMBRANE_MIN_FLUX_RATIO + 1e-6, "not clamped: {f}");
    }

    #[test]
    fn damping_strictly_decreases_with_strength() {
        // MEMBRANE_DAMPING=50 clampa rápido; elegimos strengths donde no se satura.
        let a = damped_flux_factor(0.001);
        let b = damped_flux_factor(0.01);
        let c = damped_flux_factor(0.05);
        assert!(a > b && b > c, "{a} {b} {c}");
        assert!(c >= MEMBRANE_MIN_FLUX_RATIO);
    }

    #[test]
    fn damping_handles_non_finite() {
        // Semántica: non-finite ⇒ 1.0 (sin damping). Consistente con otras
        // pure fns del módulo que hacen guard de inputs.
        assert_eq!(damped_flux_factor(f32::NAN), 1.0);
        assert_eq!(damped_flux_factor(f32::INFINITY), 1.0);
        assert_eq!(damped_flux_factor(f32::NEG_INFINITY), 1.0);
    }

    // ── compute_membrane_field ─────────────────────────────────────────────

    #[test]
    fn field_is_all_ones_on_empty_mask() {
        let mut g = SpeciesGrid::new(3, 3, 50.0);
        g.seed(1, 1, s0(), 100.0);
        let empty_mask = [false; MAX_SPECIES];
        let mut field = Vec::new();
        compute_membrane_field(&g, &empty_mask, 1.0, &mut field);
        assert_eq!(field.len(), 9);
        for v in &field { assert!((v - 1.0).abs() < 1e-6); }
    }

    #[test]
    fn field_length_matches_grid() {
        let g = SpeciesGrid::new(5, 4, 50.0);
        let mut field = Vec::new();
        compute_membrane_field(&g, &mask_only_s0(), 1.0, &mut field);
        assert_eq!(field.len(), 20);
    }

    #[test]
    fn field_below_one_where_gradient_present() {
        let mut g = SpeciesGrid::new(3, 3, 50.0);
        g.seed(1, 1, s0(), 100.0); // pico concentrado ⇒ gradiente fuerte en vecinos
        let mut field = Vec::new();
        compute_membrane_field(&g, &mask_only_s0(), 1.0, &mut field);
        // Al menos una celda vecina al pico debe tener damping < 1.
        let min = field.iter().cloned().fold(f32::INFINITY, f32::min);
        assert!(min < 1.0, "all cells at 1.0 — no membrane? min={min}");
        assert!(min >= MEMBRANE_MIN_FLUX_RATIO);
    }
}
