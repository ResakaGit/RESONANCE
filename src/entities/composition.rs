use bevy::prelude::*;

use crate::blueprint::ElementId;
use crate::layers::{
    AlchemicalEngine, AlchemicalInjector, AmbientPressure, BaseEnergy, FlowVector, MatterCoherence,
    MatterState, ModifiedField, OscillatorySignature, ResonanceLink, SpatialVolume,
};

#[derive(Clone, Debug)]
pub struct PhysicsConfig {
    pub pos: Vec2,
    pub qe: f32,
    pub radius: f32,
    /// Fuente de verdad V3 para Capa 2 (Resonancia).
    /// La frecuencia real se deriva en runtime desde `AlchemicalAlmanac`.
    pub element_id: ElementId,
    pub velocity: Vec2,
    pub dissipation: f32,
}

impl PhysicsConfig {
    pub fn spawn_components(
        self,
    ) -> (
        Transform,
        Visibility,
        BaseEnergy,
        SpatialVolume,
        OscillatorySignature,
        ElementId,
        FlowVector,
    ) {
        (
            Transform::from_translation(Vec3::new(self.pos.x, self.pos.y, 0.0)),
            Visibility::default(),
            BaseEnergy::new(self.qe),
            SpatialVolume::new(self.radius),
            // Inicializamos con un valor neutro; luego `derive_frequency_from_element_id_system`
            // setea la frecuencia desde `ElementId`.
            OscillatorySignature::new(0.0, 0.0),
            self.element_id,
            FlowVector::new(self.velocity, self.dissipation),
        )
    }
}

pub struct MatterConfig {
    pub state: MatterState,
    pub bond_energy: f32,
    pub conductivity: f32,
}

impl MatterConfig {
    pub fn spawn_component(self) -> (MatterCoherence,) {
        (MatterCoherence::new(
            self.state,
            self.bond_energy,
            self.conductivity,
        ),)
    }
}

pub struct EngineConfig {
    pub max_buffer: f32,
    pub input_valve: f32,
    pub output_valve: f32,
    pub initial_buffer: f32,
}

impl EngineConfig {
    pub fn spawn_component(self) -> (AlchemicalEngine,) {
        (AlchemicalEngine::new(
            self.max_buffer,
            self.input_valve,
            self.output_valve,
            self.initial_buffer,
        ),)
    }
}

pub struct PressureConfig {
    pub delta_qe: f32,
    pub viscosity: f32,
}

impl PressureConfig {
    pub fn spawn_component(self) -> (AmbientPressure,) {
        (AmbientPressure::new(self.delta_qe, self.viscosity),)
    }
}

#[derive(Clone, Debug)]
pub struct InjectorConfig {
    pub projected_qe: f32,
    pub forced_frequency: f32,
    pub influence_radius: f32,
}

impl InjectorConfig {
    pub fn spawn_component(self) -> (AlchemicalInjector,) {
        (AlchemicalInjector::new(
            self.projected_qe,
            self.forced_frequency,
            self.influence_radius,
        ),)
    }
}

pub struct EffectConfig {
    pub target: Entity,
    pub modified_field: ModifiedField,
    pub magnitude: f32,
    pub fuel_qe: f32,
    pub dissipation_rate: f32,
}

impl EffectConfig {
    pub fn spawn_components(self) -> (BaseEnergy, FlowVector, ResonanceLink) {
        (
            BaseEnergy::new(self.fuel_qe),
            FlowVector::new(Vec2::ZERO, self.dissipation_rate),
            ResonanceLink {
                target: self.target,
                modified_field: self.modified_field,
                magnitude: self.magnitude,
            },
        )
    }
}
