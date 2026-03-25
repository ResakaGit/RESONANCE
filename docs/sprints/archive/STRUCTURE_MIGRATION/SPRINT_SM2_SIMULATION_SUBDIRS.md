# Sprint SM-2 — Subdirectorios en simulation/

**Módulo:** `src/simulation/`
**Tipo:** Refactor estructural puro. Crear subdirectorios temáticos para 41 archivos planos.
**Onda:** 0 — Paralelo con SM-1.
**Estado:** ⏳ Pendiente

## Objetivo

Organizar los 41 archivos sueltos de `simulation/` en subdirectorios por dominio. El directorio plano actual no escala: cada sprint MG (3-8) añadirá más archivos, y el directorio crecerá sin estructura. La reorganización agrupa por responsabilidad termodinámica, metabólica, lifecycle y visual.

## Diagnóstico

```
simulation/                         ← 41 archivos PLANOS
├── mod.rs                          ← re-exporta todo
├── pipeline.rs                     ← scheduling (295 LOC)
├── bootstrap.rs                    ← init events + resources
├── input.rs
├── physics.rs                      ← 663 LOC
├── pre_physics.rs                  ← 517 LOC
├── dissipation.rs
├── thermal_transfer.rs
├── metabolic_stress.rs
├── photosynthesis.rs
├── nutrient_uptake.rs
├── growth_budget.rs                ← 500 LOC
├── lifecycle_inference.rs
├── evolution_surrogate.rs          ← 641 LOC
├── fog_simulation.rs
├── phenology_visual.rs
├── morphogenesis.rs                ← futuro: MG-3+ sistemas
├── regression.rs                   ← 552 LOC (tests)
├── eco_e5_simulation_tests.rs      ← 650 LOC (tests)
├── abiogenesis/
├── pathfinding/
└── reproduction/
```

**Problema:** 41 archivos sin agrupación temática. Al añadir MG-3 (`morphogenesis_metabolic.rs`), MG-4 (`morphogenesis_shape.rs`), MG-5 (`morphogenesis_color.rs`) → 44+ archivos. Insostenible.

## Estructura objetivo

```
simulation/
├── mod.rs                          ← actualizar: pub mod de subdirs
├── pipeline.rs                     ← sin cambio (scheduling global)
├── bootstrap.rs                    ← sin cambio (init global)
├── input.rs                        ← sin cambio
├── regression.rs                   ← sin cambio (tests globales)
│
├── thermodynamic/                  ← NEW: física térmica
│   ├── mod.rs
│   ├── physics.rs                  ← ← physics.rs (663)
│   ├── pre_physics.rs              ← ← pre_physics.rs (517)
│   ├── dissipation.rs              ← ← dissipation.rs
│   └── thermal_transfer.rs         ← ← thermal_transfer.rs
│
├── metabolic/                      ← NEW: metabolismo + crecimiento
│   ├── mod.rs
│   ├── photosynthesis.rs           ← ← photosynthesis.rs
│   ├── nutrient_uptake.rs          ← ← nutrient_uptake.rs
│   ├── metabolic_stress.rs         ← ← metabolic_stress.rs
│   ├── growth_budget.rs            ← ← growth_budget.rs (500)
│   └── morphogenesis.rs            ← ← morphogenesis.rs (futuro MG-3+)
│
├── lifecycle/                      ← NEW: ciclo de vida + evolución
│   ├── mod.rs
│   ├── lifecycle_inference.rs      ← ← lifecycle_inference.rs
│   ├── evolution_surrogate.rs      ← ← evolution_surrogate.rs (641)
│   ├── phenology_visual.rs         ← ← phenology_visual.rs
│   └── fog_simulation.rs           ← ← fog_simulation.rs
│
├── abiogenesis/                    ← sin cambio
├── pathfinding/                    ← sin cambio
└── reproduction/                   ← sin cambio
```

**Archivos de test** (`eco_e5_simulation_tests.rs`, etc.) se mueven al subdirectorio más afín o se dejan en raíz si son cross-domain.

## Pasos de implementación

### SM-2A: Crear `simulation/thermodynamic/`

1. Crear directorio `src/simulation/thermodynamic/`.
2. **Mover** `physics.rs`, `pre_physics.rs`, `dissipation.rs`, `thermal_transfer.rs` al subdirectorio.
3. Crear `thermodynamic/mod.rs` con `pub use` de todo lo público.
4. Actualizar `simulation/mod.rs`: reemplazar los 4 `pub mod` individuales por `pub mod thermodynamic`.
5. Buscar y actualizar imports: `use crate::simulation::physics` → `use crate::simulation::thermodynamic::physics` (o via re-export transparente).
6. `cargo test --lib` → verde.

### SM-2B: Crear `simulation/metabolic/`

1. Crear directorio `src/simulation/metabolic/`.
2. **Mover** `photosynthesis.rs`, `nutrient_uptake.rs`, `metabolic_stress.rs`, `growth_budget.rs`, `morphogenesis.rs`.
3. Crear `metabolic/mod.rs` con `pub use`.
4. Actualizar `simulation/mod.rs`.
5. Actualizar imports externos.
6. `cargo test --lib` → verde.

### SM-2C: Crear `simulation/lifecycle/`

1. Crear directorio `src/simulation/lifecycle/`.
2. **Mover** `lifecycle_inference.rs`, `evolution_surrogate.rs`, `phenology_visual.rs`, `fog_simulation.rs`.
3. Crear `lifecycle/mod.rs` con `pub use`.
4. Actualizar `simulation/mod.rs`.
5. Actualizar imports.
6. `cargo test --lib` → verde.

### SM-2D: Limpiar simulation/mod.rs

1. Verificar que `simulation/mod.rs` es limpio: solo `pub mod` de subdirectorios + re-exports selectivos.
2. Verificar que `pipeline.rs` sigue funcionando (importa sistemas de subdirectorios).
3. Actualizar `pipeline.rs` si los paths de sistemas cambiaron.

## Tácticas

- **Re-exports transparentes en cada mod.rs.** El pattern:
  ```rust
  // simulation/thermodynamic/mod.rs
  pub mod physics;
  pub mod pre_physics;
  pub mod dissipation;
  pub mod thermal_transfer;
  pub use physics::*;
  pub use pre_physics::*;
  // etc.
  ```
  Así los imports existentes `crate::simulation::physics_system` siguen funcionando si se re-exportan desde `simulation/mod.rs`.

- **pipeline.rs es el archivo crítico.** Todos los sistemas se registran ahí. Actualizar paths de funciones:
  ```rust
  // Antes:
  use crate::simulation::physics::thermal_system;
  // Después:
  use crate::simulation::thermodynamic::physics::thermal_system;
  // O via re-export:
  use crate::simulation::thermodynamic::thermal_system;
  ```

- **Un subdirectorio por cada Phase del pipeline** (aproximado):
  - `thermodynamic/` → `Phase::ThermodynamicLayer`
  - `metabolic/` → `Phase::MetabolicLayer` + `Phase::ChemicalLayer`
  - `lifecycle/` → `Phase::MorphologicalLayer` (parcial)

- **No mover archivos <100 LOC** que son cross-domain (ej. `input.rs`). Dejarlos en raíz.

## NO hace

- No cambia lógica, ecuaciones, ni comportamiento.
- No renombra funciones ni sistemas.
- No modifica `Phase` enum ni pipeline ordering.
- No toca módulos fuera de `simulation/`.
- No mueve `abiogenesis/`, `pathfinding/`, `reproduction/` (ya son subdirectorios).

## Criterios de aceptación

- `cargo test --lib` pasa sin regresión.
- `cargo build` compila sin warnings nuevos.
- `simulation/` raíz tiene ≤10 archivos sueltos (pipeline, bootstrap, input, mod, regression + subdirs).
- Cada subdirectorio tiene un `mod.rs` con re-exports.
- `pipeline.rs` funciona con los nuevos paths.
- Futuros archivos MG-3+ van directamente a `metabolic/` sin crear más archivos en raíz.

## Referencias

- `src/simulation/mod.rs` — módulo raíz actual
- `src/simulation/pipeline.rs` — scheduling (archivo crítico)
- `docs/sprints/CODE_QUALITY/SPRINT_Q5_PLUGIN_SPLIT.md` — split de plugins relacionado
- `docs/sprints/MORPHOGENESIS_INFERENCE/` — track que más se beneficia de esta reorganización
