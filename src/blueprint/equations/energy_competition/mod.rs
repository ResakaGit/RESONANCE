//! EC — Energy Competition: matemática pura de pools jerárquicos y extracción competitiva.
//! Onda 0 (pool_equations) + Onda A (extraction registry) + EC-5 (dynamics) + EC-7 (scale).

pub mod dynamics;
mod extraction;
mod metabolic_interference;
mod pool_equations;
pub mod scale;

pub use dynamics::{
    CompetitionMatrix, PoolHealthStatus, PoolTrajectory, competition_intensity, competition_matrix,
    detect_collapse, detect_dominance, detect_equilibrium, predict_pool_trajectory,
};
pub use extraction::{
    ExtractionContext, ExtractionModifier, ExtractionProfile, adaptive_parasite, apex_predator,
    conservative_specialist, evaluate_aggressive_extraction, evaluate_extraction,
    opportunistic_generalist, resilient_homeostatic,
};
pub use metabolic_interference::{apply_metabolic_interference, metabolic_interference_factor};
pub use pool_equations::{
    available_for_extraction, dissipation_loss, extract_aggressive, extract_competitive,
    extract_greedy, extract_proportional, extract_regulated, is_host_collapsing,
    is_pool_equilibrium, pool_next_tick, relative_fitness, scale_extractions_to_available,
    ticks_to_collapse,
};
pub use scale::{
    CompetitiveRegime, classify_competitive_regime, infer_intake_rate, infer_pool_fitness,
    propagate_fitness_to_link,
};
