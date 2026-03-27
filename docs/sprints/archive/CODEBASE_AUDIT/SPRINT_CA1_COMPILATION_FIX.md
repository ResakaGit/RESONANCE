# Sprint CA-1 — Fix compilación

**Módulo:** `src/bridge/`, `src/entities/`, `src/world/demos/`, `src/plugins/`
**Tipo:** Limpieza de migración incompleta. Eliminar archivos legacy, completar exports.
**Onda:** 0 — Bloquea todo lo demás.
**Estado:** ✅ Cerrado (2026-03-25)

## Objetivo

Restaurar `cargo check` a verde. La migración SM-3 (bridge) y SM-4 (archetypes) dejó archivos viejos en el working tree coexistiendo con los nuevos directorios. Además, `competition_arena` nunca fue registrado en `world/demos/mod.rs`.

## Diagnóstico

```
$ cargo check
error[E0761]: file for module `presets` found at both
              "src/bridge/presets.rs" and "src/bridge/presets/mod.rs"

error[E0761]: file for module `archetypes` found at both
              "src/entities/archetypes.rs" and "src/entities/archetypes/mod.rs"

error[E0432]: unresolved imports
              `crate::world::COMPETITION_ARENA_SLUG`,
              `crate::world::spawn_competition_demo_startup_system`
```

| Archivo viejo (legacy) | Archivo nuevo (migrado) | LOC viejo | LOC nuevo |
|------------------------|------------------------|-----------|-----------|
| `src/bridge/presets.rs` | `src/bridge/presets/mod.rs` + `combat.rs` + `ecosystem.rs` + `physics.rs` | 35,657 bytes | 27,995 + 1,647 + 2,352 + 1,708 bytes |
| `src/entities/archetypes.rs` | `src/entities/archetypes/mod.rs` + `competition.rs` + `flora.rs` + `heroes.rs` + `morphogenesis.rs` + `world_entities.rs` | 44,009 bytes | 214 + 4,756 + 9,840 + 8,375 + 14,231 + 11,814 bytes |

## Pasos de implementación

### CA-1A: Eliminar archivos legacy de bridge

1. Verificar que `src/bridge/presets/mod.rs` re-exporta todo lo que `src/bridge/presets.rs` exportaba.
2. Diff de exports: `grep "pub " src/bridge/presets.rs` vs `grep "pub use" src/bridge/presets/mod.rs`.
3. Si cobertura completa → eliminar `src/bridge/presets.rs`.
4. `cargo check` — verificar que el error E0761 de bridge desaparece.

### CA-1B: Eliminar archivos legacy de entities

1. Verificar que `src/entities/archetypes/mod.rs` re-exporta todo lo que `src/entities/archetypes.rs` exportaba.
2. Diff de exports análogo a CA-1A.
3. Si cobertura completa → eliminar `src/entities/archetypes.rs`.
4. `cargo check` — verificar que el error E0761 de entities desaparece.

### CA-1C: Registrar competition_arena en demos

1. Editar `src/world/demos/mod.rs`:
   ```rust
   pub mod competition_arena;
   pub mod round_world_rosa;

   pub use competition_arena::{
       COMPETITION_ARENA_SLUG, spawn_competition_demo_startup_system,
   };
   pub use round_world_rosa::{ ... };  // existente
   ```
2. Verificar que `src/world/mod.rs` re-exporta desde `demos::`.
3. `cargo check` — verificar que el error E0432 desaparece.

### CA-1D: Decidir sobre `attention_gating_system`

El sistema existe en `simulation/thermodynamic/sensory.rs:363` pero no está registrado en ningún plugin.

**Opción A — Registrar:** Agregar en `pipeline.rs` dentro de `Phase::ThermodynamicLayer`:
```rust
attention_gating_system.in_set(Phase::ThermodynamicLayer)
```
Requiere que `AttentionGrid` exista como Resource (verificar).

**Opción B — Eliminar:** Si `AttentionGrid` no existe o el feature no está listo, eliminar `attention_gating_system` y `QuantumSuspension` para no acumular código muerto.

**Decisión:** Requiere input del autor. Marcar como blocked hasta decidir.

### CA-1E: Verificar errores secundarios

Tras resolver CA-1A/B/C, correr `cargo check` de nuevo. Errores que podrían aparecer:
- `SymmetryMode` no encontrado en `blueprint::equations` (importado en `layers/body_plan_layout.rs:6`)
- `inferred_world_geometry` módulo faltante (importado en `worldgen/systems/terrain_visual_mesh.rs:7`)
- `inferred_sun_direction`, `inferred_sun_intensity`, `inferred_fog_params` faltantes (importados en `simulation/metabolic/atmosphere_inference.rs`)

Estos son de tracks IWG (Inferred World Geometry) en progreso. Documentar estado y determinar si necesitan stubs o si los archivos `.rs` nuevos (untracked `??`) deben registrarse en sus respectivos `mod.rs`.

## Tácticas

- **Verificar antes de borrar.** Cada archivo viejo se compara con el nuevo antes de eliminarlo. Si hay funciones en el viejo que no migraron, extraerlas primero.
- **Un paso, un `cargo check`.** Después de cada delete/edit, validar. No acumular cambios sin verificar.
- **Git status como guía.** Los archivos marcados `D` (deleted en staging) son los legacy. Los `??` (untracked) son los nuevos. La migración consiste en hacer que el working tree refleje el staging.

## NO hace

- No modifica lógica de ningún sistema.
- No reescribe imports más allá de lo necesario para compilar.
- No toca tests (eso es CA-3).

## DoD

- `cargo check` verde (0 errores, warnings aceptables).
- Archivos legacy eliminados del working tree.
- `competition_arena` accesible vía `crate::world::COMPETITION_ARENA_SLUG`.
- Decisión documentada sobre `attention_gating_system`.

## Referencias

- `docs/sprints/STRUCTURE_MIGRATION/SPRINT_SM3_SPLIT_BRIDGE.md`
- `docs/sprints/STRUCTURE_MIGRATION/SPRINT_SM4_SPLIT_ARCHETYPES.md`
- `src/plugins/debug_plugin.rs:21` — consumer de `COMPETITION_ARENA_SLUG`
- `src/simulation/thermodynamic/sensory.rs:363` — `attention_gating_system`
