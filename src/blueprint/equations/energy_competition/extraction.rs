use super::pool_equations::{
    extract_aggressive, extract_competitive, extract_greedy, extract_proportional,
    extract_regulated,
};
use crate::blueprint::constants::*;
use crate::layers::pool_link::ExtractionType;

// ─── EC-3A: Contexto de Extracción ───────────────────────────────────────────

/// Contexto inmutable pasado a toda función de extracción (stack-only, Copy).
#[derive(Clone, Copy, Debug)]
pub struct ExtractionContext {
    /// Energía disponible en el pool padre post-disipación.
    pub available: f32,
    /// Ratio pool/capacity del padre.
    pub pool_ratio: f32,
    /// Número de hermanos (incluyéndose).
    pub n_siblings: u32,
    /// Fitness total de todos los hermanos.
    pub total_fitness: f32,
}

// ─── EC-3B: Modificadores de Extracción ──────────────────────────────────────

/// Modificadores aplicados en orden sobre el resultado base. Stack de hasta MAX_EXTRACTION_MODIFIERS.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExtractionModifier {
    /// Bajo estrés (`pool_ratio < threshold`), multiplica la extracción.
    StressResponse { threshold: f32, multiplier: f32 },
    /// Si `pool_ratio < min_viable`, bloquea toda extracción.
    ThresholdGated { min_viable: f32 },
    /// Escala la extracción por un factor fijo.
    ScaleFactor { factor: f32 },
    /// Clamp máximo de extracción por tick.
    CapPerTick { max_per_tick: f32 },
}

// ─── EC-3C: Extraction Profile ───────────────────────────────────────────────

/// Perfil completo de extracción: función base + stack de modificadores.
/// Es el "fenotipo funcional" evaluable como pura — no se almacena, se evalúa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtractionProfile {
    /// Función base (una de las 5 primitivas).
    pub base: ExtractionType,
    /// Parámetro primario de la función base (semántica depende del tipo).
    pub primary_param: f32,
    /// Stack de modificadores aplicados en orden.
    pub modifiers: [Option<ExtractionModifier>; MAX_EXTRACTION_MODIFIERS],
}

// ─── EC-3D: Evaluación ───────────────────────────────────────────────────────

/// Evalúa la extracción completa: función base + modificadores en orden + clamp final.
pub fn evaluate_extraction(profile: &ExtractionProfile, ctx: &ExtractionContext) -> f32 {
    fold_modifiers(eval_base(profile, ctx), &profile.modifiers, ctx).clamp(0.0, ctx.available)
}

/// Evalúa extracción agresiva con su componente de daño al pool.
/// Retorna `(taken, pool_damage)`. Modificadores afectan solo `taken`.
pub fn evaluate_aggressive_extraction(
    profile: &ExtractionProfile,
    ctx: &ExtractionContext,
    damage_rate: f32,
) -> (f32, f32) {
    let raw = extract_aggressive(ctx.available, profile.primary_param, damage_rate).0;
    let taken = fold_modifiers(raw, &profile.modifiers, ctx).clamp(0.0, ctx.available);
    (taken, taken * damage_rate.clamp(0.0, 1.0))
}

fn eval_base(profile: &ExtractionProfile, ctx: &ExtractionContext) -> f32 {
    match profile.base {
        ExtractionType::Proportional => extract_proportional(ctx.available, ctx.n_siblings),
        ExtractionType::Greedy => extract_greedy(ctx.available, profile.primary_param),
        ExtractionType::Competitive => {
            extract_competitive(ctx.available, profile.primary_param, ctx.total_fitness)
        }
        ExtractionType::Aggressive => {
            extract_aggressive(ctx.available, profile.primary_param, DAMAGE_RATE_DEFAULT).0
        }
        ExtractionType::Regulated => extract_regulated(
            ctx.available,
            ctx.pool_ratio,
            profile.primary_param,
            REGULATED_THRESHOLD_LOW_DEFAULT,
            REGULATED_THRESHOLD_HIGH_DEFAULT,
        ),
    }
}

/// Aplica el stack de modificadores en orden. Misma lógica para base y agresiva.
fn fold_modifiers(
    base: f32,
    modifiers: &[Option<ExtractionModifier>; MAX_EXTRACTION_MODIFIERS],
    ctx: &ExtractionContext,
) -> f32 {
    modifiers
        .iter()
        .flatten()
        .fold(base, |acc, m| apply_modifier(acc, m, ctx))
}

fn apply_modifier(result: f32, modifier: &ExtractionModifier, ctx: &ExtractionContext) -> f32 {
    match modifier {
        ExtractionModifier::StressResponse {
            threshold,
            multiplier,
        } => {
            if ctx.pool_ratio < *threshold {
                result * multiplier
            } else {
                result
            }
        }
        ExtractionModifier::ThresholdGated { min_viable } => {
            if ctx.pool_ratio < *min_viable {
                0.0
            } else {
                result
            }
        }
        ExtractionModifier::ScaleFactor { factor } => result * factor.max(0.0),
        ExtractionModifier::CapPerTick { max_per_tick } => result.min(max_per_tick.max(0.0)),
    }
}

// ─── EC-3F: Fenotipos Predefinidos ───────────────────────────────────────────

/// Generalista oportunista: proporcional + stress response.
/// Bajo estrés de pool extrae una porción mayor de su cuota.
pub fn opportunistic_generalist() -> ExtractionProfile {
    ExtractionProfile {
        base: ExtractionType::Proportional,
        primary_param: 0.0,
        modifiers: [
            Some(ExtractionModifier::StressResponse {
                threshold: OPPORTUNISTIC_STRESS_THRESHOLD,
                multiplier: REGULATED_AGGRESSIVE_MULT,
            }),
            None,
            None,
            None,
        ],
    }
}

/// Especialista conservador: greedy + threshold gated.
/// No extrae nada si el pool padre está demasiado bajo.
pub fn conservative_specialist(capacity: f32, min_viable: f32) -> ExtractionProfile {
    ExtractionProfile {
        base: ExtractionType::Greedy,
        primary_param: capacity,
        modifiers: [
            Some(ExtractionModifier::ThresholdGated { min_viable }),
            None,
            None,
            None,
        ],
    }
}

/// Parásito adaptativo: aggressive + cap per tick.
/// Drena agresivamente pero limitado para no destruir al host de inmediato.
pub fn adaptive_parasite(aggression: f32, max_drain: f32) -> ExtractionProfile {
    ExtractionProfile {
        base: ExtractionType::Aggressive,
        primary_param: aggression,
        modifiers: [
            Some(ExtractionModifier::CapPerTick {
                max_per_tick: max_drain,
            }),
            None,
            None,
            None,
        ],
    }
}

/// Homeostático resiliente: regulated + stress response.
/// En homeostasis normal; bajo estrés activa una reserva adicional.
pub fn resilient_homeostatic(base_rate: f32) -> ExtractionProfile {
    ExtractionProfile {
        base: ExtractionType::Regulated,
        primary_param: base_rate,
        modifiers: [
            Some(ExtractionModifier::StressResponse {
                threshold: HOMEOSTATIC_STRESS_THRESHOLD,
                multiplier: HOMEOSTATIC_STRESS_MULT,
            }),
            None,
            None,
            None,
        ],
    }
}

/// Depredador apex: greedy + scale factor alto, sin daño al pool.
/// Domina en abundancia; extrae más que cualquier especialista pasivo.
pub fn apex_predator(capacity: f32) -> ExtractionProfile {
    ExtractionProfile {
        base: ExtractionType::Greedy,
        primary_param: capacity,
        modifiers: [
            Some(ExtractionModifier::ScaleFactor {
                factor: APEX_PREDATOR_SCALE_FACTOR,
            }),
            None,
            None,
            None,
        ],
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    fn ctx(
        available: f32,
        pool_ratio: f32,
        n_siblings: u32,
        total_fitness: f32,
    ) -> ExtractionContext {
        ExtractionContext {
            available,
            pool_ratio,
            n_siblings,
            total_fitness,
        }
    }

    fn no_mod_profile(base: ExtractionType, param: f32) -> ExtractionProfile {
        ExtractionProfile {
            base,
            primary_param: param,
            modifiers: [None; MAX_EXTRACTION_MODIFIERS],
        }
    }

    // EC-3A
    #[test]
    fn extraction_context_is_copy() {
        let a = ctx(100.0, 0.5, 4, 1.0);
        let b = a;
        assert_eq!(a.available, b.available);
    }

    #[test]
    fn extraction_context_size_16_bytes() {
        assert_eq!(size_of::<ExtractionContext>(), 16);
    }

    // EC-3B
    #[test]
    fn extraction_modifier_is_copy() {
        let m = ExtractionModifier::ScaleFactor { factor: 2.0 };
        let m2 = m;
        assert_eq!(m, m2);
    }

    #[test]
    fn extraction_modifier_variants_distinct() {
        assert_ne!(
            ExtractionModifier::StressResponse {
                threshold: 0.3,
                multiplier: 1.5
            },
            ExtractionModifier::ThresholdGated { min_viable: 0.1 },
        );
        assert_ne!(
            ExtractionModifier::ScaleFactor { factor: 2.0 },
            ExtractionModifier::CapPerTick {
                max_per_tick: 100.0
            },
        );
    }

    // EC-3C
    #[test]
    fn extraction_profile_is_copy() {
        let p = no_mod_profile(ExtractionType::Proportional, 0.0);
        let p2 = p;
        assert_eq!(p, p2);
    }

    #[test]
    fn extraction_profile_size_under_128_bytes() {
        assert!(
            size_of::<ExtractionProfile>() < 128,
            "size={}",
            size_of::<ExtractionProfile>()
        );
    }

    #[test]
    fn profile_zero_modifiers_equals_base_function() {
        let profile = no_mod_profile(ExtractionType::Proportional, 0.0);
        let c = ctx(1000.0, 0.5, 4, 1.0);
        let expected = extract_proportional(1000.0, 4);
        assert!((evaluate_extraction(&profile, &c) - expected).abs() < 1e-4);
    }

    #[test]
    fn profile_four_modifiers_applied_in_order() {
        // scale(2) then cap(300): result after scale = 500, then capped to 300
        let profile = ExtractionProfile {
            base: ExtractionType::Proportional,
            primary_param: 0.0,
            modifiers: [
                Some(ExtractionModifier::ScaleFactor { factor: 2.0 }),
                Some(ExtractionModifier::CapPerTick {
                    max_per_tick: 300.0,
                }),
                None,
                None,
            ],
        };
        let c = ctx(1000.0, 0.5, 4, 1.0); // base = 1000/4 = 250; *2 = 500; capped 300
        let result = evaluate_extraction(&profile, &c);
        assert!((result - 300.0).abs() < 1e-4, "got {result}");
    }

    // EC-3D
    #[test]
    fn proportional_pure_four_siblings() {
        let profile = no_mod_profile(ExtractionType::Proportional, 0.0);
        let c = ctx(1000.0, 0.5, 4, 1.0);
        assert!((evaluate_extraction(&profile, &c) - 250.0).abs() < 1e-4);
    }

    #[test]
    fn greedy_cap_per_tick_never_exceeds_cap() {
        let profile = ExtractionProfile {
            base: ExtractionType::Greedy,
            primary_param: 99999.0,
            modifiers: [
                Some(ExtractionModifier::CapPerTick {
                    max_per_tick: 200.0,
                }),
                None,
                None,
                None,
            ],
        };
        for avail in [100.0, 500.0, 1000.0, 5000.0] {
            let c = ctx(avail, 0.5, 1, 1.0);
            let result = evaluate_extraction(&profile, &c);
            assert!(result <= 200.0 + 1e-4, "avail={avail} result={result}");
        }
    }

    #[test]
    fn competitive_stress_response_extracts_more_under_stress() {
        let profile = ExtractionProfile {
            base: ExtractionType::Competitive,
            primary_param: 0.5,
            modifiers: [
                Some(ExtractionModifier::StressResponse {
                    threshold: 0.3,
                    multiplier: 1.5,
                }),
                None,
                None,
                None,
            ],
        };
        let stressed = ctx(1000.0, 0.2, 2, 1.0); // pool_ratio < 0.3
        let not_stressed = ctx(1000.0, 0.5, 2, 1.0);
        let r_stress = evaluate_extraction(&profile, &stressed);
        let r_normal = evaluate_extraction(&profile, &not_stressed);
        assert!(r_stress > r_normal, "stress={r_stress} normal={r_normal}");
    }

    #[test]
    fn regulated_threshold_gated_zero_when_below_min_viable() {
        let profile = ExtractionProfile {
            base: ExtractionType::Regulated,
            primary_param: 100.0,
            modifiers: [
                Some(ExtractionModifier::ThresholdGated { min_viable: 0.1 }),
                None,
                None,
                None,
            ],
        };
        let c = ctx(1000.0, 0.05, 2, 1.0); // pool_ratio < 0.1
        assert_eq!(evaluate_extraction(&profile, &c), 0.0);
    }

    #[test]
    fn evaluate_extraction_result_in_zero_available_range() {
        let profiles = [
            no_mod_profile(ExtractionType::Proportional, 0.0),
            no_mod_profile(ExtractionType::Greedy, 500.0),
            no_mod_profile(ExtractionType::Competitive, 0.5),
            no_mod_profile(ExtractionType::Aggressive, 0.8),
            no_mod_profile(ExtractionType::Regulated, 100.0),
        ];
        let c = ctx(1000.0, 0.5, 3, 1.5);
        for p in &profiles {
            let r = evaluate_extraction(p, &c);
            assert!(
                r >= 0.0 && r <= c.available + 1e-4,
                "base={:?} r={r}",
                p.base
            );
        }
    }

    #[test]
    fn evaluate_extraction_deterministic_100_calls() {
        let profile = no_mod_profile(ExtractionType::Competitive, 0.4);
        let c = ctx(800.0, 0.6, 3, 1.2);
        let first = evaluate_extraction(&profile, &c);
        for _ in 0..99 {
            assert_eq!(evaluate_extraction(&profile, &c), first);
        }
    }

    // EC-3E
    #[test]
    fn evaluate_aggressive_extraction_pool_damage_is_taken_times_rate() {
        let profile = no_mod_profile(ExtractionType::Aggressive, 0.5);
        let c = ctx(1000.0, 0.6, 2, 1.0);
        let (taken, pool_damage) = evaluate_aggressive_extraction(&profile, &c, 0.1);
        assert!(
            (pool_damage - taken * 0.1).abs() < 1e-4,
            "taken={taken} dmg={pool_damage}"
        );
    }

    #[test]
    fn evaluate_aggressive_modifiers_affect_taken_not_damage_rate() {
        let base_profile = no_mod_profile(ExtractionType::Aggressive, 0.5);
        let scaled_profile = ExtractionProfile {
            base: ExtractionType::Aggressive,
            primary_param: 0.5,
            modifiers: [
                Some(ExtractionModifier::ScaleFactor { factor: 0.5 }),
                None,
                None,
                None,
            ],
        };
        let c = ctx(1000.0, 0.6, 2, 1.0);
        let (t_base, d_base) = evaluate_aggressive_extraction(&base_profile, &c, 0.1);
        let (t_scaled, d_scaled) = evaluate_aggressive_extraction(&scaled_profile, &c, 0.1);
        // scaled takes less
        assert!(t_scaled < t_base, "t_scaled={t_scaled} t_base={t_base}");
        // damage rate stays proportional to taken
        assert!((d_base - t_base * 0.1).abs() < 1e-4);
        assert!((d_scaled - t_scaled * 0.1).abs() < 1e-4);
    }

    // EC-3F
    #[test]
    fn opportunistic_generalist_is_valid_profile() {
        let p = opportunistic_generalist();
        assert_eq!(p.base, ExtractionType::Proportional);
        assert!(p.modifiers[0].is_some());
    }

    #[test]
    fn opportunistic_generalist_extracts_more_under_stress() {
        let p = opportunistic_generalist();
        let stressed = ctx(1000.0, 0.2, 4, 1.0);
        let not_stressed = ctx(1000.0, 0.6, 4, 1.0);
        assert!(evaluate_extraction(&p, &stressed) > evaluate_extraction(&p, &not_stressed));
    }

    #[test]
    fn conservative_specialist_zero_below_min_viable() {
        let p = conservative_specialist(500.0, 0.3);
        let c = ctx(1000.0, 0.1, 2, 1.0); // pool_ratio < min_viable
        assert_eq!(evaluate_extraction(&p, &c), 0.0);
    }

    #[test]
    fn adaptive_parasite_is_valid_profile() {
        let p = adaptive_parasite(0.7, 300.0);
        assert_eq!(p.base, ExtractionType::Aggressive);
        assert!(p.modifiers[0].is_some());
    }

    #[test]
    fn resilient_homeostatic_is_valid_profile() {
        let p = resilient_homeostatic(80.0);
        assert_eq!(p.base, ExtractionType::Regulated);
        assert!(p.modifiers[0].is_some());
    }

    #[test]
    fn apex_predator_extracts_more_than_conservative_specialist() {
        let apex = apex_predator(400.0);
        let cons = conservative_specialist(400.0, 0.1);
        let c = ctx(1000.0, 0.5, 1, 1.0); // pool_ratio well above min_viable
        let r_apex = evaluate_extraction(&apex, &c);
        let r_cons = evaluate_extraction(&cons, &c);
        assert!(r_apex > r_cons, "apex={r_apex} cons={r_cons}");
    }

    #[test]
    fn apex_predator_is_valid_profile() {
        let p = apex_predator(600.0);
        assert_eq!(p.base, ExtractionType::Greedy);
        assert!(p.modifiers[0].is_some());
    }
}
