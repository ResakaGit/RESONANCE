//! Morfogénesis inferida — ecuaciones puras MG-1.
//!
//! Módulo acotado al track morfogénesis: evita inflar `equations.rs` y concentra constantes en
//! [`crate::blueprint::constants::morphogenesis`]. Los sistemas y el resto del blueprint pueden usar
//! `crate::blueprint::equations::…` vía re-export o importar desde aquí.

mod thermodynamics;
mod constructal;
mod surface;

pub use thermodynamics::*;
pub use constructal::*;
pub use surface::*;

use super::constants::morphogenesis as mg;

// ── Saneo numérico (una sola política; evita repetir if is_finite en cada API pública) ──

#[inline]
pub(super) fn san_nonneg(x: f32) -> f32 {
    if x.is_finite() { x.max(0.0) } else { 0.0 }
}

#[inline]
pub(super) fn san_velocity_mag(x: f32) -> f32 {
    if x.is_finite() { x.abs() } else { 0.0 }
}

#[inline]
pub(super) fn san_finite_or_zero(x: f32) -> f32 {
    if x.is_finite() { x } else { 0.0 }
}

#[inline]
pub(super) fn san_efficiency_01(x: f32) -> f32 {
    if x.is_finite() { x.clamp(0.0, 1.0) } else { 0.0 }
}

#[inline]
pub(super) fn san_emissivity(emissivity: f32) -> f32 {
    if emissivity.is_finite() {
        emissivity.clamp(0.0, 1.0)
    } else {
        mg::DEFAULT_EMISSIVITY
    }
}

#[inline]
pub(super) fn san_convection_or_default(convection_coeff: f32) -> f32 {
    if convection_coeff.is_finite() {
        convection_coeff.max(0.0)
    } else {
        mg::DEFAULT_CONVECTION_COEFF
    }
}

#[cfg(test)]
mod tests;
