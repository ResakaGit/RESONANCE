//! AP-0/1/2: Chemistry substrate constants.
//! Toda constante física deriva de las 4 fundamentales (ver `derived_thresholds.rs`):
//! `KLEIBER_EXPONENT`, `DISSIPATION_{SOLID,LIQUID,GAS,PLASMA}`, `COHERENCE_BANDWIDTH`,
//! `DENSITY_SCALE`.  Las constantes puramente topológicas/numéricas (tamaños de
//! arrays, pisos epsilon) se documentan como tales.
//!
//! All physical constants derived from the 4 fundamentals.

use crate::blueprint::equations::derived_thresholds::{
    COHERENCE_BANDWIDTH, DENSITY_SCALE, DISSIPATION_LIQUID, DISSIPATION_PLASMA, DISSIPATION_SOLID,
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

// ── Membrane + fission (reservado AP-3/4) ──────────────────────────────────

/// Ratio de presión para disparar fisión.  `= DISSIPATION_PLASMA / DISSIPATION_SOLID`.
/// Expuesto aquí para coherencia de la cadena completa.
pub const FISSION_PRESSURE_RATIO: f32 = DISSIPATION_PLASMA / DISSIPATION_SOLID;

// ── Compile-time sanity ─────────────────────────────────────────────────────

const _: () = assert!(MAX_SPECIES <= 255, "SpeciesId fits in u8");
const _: () = assert!(SPECIES_ID_NONE as usize >= MAX_SPECIES, "sentinel not collide");
const _: () = assert!(SPECIES_DIFFUSION_RATE <= DIFFUSION_CFL_MAX, "CFL stable");
const _: () = assert!(REACTION_EFFICIENCY > 0.0 && REACTION_EFFICIENCY < 1.0, "0 < eff < 1");
const _: () = assert!(RAF_MIN_CLOSURE_REACTIONS >= 2, "trivial closures filtered out");

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
    fn fission_ratio_is_plasma_over_solid() {
        assert!((FISSION_PRESSURE_RATIO - 50.0).abs() < 1e-3);
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
