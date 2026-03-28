//! Bootstrap de recursos, eventos y plugins compartidos de la simulación (sprint Q5).
//! Mantiene el mismo orden y tipos que el antiguo `SimulationPlugin::build` interno.

use bevy::prelude::*;

use crate::blueprint::{
    AlchemicalAlmanac, AlmanacElementsState, ElementDef, ElementDefRonLoader, EntityLookup,
    IdGenerator, setup_entity_id_observers,
};
use crate::bridge::BridgeConfigPlugin;
use crate::eco::boundary_field::EcoBoundaryField;
use crate::eco::climate::{ClimateAssetState, ClimateConfig, ClimateConfigLoader};
use crate::eco::context_lookup::EcoPlayfieldMargin;
use crate::events::{
    AbilityCastEvent, AbilitySelectionEvent, AllianceDefectEvent, AllianceProposedEvent,
    CatalysisEvent, CatalysisRequest, CollisionEvent,
    CultureConflictEvent, CultureEmergenceEvent,
    DeathEvent, DeltaEnergyCommit, GrimoireProjectileCastPending, GrimoireSelfBuffCastPending,
    HomeostasisAdaptEvent, HungerEvent, PhaseTransitionEvent, PreyConsumedEvent,
    ThreatDetectedEvent,
    SeasonChangeEvent, StructuralLinkBreakEvent, WorldgenMutationEvent,
};
use crate::runtime_platform::intent_projection_3d::CameraBasisForSim;
use crate::runtime_platform::simulation_tick::V6RuntimeConfig;
use crate::simulation::ability_targeting::TargetingState;
use crate::simulation::nutrient_uptake::NutrientUptakeCursor;
use crate::simulation::observers::setup_lifecycle_observers;
use crate::simulation::photosynthesis::IrradianceUpdateCursor;
use crate::simulation::states::{GameState, PlayState};
use crate::topology::config::{TerrainConfig, TerrainConfigAssetState, TerrainConfigRonLoader};
use crate::topology::{TerrainField, TerrainMutationEvent};
use crate::world::fog_of_war::FogOfWarGrid;
use crate::world::{PerceptionCache, Scoreboard, SpatialIndex};
use crate::worldgen::WorldgenWarmupConfig;
use crate::worldgen::systems::terrain::TerrainMutationQueue;
use crate::blueprint::constants::{DEFAULT_GRID_DIMS, DEFAULT_GRID_ORIGIN};
use crate::worldgen::{EnergyFieldGrid, FIELD_CELL_SIZE, NutrientFieldGrid};

/// Inicializa estado de worldgen, grid de energía, almanaque, eventos y `BridgeConfigPlugin`.
pub fn init_simulation_bootstrap(app: &mut App) {
    app.init_resource::<V6RuntimeConfig>()
        .init_resource::<CameraBasisForSim>()
        .init_resource::<crate::blueprint::equations::derived_thresholds::SelfSustainingQeMin>()
        .init_state::<GameState>()
        .add_sub_state::<PlayState>()
        .init_resource::<WorldgenWarmupConfig>();

    app.add_event::<CollisionEvent>()
        .add_event::<PhaseTransitionEvent>()
        .add_event::<CatalysisRequest>()
        .add_event::<DeltaEnergyCommit>()
        .add_event::<CatalysisEvent>()
        .add_event::<DeathEvent>()
        .add_event::<StructuralLinkBreakEvent>()
        .add_event::<HomeostasisAdaptEvent>()
        .add_event::<AbilitySelectionEvent>()
        .add_event::<AbilityCastEvent>()
        .add_event::<GrimoireProjectileCastPending>()
        .add_event::<GrimoireSelfBuffCastPending>()
        .add_event::<SeasonChangeEvent>()
        .add_event::<WorldgenMutationEvent>()
        .add_event::<TerrainMutationEvent>()
        .add_event::<HungerEvent>()
        .add_event::<PreyConsumedEvent>()
        .add_event::<ThreatDetectedEvent>()
        .add_event::<CultureEmergenceEvent>()
        .add_event::<CultureConflictEvent>()
        .add_event::<AllianceProposedEvent>()
        .add_event::<AllianceDefectEvent>();

    setup_lifecycle_observers(app);
    setup_entity_id_observers(app);

    let default_energy_grid =
        EnergyFieldGrid::new(DEFAULT_GRID_DIMS, DEFAULT_GRID_DIMS, FIELD_CELL_SIZE, DEFAULT_GRID_ORIGIN);
    let default_fog_grid = FogOfWarGrid::aligned_with_energy_field(&default_energy_grid);
    let default_nutrient_grid = NutrientFieldGrid::align_with_energy_grid(&default_energy_grid);

    app.init_resource::<IdGenerator>()
        .init_resource::<EntityLookup>()
        .init_resource::<TargetingState>()
        .init_resource::<NutrientUptakeCursor>()
        .init_resource::<super::abiogenesis::AbiogenesisCursor>()
        .init_resource::<IrradianceUpdateCursor>()
        .init_resource::<Scoreboard>()
        .init_resource::<SpatialIndex>()
        .init_resource::<PerceptionCache>()
        .init_resource::<crate::worldgen::systems::materialization::SeasonTransition>()
        .init_resource::<crate::worldgen::systems::materialization::NucleusFreqTrack>()
        .init_resource::<crate::worldgen::systems::performance::WorldgenPerfSettings>()
        .init_resource::<crate::worldgen::systems::performance::WorldgenLodContext>()
        .init_resource::<crate::worldgen::systems::performance::MaterializationCellCache>()
        .init_resource::<crate::worldgen::CellFieldSnapshotCache>()
        .init_resource::<crate::worldgen::systems::performance::MatBudgetCounters>()
        .init_resource::<crate::worldgen::systems::performance::MatCacheStats>()
        .init_resource::<crate::worldgen::systems::performance::PropagationWriteBudget>()
        .init_resource::<crate::worldgen::systems::performance::VisualDerivationFrameState>()
        .init_resource::<TerrainMutationQueue>()
        .insert_resource(default_energy_grid)
        .insert_resource(default_nutrient_grid)
        .insert_resource(default_fog_grid)
        .insert_resource(TerrainField::new(
            DEFAULT_GRID_DIMS,
            DEFAULT_GRID_DIMS,
            FIELD_CELL_SIZE,
            DEFAULT_GRID_ORIGIN,
            0,
        ))
        .init_resource::<EcoBoundaryField>()
        .init_resource::<EcoPlayfieldMargin>()
        .init_resource::<AlchemicalAlmanac>()
        .init_resource::<AlmanacElementsState>()
        .init_asset::<ElementDef>()
        .init_asset_loader::<ElementDefRonLoader>()
        .init_asset::<ClimateConfig>()
        .init_asset_loader::<ClimateConfigLoader>()
        .init_resource::<ClimateAssetState>()
        .init_asset::<TerrainConfig>()
        .init_asset_loader::<TerrainConfigRonLoader>()
        .init_resource::<TerrainConfigAssetState>()
        .add_plugins(BridgeConfigPlugin);
}
