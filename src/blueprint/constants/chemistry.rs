//! AP-0/1/2: Chemistry substrate constants.
//! Toda constante física deriva de las 4 fundamentales (ver `derived_thresholds.rs`):
//! `KLEIBER_EXPONENT`, `DISSIPATION_{SOLID,LIQUID,GAS,PLASMA}`, `COHERENCE_BANDWIDTH`,
//! `DENSITY_SCALE`.  Las constantes puramente topológicas/numéricas (tamaños de
//! arrays, pisos epsilon) se documentan como tales.
//!
//! All physical constants derived from the 4 fundamentals.

use crate::blueprint::equations::derived_thresholds::{
    COHERENCE_BANDWIDTH, DENSITY_SCALE, DISSIPATION_GAS, DISSIPATION_LIQUID,
};

// ── Topología (límites de layout, no físicos) ───────────────────────────────

/// Máximo de especies por celda. Cache-line friendly: 32 × f32 = 128 B.
pub const MAX_SPECIES: usize = 32;

/// Máximo de reactivos por reacción.
pub const MAX_REACTANTS_PER_REACTION: usize = 4;

/// Máximo de productos por reacción.
pub const MAX_PRODUCTS_PER_REACTION: usize = 4;

/// Tamaño objetivo de red para `ReactionNetwork` (soft cap).
pub const MAX_REACTIONS_PER_NETWORK: usize = 256;

/// Sentinel para `SpeciesId::NONE` en slots no usados de arrays.
pub const SPECIES_ID_NONE: u8 = u8::MAX;

// ── Cinética (Axioms 4, 7, 8) ───────────────────────────────────────────────

/// Eficiencia de reacción = fracción de masa conservada como producto.
/// `= 1 - DISSIPATION_LIQUID`. Axiom 4.
pub const REACTION_EFFICIENCY: f32 = 1.0 - DISSIPATION_LIQUID;

/// Tasa de difusión inter-celular por unidad de tiempo. `= DISSIPATION_LIQUID`. Axiom 7.
pub const SPECIES_DIFFUSION_RATE: f32 = DISSIPATION_LIQUID;

/// Límite CFL para difusión 4-vecino (`r ≤ 1/4`). Propiedad numérica del stencil.
pub const DIFFUSION_CFL_MAX: f32 = 0.25;

/// Bandwidth por defecto de alineación de frecuencia en reacciones.
/// `= COHERENCE_BANDWIDTH` — la ventana espectral canónica del simulador.
pub const REACTION_FREQ_BANDWIDTH_DEFAULT: f32 = COHERENCE_BANDWIDTH;

// ── RAF detection (Hordijk-Steel) ──────────────────────────────────────────

/// Número mínimo de reacciones para considerar una closure no-trivial.
/// NOTA: filtra cadenas cortas, pero no garantiza ciclo cerrado — la distinción
/// topológica (linear vs. cyclic) queda para AP-4.
pub const RAF_MIN_CLOSURE_REACTIONS: usize = 3;

/// Umbral de presencia para considerar una especie "food" de una RAF.
/// `= DISSIPATION_LIQUID × DENSITY_SCALE`:  la concentración debe superar la
/// tasa a la que la difusión la dispersa, en la escala espacial canónica.
pub const FOOD_PRESENCE_THRESHOLD: f32 = DISSIPATION_LIQUID * DENSITY_SCALE;

// ── Kinetic stability (Pross) ───────────────────────────────────────────────

/// Umbral de estabilidad cinética. `K ≥ 1` ⇒ persistente.  Es la definición
/// (reconstrucción ≥ decay), no un número calibrado.
pub const KINETIC_STABILITY_PERSISTENT: f32 = 1.0;

/// Guarda numérica contra división por cero en `reconstruction / decay`.
/// Escalar adimensional — no tiene derivación física.
pub const KINETIC_STABILITY_EPSILON: f32 = 1e-9;

// ── Membrane (AP-3, ADR-038) ────────────────────────────────────────────────

/// Ganancia exponencial del damping de flux por gradiente de productos.
/// `= 1.0 / DISSIPATION_LIQUID`.  Axiom 7: atenuación proporcional a la cohesión
/// del borde, derivada de la escala canónica de difusión líquida (no calibrada).
pub const MEMBRANE_DAMPING: f32 = 1.0 / DISSIPATION_LIQUID;

/// Cota inferior del factor de flux (R1 de ADR-038): garantiza Axiom 4 estricto.
/// Ninguna celda queda perfectamente sellada — siempre escapa ≥ 1 % del flux.
pub const MEMBRANE_MIN_FLUX_RATIO: f32 = 0.01;

// ── Fission (AP-4 / AP-6d calibración) ─────────────────────────────────────

/// Ratio de presión para disparar fisión.  `= DISSIPATION_GAS / DISSIPATION_LIQUID = 4`.
///
/// La fisión es un evento **fluido-mecánico** — mass transport por un pinch
/// de membrana.  Su umbral natural es la transición gas↔líquido: el punto
/// donde la difusión empieza a dominar sobre la cohesión del solvente.
/// Plasma↔sólido (= 50) abarca regímenes electrónicos/cristalográficos
/// irrelevantes a la física de vesícula.
///
/// Revisión 2026-04-14 (AP-6d): anterior `DISSIPATION_PLASMA/DISSIPATION_SOLID = 50`
/// era derivable pero no físicamente anclada.  Con la fórmula adimensional
/// corregida de `pressure_ratio`, el steady-state empírico de formose da
/// K ≈ 13 — cruza holgadamente el umbral 4 y queda bajo el 50 arbitrario.
/// Ver ADR-039 §Revisión 2026-04-14-b.
pub const FISSION_PRESSURE_RATIO: f32 = DISSIPATION_GAS / DISSIPATION_LIQUID;

/// Fracción del máximo de `strength_field` que define la cota inferior de un
/// blob detectable en el harness AP-5.  Topológica, no calibrada: filtra ruido
/// numérico de la inferencia de membrana sin cortar estructuras reales.
pub const BLOB_STRENGTH_FRACTION: f32 = 0.05;

// ── Compile-time sanity ─────────────────────────────────────────────────────

const _: () = assert!(MAX_SPECIES <= 255, "SpeciesId fits in u8");
const _: () = assert!(SPECIES_ID_NONE as usize >= MAX_SPECIES, "sentinel not collide");
const _: () = assert!(SPECIES_DIFFUSION_RATE <= DIFFUSION_CFL_MAX, "CFL stable");
const _: () = assert!(REACTION_EFFICIENCY > 0.0 && REACTION_EFFICIENCY < 1.0, "0 < eff < 1");
const _: () = assert!(RAF_MIN_CLOSURE_REACTIONS >= 2, "trivial closures filtered out");
const _: () = assert!(MEMBRANE_DAMPING > 1.0, "damping amplifies gradient");
const _: () = assert!(
    MEMBRANE_MIN_FLUX_RATIO > 0.0 && MEMBRANE_MIN_FLUX_RATIO < 1.0,
    "strict escape floor",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn efficiency_axiom_4_strict() {
        // Axiom 4: toda reacción disipa. No hay eficiencia 100%.
        assert!(REACTION_EFFICIENCY < 1.0);
        assert!(REACTION_EFFICIENCY > 0.9, "calibración razonable");
    }

    #[test]
    fn diffusion_respects_cfl() {
        assert!(SPECIES_DIFFUSION_RATE <= DIFFUSION_CFL_MAX);
    }

    #[test]
    fn fission_ratio_is_gas_over_liquid() {
        // AP-6d: transición gas→líquido (régimen fluido, donde vesículas viven).
        // 0.08 / 0.02 = 4.0 exacto.
        assert!((FISSION_PRESSURE_RATIO - 4.0).abs() < 1e-5);
    }

    #[test]
    fn membrane_damping_is_inverse_liquid_dissipation() {
        let expected = 1.0 / DISSIPATION_LIQUID;
        assert!((MEMBRANE_DAMPING - expected).abs() < 1e-3);
    }

    #[test]
    fn membrane_min_flux_ratio_strictly_allows_escape() {
        assert!(MEMBRANE_MIN_FLUX_RATIO > 0.0);
        assert!(MEMBRANE_MIN_FLUX_RATIO < 1.0);
    }

    #[test]
    fn species_id_sentinel_out_of_band() {
        assert!((SPECIES_ID_NONE as usize) >= MAX_SPECIES);
    }

    #[test]
    fn bandwidth_matches_coherence_bandwidth() {
        assert_eq!(REACTION_FREQ_BANDWIDTH_DEFAULT, COHERENCE_BANDWIDTH);
    }

    #[test]
    fn food_threshold_derives_from_fundamentals() {
        let expected = DISSIPATION_LIQUID * DENSITY_SCALE;
        assert_eq!(FOOD_PRESENCE_THRESHOLD, expected);
    }
}
