pub mod archetypes;
pub mod builder;
pub mod composition;
pub(crate) mod constants;
pub(crate) mod lifecycle_observers;

pub use composition::{
    EffectConfig, EngineConfig, InjectorConfig, MatterConfig, PhysicsConfig, PressureConfig,
};

pub use archetypes::{
    BiomeType, HeroClass, spawn_aquatic_organism, spawn_desert_creature, spawn_desert_plant,
    spawn_forest_plant,
};
pub use builder::EntityBuilder;
