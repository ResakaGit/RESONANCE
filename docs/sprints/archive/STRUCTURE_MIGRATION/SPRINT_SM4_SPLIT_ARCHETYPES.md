# Sprint SM-4 — Split entities/archetypes

**Módulo:** `src/entities/`
**Tipo:** Refactor estructural puro. Partir `archetypes.rs` (919 LOC) por categoría de entidad.
**Onda:** A — Paralelo con SM-3.
**Estado:** ⏳ Pendiente

## Objetivo

Partir `entities/archetypes.rs` (919 LOC) en archivos por categoría: flora, fauna, terrain, projectiles. Cada sprint MG-8 y cada demo nueva añade spawn functions; el archivo solo crece. Separar por categoría permite que múltiples tracks trabajen en paralelo sin merge conflicts.

## Diagnóstico

| Archivo | LOC | Contenido |
|---------|-----|-----------|
| `entities/archetypes.rs` | **919** | Todas las `spawn_*` functions: flora, fauna, terrain, demos, projectiles |

## Estructura actual

```
entities/
├── mod.rs
├── builder.rs           ← EntityBuilder API (264 LOC, bien)
├── archetypes.rs        ← 919 LOC (ALL spawn functions)
├── composition.rs
├── lifecycle_observers.rs
└── seed_templates.rs
```

## Estructura objetivo

```
entities/
├── mod.rs               ← actualizar re-exports
├── builder.rs           ← sin cambio
├── composition.rs       ← sin cambio
├── lifecycle_observers.rs ← sin cambio
├── seed_templates.rs    ← sin cambio
└── archetypes/          ← NEW: split de archetypes.rs
    ├── mod.rs            ← re-exports de todo pub
    ├── flora.rs          ← spawn_rose, spawn_tree, spawn_grass, etc.
    ├── fauna.rs          ← spawn_creature, spawn_predator, etc.
    ├── terrain.rs        ← spawn_terrain_cell, spawn_rock, etc.
    └── demo.rs           ← spawn_demo_*, spawn_test_*, proving grounds
```

## Pasos de implementación

### SM-4A: Analizar y categorizar

1. Leer `archetypes.rs` completo.
2. Categorizar cada `pub fn spawn_*` en: flora, fauna, terrain, demo/test.
3. Documentar dependencias entre funciones (helpers compartidos).
4. Helpers compartidos → quedan en `archetypes/mod.rs` o en un `archetypes/common.rs`.

### SM-4B: Partir

1. Crear `entities/archetypes/`.
2. Mover funciones por categoría a cada archivo.
3. Helpers compartidos en `mod.rs` o `common.rs`.
4. Crear `archetypes/mod.rs` con `pub use` de todo.
5. Actualizar `entities/mod.rs`: `pub mod archetypes` (mismo nombre, ahora es directorio).
6. `cargo test --lib` → verde.

### SM-4C: Verificar imports

1. Buscar `use crate::entities::archetypes::spawn_*` en todo el codebase.
2. Verificar que re-exports mantienen paths existentes.
3. Actualizar si es necesario.

## Tácticas

- **Categorización por OrganManifest/tipo.** Flora = entidades con OrganManifest de plantas. Fauna = con manifest animal. Terrain = sin manifest. Demo = funciones `spawn_demo_*` y `spawn_test_*`.
- **Helpers compartidos como funciones privadas en mod.rs.** Si hay un `fn common_setup()` usado por flora y fauna, va en `mod.rs` como `pub(super)`.
- **Cada archivo ~200-300 LOC.** 919 / 4 = ~230 por archivo. Si una categoría es muy pequeña (ej. projectiles), fusionar con demo.

## NO hace

- No modifica `EntityBuilder` ni su API.
- No añade nuevos archetypes.
- No cambia signatures de funciones `spawn_*`.
- No toca `composition.rs`, `lifecycle_observers.rs`, `seed_templates.rs`.

## Criterios de aceptación

- `cargo test --lib` pasa sin regresión.
- Ningún archivo en `entities/archetypes/` supera 400 LOC.
- Todos los `spawn_*` accesibles via `crate::entities::archetypes::spawn_*` (re-export).
- `RESONANCE_MAP=demo_arena cargo run` funciona sin cambios.

## Referencias

- `src/entities/mod.rs` — módulo raíz
- `src/entities/builder.rs` — EntityBuilder (no se toca)
- `docs/arquitectura/blueprint_entities.md` — contrato de entidades
