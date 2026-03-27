# Sprint G9 — Event Ordering Explicito Cross-Phase

**Tipo:** Refactor — asegurar orden determinista de eventos.
**Riesgo:** BAJO — solo agrega constraints de ordering.
**Onda:** 0 — Sin dependencias. Aplicable gradualmente.
**Estado:** Pendiente

## Objetivo

Garantizar que todo productor de eventos se ejecuta antes de su consumidor, incluso cross-phase. Actualmente, `.chain()` ordena sistemas dentro de cada `Phase`, pero no hay garantia de que un `EventWriter` en `Phase::Physics` se ejecute antes de su `EventReader` en `Phase::Reactions`.

## Estado actual en Resonance

- Pipeline usa `.chain()` dentro de cada `Phase`
- Eventos cross-phase (`CollisionEvent` en Physics → consumers en Reactions) dependen del orden de Phases en `SystemSet`
- No hay `.after()` explicito para event producers/consumers cross-phase
- 9 eventos definidos en `events.rs`

## Responsabilidades

### Paso 1 — Mapear producer → consumer para cada evento

| Evento | Producer (Phase) | Consumer(s) (Phase) | Ordering actual |
|--------|-----------------|---------------------|----------------|
| `CollisionEvent` | `collision_interference_system` (Physics) | `reactions::*` (Reactions) | Implicito via Phase order |
| `PhaseTransitionEvent` | `phase_transition_system` (Reactions) | `post::*` (PostPhysics) | Implicito via Phase order |
| `CatalysisEvent` | `catalysis_resolution_system` (Reactions) | `post::*` (PostPhysics) | Implicito via Phase order |
| `DeathEvent` | multiple | multiple | No determinista |
| `StructuralLinkBreakEvent` | `structural_runtime_system` (PrePhysics) | ? | Verificar |
| `HomeostasisAdaptEvent` | `homeostasis_adapt_system` (PrePhysics) | ? | Verificar |
| `GrimoireProjectileCastPending` | `will_input_system` (Input) | `bootstrap` (Input/PrePhysics) | Verificar |
| `SeasonChangeEvent` | external | multiple | Sin orden |
| `WorldgenMutationEvent` | worldgen systems | worldgen systems | Verificar |

### Paso 2 — Verificar que Phase ordering es suficiente

Para eventos que cruzan Phases, el orden de `SystemSet` (Input → PrePhysics → Physics → Reactions → PostPhysics) deberia ser suficiente. Verificar en `pipeline.rs` que los sets estan correctamente encadenados:

```rust
app.configure_sets(FixedUpdate, (
    Phase::Input,
    Phase::PrePhysics,
    Phase::Physics,
    Phase::Reactions,
    Phase::PostPhysics,
).chain());
```

### Paso 3 — Agregar `.after()` donde sea necesario

Para eventos producidos y consumidos dentro de la misma Phase:

```rust
// Ejemplo: si collision produce DeathEvent y death_handler tambien esta en Physics
app.add_systems(FixedUpdate, (
    collision_system,      // puede producir DeathEvent
    death_handler_system,  // consume DeathEvent
).chain().in_set(Phase::Physics));
```

### Paso 4 — Documentar event flow

Agregar comentario en `events.rs` que documente el flujo de cada evento:

```rust
/// Emitted by: collision_interference_system (Phase::Physics)
/// Consumed by: damage_resolution (Phase::Reactions)
pub struct CollisionEvent { ... }
```

## Tacticas

- **No cambiar orden de Phases.** Solo agregar `.after()` / `.chain()` donde falte.
- **Priority: DeathEvent.** Es el unico evento con productores multiples sin orden explicito.
- **Tests de order.** Agregar test que verifique que events de un frame se consumen en el mismo frame (no leak al siguiente).

## NO hace

- No cambia el orden de Phases.
- No agrega eventos nuevos.
- No modifica logica de ningun sistema.
- No cambia el tipo de los eventos.

## Criterio de aceptacion

- [ ] Todos los pares producer/consumer estan documentados
- [ ] `.chain()` o `.after()` explicito para pares dentro de la misma Phase
- [ ] Phase ordering verificado como `.chain()` en `pipeline.rs`
- [ ] Cada evento en `events.rs` tiene doc comment con producer/consumer
- [ ] `cargo check` pasa
- [ ] `cargo test` — 575+ tests pasan
- [ ] No hay eventos que leaken al siguiente frame

## Esfuerzo estimado

~1-2 horas. La mayor parte es auditar el pipeline existente y documentar.
