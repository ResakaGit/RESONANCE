use bevy::prelude::*;

use crate::blueprint::ElementId;
use crate::layers::link::{ModifiedField, ResonanceLink};
use crate::layers::will::{
    AbilityCastSpec, AbilityOutput, AbilitySlot, AbilityTarget, Channeling, Grimoire, TargetingMode,
};
use crate::layers::*;

/// Plugin que registra todos los tipos de componentes de las 14 capas
/// para reflexión, inspector y serialización.
pub struct LayersPlugin;

impl Plugin for LayersPlugin {
    fn build(&self, app: &mut App) {
        app
            // Tipos “value objects” usados dentro de componentes.
            .register_type::<ElementId>()
            // Sprint G11 — IDs persistentes
            .register_type::<crate::blueprint::ChampionId>()
            .register_type::<crate::blueprint::WorldEntityId>()
            .register_type::<crate::blueprint::EffectId>()
            .register_type::<crate::blueprint::IdGenerator>()
            // Jerarquía #[require] (sprint G6)
            .register_type::<crate::layers::AlchemicalBase>()
            .register_type::<crate::layers::WaveEntity>()
            .register_type::<crate::layers::MobileEntity>()
            .register_type::<crate::layers::Champion>()
            // Shape inference marker (Gap 1 / MG-9)
            .register_type::<crate::layers::HasInferredShape>()
            // Capa 0
            .register_type::<BaseEnergy>()
            // Capa 1
            .register_type::<SpatialVolume>()
            // Capa 2
            .register_type::<OscillatorySignature>()
            // Capa 3
            .register_type::<FlowVector>()
            // Capa 4
            .register_type::<MatterCoherence>()
            .register_type::<MatterState>()
            .register_type::<NutrientProfile>()
            .register_type::<GrowthBudget>()
            .register_type::<AllometricRadiusAnchor>()
            .register_type::<IrradianceReceiver>()
            // Capa 5
            .register_type::<AlchemicalEngine>()
            // Capa 6
            .register_type::<AmbientPressure>()
            // Capa 7
            .register_type::<WillActuator>()
            .register_type::<Grimoire>()
            .register_type::<AbilityOutput>()
            .register_type::<AbilityCastSpec>()
            .register_type::<TargetingMode>()
            .register_type::<AbilityTarget>()
            .register_type::<AbilitySlot>()
            .register_type::<Channeling>()
            .register_type::<DespawnOnContact>()
            .register_type::<OnContactEffect>()
            .register_type::<ProjectedQeFromEnergy>()
            // D1: Behavioral Intelligence
            .register_type::<BehavioralAgent>()
            .register_type::<BehaviorIntent>()
            .register_type::<BehaviorMode>()
            .register_type::<BehaviorCooldown>()
            .register_type::<EnergyAssessment>()
            .register_type::<SensoryAwareness>()
            // Inferencia desacoplada (perfil/capacidades/intenciones)
            .register_type::<InferenceProfile>()
            .register_type::<CapabilitySet>()
            .register_type::<GrowthIntent>()
            .register_type::<LifecycleStage>()
            .register_type::<LifecycleStageCache>()
            .register_type::<GeometryPrimitive>()
            .register_type::<OrganRole>()
            .register_type::<OrganSpec>()
            .register_type::<OrganManifest>()
            .register_type::<crate::layers::ExergyNode>()
            .register_type::<crate::layers::ExergyEdge>()
            .register_type::<crate::layers::MetabolicGraph>()
            .register_type::<crate::layers::EntropyLedger>()
            .register_type::<crate::layers::MorphogenesisShapeParams>()
            .register_type::<crate::layers::InferredAlbedo>()
            .register_type::<crate::layers::MorphogenesisSurface>()
            // M2: LOD macro-step marker
            .register_type::<crate::layers::MacroStepTarget>()
            // Energy Competition (EC-2)
            .register_type::<crate::layers::EnergyPool>()
            .register_type::<crate::layers::ExtractionType>()
            .register_type::<crate::layers::PoolParentLink>()
            // Capa 8
            .register_type::<AlchemicalInjector>()
            // Capa 9
            .register_type::<MobaIdentity>()
            .register_type::<Faction>()
            .register_type::<RelationalTag>()
            .register_type::<crate::layers::VisionProvider>()
            .register_type::<crate::layers::VisionFogAnchor>()
            .register_type::<crate::layers::VisionBlocker>()
            .register_type::<crate::layers::FogHiddenMask>()
            // Capa 10
            .register_type::<ResonanceLink>()
            .register_type::<ModifiedField>()
            .register_type::<crate::layers::ResonanceFlowOverlay>()
            .register_type::<crate::layers::ResonanceMotorOverlay>()
            .register_type::<crate::layers::ResonanceThermalOverlay>();

        // Componentes auxiliares (V4)
        app.register_type::<crate::layers::ContainedIn>()
            .register_type::<crate::layers::ContactType>()
            // Capa 11
            .register_type::<crate::layers::TensionField>()
            .register_type::<crate::layers::FieldFalloffMode>()
            // Capa 12
            .register_type::<crate::layers::Homeostasis>()
            // Capa 13
            .register_type::<crate::layers::StructuralLink>()
            // Capa V5 (optimización)
            .register_type::<crate::layers::PerformanceCachePolicy>()
            .register_type::<crate::layers::CacheScope>()
            // Eco-Boundaries + materialización E6
            .register_type::<crate::eco::contracts::ZoneClass>()
            .register_type::<crate::eco::contracts::TransitionType>()
            .register_type::<crate::eco::contracts::BoundaryMarker>()
            .register_type::<crate::worldgen::BoundaryVisual>()
            .register_type::<crate::worldgen::PendingEnergyVisualRebuild>()
            .register_type::<crate::worldgen::PhenologyVisualParams>()
            .register_type::<crate::worldgen::PhenologyPhaseCache>()
            .register_type::<crate::blueprint::ElementPhenologyDef>()
            .register_type::<crate::worldgen::WorldArchetype>();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::LayersPlugin;
    use crate::layers::{
        GeometryPrimitive, LifecycleStage, LifecycleStageCache, OrganManifest, OrganRole, OrganSpec,
    };

    #[test]
    fn layers_plugin_registers_li1_reflect_types() {
        let mut app = App::new();
        app.add_plugins(LayersPlugin);

        let registry = app.world().resource::<AppTypeRegistry>().read();
        assert!(registry.get(std::any::TypeId::of::<LifecycleStage>()).is_some());
        assert!(registry.get(std::any::TypeId::of::<LifecycleStageCache>()).is_some());
        assert!(registry.get(std::any::TypeId::of::<GeometryPrimitive>()).is_some());
        assert!(registry.get(std::any::TypeId::of::<OrganRole>()).is_some());
        assert!(registry.get(std::any::TypeId::of::<OrganSpec>()).is_some());
        assert!(registry.get(std::any::TypeId::of::<OrganManifest>()).is_some());
    }
}
