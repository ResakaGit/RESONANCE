use bevy::math::Vec2;
use bevy::prelude::*;

use crate::blueprint::recipes::EffectRecipe;
use crate::entities::{InjectorConfig, PhysicsConfig};
use crate::layers::{AbilityTarget, MatterState};

/// Jugador eligió slot QWER (puede armar targeting).
///
/// - **Emite:** `grimoire_cast_intent_system` (`Phase::Input`).
#[derive(Event, Debug, Clone, Copy)]
pub struct AbilitySelectionEvent {
    pub caster: Entity,
    pub slot_index: usize,
}

/// Cast confirmado (NoTarget, punto, dirección). Buffer se drena en PrePhysics vía pending.
///
/// - **Emite:** `enqueue_grimoire_cast_intent` / `channeling_grimoire_emit_system`.
#[derive(Event, Debug, Clone)]
pub struct AbilityCastEvent {
    pub caster: Entity,
    pub slot_index: usize,
    pub target: AbilityTarget,
}

/// Intención de proyectil desde grimorio (sin spawn ni gasto de buffer).
///
/// - **Emite:** `grimoire_cast_intent_system` (`Phase::Input`, `InputChannelSet::SimulationRest`).
/// - **Consume:** `grimoire_cast_resolve_system` (`Phase::ThermodynamicLayer`, antes de la cadena worldgen;
///   encadenado con `update_spatial_index_system` en `register_prephysics_worldgen_through_delta`).
///   En el mismo tick procesa **primero** todos los pending de proyectil y **después** los de
///   self-buff ([`GrimoireSelfBuffCastPending`]) en un solo sistema.
/// - **Orden cross-phase:** garantizado por `.chain()` de fases (`Input` → `PrePhysics`) en
///   `simulation::pipeline::register_simulation_pipeline`.
#[derive(Event, Debug, Clone)]
pub struct GrimoireProjectileCastPending {
    pub caster: Entity,
    pub cost_qe: f32,
    pub physics: PhysicsConfig,
    pub injector: InjectorConfig,
    pub effect: Option<EffectRecipe>,
    pub despawn_on_contact: bool,
}

/// Intención de self-buff desde grimorio; el buffer se descuenta en PrePhysics.
///
/// - **Emite:** `grimoire_cast_intent_system` (`Phase::Input`, `InputChannelSet::SimulationRest`).
/// - **Consume:** `grimoire_cast_resolve_system` (`Phase::ThermodynamicLayer`): se resuelve **después** de
///   drenar todos los [`GrimoireProjectileCastPending`] del mismo tick (orden fijo en código).
#[derive(Event, Debug, Clone)]
pub struct GrimoireSelfBuffCastPending {
    pub caster: Entity,
    pub cost_qe: f32,
    pub recipe: EffectRecipe,
}

/// Nuevo destino click-to-move: el consumidor recalcula `NavPath` (lazy, no cada frame).
///
/// - **Emite:** `emit_path_request_on_goal_change_system` (`FixedUpdate`, `InputChannelSet::PlatformWill`).
/// - **Consume:** `pathfinding_compute_system` (misma cadena, después del emit).
#[derive(Event, Debug, Clone, Copy)]
pub struct PathRequestEvent {
    pub goal_xz: Vec2,
}

/// Colisión narrow-phase con interferencia y transferencia térmica ya aplicada.
///
/// - **Emite:** `collision_interference_system` (`Phase::AtomicLayer`, último eslabón de la cadena
///   physics; el `send` ocurre después de los `drain`/`inject` del par).
/// - **Consume:** ningún sistema registrado en el pipeline actual (telemetría / extensiones futuras).
/// - **Orden cross-phase:** cualquier lector futuro en `Phase::ChemicalLayer` o posterior debe quedar
///   después de `Phase::AtomicLayer` vía cadena de fases.
#[derive(Event, Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub interference: f32,
    pub transferred_qe: f32,
}

/// Cambio de estado de materia (Capa 4) por temperatura equivalente.
///
/// - **Emite:** `state_transitions_system` (`Phase::ChemicalLayer`, en la cadena reactions).
/// - **Consume:** ningún sistema registrado en el pipeline actual.
/// - **Orden intra-fase:** antes de `catalysis_spatial_filter_system` por `.chain()` en
///   `reactions::register_reactions_phase_systems`.
#[derive(Event, Debug, Clone)]
pub struct PhaseTransitionEvent {
    pub entity: Entity,
    pub previous_state: MatterState,
    pub new_state: MatterState,
}

/// Petición cruda de impacto (Filtro Espacial superado).
///
/// - **Emite:** `catalysis_spatial_filter_system` (`Phase::ChemicalLayer`).
/// - **Consume:** `catalysis_math_strategy_system`.
#[derive(Event, Debug, Clone)]
pub struct CatalysisRequest {
    pub spell: Entity,
    pub target: Entity,
    pub caster: Option<Entity>,
    pub on_contact_effect: Option<EffectRecipe>,
    pub despawn_on_contact: bool,
}

/// Aprobación termodinámica (Estrategia evaluada).
///
/// - **Emite:** `catalysis_math_strategy_system` (`Phase::ChemicalLayer`).
/// - **Consume:** `catalysis_energy_reducer_system`, `catalysis_side_effects_system`.
#[derive(Event, Debug, Clone)]
pub struct DeltaEnergyCommit {
    pub spell: Entity,
    pub target: Entity,
    pub caster: Option<Entity>,
    pub result_qe: f32,
    pub interference: f32,
    pub positive_freq_delta: Option<f32>,
    pub bond_weakening_factor: Option<f32>,
    pub on_contact_effect: Option<EffectRecipe>,
    pub despawn_on_contact: bool,
}

/// Resolución energética hechizo ↔ objetivo (catálisis).
///
/// - **Emite:** `catalysis_side_effects_system` (`Phase::ChemicalLayer`, después de
///   `catalysis_energy_reducer_system`).
/// - **Consume:** ningún sistema registrado en el pipeline actual (telemetría/extensiones).
/// - **Orden intra-fase:** `state_transitions_system -> catalysis_spatial_filter_system ->
///   catalysis_math_strategy_system -> catalysis_energy_reducer_system ->
///   catalysis_side_effects_system`.
#[derive(Event, Debug, Clone)]
pub struct CatalysisEvent {
    pub caster: Option<Entity>,
    pub target: Entity,
    pub spell: Entity,
    pub interference: f32,
    pub applied_qe: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    Dissipation,
    Destruction,
    Annihilation,
    StructuralCollapse,
    Overload,
    Predation,
}

/// Entidad alcanzó `qe` mínimo vía [`crate::layers::EnergyOps::drain`].
///
/// - **Emite (no exhaustivo):** `dissipation_system`, `collision_interference_system`
///   (`Phase::AtomicLayer`); `contained_thermal_transfer_system`, `structural_constraint_system`,
///   `engine_processing_system` (`Phase::ThermodynamicLayer`); `catalysis_energy_reducer_system`,
///   `catalysis_side_effects_system`, `homeostasis_system` (`Phase::ChemicalLayer`).
/// - **Consume:** `worldgen_nucleus_death_notify_system` luego `faction_identity_system`
///   (`Phase::MetabolicLayer`, cadena en `worldgen::systems::prephysics::register_postphysics_nucleus_death_before_faction`);
///   tests y utilidades pueden leer vía `EventReader` / `EventCursor`.
/// - **Orden cross-phase:** productores ≤ `Phase::ChemicalLayer` preceden a PostPhysics por cadena de fases.
#[derive(Event, Debug, Clone)]
pub struct DeathEvent {
    pub entity: Entity,
    pub cause: DeathCause,
}

/// Ruptura de enlace estructural (estrés > umbral).
///
/// - **Emite:** `structural_constraint_system` (`Phase::ThermodynamicLayer`, cadena principal tras
///   `containment_system`).
/// - **Consume:** ningún sistema registrado en el pipeline actual (observabilidad / futuro).
#[derive(Event, Debug, Clone)]
pub struct StructuralLinkBreakEvent {
    pub source: Entity,
    pub target: Entity,
    pub stress: f32,
}

/// Homeostasis adaptó frecuencia con costo `qe`.
///
/// - **Emite:** `homeostasis_system` (`Phase::ChemicalLayer`, último en la cadena reactions).
/// - **Consume:** ningún sistema registrado en el pipeline actual.
#[derive(Event, Debug, Clone)]
pub struct HomeostasisAdaptEvent {
    pub entity: Entity,
    pub from_hz: f32,
    pub to_hz: f32,
    pub qe_cost: f32,
}

/// Inicio de transición de estación (preset por nombre).
///
/// - **Emite:** runtime / comandos / tests (`SeasonChangeEvent`); no hay productor ECS único en el
///   loop principal.
/// - **Consume:** `season_change_begin_system` (`Phase::ThermodynamicLayer`, cadena worldgen en
///   `register_prephysics_worldgen_through_delta`).
#[derive(Event, Debug, Clone)]
pub struct SeasonChangeEvent {
    pub preset_name: String,
}

/// Observabilidad: mutaciones de núcleos y estación en worldgen.
///
/// - **Emite (`Phase::ThermodynamicLayer`, cadena worldgen):**
///   - `worldgen_nucleus_death_notify_system` → `NucleusDestroyed` (desde `DeathEvent` + `EnergyNucleus`);
///   - `worldgen_runtime_nucleus_created_system` → `NucleusCreated`;
///   - `season_transition_tick_system` → `SeasonApplied` al cerrar la transición;
///   - `worldgen_nucleus_freq_changed_notify_system` → `NucleusModified` (fuera de `SeasonTransition` activa).
/// - **Relacionado (no emite este evento):** `season_change_begin_system` solo lee
///   [`SeasonChangeEvent`] y arma `SeasonTransition`; el `SeasonApplied` lo envía
///   `season_transition_tick_system` al terminar los ticks de interpolación.
/// - **Consume:** ningún `EventReader` de gameplay registrado; tests usan `EventCursor` sobre
///   `Events<WorldgenMutationEvent>`.
///
/// `NucleusDestroyed` se correlaciona con `DeathEvent` + `EnergyNucleus`. `Entity` puede ser
/// inválida tras el frame.
#[derive(Event, Debug, Clone)]
pub enum WorldgenMutationEvent {
    NucleusDestroyed {
        entity: Entity,
        position: Vec2,
    },
    NucleusCreated {
        entity: Entity,
        position: Vec2,
    },
    NucleusModified {
        entity: Entity,
        old_freq: f32,
        new_freq: f32,
    },
    SeasonApplied {
        preset_name: String,
    },
}

/// Entidad con hambre (saciedad bajo umbral).
///
/// - **Emite:** `trophic_satiation_decay_system` (`Phase::MetabolicLayer`).
/// - **Consume:** D1 behavioral intelligence (futuro: dispara BehaviorMode::Forage/Hunt).
#[derive(Event, Debug, Clone)]
pub struct HungerEvent {
    pub entity: Entity,
    pub deficit_qe: f32,
}

/// Threat level exceeded panic threshold (D5 sensory perception).
///
/// - **Emite:** `sensory_awareness_event_system` (`Phase::Input`, after scan + memory).
/// - **Consume:** D1 behavioral intelligence (future: triggers panic override).
#[derive(Event, Debug, Clone)]
pub struct ThreatDetectedEvent {
    pub entity: Entity,
    pub threat: Entity,
    pub threat_level: f32,
}

/// Predador consumió presa con transferencia de qe.
///
/// - **Emite:** `trophic_predation_attempt_system` (`Phase::MetabolicLayer`).
/// - **Consume:** telemetría / futuro D6 social (alerta de caza).
#[derive(Event, Debug, Clone)]
pub struct PreyConsumedEvent {
    pub predator: Entity,
    pub prey: Entity,
    pub qe_transferred: f32,
}
