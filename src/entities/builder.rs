use bevy::prelude::*;

use crate::blueprint::ElementId;
use crate::blueprint::equations;
use crate::layers::{
    AlchemicalEngine, AlchemicalInjector, AllometricRadiusAnchor, AmbientPressure, BaseEnergy,
    Faction, FlowVector, GrowthBudget, Homeostasis, IrradianceReceiver, MatterCoherence,
    MatterState, MetabolicGraph, MobaIdentity, MorphogenesisShapeParams, NutrientProfile,
    OrganManifest, OscillatorySignature, SpatialVolume, StructuralLink, TensionField, WillActuator,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::kinematics_3d_adapter::V6RuntimeEntity;

/// Builder unificado de entidad por capas incrementales (Sprint 05).
#[derive(Debug, Default)]
pub struct EntityBuilder {
    name: Option<String>,
    position: Vec2,
    energy: Option<BaseEnergy>,
    volume: Option<SpatialVolume>,
    wave: Option<(ElementId, OscillatorySignature)>,
    flow: Option<FlowVector>,
    matter: Option<MatterCoherence>,
    engine: Option<AlchemicalEngine>,
    ambient: Option<AmbientPressure>,
    will: Option<WillActuator>,
    injector: Option<AlchemicalInjector>,
    identity: Option<MobaIdentity>,
    tension_field: Option<TensionField>,
    homeostasis: Option<Homeostasis>,
    structural_link: Option<StructuralLink>,
    nutrient: Option<NutrientProfile>,
    growth_budget: Option<GrowthBudget>,
    organ_manifest: Option<OrganManifest>,
    metabolic_graph: Option<MetabolicGraph>,
    irradiance: Option<IrradianceReceiver>,
    /// `Some(y)` → sim `(sx,sy)` en mundo `(sx, y, sy)`; `None` → legacy `(sx, sy, 0)`.
    ground_plane_y: Option<f32>,
    /// Registra observer `OnAdd<BaseEnergy>` antes del insert (solo héroes / spawn con log de L0).
    observe_base_energy_on_spawn: bool,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn at(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Alinea spawn al plano XZ con altura Y fija (perfil full3d).
    pub fn sim_world_layout(mut self, layout: &SimWorldTransformParams) -> Self {
        if layout.use_xz_ground {
            self.ground_plane_y = Some(layout.standing_y);
        }
        self
    }

    /// Sprint G7: observer por entidad; debe ejecutarse antes de insertar `BaseEnergy`.
    pub fn observe_hero_base_energy_spawn(mut self) -> Self {
        self.observe_base_energy_on_spawn = true;
        self
    }

    /// Capa 0
    pub fn energy(mut self, qe: f32) -> Self {
        self.energy = Some(BaseEnergy::new(qe));
        self
    }

    /// Capa 1
    pub fn volume(mut self, radius: f32) -> Self {
        self.volume = Some(SpatialVolume::new(radius));
        self
    }

    /// Capa 2
    ///
    /// Nota: la frecuencia efectiva se deriva desde `ElementId` en runtime.
    pub fn wave(mut self, element_id: ElementId) -> Self {
        self.wave = Some((element_id, OscillatorySignature::new(0.0, 0.0)));
        self
    }

    /// Capa 2 from frequency Hz (axiomatic abiogenesis — no element lookup).
    ///
    /// Sets OscillatorySignature directly from Hz. ElementId derived from frequency band.
    pub fn wave_from_hz(mut self, frequency_hz: f32) -> Self {
        use crate::blueprint::equations::element_symbol_from_frequency;
        let sym = element_symbol_from_frequency(frequency_hz);
        self.wave = Some((
            ElementId::from_name(sym),
            OscillatorySignature::new(frequency_hz, 0.0),
        ));
        self
    }

    /// Capa 3
    pub fn flow(mut self, velocity: Vec2, dissipation: f32) -> Self {
        self.flow = Some(FlowVector::new(velocity, dissipation));
        self
    }

    /// Capa 4
    pub fn matter(mut self, state: MatterState, bond_energy: f32, conductivity: f32) -> Self {
        self.matter = Some(MatterCoherence::new(state, bond_energy, conductivity));
        self
    }

    /// Capa 5
    pub fn motor(
        mut self,
        max_buffer: f32,
        input_valve: f32,
        output_valve: f32,
        initial_buffer: f32,
    ) -> Self {
        self.engine = Some(AlchemicalEngine::new(
            max_buffer,
            input_valve,
            output_valve,
            initial_buffer,
        ));
        self
    }

    /// Capa 6
    pub fn ambient(mut self, delta_qe: f32, viscosity: f32) -> Self {
        self.ambient = Some(AmbientPressure::new(delta_qe, viscosity));
        self
    }

    /// Capa 7
    pub fn will_default(mut self) -> Self {
        self.will = Some(WillActuator::default());
        self
    }

    /// Capa 8
    pub fn injector(
        mut self,
        projected_qe: f32,
        forced_frequency: f32,
        influence_radius: f32,
    ) -> Self {
        self.injector = Some(AlchemicalInjector::new(
            projected_qe,
            forced_frequency,
            influence_radius,
        ));
        self
    }

    /// Capa 9
    pub fn identity(mut self, faction: Faction, tag_bits: u8, critical_multiplier: f32) -> Self {
        self.identity = Some(MobaIdentity {
            faction,
            relational_tags: tag_bits,
            critical_multiplier,
        });
        self
    }

    /// Capa 11
    pub fn tension_field(mut self, field: TensionField) -> Self {
        self.tension_field = Some(field);
        self
    }

    /// Capa 12
    pub fn homeostasis(mut self, homeostasis: Homeostasis) -> Self {
        self.homeostasis = Some(homeostasis);
        self
    }

    /// Capa 13
    pub fn structural_link(mut self, link: StructuralLink) -> Self {
        self.structural_link = Some(link);
        self
    }

    /// Capa 4 (metabólica): nutrientes normalizados.
    pub fn nutrient(mut self, carbon: f32, nitrogen: f32, phosphorus: f32, water: f32) -> Self {
        self.nutrient = Some(NutrientProfile::new(carbon, nitrogen, phosphorus, water));
        self
    }

    /// Capa 4 (metabólica): presupuesto de biomasa disponible.
    pub fn growth_budget(mut self, biomass: f32, limiter: u8, efficiency: f32) -> Self {
        self.growth_budget = Some(GrowthBudget::new(biomass, limiter, efficiency));
        self
    }

    /// Manifesto de órganos (consumido por `with_metabolic_graph_inferred`).
    pub fn with_organ_manifest(mut self, manifest: OrganManifest) -> Self {
        self.organ_manifest = Some(manifest);
        self
    }

    /// Infiere MetabolicGraph desde OrganManifest (requiere `with_organ_manifest` previo).
    pub fn with_metabolic_graph_inferred(mut self, t_core: f32, t_env: f32) -> Self {
        let manifest = self
            .organ_manifest
            .as_ref()
            .expect("with_organ_manifest must be called before with_metabolic_graph_inferred");
        self.metabolic_graph = Some(equations::metabolic_graph_from_manifest(
            manifest, t_core, t_env,
        ));
        self
    }

    /// MetabolicGraph construido manualmente (vía MetabolicGraphBuilder).
    pub fn with_metabolic_graph(mut self, graph: MetabolicGraph) -> Self {
        self.metabolic_graph = Some(graph);
        self
    }

    /// Irradiancia recibida (extensión Capa 1).
    pub fn irradiance(mut self, photon_density: f32, absorbed_fraction: f32) -> Self {
        self.irradiance = Some(IrradianceReceiver::new(photon_density, absorbed_fraction));
        self
    }

    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let snapshot_bridge_3d = self.energy.is_some() && self.volume.is_some();
        let translation = if let Some(gy) = self.ground_plane_y {
            Vec3::new(self.position.x, gy, self.position.y)
        } else {
            Vec3::new(self.position.x, self.position.y, 0.0)
        };
        let mut entity = commands.spawn((
            Transform::from_translation(translation),
            Visibility::default(),
        ));

        if self.observe_base_energy_on_spawn {
            entity.observe(super::lifecycle_observers::on_hero_base_energy_added);
        }

        if let Some(name) = self.name {
            entity.insert(Name::new(name));
        }
        if let Some(energy) = self.energy {
            entity.insert(energy);
        }
        if let Some(volume) = self.volume {
            let base_radius = volume.radius;
            entity.insert((volume, AllometricRadiusAnchor::new(base_radius)));
        }
        if let Some((element_id, wave)) = self.wave {
            entity.insert((element_id, wave));
        }
        if let Some(flow) = self.flow {
            entity.insert(flow);
        }
        if let Some(matter) = self.matter {
            entity.insert(matter);
        }
        if let Some(engine) = self.engine {
            entity.insert(engine);
        }
        if let Some(ambient) = self.ambient {
            entity.insert(ambient);
        }
        if let Some(will) = self.will {
            entity.insert(will);
        }
        if let Some(injector) = self.injector {
            entity.insert(injector);
        }
        if let Some(identity) = self.identity {
            entity.insert(identity);
        }
        if let Some(field) = self.tension_field {
            entity.insert(field);
        }
        if let Some(homeostasis) = self.homeostasis {
            entity.insert(homeostasis);
        }
        if let Some(link) = self.structural_link {
            entity.insert(link);
        }
        if let Some(nutrient) = self.nutrient {
            entity.insert(nutrient);
        }
        if let Some(growth_budget) = self.growth_budget {
            entity.insert(growth_budget);
        }
        if let Some(graph) = self.metabolic_graph {
            entity.insert((graph, MorphogenesisShapeParams::default()));
        }
        if let Some(irradiance) = self.irradiance {
            entity.insert(irradiance);
        }

        let id = entity.id();
        // Puente PBR 3D: solo entidades con volumen + energía (Capa 0+1) participan en snapshot.
        if snapshot_bridge_3d {
            commands.entity(id).insert(V6RuntimeEntity);
        }

        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::{LifecycleStage, MetabolicGraphBuilder, OrganRole, OrganSpec};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn with_metabolic_graph_inserts_graph_on_spawn() {
        let mut app = test_app();
        let graph = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0)
            .add_node(OrganRole::Stem, 0.8, 5.0)
            .add_edge(0, 1, 50.0)
            .build()
            .unwrap();
        let mut commands = app.world_mut().commands();
        let entity = EntityBuilder::new()
            .energy(100.0)
            .volume(1.0)
            .with_metabolic_graph(graph)
            .spawn(&mut commands);
        drop(commands);
        app.update();
        assert!(app.world().entity(entity).contains::<MetabolicGraph>());
    }

    #[test]
    fn with_metabolic_graph_also_inserts_shape_params() {
        let mut app = test_app();
        let graph = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0)
            .build()
            .unwrap();
        let mut commands = app.world_mut().commands();
        let entity = EntityBuilder::new()
            .energy(100.0)
            .with_metabolic_graph(graph)
            .spawn(&mut commands);
        drop(commands);
        app.update();
        assert!(
            app.world()
                .entity(entity)
                .contains::<MorphogenesisShapeParams>()
        );
    }

    #[test]
    fn with_metabolic_graph_inferred_creates_from_manifest() {
        let mut app = test_app();
        let mut manifest = OrganManifest::new(LifecycleStage::Mature);
        manifest.push(OrganSpec::new(OrganRole::Root, 1, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Leaf, 1, 1.0));
        let mut commands = app.world_mut().commands();
        let entity = EntityBuilder::new()
            .energy(200.0)
            .volume(1.0)
            .with_organ_manifest(manifest)
            .with_metabolic_graph_inferred(400.0, 300.0)
            .spawn(&mut commands);
        drop(commands);
        app.update();
        let graph = app.world().entity(entity).get::<MetabolicGraph>().unwrap();
        assert_eq!(graph.node_count(), 3);
    }

    #[test]
    #[should_panic(expected = "with_organ_manifest must be called")]
    fn with_metabolic_graph_inferred_without_manifest_panics() {
        let _ = EntityBuilder::new().with_metabolic_graph_inferred(400.0, 300.0);
    }

    #[test]
    fn irradiance_inserts_receiver_on_spawn() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let entity = EntityBuilder::new()
            .energy(100.0)
            .irradiance(50.0, 0.6)
            .spawn(&mut commands);
        drop(commands);
        app.update();
        let ir = app
            .world()
            .entity(entity)
            .get::<IrradianceReceiver>()
            .unwrap();
        assert!((ir.photon_density - 50.0).abs() < 1e-6);
    }

    #[test]
    fn full_morphogenesis_chain_produces_valid_entity() {
        let mut app = test_app();
        let mut manifest = OrganManifest::new(LifecycleStage::Mature);
        manifest.push(OrganSpec::new(OrganRole::Core, 1, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Fin, 1, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Sensory, 1, 1.0));
        let mut commands = app.world_mut().commands();
        let entity = EntityBuilder::new()
            .energy(500.0)
            .volume(2.0)
            .flow(Vec2::new(4.0, 0.0), 0.05)
            .ambient(0.0, 1000.0)
            .irradiance(5.0, 0.3)
            .with_organ_manifest(manifest)
            .with_metabolic_graph_inferred(400.0, 280.0)
            .spawn(&mut commands);
        drop(commands);
        app.update();
        let e = app.world().entity(entity);
        assert!(e.contains::<BaseEnergy>());
        assert!(e.contains::<SpatialVolume>());
        assert!(e.contains::<FlowVector>());
        assert!(e.contains::<AmbientPressure>());
        assert!(e.contains::<MetabolicGraph>());
        assert!(e.contains::<MorphogenesisShapeParams>());
        assert!(e.contains::<IrradianceReceiver>());
    }
}
