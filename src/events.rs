use bevy::prelude::*;

use crate::blueprint::recipes::EffectRecipe;
use crate::entities::{InjectorConfig, PhysicsConfig};
use crate::layers::{AbilityTarget, Faction, MatterState};

/// Jugador eligió slot QWER (puede armar targeting).
///
/// - **Emite:** `grimoire_slot_selection_system` (`Phase::Input`, `InputChannelSet::SimulationRest`).
/// - **Consume:** ningún sistema registrado en el pipeline actual (observabilidad / UI futura).
#[derive(Event, Debug, Clone, Copy)]
pub struct AbilitySelectionEvent {
    pub caster: Entity,
    pub slot_index: usize,
}

/// Cast confirmado (NoTarget, punto, dirección). Emitido como señal de log/telemetría.
///
/// - **Emite:** `grimoire_channeling_start_system` / `ability_point_target_pick_system` /
///   `enqueue_grimoire_cast_intent` (`Phase::Input`, `InputChannelSet::SimulationRest`);
///   `channeling_grimoire_emit_system` (`Phase::ThermodynamicLayer`).
/// - **Consume:** ningún sistema registrado en el pipeline actual (telemetría / extensiones futuras).
/// - **Nota:** la ejecución real del hechizo pasa por [`GrimoireProjectileCastPending`] /
///   [`GrimoireSelfBuffCastPending`], no por este evento.
#[derive(Event, Debug, Clone)]
pub struct AbilityCastEvent {
    pub caster: Entity,
    pub slot_index: usize,
    pub target: AbilityTarget,
}

/// Intención de proyectil desde grimorio (sin spawn ni gasto de buffer).
///
/// - **Emite:** `grimoire_channeling_start_system` (`Phase::Input`, `InputChannelSet::SimulationRest`).
/// - **Consume:** `grimoire_cast_resolve_system` (`Phase::ThermodynamicLayer`; es el segundo sistema
///   en la cadena `channeling_grimoire_emit_system → grimoire_cast_resolve_system`, registrada en
///   `register_grimoire_and_spatial_index`). En el mismo tick procesa **primero** todos los pending
///   de proyectil y **después** los de self-buff ([`GrimoireSelfBuffCastPending`]) en un solo sistema.
/// - **Orden cross-phase:** garantizado por `.chain()` de fases (`Phase::Input` → `Phase::ThermodynamicLayer`)
///   en `simulation::pipeline::register_simulation_pipeline`.
#[derive(Event, Debug, Clone)]
pub struct GrimoireProjectileCastPending {
    pub caster: Entity,
    pub cost_qe: f32,
    pub physics: PhysicsConfig,
    pub injector: InjectorConfig,
    pub effect: Option<EffectRecipe>,
    pub despawn_on_contact: bool,
}

/// Intención de self-buff desde grimorio; el buffer se descuenta en ThermodynamicLayer.
///
/// - **Emite:** `grimoire_channeling_start_system` (`Phase::Input`, `InputChannelSet::SimulationRest`).
/// - **Consume:** `grimoire_cast_resolve_system` (`Phase::ThermodynamicLayer`): se resuelve **después**
///   de drenar todos los [`GrimoireProjectileCastPending`] del mismo tick (orden fijo en código).
/// - **Orden cross-phase:** garantizado por `.chain()` de fases (`Phase::Input` → `Phase::ThermodynamicLayer`)
///   en `simulation::pipeline::register_simulation_pipeline`.
#[derive(Event, Debug, Clone)]
pub struct GrimoireSelfBuffCastPending {
    pub caster: Entity,
    pub cost_qe: f32,
    pub recipe: EffectRecipe,
}

/// Nuevo destino click-to-move: el consumidor recalcula `NavPath` (lazy, no cada tick).
///
/// - **Emite:** `emit_path_request_on_goal_change_system` (`Phase::Input`,
///   `InputChannelSet::PlatformWill`; solo en el perfil `full3d`, vía `Compat2d3dPlugin`).
/// - **Consume:** `pathfinding_compute_system` (`Phase::Input`, `InputChannelSet::PlatformWill`,
///   `.after(emit_path_request_on_goal_change_system)`).
/// - **Orden intra-fase:** producer → consumer dentro del mismo `InputChannelSet::PlatformWill`
///   por `.chain()` en `Compat2d3dPlugin`; no cruza fronteras de `Phase`.
#[derive(Event, Debug, Clone, Copy)]
pub struct PathRequestEvent {
    pub goal_xz: Vec2,
}

/// Colisión narrow-phase con interferencia y transferencia térmica ya aplicada.
///
/// - **Emite:** `collision_interference_system` (`Phase::AtomicLayer`; registrado en
///   `physics::register_physics_phase_systems`, último eslabón de su cadena; el `send` ocurre
///   después del `drain`/`inject` del par).
/// - **Consume:** ningún sistema registrado en el pipeline actual (telemetría / extensiones futuras).
/// - **Orden cross-phase:** cualquier lector futuro en `Phase::ChemicalLayer` o posterior está
///   automáticamente después de `Phase::AtomicLayer` por la cadena global de fases.
#[derive(Event, Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub interference: f32,
    pub transferred_qe: f32,
}

/// Cambio de estado de materia (Capa 4) por temperatura equivalente.
///
/// - **Emite:** `state_transitions_system` (`Phase::ChemicalLayer`; registrado en
///   `reactions::register_reactions_phase_systems` como 7.º sistema en la cadena, después de
///   `nutrient_depletion_system`).
/// - **Consume:** ningún sistema registrado en el pipeline actual (observabilidad / extensiones).
/// - **Orden intra-fase:** `state_transitions_system` → `catalysis_spatial_filter_system` → …
///   por `.chain()` en `reactions::register_reactions_phase_systems`.
#[derive(Event, Debug, Clone)]
pub struct PhaseTransitionEvent {
    pub entity: Entity,
    pub previous_state: MatterState,
    pub new_state: MatterState,
}

/// Petición cruda de impacto (Filtro Espacial superado).
///
/// - **Emite:** `catalysis_spatial_filter_system` (`Phase::ChemicalLayer`; cadena reactions).
/// - **Consume:** `catalysis_math_strategy_system` (`Phase::ChemicalLayer`; `.after()` por `.chain()`
///   en `reactions::register_reactions_phase_systems`).
/// - **Orden intra-fase:** `state_transitions_system` → `catalysis_spatial_filter_system` →
///   `catalysis_math_strategy_system` → …
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
/// - **Emite:** `catalysis_math_strategy_system` (`Phase::ChemicalLayer`; cadena reactions).
/// - **Consume:** `catalysis_energy_reducer_system` y `catalysis_side_effects_system`
///   (`Phase::ChemicalLayer`; ambos `.after()` por `.chain()` en `reactions::register_reactions_phase_systems`).
/// - **Orden intra-fase:** `catalysis_math_strategy_system` → `catalysis_energy_reducer_system` →
///   `catalysis_side_effects_system`.
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
/// - **Emite:** `catalysis_side_effects_system` (`Phase::ChemicalLayer`; último consumidor de
///   [`DeltaEnergyCommit`], después de `catalysis_energy_reducer_system`).
/// - **Consume:** ningún sistema registrado en el pipeline actual (telemetría / extensiones futuras).
/// - **Orden intra-fase (cadena completa en `reactions::register_reactions_phase_systems`):**
///   `state_transitions_system` → `catalysis_spatial_filter_system` →
///   `catalysis_math_strategy_system` → `catalysis_energy_reducer_system` →
///   `catalysis_side_effects_system` → `homeostasis_system`.
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
///   (`Phase::AtomicLayer`; `physics::register_physics_phase_systems`);
///   `contained_thermal_transfer_system`, `structural_constraint_system`,
///   `engine_processing_system` (`Phase::ThermodynamicLayer`);
///   `catalysis_energy_reducer_system`, `catalysis_side_effects_system`, `homeostasis_system`,
///   `metabolic_stress_death_system` (`Phase::ChemicalLayer` / `Phase::MetabolicLayer`).
/// - **Consume:** `nutrient_return_on_death_system` (`Phase::ChemicalLayer`; último en cadena reactions);
///   `worldgen_nucleus_death_notify_system` → `faction_identity_system`
///   (`Phase::MetabolicLayer`; cadena en
///   `worldgen::systems::prephysics::register_postphysics_nucleus_death_before_faction`);
///   `trophic_decomposer_system` / `nutrient_uptake::nutrient_uptake_system` también leen
///   `DeathEvent` indirectamente como guardia de consistencia.
/// - **Orden cross-phase:** productores en `Phase::ThermodynamicLayer` / `Phase::AtomicLayer` /
///   `Phase::ChemicalLayer` preceden a los consumidores en `Phase::MetabolicLayer` por la cadena
///   global de fases; el consumo de `nutrient_return_on_death_system` en `Phase::ChemicalLayer`
///   garantiza orden intra-fase vía `.chain()` en `reactions::register_reactions_phase_systems`.
#[derive(Event, Debug, Clone)]
pub struct DeathEvent {
    pub entity: Entity,
    pub cause: DeathCause,
}

/// Ruptura de enlace estructural (estrés > umbral).
///
/// - **Emite:** `structural_constraint_system` (`Phase::ThermodynamicLayer`; segundo sistema en la
///   cadena `containment_system → structural_constraint_system → contained_thermal_transfer_system
///   → …` registrada en `pipeline::register_simulation_pipeline`).
/// - **Consume:** ningún sistema registrado en el pipeline actual (observabilidad / futuro).
/// - **Nota:** el `StructuralLink` se elimina con `commands.entity(source).remove::<StructuralLink>()`
///   en el mismo tick que el evento, sin requerir consumidor ECS.
#[derive(Event, Debug, Clone)]
pub struct StructuralLinkBreakEvent {
    pub source: Entity,
    pub target: Entity,
    pub stress: f32,
}

/// Homeostasis adaptó frecuencia con costo `qe`.
///
/// - **Emite:** `homeostasis_system` (`Phase::ChemicalLayer`; penúltimo en la cadena reactions,
///   antes de `thermoregulation_cost_system` y `homeostasis_stability_check_system`, según
///   `reactions::register_reactions_phase_systems`).
/// - **Consume:** ningún sistema registrado en el pipeline actual (telemetría / observabilidad).
#[derive(Event, Debug, Clone)]
pub struct HomeostasisAdaptEvent {
    pub entity: Entity,
    pub from_hz: f32,
    pub to_hz: f32,
    pub qe_cost: f32,
}

/// Inicio de transición de estación (preset por nombre).
///
/// - **Emite:** runtime externo / comandos / tests; no hay productor ECS único registrado en el
///   loop principal (se envía desde fuera del pipeline, e.g., UI o scripts de test).
/// - **Consume:** `season_change_begin_system` (`Phase::ThermodynamicLayer`; primer sistema de la
///   cadena worldgen en `register_prephysics_worldgen_through_delta` / `register_worldgen_core_prephysics_chain`).
/// - **Orden cross-phase:** el evento llega siempre antes de que `Phase::ThermodynamicLayer` lo
///   consuma (el frame de envío o el siguiente, según cuándo se envíe en el tick).
#[derive(Event, Debug, Clone)]
pub struct SeasonChangeEvent {
    pub preset_name: String,
}

/// Observabilidad: mutaciones de núcleos y estación en worldgen.
///
/// - **Emite:**
///   - `NucleusDestroyed`: `worldgen_nucleus_death_notify_system` (`Phase::MetabolicLayer`; consume
///     [`DeathEvent`] + filtra `EnergyNucleus`; cadena `fog_of_war_provider_system →
///     worldgen_nucleus_death_notify_system → faction_identity_system` en
///     `register_postphysics_nucleus_death_before_faction`).
///   - `NucleusCreated`: `worldgen_runtime_nucleus_created_system` (`Phase::ThermodynamicLayer`;
///     cadena worldgen en `register_prephysics_worldgen_through_delta`).
///   - `SeasonApplied`: `season_transition_tick_system` (`Phase::ThermodynamicLayer`; al cerrar la
///     interpolación de temporada). `season_change_begin_system` solo arma `SeasonTransition`; no
///     emite este evento.
///   - `NucleusModified`: `worldgen_nucleus_freq_changed_notify_system` (`Phase::ThermodynamicLayer`;
///     solo fuera de `SeasonTransition` activa).
/// - **Consume:** ningún `EventReader` de gameplay registrado; tests usan `EventCursor` sobre
///   `Events<WorldgenMutationEvent>`.
///
/// `NucleusDestroyed`: `Entity` puede ser inválida tras el frame de emisión.
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
/// - **Emite:** `trophic_satiation_decay_system` (`Phase::MetabolicLayer`; primer sistema en la
///   cadena trophic, registrada en `pipeline::register_simulation_pipeline`).
/// - **Consume:** ningún sistema registrado en el pipeline actual. Reservado para D1 behavioral
///   intelligence (futuro: dispara `BehaviorMode::Forage` / `BehaviorMode::Hunt`).
#[derive(Event, Debug, Clone)]
pub struct HungerEvent {
    pub entity: Entity,
    pub deficit_qe: f32,
}

/// Threat level exceeded panic threshold (D5 sensory perception).
///
/// - **Emite:** `sensory_awareness_event_system` (`Phase::Input`; third system in the chain
///   `sensory_frequency_scan_system → sensory_threat_memory_system → sensory_awareness_event_system`,
///   registered in `pipeline::register_simulation_pipeline` `.after(InputChannelSet::PlatformWill)
///   .before(BehaviorSet::Assess)`).
/// - **Consume:** ningún sistema registrado en el pipeline actual. Reservado para D1 behavioral
///   intelligence (future: triggers panic override in `BehaviorSet::Assess`).
#[derive(Event, Debug, Clone)]
pub struct ThreatDetectedEvent {
    pub entity: Entity,
    pub threat: Entity,
    pub threat_level: f32,
}

/// Facción superó los tres umbrales de emergencia cultural (CE track).
///
/// Rising-edge only: emitido cuando `was_emergent` transiciona false → true.
/// Derivado de `OscillatorySignature` (L2) + `Homeostasis` (L12) + catálisis.
///
/// - **Emite:** `culture_observation_system` (`Phase::MetabolicLayer`).
/// - **Consume:** ningún sistema registrado en el pipeline actual (observabilidad / UI futura).
#[derive(Event, Debug, Clone, Copy)]
pub struct CultureEmergenceEvent {
    /// Facción que alcanzó emergencia cultural.
    pub faction: Faction,
    /// Índice cultural compuesto [0..1]: coherencia × síntesis × resiliencia × longevidad.
    pub culture_index: f32,
    /// Coherencia de frecuencias del grupo en el tick de emergencia [0..1].
    pub coherence: f32,
}

/// Interferencia destructiva activa entre dos facciones (anti-cultura / conflicto).
///
/// Emitido cuando `cos(Δfreq_inter_group) < CULTURE_CONFLICT_THRESHOLD`.
/// Cada observación donde el conflicto persiste emite un evento (no rising-edge).
///
/// - **Emite:** `culture_observation_system` (`Phase::MetabolicLayer`).
/// - **Consume:** ningún sistema registrado en el pipeline actual (observabilidad / futuro D1 behavioral).
#[derive(Event, Debug, Clone, Copy)]
pub struct CultureConflictEvent {
    /// Primera facción del par (por índice FACTIONS[i]).
    pub faction_a: Faction,
    /// Segunda facción del par (por índice FACTIONS[j], j > i).
    pub faction_b: Faction,
    /// Potencial de interferencia inter-grupo (cos(Δfreq) < 0).
    pub conflict_potential: f32,
}

/// Predador consumió presa con transferencia de qe.
///
/// - **Emite:** `trophic_predation_attempt_system` (`Phase::MetabolicLayer`; tercer sistema en la
///   cadena trophic: `trophic_satiation_decay_system → trophic_herbivore_forage_system →
///   trophic_predation_attempt_system → trophic_decomposer_system`).
/// - **Consume:** ningún sistema registrado en el pipeline actual (telemetría / futuro D6 social
///   para alerta de caza).
#[derive(Event, Debug, Clone)]
pub struct PreyConsumedEvent {
    pub predator: Entity,
    pub prey: Entity,
    pub qe_transferred: f32,
}

/// AC-5: Emitted when two entities enter a Nash-stable cooperation.
///
/// - **Emits:** `cooperation_evaluation_system` (`Phase::MetabolicLayer`)
/// - **Consumes:** future social/AI systems (coalition formation, shared territory)
#[derive(Event, Debug, Clone)]
pub struct AllianceProposedEvent {
    pub initiator: Entity,
    pub partner: Entity,
    /// Expected qe/tick gain for the initiator from cooperation.
    pub expected_gain: f32,
}

/// AC-5: Emitted when an entity defects from a cooperation (Nash condition broken).
///
/// - **Emits:** `cooperation_evaluation_system` (`Phase::MetabolicLayer`)
/// - **Consumes:** future social/AI systems (betrayal tracking, reputation)
#[derive(Event, Debug, Clone)]
pub struct AllianceDefectEvent {
    pub defector: Entity,
    pub abandoned: Entity,
    /// Temptation value — how much the defector expected to gain solo.
    pub defection_temptation: f32,
}
