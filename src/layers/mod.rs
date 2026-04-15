pub mod behavior;
pub mod body_plan_layout;
pub mod coherence;
pub mod containment;
pub mod converged;
pub mod derived;
pub mod energy;
pub mod energy_pool;
pub mod energy_tag;
pub mod engine;
pub mod entropy_ledger;
pub mod epigenetics;
pub mod flow;
pub mod gompertz_cache;
pub mod growth;
pub mod has_inferred_shape;
pub mod homeostasis;
pub mod identity;
pub mod inference;
pub mod inferred_albedo;
pub mod injector;
pub mod internal_field;
pub mod irradiance;
pub mod kleiber_cache;
pub mod language;
pub mod link;
pub mod macro_step;
pub mod markers;
pub mod metabolic_graph;
pub mod morphogenesis_surface;
pub mod niche;
pub mod nutrient;
pub mod organ;
pub mod oscillatory;
pub mod other_model;
pub mod performance_cache;
pub mod pool_ledger;
pub mod pool_link;
pub mod pressure;
pub mod reflected_spectrum;
pub mod self_model;
pub mod senescence;
pub mod shape_params;
pub mod social_communication;
pub mod structural_link;
pub mod symbiosis;
pub mod tension_field;
pub mod timescale;
pub mod trophic;
pub mod vision_fog;
pub mod volume;
pub mod will;

// Componentes — re-exportación pública
pub use behavior::{
    BehaviorCooldown, BehaviorIntent, BehaviorMode, BehavioralAgent, EnergyAssessment,
    SensoryAwareness,
};
pub use body_plan_layout::BodyPlanLayout;
pub use coherence::{MatterCoherence, MatterState};
pub use containment::{ContactType, ContainedIn};
pub use energy::BaseEnergy;
pub use energy_pool::EnergyPool;
pub use energy_tag::EnergyTag;
pub use engine::AlchemicalEngine;
pub use entropy_ledger::EntropyLedger;
pub use flow::FlowVector;
pub use growth::{AllometricRadiusAnchor, GrowthBudget};
pub use has_inferred_shape::HasInferredShape;
pub use homeostasis::Homeostasis;
pub use identity::{Faction, MobaIdentity, RelationalTag};
pub use inference::{
    AnimalSpec, CapabilitySet, EnvContext, GrowthIntent, InferenceProfile, PendingMorphRebuild,
    TrophicClass,
};
pub use inferred_albedo::InferredAlbedo;
pub use injector::AlchemicalInjector;
pub use internal_field::InternalEnergyField;
pub use irradiance::IrradianceReceiver;
pub use link::{
    ModifiedField, ResonanceFlowOverlay, ResonanceLink, ResonanceMotorOverlay,
    ResonanceThermalOverlay,
};
pub use macro_step::MacroStepTarget;
pub use markers::{AlchemicalBase, Champion, MobileEntity, WaveEntity};
pub use metabolic_graph::{
    ExergyEdge, ExergyNode, METABOLIC_GRAPH_MAX_EDGES, METABOLIC_GRAPH_MAX_NODES, MetabolicGraph,
    MetabolicGraphBuilder, MetabolicGraphError,
};
pub use morphogenesis_surface::MorphogenesisSurface;
pub use nutrient::NutrientProfile;
pub use organ::{
    GeometryPrimitive, LifecycleStage, LifecycleStageCache, MAX_ORGANS_PER_ENTITY,
    ORGAN_ROLE_PRIMITIVE, OrganManifest, OrganRole, OrganSpec,
};
pub use oscillatory::{OscillatorySignature, compute_interference_total};
pub use performance_cache::{CacheScope, PerformanceCachePolicy};
pub use pool_ledger::PoolConservationLedger;
pub use pool_link::{ExtractionType, PoolParentLink};
pub use pressure::AmbientPressure;
pub use reflected_spectrum::ReflectedSpectrum;
pub use shape_params::MorphogenesisShapeParams;
pub use social_communication::{PackMembership, PackRole};
pub use structural_link::StructuralLink;
pub use tension_field::{FieldFalloffMode, TensionField};
pub use trophic::{TrophicConsumer, TrophicState};
pub use vision_fog::{FogHiddenMask, VisionBlocker, VisionFogAnchor, VisionProvider};
pub use volume::SpatialVolume;
pub use will::{
    AbilityCastSpec, AbilityOutput, AbilitySlot, AbilityTarget, Channeling, DespawnOnContact,
    Grimoire, MAX_GRIMOIRE_ABILITIES, OnContactEffect, ProjectedQeFromEnergy, TargetingMode,
    WillActuator,
};

pub use converged::{Converged, hash_f32, hash_pos};
pub use epigenetics::EpigeneticState;
pub use gompertz_cache::GompertzCache;
pub use kleiber_cache::KleiberCache;
pub use language::{LanguageCapacity, MAX_VOCAB_SIZE};
pub use niche::NicheProfile;
pub use other_model::{MAX_MODELS, OtherModel, OtherModelSet};
pub use self_model::{FunctionallyConscious, SelfModel};
pub use senescence::SenescenceProfile;
pub use symbiosis::{SymbiosisLink, SymbiosisType};
pub use timescale::TimescaleAdapter;

pub mod organ_energy_slots;
pub use organ_energy_slots::OrganEnergySlots;

// AUTOPOIESIS track (AP-0/1/2) — chemistry substrate.
pub mod reaction;
pub mod reaction_network;
pub mod species_grid;
pub mod closure;
pub use reaction::{Reaction, SpeciesId, StoichEntry};
pub use reaction_network::{
    ReactionId, ReactionNetwork, ReactionNetworkError, ReactionNetworkSpec, ReactionSpec,
    StoichSpec,
};
pub use species_grid::{SpeciesCell, SpeciesGrid};
pub use closure::{CLOSURE_HISTORY_LEN, ClosureHistory, ClosureMetrics};

// AUTOPOIESIS track (AP-3, ADR-038) — emergent membrane mask (bridge AP-1→AP-3).
pub mod closure_membrane_mask;
pub use closure_membrane_mask::ClosureMembraneMask;

// AUTOPOIESIS track (AP-4, ADR-039) — lineage genealogy registry.
pub mod lineage_registry;
pub use lineage_registry::{LineageRecord, LineageRegistry};

// AUTOPOIESIS track (AP-6c, ADR-041) — per-cell lineage tag (Bevy-free data).
pub mod lineage_grid;
pub use lineage_grid::LineageGrid;

// Ops — SystemParam adapters
pub use derived::PhysicsOps;
pub use energy::EnergyOps;
pub use oscillatory::InterferenceOps;
