pub mod allometric_growth;
pub mod axiomatic_split;
pub mod body_plan_layout_inference;
pub mod competitive_exclusion;
pub mod constructal_body_plan;
pub mod entity_shape_inference;
pub mod env_scenario;
pub mod evolution_surrogate;
pub mod inference_growth;
pub mod internal_field_diffusion;
pub mod morpho_adaptation;
pub mod organ_lifecycle;
pub mod state_transitions;

pub use body_plan_layout_inference::body_plan_layout_inference_system;
pub use constructal_body_plan::constructal_body_plan_system;
pub use entity_shape_inference::entity_shape_inference_system;
pub use state_transitions::{enter_game_state_playing_system, transition_to_active_system};
