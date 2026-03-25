//! EC — Energy Competition: matemática pura de pools jerárquicos y extracción competitiva.
//! Onda 0 (pool_equations) + Onda A (extraction registry).

mod pool_equations;
mod extraction;

pub use pool_equations::{
    available_for_extraction, dissipation_loss, extract_aggressive, extract_competitive,
    extract_greedy, extract_proportional, extract_regulated, is_host_collapsing,
    is_pool_equilibrium, pool_next_tick, relative_fitness, scale_extractions_to_available,
    ticks_to_collapse,
};
pub use extraction::{
    evaluate_aggressive_extraction, evaluate_extraction,
    adaptive_parasite, apex_predator, conservative_specialist,
    opportunistic_generalist, resilient_homeostatic,
    ExtractionContext, ExtractionModifier, ExtractionProfile,
};
