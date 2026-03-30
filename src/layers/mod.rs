pub mod behavior;
pub mod body_plan_layout;
pub mod epigenetics;
pub mod language;
pub mod niche;
pub mod other_model;
pub mod self_model;
pub mod senescence;
pub mod symbiosis;
pub mod timescale;
pub mod coherence;
pub mod containment;
pub mod derived;
pub mod energy;
pub mod energy_pool;
pub mod internal_field;
pub mod engine;
pub mod entropy_ledger;
pub mod flow;
pub mod growth;
pub mod has_inferred_shape;
pub mod homeostasis;
pub mod identity;
pub mod inference;
pub mod inferred_albedo;
pub mod injector;
pub mod irradiance;
pub mod link;
pub mod markers;
pub mod metabolic_graph;
pub mod morphogenesis_surface;
pub mod nutrient;
pub mod oscillatory;
pub mod macro_step;
pub mod organ;
pub mod performance_cache;
pub mod pool_ledger;
pub mod pool_link;
pub mod pressure;
pub mod shape_params;
pub mod social_communication;
pub mod structural_link;
pub mod tension_field;
pub mod trophic;
pub mod vision_fog;
pub mod volume;
pub mod will;
pub mod kleiber_cache;
pub mod gompertz_cache;
pub mod converged;

// Componentes — re-exportación pública
pub use body_plan_layout::BodyPlanLayout;
pub use has_inferred_shape::HasInferredShape;
pub use behavior::{
    BehaviorCooldown, BehaviorIntent, BehaviorMode, BehavioralAgent, EnergyAssessment,
    SensoryAwareness,
};
pub use coherence::{MatterCoherence, MatterState};
pub use containment::{ContactType, ContainedIn};
pub use energy::BaseEnergy;
pub use energy_pool::EnergyPool;
pub use entropy_ledger::EntropyLedger;
pub use engine::AlchemicalEngine;
pub use internal_field::InternalEnergyField;
pub use flow::FlowVector;
pub use growth::{AllometricRadiusAnchor, GrowthBudget};
pub use homeostasis::Homeostasis;
pub use identity::{Faction, MobaIdentity, RelationalTag};
pub use inference::{AnimalSpec, CapabilitySet, EnvContext, GrowthIntent, InferenceProfile, PendingMorphRebuild, TrophicClass};
pub use trophic::{TrophicConsumer, TrophicState};
pub use inferred_albedo::InferredAlbedo;
pub use injector::AlchemicalInjector;
pub use irradiance::IrradianceReceiver;
pub use link::{
    ModifiedField, ResonanceFlowOverlay, ResonanceLink, ResonanceMotorOverlay,
    ResonanceThermalOverlay,
};
pub use markers::{AlchemicalBase, Champion, MobileEntity, WaveEntity};
pub use metabolic_graph::{
    ExergyEdge, ExergyNode, MetabolicGraph, MetabolicGraphBuilder, MetabolicGraphError,
    METABOLIC_GRAPH_MAX_EDGES, METABOLIC_GRAPH_MAX_NODES,
};
pub use nutrient::NutrientProfile;
pub use oscillatory::{OscillatorySignature, compute_interference_total};
pub use macro_step::MacroStepTarget;
pub use pool_ledger::PoolConservationLedger;
pub use pool_link::{ExtractionType, PoolParentLink};
pub use organ::{
    GeometryPrimitive, LifecycleStage, LifecycleStageCache, OrganManifest, OrganRole, OrganSpec,
    MAX_ORGANS_PER_ENTITY, ORGAN_ROLE_PRIMITIVE,
};
pub use performance_cache::{CacheScope, PerformanceCachePolicy};
pub use pressure::AmbientPressure;
pub use morphogenesis_surface::MorphogenesisSurface;
pub use shape_params::MorphogenesisShapeParams;
pub use social_communication::{PackMembership, PackRole};
pub use structural_link::StructuralLink;
pub use tension_field::{FieldFalloffMode, TensionField};
pub use vision_fog::{FogHiddenMask, VisionBlocker, VisionFogAnchor, VisionProvider};
pub use volume::SpatialVolume;
pub use will::{
    AbilityCastSpec, AbilityOutput, AbilitySlot, AbilityTarget, Channeling, DespawnOnContact,
    Grimoire, MAX_GRIMOIRE_ABILITIES, OnContactEffect, ProjectedQeFromEnergy, TargetingMode,
    WillActuator,
};

pub use epigenetics::EpigeneticState;
pub use language::{LanguageCapacity, MAX_VOCAB_SIZE};
pub use niche::NicheProfile;
pub use other_model::{OtherModel, OtherModelSet, MAX_MODELS};
pub use self_model::{FunctionallyConscious, SelfModel};
pub use senescence::SenescenceProfile;
pub use kleiber_cache::KleiberCache;
pub use gompertz_cache::GompertzCache;
pub use converged::{Converged, hash_f32, hash_pos};
pub use symbiosis::{SymbiosisLink, SymbiosisType};
pub use timescale::TimescaleAdapter;

// Ops — SystemParam adapters
pub use derived::PhysicsOps;
pub use energy::EnergyOps;
pub use oscillatory::InterferenceOps;
