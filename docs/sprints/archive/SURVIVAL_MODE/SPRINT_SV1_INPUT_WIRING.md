# Sprint SV-1 — Input Wiring: InputCommand → WillActuator

**Modulo:** `src/sim_world.rs` (5 LOC change)
**Tipo:** Wiring de contrato existente.
**Estado:** ✅ Completado (2026-03-28)

---

## Contexto: que ya existe

**Lo que SÍ existe:**

- `InputCommand::MoveToward { entity_id: u64, goal: [f32; 2] }` — definido en `sim_world.rs:87`.
- `InputCommand::CastAbility { entity_id: u64, slot: u8, target: AbilityTargetCmd }` — definido en `sim_world.rs:88`.
- `SimWorld::tick(&[InputCommand])` — recibe commands, llama `apply_input()`.
- `WillActuator` (L7) — `movement_intent: Vec2` que el pipeline lee para mover la entidad.
- `WorldEntityId` — strong ID que mapea `entity_id: u64` → Bevy `Entity`.
- `IdGenerator` / `EntityLookup` resources — registran WorldEntityId → Entity.

**Lo que NO existe:**

1. `apply_input()` está vacío — es un no-op con un TODO.
2. No hay resolución de `entity_id → Bevy Entity`.
3. No hay escritura de `InputCommand → WillActuator.movement_intent`.

---

## Objetivo

Implementar `apply_input()` para que `InputCommand::MoveToward` escriba el intent
en el `WillActuator` de la entidad correspondiente. 5 LOC.

---

## Responsabilidades

### SV-1A: Entity lookup

```rust
// En SimWorld::apply_input():

fn apply_input(&mut self, commands: &[InputCommand]) {
    let world = self.app.world_mut();
    for cmd in commands {
        match cmd {
            InputCommand::MoveToward { entity_id, goal } => {
                // Resolve entity_id → Bevy Entity via query
                let mut q = world.query::<(&WorldEntityId, &mut WillActuator)>();
                for (id, mut will) in q.iter_mut(world) {
                    if id.0 as u64 == *entity_id {
                        let dx = goal[0] - /* current pos */;
                        let dy = goal[1] - /* current pos */;
                        let len = (dx * dx + dy * dy).sqrt().max(0.01);
                        will.movement_intent = Vec2::new(dx / len, dy / len);
                        break;
                    }
                }
            }
            InputCommand::CastAbility { entity_id, slot, target } => {
                // Future: route to Grimoire
                let _ = (entity_id, slot, target);
            }
        }
    }
}
```

**Nota:** El query necesita `Transform` para obtener la posición actual. Ajustar firma.

---

## NO hace

- No crea binario — eso es SV-2.
- No implementa game over — eso es SV-3.
- No modifica ningún system de simulación.
- No toca `batch/`.

---

## Dependencias

- `crate::layers::WillActuator` — target component.
- `crate::blueprint::WorldEntityId` — entity resolution.

---

## Criterios de aceptacion

### SV-1A (Input wiring)
- `SimWorld::tick(&[InputCommand::MoveToward { entity_id: 0, goal: [5.0, 5.0] }])` modifica `WillActuator` de entity 0.
- Entity sin `WorldEntityId` matching → no crash, no-op.
- Empty commands → no change.
- `CastAbility` → no crash (future implementation).

### General
- `cargo test --lib` sin regresion.
- `SimWorld` tests existentes siguen pasando.

---

## Referencias

- `src/sim_world.rs:246` — `apply_input()` actual (no-op)
- `src/layers/will.rs` — `WillActuator` API
- `src/blueprint/ids/types.rs` — `WorldEntityId`
