# Sprint SM-1 — Split worldgen/materialization + inference

**Módulo:** `src/worldgen/`
**Tipo:** Refactor estructural puro. Mover código, crear subdirectorios, actualizar re-exports.
**Onda:** 0 — Paralelo con SM-2.
**Estado:** ⏳ Pendiente

## Objetivo

Partir los 3 archivos más grandes de `worldgen/` en subdirectorios temáticos sin cambiar una sola línea de lógica. El monolito actual (22 archivos, 3 de >800 LOC) se reorganiza en dominios claros: `materialization/`, `inference/`.

## Diagnóstico

| Archivo | LOC | Problema |
|---------|-----|---------|
| `systems/materialization.rs` | **1,851** | God-file: spawn + rules + visual + terrain en un archivo |
| `shape_inference.rs` | **849** | Inferencia de forma monolítica, mezclada con helpers |
| `materialization_rules.rs` | **836** | Reglas de materialización separadas pero gigantes |
| `systems/propagation.rs` | **810** | Propagación monolítica: step + field en uno |
| `organ_inference.rs` | **636** | Inferencia de órganos, separada pero relacionada a shape |
| `visual_derivation.rs` | **624** | Derivación visual, separada pero acoplada a inferencia |

## Estructura actual

```
worldgen/
├── mod.rs
├── field_grid.rs
├── lod.rs
├── map_config.rs
├── materialization_rules.rs        ← 836 LOC
├── nucleus.rs
├── nutrient_field.rs
├── organ_inference.rs              ← 636 LOC
├── propagation.rs
├── shape_inference.rs              ← 849 LOC
├── visual_derivation.rs            ← 624 LOC
├── cell_field_snapshot/
│   └── (3 files)
└── systems/
    ├── mod.rs
    ├── materialization.rs          ← 1,851 LOC
    ├── propagation.rs              ← 810 LOC
    ├── visual.rs                   ← 758 LOC
    ├── startup.rs
    ├── terrain.rs
    ├── phenology_visual.rs
    ├── prephysics.rs
    └── performance.rs
```

## Estructura objetivo

```
worldgen/
├── mod.rs                          ← actualizar: pub mod materialization, inference
├── field_grid.rs
├── lod.rs
├── map_config.rs
├── nucleus.rs
├── nutrient_field.rs
├── propagation.rs
├── materialization/                ← NEW
│   ├── mod.rs                      ← re-exports de todo lo público
│   ├── rules.rs                    ← ← materialization_rules.rs (836, movido)
│   ├── spawn.rs                    ← ← top ~600 LOC de systems/materialization.rs
│   ├── transform.rs                ← ← mid ~600 LOC de systems/materialization.rs
│   └── visual.rs                   ← ← bottom ~650 LOC de systems/materialization.rs
├── inference/                      ← NEW
│   ├── mod.rs                      ← re-exports
│   ├── shape.rs                    ← ← shape_inference.rs (849, movido)
│   ├── organ.rs                    ← ← organ_inference.rs (636, movido)
│   └── visual_derivation.rs       ← ← visual_derivation.rs (624, movido)
├── cell_field_snapshot/
│   └── (3 files, sin cambio)
└── systems/
    ├── mod.rs                      ← actualizar: quitar materialization, redirigir
    ├── propagation/                ← NEW (si >800 LOC justifica split)
    │   ├── mod.rs
    │   ├── step.rs                 ← ← top ~400 LOC de systems/propagation.rs
    │   └── field.rs                ← ← bottom ~400 LOC de systems/propagation.rs
    ├── visual.rs                   ← sin cambio (758 LOC, umbral borderline)
    ├── startup.rs
    ├── terrain.rs
    ├── phenology_visual.rs
    ├── prephysics.rs
    └── performance.rs
```

## Pasos de implementación

### SM-1A: Crear `worldgen/materialization/`

1. Crear directorio `src/worldgen/materialization/`.
2. **Mover** `materialization_rules.rs` → `materialization/rules.rs`.
3. **Partir** `systems/materialization.rs` (1,851 LOC) en 3 archivos:
   - Identificar bloques lógicos: funciones de spawn, funciones de transformación, funciones visuales.
   - Mover cada bloque a `materialization/spawn.rs`, `transform.rs`, `visual.rs`.
   - **No renombrar funciones.** Solo mover.
4. Crear `materialization/mod.rs` con `pub use` de todo lo público.
5. Actualizar `worldgen/mod.rs`: reemplazar `pub mod materialization_rules` por `pub mod materialization`.
6. Actualizar `systems/mod.rs`: quitar `pub mod materialization`, redirigir imports.
7. `cargo test --lib` → verde.

### SM-1B: Crear `worldgen/inference/`

1. Crear directorio `src/worldgen/inference/`.
2. **Mover** `shape_inference.rs` → `inference/shape.rs`.
3. **Mover** `organ_inference.rs` → `inference/organ.rs`.
4. **Mover** `visual_derivation.rs` → `inference/visual_derivation.rs`.
5. Crear `inference/mod.rs` con `pub use` de todo lo público.
6. Actualizar `worldgen/mod.rs`: quitar los 3 `pub mod` individuales, añadir `pub mod inference`.
7. `cargo test --lib` → verde.

### SM-1C: Partir `systems/propagation.rs` (opcional)

1. Solo si >800 LOC y tiene bloques lógicos claros.
2. Crear `systems/propagation/` con `mod.rs`, `step.rs`, `field.rs`.
3. Mover funciones de step a `step.rs`, funciones de field a `field.rs`.
4. Actualizar `systems/mod.rs`.
5. `cargo test --lib` → verde.

## Tácticas

- **Un commit por archivo movido.** `git mv` preserva historia. Si hay split (systems/materialization.rs → 3 archivos), un commit por el split completo.
- **Re-exports preservan API.** Todo `pub fn` y `pub struct` que antes era accesible desde `worldgen::` sigue siéndolo tras el move.
- **Grep para encontrar imports.** Buscar `use crate::worldgen::materialization_rules` y `use crate::worldgen::shape_inference` etc. para actualizar paths.
- **No tocar lógica.** Si ves un bug al mover, NO lo arregles en este sprint. Anótalo y créale un issue.

## NO hace

- No cambia lógica, ecuaciones, ni comportamiento de ningún sistema.
- No renombra funciones, structs, ni constantes públicas.
- No añade ni quita tests (solo los mueve si están inline en los archivos partidos).
- No toca módulos fuera de `worldgen/`.

## Criterios de aceptación

- `cargo test --lib` pasa sin regresión.
- `cargo build` compila sin warnings nuevos.
- Ningún archivo en `worldgen/` supera 700 LOC.
- `worldgen/mod.rs` tiene `pub mod materialization` y `pub mod inference`.
- Todos los imports externos (`use crate::worldgen::...`) siguen funcionando via re-exports.
- `git log --follow` funciona para archivos movidos (historia preservada).

## Referencias

- `src/worldgen/mod.rs` — módulo raíz actual
- `src/worldgen/systems/mod.rs` — sistemas actuales
- `docs/sprints/MIGRATION/README.md` — track previo M1–M5 como precedente
