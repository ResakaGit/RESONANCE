# Blueprint: Pipeline de Simulacion (`simulation/`)

Orquesta toda la logica de gameplay en `FixedUpdate` con timestep fijo.
6 fases encadenadas en `SystemSet`s. Cada sistema pertenece a exactamente una fase.
La fase `ThermodynamicLayer` corre en `Playing` (incluye `Warmup`); las demas solo en `Active`.

## Pipeline completo

```mermaid
sequenceDiagram
    participant Clock as SimulationClockSet
    participant Input as Phase::Input
    participant Thermo as Phase::ThermodynamicLayer
    participant Atomic as Phase::AtomicLayer
    participant Chem as Phase::ChemicalLayer
    participant Meta as Phase::MetabolicLayer
    participant Morph as Phase::MorphologicalLayer

    Clock->>Clock: advance_simulation_clock<br/>bridge_phase_tick<br/>bridge_optimizer_log

    Clock->>Input: PlatformWill then SimulationRest
    Note over Input: almanac sync, grimoire enqueue<br/>ET-2 theory of mind<br/>targeting, D1 behavior (Assess->Decide)<br/>D5 sensory perception<br/>GS-1 lockstep input gate

    Input->>Thermo: ---
    Note over Thermo: containment, structural L13<br/>resonance link L10 reset+apply<br/>engine overload L5, irradiance<br/>perception cache<br/>V7 worldgen (propagation, mat, eco)<br/>nutrient field<br/>GS-5 nucleus intake/decay

    Thermo->>Atomic: ---
    Note over Atomic: dissipation, will->velocity<br/>D3 locomotion cost<br/>movement integration<br/>spatial index rebuild<br/>tension field L11, collision<br/>AC-2 Kuramoto entrainment

    Atomic->>Chem: ---
    Note over Chem: osmosis, nutrient uptake<br/>photosynthesis, state transitions<br/>catalysis chain<br/>D4 homeostasis + thermoregulation

    Chem->>Meta: ---
    Note over Meta: growth budget, allometric growth<br/>D2 trophic, D6 social, D9 ecology<br/>culture, AC-5 cooperation, ET-5 symbiosis, ET-9 niche<br/>MG3 metabolic graph, MG6 entropy<br/>GS-5 victory, fog of war<br/>SF-4 metrics export

    Meta->>Morph: ---
    Note over Morph: MG4 shape, MG7 rugosity, MG5 albedo<br/>ET-6 epigenetic adaptation<br/>constructal body plan (axiomatic)<br/>organ growth LI3, evolution surrogate<br/>D7 reproduction (flora + fauna), abiogenesis (axiomatic)<br/>D8 morpho adaptation<br/>bridge metrics, terrain mesh<br/>water, atmosphere<br/>SF-5 checkpoint save
```

## Sub-sets dentro de fases

```mermaid
flowchart LR
    subgraph "Phase::Input"
        PW["InputChannelSet::PlatformWill"]
        SR["InputChannelSet::SimulationRest"]
        BA["BehaviorSet::Assess"]
        BD["BehaviorSet::Decide"]
        PW --> SR
        PW --> BA --> BD
    end
```

## Tipos exportados

| Tipo | Archivo | Rol |
|------|---------|-----|
| `Phase` (6 variantes) | `mod.rs` | SystemSet principal del pipeline |
| `InputChannelSet` | `mod.rs` | Orden dentro de Input |
| `BehaviorSet` | `behavior.rs` | Sub-fases D1: Assess, Decide |
| `GameState` | `states.rs` | Loading / Playing |
| `PlayState` | `states.rs` | Warmup / Active / Paused |
| `PlayerControlled` | `player_controlled.rs` | Marker del heroe local |
| `SpellMarker` | `reactions.rs` | Marker de proyectil activo |

## Sistemas clave por fase

### Input
- `platform_will_system` — input de plataforma a `WillActuator`
- `grimoire_enqueue_system` — encola habilidades
- `ability_targeting_system` — resuelve targets
- `behavior_assess_system` / `behavior_decide_system` — D1 IA
- `sensory_perception_system` — D5 campo sensorial
- `lockstep_input_gate_system` — GS-1 netcode determinista

### ThermodynamicLayer
- `containment_system` — relaciones host/contained
- `structural_runtime_system` — spring joints L13
- `resonance_link_system` — buffs/debuffs L10
- `engine_tick_system` — AlchemicalEngine L5
- `irradiance_system` — irradiancia solar
- `perception_cache_system` — PerceptionCache
- V7 worldgen: propagation, materialization, eco boundaries

### AtomicLayer
- `dissipation_system` — perdida de energia por flujo
- `will_to_velocity_system` — L7 intent a L3 velocity
- `locomotion_cost_system` — D3 costo energetico
- `movement_system` — integra posicion
- `update_spatial_index_system` — rebuild SpatialIndex
- `tension_field_system` — L11 fuerzas a distancia
- `collision_system` — colisiones + contacto
- `entrainment_system` — AC-2 Kuramoto sync

### ChemicalLayer
- `osmosis_system` — transferencia osmotica
- `nutrient_uptake_system` — absorcion de nutrientes
- `photosynthesis_system` — luz a qe
- `state_transition_system` — cambios MatterState
- `catalysis_chain_system` — reacciones
- `homeostasis_system` — D4 adaptacion de frecuencia
- `thermoregulation_system` — D4 termorregulacion

### MetabolicLayer
- `growth_budget_system` — presupuesto de crecimiento
- `trophic_system` — D2 depredacion
- `social_communication_system` — D6 manadas
- `ecology_dynamics_system` — D9 dinamica ecologica
- `cooperation_system` — AC-5 cooperacion emergente
- `victory_check_system` — GS-5 condicion de victoria
- `fog_of_war_system` — niebla de guerra
- `metrics_batch_system` — SF-4 metricas

### MorphologicalLayer
- `shape_optimization_system` — MG4 forma optima
- `rugosity_system` — MG7 rugosidad superficial
- `albedo_inference_system` — MG5 albedo
- `organ_lifecycle_system` — LI3 organos
- `evolution_surrogate_system` — mutacion/seleccion
- `reproduction_system` — D7 reproduccion
- `abiogenesis_system` — generacion espontanea
- `morpho_adaptation_system` — D8 adaptacion
- `checkpoint_save_system` — SF-5 guardado

## Dependencias

- `crate::layers` — lee/escribe las 14 capas
- `crate::blueprint::equations` — matematica pura
- `crate::blueprint::constants` — tuning
- `crate::world` — SpatialIndex, FogOfWarGrid, PerceptionCache
- `crate::worldgen` — V7 worldgen systems
- `crate::events` — contratos de eventos

## Invariantes

- Todo sistema de gameplay en `FixedUpdate`, nunca en `Update` (salvo derivacion visual)
- Todo sistema asignado a exactamente un `Phase::*`
- Producers `.before()` o `.chain()` con consumers — nunca eventos sin orden
- Determinismo: mismo input, mismo output (requisito netcode GS-1)
- `SpatialIndex` actualizado antes de queries de vecindad
