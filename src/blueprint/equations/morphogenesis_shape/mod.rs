use bevy::math::{Vec2, Vec3};

// ═══════════════════════════════════════════════
// Shape inference — energy field → GeometryInfluence parameters (stateless)
// ═══════════════════════════════════════════════

/// Energy gradient from 4-connected neighbor accumulated_qe (finite differences, 2D grid).
/// Returns the direction of steepest energy increase in the XZ sim plane.
#[inline]
pub fn energy_gradient_2d(qe_left: f32, qe_right: f32, qe_down: f32, qe_up: f32) -> Vec2 {
    let safe = |v: f32| if v.is_finite() { v } else { 0.0 };
    Vec2::new(
        (safe(qe_right) - safe(qe_left)) * 0.5,
        (safe(qe_up) - safe(qe_down)) * 0.5,
    )
}

/// Shape growth direction: vertical (Y-up) biased by the horizontal energy gradient.
/// `gradient_blend` controls how much the gradient pulls the direction sideways (\[0,1\]).
#[inline]
pub fn shape_inferred_direction(gradient: Vec2, gradient_blend: f32) -> Vec3 {
    let b = gradient_blend.clamp(0.0, 1.0);
    let horizontal = Vec3::new(gradient.x, 0.0, gradient.y);
    let dir = Vec3::Y + horizontal * b;
    let n = dir.normalize_or_zero();
    if n.length_squared() < 1e-12 {
        Vec3::Y
    } else {
        n
    }
}

/// Length budget for inferred shape: scales linearly with normalized energy.
#[inline]
pub fn shape_inferred_length(qe_norm: f32, base_length: f32, qe_scale: f32) -> f32 {
    let q = if qe_norm.is_finite() {
        qe_norm.clamp(0.0, 1.0)
    } else {
        0.0
    };
    (base_length + q * qe_scale).max(0.1)
}

/// Resistance: higher bond energy → more resistance → straighter shapes.
#[inline]
pub fn shape_inferred_resistance(
    bond_energy: f32,
    default_resistance: f32,
    bond_scale: f32,
) -> f32 {
    let be = if bond_energy.is_finite() {
        bond_energy.max(0.0)
    } else {
        0.0
    };
    (default_resistance + be * bond_scale).clamp(0.05, 5.0)
}

/// Presupuesto de ramificación por profundidad (dominancia apical cuadrática).
#[inline]
pub fn branch_budget(growth_budget: f32, depth: u32, max_depth: u32) -> u32 {
    if !growth_budget.is_finite() || growth_budget <= 0.0 || max_depth == 0 || depth >= max_depth {
        return 0;
    }
    let factor = branch_depth_factor(depth, max_depth);
    (growth_budget.max(0.0) * factor * factor).floor() as u32
}

/// Factor lineal de atenuación por profundidad para Capa 5.
#[inline]
pub fn branch_depth_factor(depth: u32, max_depth: u32) -> f32 {
    if max_depth == 0 {
        return 0.0;
    }
    let d = depth as f32 / max_depth as f32;
    (1.0 - d).clamp(0.0, 1.0)
}

/// Atenuación morfológica de Capa 5 por profundidad.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn branch_attenuation_values(
    length_budget: f32,
    radius_base: f32,
    energy_strength: f32,
    qe_norm: f32,
    detail: f32,
    depth: u32,
    max_depth: u32,
    radius_decay: f32,
    energy_decay: f32,
    qe_decay: f32,
    detail_decay: f32,
) -> (f32, f32, f32, f32, f32) {
    let length = length_budget * branch_depth_factor(depth, max_depth);
    let radius = radius_base * radius_decay.max(0.0);
    let energy = energy_strength * energy_decay.max(0.0);
    let qe = qe_norm * qe_decay.max(0.0);
    let detail = detail * detail_decay.max(0.0);
    (length, radius, energy, qe, detail)
}
