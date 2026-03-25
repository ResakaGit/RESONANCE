use super::finite_helpers::{finite_non_negative, finite_unit};
use crate::blueprint::constants::*;

pub fn starvation_threshold(base_threshold: f32, resilience: f32) -> f32 {
    let base = if base_threshold.is_finite() {
        base_threshold.max(0.0)
    } else {
        0.0
    };
    let r = if resilience.is_finite() {
        resilience.clamp(0.0, 1.0)
    } else {
        0.0
    };
    // Alta resiliencia baja el umbral (sobrevive con menos)
    base * (1.0 - r * 0.8)
}

/// Stress metabólico: ratio de qe actual vs umbral (qe/threshold).
/// Valores ≥ 1.0 indican qe por encima del umbral de inanición adaptativo.
/// Si `threshold <= 0` o no es finito, no hay escala definida: retorna 1.0 (sin estrés numérico).
#[inline]
pub fn metabolic_viability(qe: f32, threshold: f32) -> f32 {
    if threshold <= 0.0 || !threshold.is_finite() {
        return 1.0;
    }
    let q = if qe.is_finite() { qe.max(0.0) } else { 0.0 };
    q / threshold
}

/// Ganancia ambiental de intake (`>= 0`) a partir de señales exógenas normalizadas.
#[inline]
pub fn env_intake_gain(food_density_t: f32, medium_density_t: f32) -> f32 {
    let food = finite_unit(food_density_t);
    let medium = finite_unit(medium_density_t);
    (ENV_INTAKE_GAIN_FLOOR + food * ENV_INTAKE_FOOD_WEIGHT + medium * ENV_INTAKE_MEDIUM_WEIGHT).max(0.0)
}

/// Penalización de mantenimiento (`>= 0`) por temperatura y predación.
#[inline]
pub fn env_maintenance_penalty(temperature_t: f32, predation_pressure_t: f32) -> f32 {
    let temp = finite_unit(temperature_t);
    let predation = finite_unit(predation_pressure_t);
    let thermal_deviation = (temp - 0.5).abs() * 2.0;
    (thermal_deviation * ENV_MAINT_TEMPERATURE_SCALE + predation * ENV_MAINT_PREDATION_SCALE).max(0.0)
}

/// Penalización de estrés (`>= 0`) por presión de caza y densidad del medio.
#[inline]
pub fn env_stress_penalty(predation_pressure_t: f32, medium_density_t: f32) -> f32 {
    let predation = finite_unit(predation_pressure_t);
    let medium = finite_unit(medium_density_t);
    (predation * ENV_STRESS_PREDATION_SCALE + medium * ENV_STRESS_MEDIUM_SCALE).max(0.0)
}

/// Viabilidad efectiva para inferencia morfológica bajo entorno exógeno.
#[inline]
pub fn organ_viability_score(
    base_viability: f32,
    intake_gain: f32,
    maintenance_penalty: f32,
    stress_penalty: f32,
) -> f32 {
    let base = finite_non_negative(base_viability);
    let intake = finite_non_negative(intake_gain);
    let maintenance = finite_non_negative(maintenance_penalty);
    let stress = finite_non_negative(stress_penalty);
    let score = base * intake - maintenance - stress;
    if score.is_finite() { score.max(0.0) } else { 0.0 }
}

/// Viabilidad base sin entorno para inferencia de órganos.
#[inline]
pub fn organ_base_viability(qe: f32, growth_efficiency: f32) -> f32 {
    let qe_ref = ORGAN_BASE_VIABILITY_QE_REFERENCE.max(1.0);
    let qe_norm = (finite_non_negative(qe) / qe_ref).clamp(0.0, 1.0);
    let efficiency = finite_unit(growth_efficiency);
    (qe_norm * ORGAN_BASE_VIABILITY_QE_WEIGHT + efficiency * ORGAN_BASE_VIABILITY_EFFICIENCY_WEIGHT)
        .clamp(0.0, 1.0)
}

/// Intake relativo por clase trófica y disponibilidad de alimento.
#[inline]
pub fn trophic_intake_factor(trophic: crate::layers::TrophicClass, food_density_t: f32) -> f32 {
    let food = finite_unit(food_density_t);
    let class_factor = TROPHIC_INTAKE_FACTOR[trophic as usize];
    (class_factor * food).max(0.0)
}

/// Asimilación energética luego de intake bruto.
#[inline]
pub fn trophic_assimilation(intake_qe: f32, metabolic_efficiency: f32, temperature_t: f32) -> f32 {
    let intake = finite_non_negative(intake_qe);
    let efficiency = finite_unit(metabolic_efficiency);
    let temp = finite_unit(temperature_t);
    let temp_penalty = ((temp - 0.5).abs() * 2.0 * TROPHIC_ASSIMILATION_TEMP_PENALTY).clamp(0.0, 1.0);
    (intake * efficiency * (1.0 - temp_penalty)).max(0.0)
}

/// Costo de mantenimiento energético por masa y sesgos funcionales.
#[inline]
pub fn trophic_maintenance_cost(
    mass_t: f32,
    mobility_bias: f32,
    armor_bias: f32,
    predation_pressure_t: f32,
    medium_density_t: f32,
) -> f32 {
    let mass = finite_non_negative(mass_t);
    let mobility = finite_unit(mobility_bias);
    let armor = finite_unit(armor_bias);
    let predation = finite_unit(predation_pressure_t);
    let medium = finite_unit(medium_density_t);
    let load = TROPHIC_MAINTENANCE_BASE
        + mobility * TROPHIC_MAINTENANCE_MOBILITY_WEIGHT
        + armor * TROPHIC_MAINTENANCE_ARMOR_WEIGHT
        + predation * TROPHIC_MAINTENANCE_PREDATION_WEIGHT
        + medium * TROPHIC_MAINTENANCE_MEDIUM_WEIGHT;
    (mass * load).max(0.0)
}

/// Delta neto de qe tras competencia del entorno.
#[inline]
pub fn trophic_net_qe_delta(assimilation_qe: f32, maintenance_qe: f32, competition_t: f32) -> f32 {
    let assimilation = finite_non_negative(assimilation_qe);
    let maintenance = finite_non_negative(maintenance_qe);
    let competition = finite_unit(competition_t);
    let competition_penalty = maintenance * competition * TROPHIC_COMPETITION_PENALTY_SCALE;
    let out = assimilation - maintenance - competition_penalty;
    if out.is_finite() { out } else { 0.0 }
}

/// Score de supervivencia base para surrogate evolutivo (LI9).
#[inline]
pub fn evolution_survival_score(viability: f32, net_qe_delta: f32) -> f32 {
    finite_non_negative(viability) + net_qe_delta
}

/// Score reproductivo condicionado por capacidad y sesgo.
#[inline]
pub fn evolution_reproduction_score(can_reproduce: bool, net_qe_delta: f32, reproduction_bias: f32) -> f32 {
    if can_reproduce {
        (net_qe_delta * finite_unit(reproduction_bias)).max(0.0)
    } else {
        0.0
    }
}

/// Agregación final de fitness surrogate (LI9).
#[inline]
pub fn evolution_aggregate_fitness(
    survival_score: f32,
    reproduction_score: f32,
    maintenance_cost: f32,
    maintenance_weight: f32,
) -> f32 {
    finite_non_negative(survival_score)
        + finite_non_negative(reproduction_score)
        - finite_non_negative(maintenance_cost) * finite_non_negative(maintenance_weight)
}
