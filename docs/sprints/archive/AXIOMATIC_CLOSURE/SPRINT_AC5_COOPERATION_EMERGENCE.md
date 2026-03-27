# Sprint AC-5 — Cooperation Emergence (Game Theory)

**Módulo:** `src/simulation/` (nuevo `cooperation.rs`), `src/events.rs`, `src/blueprint/equations/emergence/symbiosis.rs`
**Tipo:** Ecuaciones puras (extensión) + sistema de evaluación + 2 eventos nuevos
**Eje axiomático:** Axioma 3 consecuencia — cooperation emerges when E[cooperate] > E[compete]
**Estado:** 🔒 Requiere AC-1
**Oleada:** D

---

## Contexto: qué ya existe

**Lo que SÍ existe:**

- `blueprint/equations/emergence/symbiosis.rs`:
  - `is_symbiosis_stable(a_with_b, a_without_b, b_with_a, b_without_a)` → bool (Nash check)
  - `coevolution_pressure(extraction, resistance)` → f32
  - `mutualism_benefit(base, factor)` y `parasitism_drain(base, rate)` → escalares

- `simulation/metabolic/social_communication.rs` — `pack_formation_system`:
  - Los packs se forman por facción y proximidad. No evalúan si cooperar es rentable.
  - `PackMembership`, `PackRole { Leader, Follower, Scout }` existen.

- `blueprint/equations/energy_competition/extraction.rs` — `ExtractionContext`:
  - `available`, `pool_ratio`, `n_siblings`, `total_fitness` permiten estimar extracción.

**Lo que NO existe:**

1. Estimación de `extraction_if_in_group` vs `extraction_if_solo` para una entidad.
2. Sistema que compare las dos y evalúe si la condición de cooperación se cumple.
3. Lógica de deserción endógena cuando la condición deja de cumplirse.
4. Eventos `AllianceProposedEvent`, `AllianceDefectEvent`.

---

## Objetivo

Implementar la condición axiomática de cooperación como sistema activo:

```
cooperate(A, B) when:
    E[ΔE_A | cooperate with B] > E[ΔE_A | compete alone]   AND
    E[ΔE_B | cooperate with A] > E[ΔE_B | compete alone]
```

El resultado no son reglas sino consecuencias físicas: si la condición se cumple,
los dos emiten señales de alianza. Si deja de cumplirse, emiten señal de deserción.
El pack system existente acepta/rechaza en base a esas señales.

---

## Responsabilidades

### AC-5A: Ecuaciones puras — estimación de extracción

```rust
// src/blueprint/equations/emergence/symbiosis.rs — agregar

/// Estima la extracción esperada para una entidad en un pool con n_competitors.
/// Usa el modelo de extracción proporcional como baseline neutral.
/// Simplified: available / (n_competitors + 1.0)
pub fn extraction_estimate_in_group(
    available: f32,
    own_fitness: f32,
    group_total_fitness: f32,
    n_members: f32,
) -> f32 {
    if group_total_fitness <= 0.0 { return 0.0; }
    available * (own_fitness / group_total_fitness).min(1.0 / n_members.max(1.0))
}

/// Estima la extracción en modo solitario (competición directa 1-a-1).
pub fn extraction_estimate_solo(
    available: f32,
    own_fitness: f32,
    competitor_fitness: f32,
) -> f32 {
    if own_fitness + competitor_fitness <= 0.0 { return 0.0; }
    available * (own_fitness / (own_fitness + competitor_fitness))
}

/// Retorna true si cooperar es Nash-stable para ambas partes.
/// Ya existe is_symbiosis_stable() — esta función la llama con los estimados.
pub fn cooperation_is_beneficial(
    available_pool: f32,
    fitness_a: f32, fitness_b: f32,
    group_total_fitness_with_both: f32,
    group_size_with_both: f32,
) -> bool {
    let a_in_group = extraction_estimate_in_group(
        available_pool, fitness_a, group_total_fitness_with_both, group_size_with_both,
    );
    let b_in_group = extraction_estimate_in_group(
        available_pool, fitness_b, group_total_fitness_with_both, group_size_with_both,
    );
    let a_solo = extraction_estimate_solo(available_pool, fitness_a, fitness_b);
    let b_solo = extraction_estimate_solo(available_pool, fitness_b, fitness_a);

    is_symbiosis_stable(a_in_group, a_solo, b_in_group, b_solo)
}
```

### AC-5B: Eventos nuevos

```rust
// src/events.rs — agregar (sigue el patrón de los 15 eventos existentes)

/// Emitido cuando dos entidades evalúan que cooperar es energéticamente beneficioso.
/// No forma la alianza directamente — el pack_formation_system la acepta o rechaza
/// según compatibilidad de facción y otros filtros.
#[derive(Event, Debug, Clone)]
pub struct AllianceProposedEvent {
    pub initiator: Entity,
    pub target: Entity,
    pub expected_gain_delta: f32,   // cuánto gana initiator vs competir solo
}

/// Emitido cuando una entidad evalúa que ya no le conviene estar en alianza.
#[derive(Event, Debug, Clone)]
pub struct AllianceDefectEvent {
    pub defector: Entity,
    pub group_id: u32,
    pub ticks_since_unprofitable: u32,
}
```

### AC-5C: Sistema de evaluación

```rust
// src/simulation/cooperation.rs  (nuevo)

use bevy::prelude::*;
use crate::layers::{BaseEnergy, OscillatorySignature};
use crate::simulation::metabolic::social_communication::{PackMembership, PackRole};
use crate::blueprint::equations::emergence::symbiosis as symbiosis_eq;
use crate::simulation::mod::Phase;

/// Evalúa la condición de cooperación por pares de entidades adyacentes.
/// Emite AllianceProposedEvent si cooperar > competir para ambas.
/// Emite AllianceDefectEvent si la condición deja de cumplirse por N ticks.
/// Phase::MetabolicLayer, after pack_formation_system, before pack_cohesion.
pub fn cooperation_evaluation_system(
    candidates: Query<(Entity, &Transform, &BaseEnergy, Option<&PackMembership>)>,
    all_entities: Query<(Entity, &Transform, &BaseEnergy, Option<&PackMembership>)>,
    spatial: Res<SpatialIndex>,
    config: Res<CooperationConfig>,
    mut alliance_events: EventWriter<AllianceProposedEvent>,
    mut defect_events: EventWriter<AllianceDefectEvent>,
) {
    // Para cada entidad sin pack (o en pack pequeño):
    //   Query vecinos dentro de COOPERATION_EVAL_RADIUS
    //   Para el pool más cercano que ambos están disputando:
    //     Evaluar cooperation_is_beneficial()
    //   Si beneficial → emit AllianceProposedEvent
    //   Si ya en pack y beneficial < 0 por N ticks → emit AllianceDefectEvent
}
```

**Nota de implementación:**
El sistema no necesita persistir el historial de N ticks directamente en el componente.
Se puede usar un `Local<HashMap<Entity, u32>>` para contar ticks de deserción
(dado que es estado de sistema, no de entidad). Esto es aceptable para este sistema
de evaluación — no es un hot path (no es O(n²) descontrolado porque usa spatial query).

### AC-5D: Config

```rust
// Resource para configurar umbrales
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct CooperationConfig {
    pub eval_radius: f32,            // radio para buscar candidatos de alianza
    pub defect_ticks_threshold: u32, // ticks de pérdida antes de desertar
    pub min_pool_size: f32,          // pool mínimo para que valga evaluar
}

impl Default for CooperationConfig {
    fn default() -> Self {
        Self {
            eval_radius: 12.0,
            defect_ticks_threshold: 10,
            min_pool_size: 20.0,
        }
    }
}
```

---

## No hace

- No fuerza alianzas — emite propuestas. El pack_formation_system existente mantiene
  el control de quién acepta.
- No implementa memoria de traición — ese es un sistema de reputación futuro.
- No evalúa groups de >2 (cooperación n-aria) — pares primero, grupos después.
- No requiere `HashMap` en el query de spatial — usa `SpatialIndex::query_radius`.

---

## Lo que emerge

Con AC-1 + AC-5:

- **Alianzas entre distintas bandas frecuenciales:** Si un Terra y un Aqua están
  disputando el mismo pool, y juntos extraen más que solos, forman alianza — aunque
  con fricción de interferencia (AC-1). La alianza emerge cuando el beneficio de
  coordinación supera la pérdida por desincronización.

- **Deserción endógena:** Si el pool se agota (Axioma 2: existence is temporary), la
  condición Nash deja de cumplirse. Los aliados se disuelven sin que ningún sistema
  "decida" separarse. La deserción es física.

- **Tamaño óptimo de grupo:** Un grupo de 10 extrae menos per-capita que uno de 3
  sobre el mismo pool (Axioma 5: conservation). El sistema de cooperación evita grupos
  sobredimensionados — se forman hasta el punto de equilibrio Nash.

---

## Criterios de aceptación

### AC-5A (Ecuaciones)

```
extraction_estimate_in_group(100.0, 1.0, 2.0, 2.0) → 50.0  (mismo fitness, 2 miembros)
extraction_estimate_solo(100.0, 1.0, 1.0)           → 50.0  (fitness igual)
extraction_estimate_solo(100.0, 2.0, 1.0)           → 66.7  (más fit)

cooperation_is_beneficial con:
    available=100, fitness_a=1, fitness_b=1, group_total=2, size=2
    → depende de is_symbiosis_stable: 50 vs 50 → stable (no diferencia)

Mutualism case: si group extraction es mejor que solo (pool con sinergia):
    cooperation_is_beneficial → true
```

### AC-5C (Sistema)

Test (MinimalPlugins + AC-1):
- Dos entidades comparten pool escaso: si `cooperation_is_beneficial()` → `AllianceProposedEvent` emitido.
- Pack donde la condición deja de cumplirse N ticks consecutivos → `AllianceDefectEvent` emitido.
- Tres entidades compiten un pool pequeño: la tercera no entra al pack si la condición es negativa.

### General

- `cargo test --lib` sin regresión.
- `AllianceProposedEvent` y `AllianceDefectEvent` registrados en bootstrap.
- `CooperationConfig` registrado como `Resource`.

---

## Dependencias

- AC-1 — interference modula extracción real (sin esto, los estimados son demasiado simples)
- `events.rs` — registro de nuevos eventos
- `blueprint/equations/emergence/symbiosis.rs` — `is_symbiosis_stable()` (ya existe)
- `simulation/metabolic/social_communication.rs` — pack_formation_system (ordering)
- `world/mod.rs` — `SpatialIndex`

---

## Referencias

- `src/blueprint/equations/emergence/symbiosis.rs` — `is_symbiosis_stable()`, `coevolution_pressure()`
- `src/simulation/metabolic/social_communication.rs` — sistema de packs existente
- `docs/design/AXIOMATIC_CLOSURE.md §3 Tier 3` — Cooperation game theory design
- Axioma 3: "Cooperation emerges when E[ΔE_A | cooperate] > E[ΔE_A | compete]"
- Axioma 3: "Neither cooperation nor competition is moral. Both are physics."
