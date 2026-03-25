//! Marcadores de jerarquía `#[require]` (Bevy 0.15), sprint G6.
//!
//! **Qué garantizan:** al hacer `spawn(AlchemicalBase | WaveEntity | MobileEntity | Champion)`,
//! Bevy inserta la cadena transitiva de componentes con sus `Default` (SSOT en cada capa vía
//! `blueprint/constants/`). Ver contratos ejecutables en `tests/require_marker_hierarchy.rs`.
//!
//! **Qué no garantizan (spawn mínimo):** control de jugador (`PlayerControlled`), grimorio
//! (`Grimoire`), adapter visual/runtime (`V6RuntimeEntity` desde [`crate::entities::EntityBuilder`]),
//! navegación, ni el resto de las 14 capas opcionales (p. ej. L6 `AmbientPressure`, L8
//! `AlchemicalInjector`, L10+). Los héroes de partida siguen naciendo por
//! [`crate::entities::archetypes::spawn_hero_layers`] / [`crate::entities::EntityBuilder`], que
//! **no** insertan estos markers hoy — no uses `With<Champion>` en gameplay hasta alinear
//! arquetipo y marker si querés un único contrato.
//!
//! Complementan el builder; no lo reemplazan.

use bevy::prelude::*;

use super::{
    AlchemicalEngine, BaseEnergy, FlowVector, MatterCoherence, MobaIdentity, OscillatorySignature,
    SpatialVolume, WillActuator,
};

/// Base alquímica: todo lo que existe en el mundo de energía (L0–L1 + presencia espacial Bevy).
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility, BaseEnergy, SpatialVolume)]
pub struct AlchemicalBase;

/// Entidad con onda (L2–L3): interferencia y flujo sobre [`AlchemicalBase`].
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(AlchemicalBase, OscillatorySignature, FlowVector)]
pub struct WaveEntity;

/// Entidad móvil: coherencia (L4), motor (L5) y actuador (L7) sobre [`WaveEntity`].
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(WaveEntity, MatterCoherence, AlchemicalEngine, WillActuator)]
pub struct MobileEntity;

/// Etiqueta de “champion” alquímico: L9 sobre [`MobileEntity`]. **No** implica héroe MOBA
/// cableado (input, grimorio, bridge 3D); ver doc del módulo.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(MobileEntity, MobaIdentity)]
pub struct Champion;
