//! Resolución EAC2 y envolvente Hz global (EAC4) — lógica pura sobre mapas de elementos.
//! Ver `docs/sprints/ELEMENT_ALMANAC_CANON/README.md`.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::blueprint::element_id::ElementId;

use super::element_def::ElementDef;

/// Si `frequency_hz` no es finito en `from_defs`, se usa `(min+max)*k` como pico por defecto.
pub(super) const ALMANAC_DEFAULT_PEAK_FROM_BAND_MIDPOINT: f32 = 0.5;

/// EAC2: orden total entre candidatos que ya cumplen `contains(freq)`.
/// Para usar con [`Iterator::min_by`]: el **mínimo** es el elemento preferido
/// (menor anchura → mayor `freq_band.0` si empate → menor `ElementId::raw()`).
#[inline]
pub(super) fn cmp_stable_band_candidates_eac2(
    a_id: ElementId,
    a_def: &ElementDef,
    b_id: ElementId,
    b_def: &ElementDef,
) -> Ordering {
    a_def
        .freq_band_span()
        .total_cmp(&b_def.freq_band_span())
        .then_with(|| b_def.freq_band.0.total_cmp(&a_def.freq_band.0))
        .then_with(|| a_id.raw().cmp(&b_id.raw()))
}

pub(super) fn compute_game_frequency_hz_bounds(
    elements: &HashMap<ElementId, ElementDef>,
) -> Option<(f32, f32)> {
    let mut low = f32::INFINITY;
    let mut high = f32::NEG_INFINITY;
    for def in elements.values() {
        let (a, b) = def.freq_band;
        if !a.is_finite() || !b.is_finite() {
            continue;
        }
        let lo = a.min(b);
        let hi = a.max(b);
        low = low.min(lo);
        high = high.max(hi);
    }
    if low.is_finite() && high.is_finite() && high > low {
        Some((low, high))
    } else {
        None
    }
}
