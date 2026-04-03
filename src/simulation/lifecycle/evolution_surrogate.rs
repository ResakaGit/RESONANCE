use std::collections::VecDeque;

use bevy::prelude::*;

use crate::blueprint::constants::{
    EVOLUTION_BASELINE_COMPETITION, EVOLUTION_HOSTILE_COMPETITION,
    EVOLUTION_HOSTILE_PREDATION_DELTA, EVOLUTION_HOSTILE_TEMPERATURE_DELTA,
    EVOLUTION_MAINTENANCE_WEIGHT, EVOLUTION_ROLE_REPRODUCE_BIT, EVOLUTION_SCARCE_COMPETITION,
    EVOLUTION_SCARCE_FOOD_FACTOR, EVOLUTION_SURROGATE_MAX_ITERATIONS,
    MAX_EVOLUTION_EVALS_PER_FRAME,
};
use crate::blueprint::equations::{
    evolution_aggregate_fitness, evolution_reproduction_score, evolution_survival_score,
    organ_base_viability, trophic_assimilation, trophic_intake_factor, trophic_maintenance_cost,
    trophic_net_qe_delta,
};
use crate::bridge::cache::{BridgeCache, CachedValue};
use crate::bridge::config::EvolutionSurrogateBridge;
use crate::layers::{
    AnimalSpec, BaseEnergy, CapabilitySet, EnvContext, GrowthBudget, InferenceProfile, TrophicClass,
};
use crate::simulation::env_scenario::{EffectiveOrganViability, EnvScenarioSnapshot};

const SURROGATE_SCENARIO_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EvolutionCacheKey {
    pub food_bin: u8,
    pub predation_bin: u8,
    pub temperature_bin: u8,
    pub medium_bin: u8,
    pub competition_bin: u8,
    pub viability_bin: u8,
    pub role_mask: u16,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EvolutionFitnessSample {
    pub survival_score: f32,
    pub reproduction_score: f32,
    pub maintenance_cost: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EvolutionVariant {
    pub spec: AnimalSpec,
    pub viability: f32,
    pub role_mask: u16,
}

#[derive(Debug, Clone)]
pub struct EvolutionEvaluationTask {
    pub variant: EvolutionVariant,
    pub scenarios: [EnvContext; SURROGATE_SCENARIO_COUNT],
    pub scenario_cursor: usize,
    pub score_accum: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EvolutionRankedVariant {
    pub variant: EvolutionVariant,
    pub fitness: f32,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvolutionSurrogateConfig {
    pub max_evals_per_frame: u32,
    pub max_iterations: u32,
    pub top_k: usize,
    pub cache_version: u32,
}

impl Default for EvolutionSurrogateConfig {
    fn default() -> Self {
        Self {
            max_evals_per_frame: MAX_EVOLUTION_EVALS_PER_FRAME,
            max_iterations: EVOLUTION_SURROGATE_MAX_ITERATIONS,
            top_k: 3,
            cache_version: 1,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct EvolutionSurrogateQueue {
    pub pending: VecDeque<EvolutionEvaluationTask>,
}

#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct EvolutionSurrogateState {
    pub top_k: Vec<EvolutionRankedVariant>,
    pub last_processed_scenario_evals: u32,
    pub stable_iterations: u32,
}

#[inline]
fn finite_unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[inline]
fn quantize_to_u8(value: f32) -> u8 {
    let v = finite_unit(value);
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

#[inline]
pub fn evolution_cache_key_from_input(
    ctx: &EnvContext,
    viability: f32,
    role_mask: u16,
) -> EvolutionCacheKey {
    EvolutionCacheKey {
        food_bin: quantize_to_u8(ctx.food_density_t),
        predation_bin: quantize_to_u8(ctx.predation_pressure_t),
        temperature_bin: quantize_to_u8(ctx.temperature_t),
        medium_bin: quantize_to_u8(ctx.medium_density_t),
        competition_bin: quantize_to_u8(ctx.competition_t),
        viability_bin: quantize_to_u8(viability),
        role_mask,
    }
}

#[inline]
fn variant_fingerprint(variant: EvolutionVariant) -> u64 {
    let mut h: u64 = 0x8422_2325_cbf2_9ce4;
    for byte in [
        quantize_to_u8(variant.spec.metabolic_efficiency),
        quantize_to_u8(variant.spec.mobility_bias),
        quantize_to_u8(variant.spec.armor_bias),
        quantize_to_u8(variant.spec.sensor_bias),
        quantize_to_u8(variant.spec.reproduction_bias),
        quantize_to_u8(variant.spec.resilience),
        quantize_to_u8(variant.viability),
        (variant.role_mask & 0xFF) as u8,
        (variant.role_mask >> 8) as u8,
        variant.spec.trophic as u8,
    ] {
        h ^= byte as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

#[inline]
fn key_hash(key: EvolutionCacheKey, variant: EvolutionVariant, cache_version: u32) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in [
        key.food_bin,
        key.predation_bin,
        key.temperature_bin,
        key.medium_bin,
        key.competition_bin,
        key.viability_bin,
        (key.role_mask & 0xFF) as u8,
        (key.role_mask >> 8) as u8,
        (cache_version & 0xFF) as u8,
        ((cache_version >> 8) & 0xFF) as u8,
        ((cache_version >> 16) & 0xFF) as u8,
        ((cache_version >> 24) & 0xFF) as u8,
    ] {
        h ^= byte as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h ^ variant_fingerprint(variant)
}

#[inline]
fn can_reproduce_from_mask(role_mask: u16) -> bool {
    role_mask & (1 << EVOLUTION_ROLE_REPRODUCE_BIT) != 0
}

#[inline]
fn evaluate_sample(variant: EvolutionVariant, ctx: EnvContext) -> EvolutionFitnessSample {
    let intake = trophic_intake_factor(variant.spec.trophic, ctx.food_density_t);
    let assimilation =
        trophic_assimilation(intake, variant.spec.metabolic_efficiency, ctx.temperature_t);
    let maintenance = trophic_maintenance_cost(
        variant.viability.max(0.0),
        variant.spec.mobility_bias,
        variant.spec.armor_bias,
        ctx.predation_pressure_t,
        ctx.medium_density_t,
    );
    let net = trophic_net_qe_delta(assimilation, maintenance, ctx.competition_t);
    EvolutionFitnessSample {
        survival_score: evolution_survival_score(variant.viability, net),
        reproduction_score: evolution_reproduction_score(
            can_reproduce_from_mask(variant.role_mask),
            net,
            variant.spec.reproduction_bias,
        ),
        maintenance_cost: maintenance,
    }
}

#[inline]
fn aggregate_fitness(sample: EvolutionFitnessSample) -> f32 {
    evolution_aggregate_fitness(
        sample.survival_score,
        sample.reproduction_score,
        sample.maintenance_cost,
        EVOLUTION_MAINTENANCE_WEIGHT,
    )
}

fn evaluate_one_scenario_cached(
    variant: EvolutionVariant,
    scenario: EnvContext,
    cache: &mut BridgeCache<EvolutionSurrogateBridge>,
    cache_version: u32,
) -> f32 {
    let key = evolution_cache_key_from_input(&scenario, variant.viability, variant.role_mask);
    let hash = key_hash(key, variant, cache_version);
    if let Some(CachedValue::Scalar(score)) = cache.lookup(hash) {
        return score;
    }
    let score = aggregate_fitness(evaluate_sample(variant, scenario));
    cache.insert(hash, CachedValue::Scalar(score));
    score
}

#[inline]
fn evaluate_variant_average_cached(
    variant: EvolutionVariant,
    scenarios: &[EnvContext],
    cache: &mut BridgeCache<EvolutionSurrogateBridge>,
    cache_version: u32,
) -> f32 {
    if scenarios.is_empty() {
        return 0.0;
    }
    scenarios
        .iter()
        .copied()
        .map(|scenario| evaluate_one_scenario_cached(variant, scenario, cache, cache_version))
        .sum::<f32>()
        / scenarios.len() as f32
}

fn top_k_variants(
    mut ranked: Vec<EvolutionRankedVariant>,
    k: usize,
) -> Vec<EvolutionRankedVariant> {
    ranked.sort_by(|a, b| {
        b.fitness
            .total_cmp(&a.fitness)
            .then_with(|| a.variant.role_mask.cmp(&b.variant.role_mask))
            .then_with(|| variant_fingerprint(a.variant).cmp(&variant_fingerprint(b.variant)))
            .then_with(|| compare_variant_total(a.variant, b.variant))
    });
    ranked.truncate(k.min(ranked.len()));
    ranked
}

#[inline]
fn compare_variant_total(a: EvolutionVariant, b: EvolutionVariant) -> std::cmp::Ordering {
    (a.spec.trophic as u8)
        .cmp(&(b.spec.trophic as u8))
        .then_with(|| {
            a.spec
                .metabolic_efficiency
                .to_bits()
                .cmp(&b.spec.metabolic_efficiency.to_bits())
        })
        .then_with(|| {
            a.spec
                .mobility_bias
                .to_bits()
                .cmp(&b.spec.mobility_bias.to_bits())
        })
        .then_with(|| {
            a.spec
                .armor_bias
                .to_bits()
                .cmp(&b.spec.armor_bias.to_bits())
        })
        .then_with(|| {
            a.spec
                .sensor_bias
                .to_bits()
                .cmp(&b.spec.sensor_bias.to_bits())
        })
        .then_with(|| {
            a.spec
                .reproduction_bias
                .to_bits()
                .cmp(&b.spec.reproduction_bias.to_bits())
        })
        .then_with(|| {
            a.spec
                .resilience
                .to_bits()
                .cmp(&b.spec.resilience.to_bits())
        })
        .then_with(|| a.viability.to_bits().cmp(&b.viability.to_bits()))
}

#[inline]
fn build_variant(
    energy: &BaseEnergy,
    growth: &GrowthBudget,
    profile: Option<&InferenceProfile>,
    capabilities: Option<&CapabilitySet>,
    effective_viability: Option<&EffectiveOrganViability>,
) -> EvolutionVariant {
    let profile = profile.copied().unwrap_or_default();
    let capabilities = capabilities.copied().unwrap_or_default();
    // Mapping: InferenceProfile (4 fields) → AnimalSpec (6 biases).
    // armor_bias derived as complement of mobility (rigid = armored).
    let armor_bias = 1.0 - profile.mobility_bias;
    let spec = AnimalSpec::new(
        TrophicClass::Herbivore,
        growth.efficiency,
        profile.mobility_bias,
        armor_bias,
        profile.branching_bias,
        profile.growth_bias,
        profile.resilience,
    );
    let viability = effective_viability
        .map(|v| v.value)
        .unwrap_or_else(|| organ_base_viability(energy.qe(), growth.efficiency));
    EvolutionVariant {
        spec,
        viability,
        role_mask: capabilities.flags as u16,
    }
}

#[inline]
fn scenarios_from_snapshot(snapshot: EnvScenarioSnapshot) -> [EnvContext; 3] {
    let baseline = EnvContext::new(
        snapshot.food_density_t,
        snapshot.predation_pressure_t,
        snapshot.temperature_t,
        snapshot.medium_density_t,
        EVOLUTION_BASELINE_COMPETITION,
    );
    let scarce = EnvContext::new(
        (snapshot.food_density_t * EVOLUTION_SCARCE_FOOD_FACTOR).clamp(0.0, 1.0),
        snapshot.predation_pressure_t,
        snapshot.temperature_t,
        snapshot.medium_density_t,
        EVOLUTION_SCARCE_COMPETITION,
    );
    let hostile = EnvContext::new(
        snapshot.food_density_t,
        (snapshot.predation_pressure_t + EVOLUTION_HOSTILE_PREDATION_DELTA).clamp(0.0, 1.0),
        (snapshot.temperature_t + EVOLUTION_HOSTILE_TEMPERATURE_DELTA).clamp(0.0, 1.0),
        snapshot.medium_density_t,
        EVOLUTION_HOSTILE_COMPETITION,
    );
    [baseline, scarce, hostile]
}

/// Encola tareas surrogate cuando la cola queda vacía (producer LI9).
pub fn evolution_surrogate_enqueue_system(
    snapshot: Res<EnvScenarioSnapshot>,
    mut queue: ResMut<EvolutionSurrogateQueue>,
    query: Query<(
        Entity,
        &BaseEnergy,
        &GrowthBudget,
        Option<&InferenceProfile>,
        Option<&CapabilitySet>,
        Option<&EffectiveOrganViability>,
    )>,
) {
    if !queue.pending.is_empty() {
        return;
    }
    let scenarios = scenarios_from_snapshot(*snapshot);
    let mut entities = query
        .iter()
        .map(|(entity, energy, growth, profile, caps, eff)| {
            (entity, build_variant(energy, growth, profile, caps, eff))
        })
        .collect::<Vec<_>>();
    entities.sort_by_key(|(entity, _)| entity.to_bits());
    for (_, variant) in entities
        .into_iter()
        .take(MAX_EVOLUTION_EVALS_PER_FRAME as usize)
    {
        queue.pending.push_back(EvolutionEvaluationTask {
            variant,
            scenarios,
            scenario_cursor: 0,
            score_accum: 0.0,
        });
    }
}

pub fn evolution_surrogate_tick_system(
    mut queue: ResMut<EvolutionSurrogateQueue>,
    mut state: ResMut<EvolutionSurrogateState>,
    config: Res<EvolutionSurrogateConfig>,
    mut cache: ResMut<BridgeCache<EvolutionSurrogateBridge>>,
) {
    if state.stable_iterations >= config.max_iterations {
        queue.pending.clear();
        state.last_processed_scenario_evals = 0;
        return;
    }

    if queue.pending.is_empty() {
        state.last_processed_scenario_evals = 0;
        return;
    }

    let mut scenario_evals = 0u32;
    let mut finished = Vec::new();

    while scenario_evals < config.max_evals_per_frame {
        let Some(mut task) = queue.pending.pop_front() else {
            break;
        };
        let scenario = task.scenarios[task.scenario_cursor];
        let score =
            evaluate_one_scenario_cached(task.variant, scenario, &mut cache, config.cache_version);
        task.score_accum += score;
        task.scenario_cursor += 1;
        scenario_evals += 1;

        if task.scenario_cursor >= SURROGATE_SCENARIO_COUNT {
            finished.push(EvolutionRankedVariant {
                variant: task.variant,
                fitness: task.score_accum / SURROGATE_SCENARIO_COUNT as f32,
            });
        } else {
            queue.pending.push_back(task);
        }
    }

    state.last_processed_scenario_evals = scenario_evals;
    if finished.is_empty() {
        return;
    }

    let mut union = state.top_k.clone();
    union.extend(finished);
    let next_top_k = top_k_variants(union, config.top_k);
    if state.top_k == next_top_k {
        state.stable_iterations = state.stable_iterations.saturating_add(1);
    } else {
        state.stable_iterations = 0;
    }
    state.top_k = next_top_k;

    if state.stable_iterations >= config.max_iterations {
        queue.pending.clear();
    }
}

pub fn run_surrogate_iterations(
    variants: &[EvolutionVariant],
    scenarios: &[EnvContext],
    top_k: usize,
    max_iterations: u32,
    cache: &mut BridgeCache<EvolutionSurrogateBridge>,
    cache_version: u32,
) -> (Vec<EvolutionRankedVariant>, u32) {
    let mut prev: Vec<EvolutionRankedVariant> = Vec::new();
    let mut iter = 0u32;
    while iter < max_iterations {
        iter += 1;
        let ranked = variants
            .iter()
            .copied()
            .map(|variant| EvolutionRankedVariant {
                variant,
                fitness: evaluate_variant_average_cached(variant, scenarios, cache, cache_version),
            })
            .collect::<Vec<_>>();
        let current = top_k_variants(ranked, top_k);
        if current == prev {
            return (current, iter);
        }
        prev = current;
    }
    (prev, iter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::config::CachePolicy;
    use crate::layers::TrophicClass;

    fn base_spec() -> AnimalSpec {
        AnimalSpec::new(TrophicClass::Herbivore, 0.8, 0.2, 0.2, 0.4, 0.6, 0.5)
    }

    fn rich_env() -> EnvContext {
        EnvContext::new(0.95, 0.2, 0.5, 0.45, 0.2)
    }

    fn poor_env() -> EnvContext {
        EnvContext::new(0.1, 0.2, 0.5, 0.45, 0.2)
    }

    #[test]
    fn cache_key_quantization_is_stable_for_equal_inputs() {
        let a = evolution_cache_key_from_input(&rich_env(), 0.45, 0b1011);
        let b = evolution_cache_key_from_input(&rich_env(), 0.45, 0b1011);
        assert_eq!(a, b);
    }

    #[test]
    fn competition_changes_cache_key() {
        let a = EnvContext::new(0.4, 0.2, 0.5, 0.5, 0.1);
        let b = EnvContext::new(0.4, 0.2, 0.5, 0.5, 0.9);
        let ka = evolution_cache_key_from_input(&a, 0.5, 1 << 6);
        let kb = evolution_cache_key_from_input(&b, 0.5, 1 << 6);
        assert_ne!(ka, kb);
    }

    #[test]
    fn cache_hit_returns_same_fitness_as_recompute_within_epsilon() {
        let variant = EvolutionVariant {
            spec: base_spec(),
            viability: 0.7,
            role_mask: 1 << 6,
        };
        let scenarios = [rich_env(), poor_env()];
        let mut cache = BridgeCache::<EvolutionSurrogateBridge>::new(32, CachePolicy::Lru);
        let first = super::evaluate_variant_average_cached(variant, &scenarios, &mut cache, 1);
        let recompute = scenarios
            .iter()
            .copied()
            .map(|s| super::aggregate_fitness(super::evaluate_sample(variant, s)))
            .sum::<f32>()
            / scenarios.len() as f32;
        let second = super::evaluate_variant_average_cached(variant, &scenarios, &mut cache, 1);
        assert!((first - second).abs() < 1e-6);
        assert!((first - recompute).abs() < 1e-6);
        assert!(cache.stats().hits > 0);
    }

    #[test]
    fn cache_version_change_forces_cache_miss() {
        let variant = EvolutionVariant {
            spec: base_spec(),
            viability: 0.7,
            role_mask: 1 << 6,
        };
        let scenario = rich_env();
        let mut cache = BridgeCache::<EvolutionSurrogateBridge>::new(32, CachePolicy::Lru);

        let _ = super::evaluate_one_scenario_cached(variant, scenario, &mut cache, 1);
        let misses_after_v1 = cache.stats().misses;
        let _ = super::evaluate_one_scenario_cached(variant, scenario, &mut cache, 1);
        let misses_after_v1_hit = cache.stats().misses;
        let _ = super::evaluate_one_scenario_cached(variant, scenario, &mut cache, 2);
        let misses_after_v2 = cache.stats().misses;

        assert_eq!(misses_after_v1, misses_after_v1_hit);
        assert!(misses_after_v2 > misses_after_v1_hit);
    }

    #[test]
    fn low_food_penalizes_high_maintenance_traits() {
        let high_maintenance = EvolutionVariant {
            spec: AnimalSpec::new(TrophicClass::Herbivore, 0.8, 1.0, 1.0, 0.4, 0.6, 0.5),
            viability: 0.8,
            role_mask: 1 << 6,
        };
        let low_maintenance = EvolutionVariant {
            spec: AnimalSpec::new(TrophicClass::Herbivore, 0.8, 0.1, 0.1, 0.4, 0.6, 0.5),
            viability: 0.8,
            role_mask: 1 << 6,
        };
        let mut cache = BridgeCache::<EvolutionSurrogateBridge>::new(64, CachePolicy::Lru);
        let hi = super::evaluate_one_scenario_cached(high_maintenance, poor_env(), &mut cache, 1);
        let lo = super::evaluate_one_scenario_cached(low_maintenance, poor_env(), &mut cache, 1);
        assert!(lo > hi, "low={lo}, high={hi}");
    }

    #[test]
    fn top_k_converges_within_iteration_limit_for_simple_fixture() {
        let variants = [
            EvolutionVariant {
                spec: base_spec(),
                viability: 0.8,
                role_mask: 1 << 6,
            },
            EvolutionVariant {
                spec: AnimalSpec::new(TrophicClass::Herbivore, 0.6, 0.9, 0.9, 0.2, 0.3, 0.4),
                viability: 0.6,
                role_mask: 1 << 6,
            },
            EvolutionVariant {
                spec: AnimalSpec::new(TrophicClass::PrimaryProducer, 0.9, 0.1, 0.1, 0.1, 0.7, 0.7),
                viability: 0.9,
                role_mask: 1 << 6,
            },
        ];
        let scenarios = [rich_env(), poor_env()];
        let mut cache = BridgeCache::<EvolutionSurrogateBridge>::new(128, CachePolicy::Lru);
        let (_top, iterations) =
            run_surrogate_iterations(&variants, &scenarios, 2, 8, &mut cache, 1);
        assert!(iterations <= 3);
    }

    #[test]
    fn frame_budget_limits_number_of_processed_scenario_evals() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(EvolutionSurrogateConfig {
            max_evals_per_frame: 3,
            max_iterations: 8,
            top_k: 2,
            cache_version: 1,
        });
        app.insert_resource(EvolutionSurrogateQueue {
            pending: (0..4)
                .map(|i| EvolutionEvaluationTask {
                    variant: EvolutionVariant {
                        spec: base_spec(),
                        viability: 0.5 + i as f32 * 0.01,
                        role_mask: 1 << 6,
                    },
                    scenarios: [rich_env(), poor_env(), rich_env()],
                    scenario_cursor: 0,
                    score_accum: 0.0,
                })
                .collect(),
        });
        app.insert_resource(EvolutionSurrogateState::default());
        app.insert_resource(BridgeCache::<EvolutionSurrogateBridge>::new(
            256,
            CachePolicy::Lru,
        ));
        app.add_systems(Update, evolution_surrogate_tick_system);
        app.update();
        let state = app.world().resource::<EvolutionSurrogateState>();
        assert_eq!(state.last_processed_scenario_evals, 3);
        assert_eq!(
            app.world()
                .resource::<EvolutionSurrogateQueue>()
                .pending
                .len(),
            4
        );
    }

    #[test]
    fn stable_iterations_limit_clears_pending_work() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(EvolutionSurrogateConfig {
            max_evals_per_frame: 3,
            max_iterations: 1,
            top_k: 2,
            cache_version: 1,
        });
        app.insert_resource(EvolutionSurrogateQueue {
            pending: (0..2)
                .map(|_| EvolutionEvaluationTask {
                    variant: EvolutionVariant {
                        spec: base_spec(),
                        viability: 0.6,
                        role_mask: 1 << 6,
                    },
                    scenarios: [rich_env(), poor_env(), rich_env()],
                    scenario_cursor: 0,
                    score_accum: 0.0,
                })
                .collect(),
        });
        app.insert_resource(EvolutionSurrogateState {
            top_k: Vec::new(),
            last_processed_scenario_evals: 0,
            stable_iterations: 1,
        });
        app.insert_resource(BridgeCache::<EvolutionSurrogateBridge>::new(
            256,
            CachePolicy::Lru,
        ));
        app.add_systems(Update, evolution_surrogate_tick_system);

        app.update();

        assert_eq!(
            app.world()
                .resource::<EvolutionSurrogateQueue>()
                .pending
                .len(),
            0
        );
        assert_eq!(
            app.world()
                .resource::<EvolutionSurrogateState>()
                .last_processed_scenario_evals,
            0
        );
    }
}
