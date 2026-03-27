//! ET-13: Geological Time LOD — ecuaciones puras. Sin deps de Bevy.

/// Nivel de LOD óptimo dado el número de entidades y el horizonte temporal.
pub fn optimal_lod_level(entity_count: u32, tick_horizon: u32, performance_budget: f32) -> u8 {
    let required_work = entity_count as f32 * tick_horizon as f32;
    if required_work <= performance_budget { return 0; }
    let compression_needed = required_work / performance_budget;
    match compression_needed as u32 {
        0..=9   => 0,
        10..=99 => 1,
        100..=999 => 2,
        _ => 3,
    }
}

/// Física simplificada para LOD > 0: energía media del grupo proyectada N ticks.
pub fn compressed_physics_step(
    population_qe: f32,
    mean_intake: f32,
    mean_dissipation: f32,
    tick_compression: u32,
) -> f32 {
    let net_per_tick = mean_intake - mean_dissipation;
    (population_qe + net_per_tick * tick_compression as f32).max(0.0)
}

/// Varianza del grupo para desagregar con ruido apropiado.
pub fn population_variance(mean_qe: f32, variance_factor: f32, group_size: u32) -> f32 {
    if group_size == 0 { return 0.0; }
    mean_qe * variance_factor / (group_size as f32).sqrt()
}

/// Qe asignado a una entidad al desagregar una población (ruido LCG deterministico).
pub fn desegregated_qe(mean_qe: f32, variance: f32, entity_seed: u32) -> f32 {
    let noise = (entity_seed.wrapping_mul(1664525).wrapping_add(1013904223)) as f32 / u32::MAX as f32;
    let centered = noise * 2.0 - 1.0;
    (mean_qe + centered * variance).max(0.0)
}

/// Tasa de extinción simplificada para LOD alto.
pub fn population_extinction_rate(mean_qe: f32, dissipation_rate: f32, environmental_stress: f32) -> f32 {
    if mean_qe <= 0.0 { return 1.0; }
    (dissipation_rate + environmental_stress) / mean_qe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimal_lod_zero_when_affordable() {
        assert_eq!(optimal_lod_level(10, 10, 1000.0), 0);
    }

    #[test]
    fn optimal_lod_increases_with_load() {
        let lod = optimal_lod_level(1000, 1000, 100.0);
        assert!(lod > 0);
    }

    #[test]
    fn compressed_physics_step_positive_net() {
        let qe = compressed_physics_step(100.0, 2.0, 1.0, 10);
        assert!((qe - 110.0).abs() < 1e-4);
    }

    #[test]
    fn compressed_physics_step_clamped_at_zero() {
        let qe = compressed_physics_step(10.0, 0.0, 5.0, 10);
        assert_eq!(qe, 0.0);
    }

    #[test]
    fn population_variance_zero_group() {
        assert_eq!(population_variance(100.0, 0.1, 0), 0.0);
    }

    #[test]
    fn desegregated_qe_deterministic() {
        let a = desegregated_qe(100.0, 10.0, 42);
        let b = desegregated_qe(100.0, 10.0, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn desegregated_qe_non_negative() {
        for seed in 0..100u32 {
            assert!(desegregated_qe(5.0, 100.0, seed) >= 0.0);
        }
    }

    #[test]
    fn population_extinction_rate_one_when_dead() {
        assert!((population_extinction_rate(0.0, 0.1, 0.1) - 1.0).abs() < 1e-5);
    }
}
