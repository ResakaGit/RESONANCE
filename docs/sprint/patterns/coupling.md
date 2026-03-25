# Reducción de Acoplamiento

Estrategias para que los 9 dominios nuevos permanezcan ortogonales entre sí y con el código existente.

---

## Principio: Acoplamiento Solo Via Capas y Eventos

```
BIEN: System A lee L0 (BaseEnergy) → System B lee L0
  → Ambos conocen L0, no se conocen entre sí

MAL: System A importa módulo de System B
  → Acoplamiento directo, no composable
```

**Regla absoluta**: Un system de dominio X NUNCA importa un system de dominio Y. Solo importa:
- `crate::layers::*` (componentes)
- `crate::blueprint::equations` (math pura)
- `crate::blueprint::constants` (tuning)
- `crate::events` (eventos)
- `bevy::prelude::*`

---

## Matriz de Dependencias por Dominio

```
            D1   D2   D3   D4   D5   D6   D7   D8   D9
D1 Behavior  —    ←    ←    ←    ←    ←    .    .    .
D2 Trophic  evt   —    .    .    ←    .    .    .    ←
D3 Locomot   .    .    —    .    .    .    .    .    .
D4 Homeo     .    .    .    —    .    .    .    ←    .
D5 Sensory   .    .    .    .    —    .    .    .    .
D6 Social   evt   .    .    .   evt   —    .    .    .
D7 Repro     .   evt   .    .    .    .    —    .    evt
D8 Morpho    .    .    .   comp  .    .    .    —    .
D9 Ecology   .   evt   .    .    .    .   evt   .    —

Leyenda:
  ←    = lee output component de ese dominio
  evt  = comunica solo via Event
  comp = lee component que otro dominio escribe
  .    = sin dependencia
```

---

## Patrón 1: Component as Contract

Los dominios se comunican via componentes que actúan como contratos:

```rust
// D1 escribe BehaviorIntent (el "qué quiere hacer")
// D3 lee BehaviorIntent para aplicar locomotion cost
// D2 lee BehaviorIntent para saber si está cazando

// Pero D3 NO importa D1. D3 solo importa el componente:
use crate::layers::behavior::BehaviorIntent;
```

**Ubicación de componentes nuevos**: `src/layers/behavior.rs` (nuevo archivo en layers/).

---

## Patrón 2: Event as Firewall

Cross-domain effects SOLO via eventos:

```rust
// D2 (Trophic) mata a una presa → DeathEvent (existente)
// D6 (Social) detecta DeathEvent de miembro de manada → reacciona
// D9 (Ecology) detecta DeathEvent → actualiza census

// Ninguno importa al otro. Todos conocen DeathEvent.
```

**Eventos nuevos necesarios**:

| Evento | Productor | Consumidores |
|--------|-----------|--------------|
| `HungerEvent` | D1 (Behavior) | D2 (Trophic: trigger foraging) |
| `ThreatDetectedEvent` | D5 (Sensory) | D1 (Behavior: trigger flee) |
| `PreyConsumedEvent` | D2 (Trophic) | D9 (Ecology: census), D6 (Social: pack share) |
| `OffspringSpawnedEvent` | D7 (Repro) | D9 (Ecology: census) |
| `PackFormedEvent` | D6 (Social) | D1 (Behavior: pack coordination mode) |
| `NicheShiftEvent` | D9 (Ecology) | D8 (Morpho: adaptive pressure) |

Total: 6 eventos nuevos en `events.rs`.

---

## Patrón 3: Thin Component Layer

Componentes nuevos viven en `src/layers/` — NUNCA en `src/simulation/`:

```
src/layers/
  behavior.rs     ← BehaviorIntent, BehaviorMode, BehavioralAgent (marker)
  sensory.rs      ← SensoryAwareness, ThreatMemory (nuevo, o extender existente)
  trophic.rs      ← TrophicRole, TrophicState (nuevo)
  social.rs       ← PackMembership, DominanceRank (nuevo)
  thermoregulation.rs ← ThermalBalance (nuevo, o usar L12 Homeostasis)
```

Systems en `src/simulation/` solo IMPORTAN estos componentes. No los definen.

---

## Patrón 4: Vertical Slice per Domain

Cada dominio es un vertical slice independiente:

```
D2 Trophic & Predation:
  src/layers/trophic.rs              → componentes (TrophicRole, TrophicState)
  src/blueprint/equations/trophic/   → ecuaciones puras
  src/blueprint/constants/trophic.rs → constantes
  src/simulation/trophic/            → systems
    mod.rs
    intake.rs
    predation_filter.rs
    capture.rs
  tests/trophic_integration.rs       → tests de integración
```

**Regla**: Se puede borrar un dominio entero sin romper compilación. Solo se pierden las features de ese dominio.

---

## Patrón 5: Feature Toggle por Marker (No Feature Flags)

Si un mapa no tiene fauna, los systems de fauna no corren:

```rust
// En pipeline registration:
app.add_systems(FixedUpdate, (
    behavior_decision_system,
    behavior_transition_system,
    behavior_will_bridge_system,
).chain()
 .in_set(Phase::Input)
 .run_if(any_with_component::<BehavioralAgent>));
```

**No hay feature flags**. No hay `#[cfg]`. La presencia o ausencia de fauna en el mundo es suficiente.

---

## Patrón 6: Equation Namespace Isolation

Ecuaciones de distintos dominios NO se llaman entre sí:

```
blueprint/equations/
  trophic/mod.rs     → solo funciones troficas
  behavior/mod.rs    → solo funciones de decisión
  locomotion/mod.rs  → solo funciones de movimiento

// BIEN: system llama equations::trophic::intake_rate(...)
// MAL:  equations::trophic::intake_rate() llama equations::locomotion::cost()
```

Si dos dominios necesitan la misma math, se extrae a `equations/shared/` o se duplica (prefiere duplicar 3 líneas a crear acoplamiento).

---

## Test de Ortogonalidad

Para verificar que un dominio es ortogonal:

```
1. ¿Puedo borrar todo el directorio sin errores de compilación? (solo warnings)
2. ¿El resto de la simulación funciona sin este dominio?
3. ¿Las ecuaciones de este dominio tienen tests que pasan sin Bevy?
4. ¿Los events de este dominio tienen 0 consumidores obligatorios?
```

Si la respuesta es "sí" a todo → dominio ortogonal. Si no → hay acoplamiento a eliminar.
