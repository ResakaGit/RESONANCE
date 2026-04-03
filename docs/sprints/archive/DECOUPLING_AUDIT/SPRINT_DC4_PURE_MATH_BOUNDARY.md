# DC-4: Pure Math Boundary Enforcement

**Objetivo:** Restablecer la frontera de pureza: mover `terrain_blocks_vision` a `blueprint/equations/`, desacoplar rendering de `simulation::sensory::AttentionGrid`, extraer inline math de 8 sistemas a funciones puras.

**Estado:** PENDIENTE
**Esfuerzo:** M (~12 archivos, ~200 LOC refactor)
**Bloqueado por:** DC-1 (enum paths en equations/)
**Desbloquea:** Pureza completa de equations/, rendering desacoplado de simulation

---

## 3 Sub-tareas

| ID | Tarea | Archivos | LOC |
|----|-------|----------|-----|
| DC-4A | Mover `terrain_blocks_vision` a equations/ | 3 | ~40 |
| DC-4B | Desacoplar rendering ↔ AttentionGrid | 3 | ~30 |
| DC-4C | Extraer inline math de sistemas | 8 | ~120 |

---

## DC-4A: Mover `terrain_blocks_vision` a equations/

### Problema

`terrain_blocks_vision` es una función pura (input→output, sin ECS) que vive en `simulation/thermodynamic/physics.rs:96-124`. Es importada desde `simulation/thermodynamic/pre_physics.rs:16` creando un cross-module import dentro de simulation/.

### Análisis de la función

```rust
// Actual: simulation/thermodynamic/physics.rs:96
pub fn terrain_blocks_vision(from: Vec2, to: Vec2, terrain: &TerrainField) -> bool {
    // Bresenham raycast + altitude interpolation
    // Pure: &TerrainField (immutable), returns bool
    // No ECS, no commands, no queries
}

// Helper: physics.rs:126
fn raycast_cells_exclusive(from: (usize, usize), to: (usize, usize)) -> Vec<(usize, usize)> {
    // Supercover line algorithm
}
```

**Dependencia:** Solo necesita `TerrainField` — un struct con `sample_at()` y `world_to_cell()`. `TerrainField` vive en `topology/terrain_field.rs`.

### Diseño

```rust
// blueprint/equations/vision.rs — NUEVO

use crate::math_types::Vec2;
use crate::topology::TerrainField;

/// Raycast discreto sobre celdas: verifica si el terreno bloquea la línea de visión
/// entre dos puntos del mundo.
///
/// Interpola altitud linealmente entre origen y destino. Si alguna celda intermedia
/// tiene altitud mayor que la interpolación, retorna true (blocked).
///
/// Discrete raycast over cells: checks if terrain blocks line of sight
/// between two world positions.
pub fn terrain_blocks_vision(from: Vec2, to: Vec2, terrain: &TerrainField) -> bool {
    // ... (código idéntico, movido sin cambios)
}

/// Supercover line: todas las celdas entre dos puntos de grid (exclusivo endpoints).
fn raycast_cells_exclusive(
    from: (usize, usize),
    to: (usize, usize),
) -> Vec<(usize, usize)> {
    // ... (código idéntico, movido sin cambios)
}
```

**Registro en facade:**
```rust
// blueprint/equations/mod.rs — añadir
pub mod vision;
pub use vision::terrain_blocks_vision;
```

**Migración de imports:**
```rust
// ANTES (pre_physics.rs:16):
use crate::simulation::physics::terrain_blocks_vision;

// DESPUÉS:
use crate::blueprint::equations::terrain_blocks_vision;
```

### Decisión: ¿Trait para TerrainField?

| Opción | Pros | Contras | Decisión |
|--------|------|---------|----------|
| Pasar `&TerrainField` directamente | Simple, zero abstraction | equations/ depende de topology/ | **SÍ** |
| Trait `TerrainSampler` | Desacopla equations de topology | Over-engineering, 1 implementor | NO |
| Pasar `Fn(usize,usize)->f32` | Máxima flexibilidad | Ergonomía reducida, peor readability | NO |

**Justificación:** `blueprint/equations/` ya importa de `math_types/` (que es topology-adjacent). La dependencia equations→topology es unidireccional y estable. TerrainField es un struct de datos, no un componente ECS.

---

## DC-4B: Desacoplar rendering ↔ AttentionGrid

### Problema

```rust
// rendering/quantized_color/systems.rs:10
use crate::simulation::thermodynamic::sensory::AttentionGrid;
```

Rendering importa directamente de simulation. Esto crea un acoplamiento render→sim que complica testing y separación de concerns.

### Análisis del uso

`factor_precision_system` lee `AttentionGrid` para modular la precisión visual (ρ). Ya tiene fallback a camera-based LOD si `AttentionGrid` no existe:

```rust
attention: Option<Res<AttentionGrid>>,  // Optional — graceful degradation
```

### Diseño: Contract Resource en shared location

**No usamos trait objects.** El hot path (every materialized entity per frame) no tolera vtable indirection.

**No creamos un trait.** Solo hay 1 implementor. La abstracción no se paga.

**Movemos `AttentionGrid` a un módulo compartido** que ni simulation ni rendering "poseen":

```rust
// runtime_platform/contracts/attention.rs — NUEVO

use bevy::prelude::*;
use crate::math_types::Vec2;

/// Grid espacial de atención perceptiva (A ∈ [0, 1]).
/// Escrita por simulation (sensory), leída por rendering (precision).
///
/// Spatial attention grid. Written by simulation, read by rendering.
/// Lives in runtime_platform/contracts/ to avoid sim→render or render→sim coupling.
#[derive(Resource, Debug, Default)]
pub struct AttentionGrid {
    pub a: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub origin: Vec2,
}

impl AttentionGrid {
    /// Atención en una posición del mundo. Retorna 0.0 si fuera de bounds.
    pub fn get_attention(&self, world_pos: Vec2) -> f32 {
        // ... (lógica idéntica a la actual)
    }

    /// Coordenadas de celda para una posición del mundo.
    fn cell_coords(&self, world_pos: Vec2) -> Option<(usize, usize)> {
        // ... (lógica idéntica)
    }
}
```

**Migración de imports:**
```rust
// ANTES (rendering/quantized_color/systems.rs:10):
use crate::simulation::thermodynamic::sensory::AttentionGrid;

// DESPUÉS:
use crate::runtime_platform::contracts::AttentionGrid;

// ANTES (simulation/thermodynamic/sensory.rs):
pub struct AttentionGrid { ... }  // definición

// DESPUÉS:
use crate::runtime_platform::contracts::AttentionGrid;
// sensory.rs solo escribe en AttentionGrid, ya no lo define
```

### Ownership contract

```
runtime_platform/contracts/attention.rs  — DEFINE el tipo (neutral territory)
simulation/thermodynamic/sensory.rs      — ESCRIBE (attention_convergence_system)
rendering/quantized_color/systems.rs     — LEE (factor_precision_system)
simulation/thermodynamic/sensory.rs      — LEE (attention_gating_system)
plugins/thermodynamic_plugin.rs          — INICIALIZA (app.init_resource)
```

**Pattern:** "Contract Resource" — tipo definido en zona neutral, escrito por un módulo, leído por otros. No hay trait, no hay indirection, no hay abstraction tax.

---

## DC-4C: Extraer inline math de sistemas

### 8 Sistemas con math inline

| Sistema | Archivo | Math inline | Función pura destino |
|---------|---------|-------------|---------------------|
| `entity_shape_inference` | lifecycle/entity_shape_inference.rs:75-80 | `hunger * 0.25`, `1.35 * resistance` | **Cubierto por DC-2** (se extraen ahí) |
| `awakening_system` | awakening.rs:66-68 | `energy.qe() / (vol * vol)` | `equations::density_from_qe_volume(qe, vol)` |
| `epigenetic_adaptation` | emergence/epigenetic_adaptation.rs:20-24 | `0.5 + dim * 0.1` threshold | `equations::gene_expression_threshold(dim)` |
| `allometric_growth` | lifecycle/allometric_growth.rs:57-63 | Epsilon write gate | OK — guard pattern, no math |
| `physics` | thermodynamic/physics.rs:75-93 | Slope friction alignment | Mostly calls equations; minor inline OK |
| `culture` | emergence/culture.rs:95-98 | Tuple packing | OK — accessor calls, not math |
| `nutrient_uptake` | metabolic/nutrient_uptake.rs:48-55 | Profile delta clamp | OK — uses helper fn already |
| `pre_physics` | thermodynamic/pre_physics.rs:48-64 | Dual-path intake | OK — volume branching, not math |

**Scope real:** 2 extracciones necesarias (awakening + epigenetic). El resto ya está delegado o es guard logic.

### Extracción 1: awakening density calculation

```rust
// ANTES (awakening.rs:66-68):
let vol = volume.radius();
let density = energy.qe() / (vol * vol);

// DESPUÉS:
let density = equations::density_from_qe_volume(energy.qe(), volume.radius());
```

```rust
// blueprint/equations/core_physics/mod.rs — añadir

/// Densidad energética normalizada: qe / radius².
/// Axiom 1: everything is energy, density determines capabilities.
pub fn density_from_qe_volume(qe: f32, radius: f32) -> f32 {
    if radius <= 0.0 { return 0.0; }
    qe / (radius * radius)
}
```

### Extracción 2: epigenetic expression threshold

```rust
// ANTES (epigenetic_adaptation.rs:20-24):
let threshold = 0.5 + dim * 0.1;
if expression > threshold { ... }

// DESPUÉS:
let threshold = equations::gene_expression_threshold(dim);
```

```rust
// blueprint/equations/emergence/epigenetic.rs — añadir o crear

/// Umbral de expresión génica dado un índice dimensional.
/// Genes de dimensiones altas requieren más presión ambiental para expresarse.
///
/// Gene expression threshold given a dimensional index.
pub fn gene_expression_threshold(dimension_index: f32) -> f32 {
    0.5 + dimension_index * EPIGENETIC_DIM_SENSITIVITY
}
```

```rust
// blueprint/constants/emergence.rs — añadir

/// Sensibilidad dimensional para expresión epigenética.
/// Cada dimensión adicional requiere +10% de presión para expresarse.
pub const EPIGENETIC_DIM_SENSITIVITY: f32 = 0.1;

/// Base threshold para expresión génica (50% presión mínima).
pub const EPIGENETIC_BASE_THRESHOLD: f32 = 0.5;
```

---

## Plan de ejecución (4 commits)

### Commit 1: Crear `equations/vision.rs` + migrar terrain_blocks_vision

- Crear archivo, mover función + helper
- Re-export desde equations/mod.rs
- Actualizar import en pre_physics.rs
- Eliminar función de physics.rs
- Tests: mover tests existentes + añadir edge cases

### Commit 2: Mover AttentionGrid a runtime_platform/contracts/

- Crear `runtime_platform/contracts/attention.rs`
- Mover struct + impl desde sensory.rs
- Actualizar imports en: sensory.rs, quantized_color/systems.rs, thermodynamic_plugin.rs
- Tests: verificar que factor_precision_system compila con nuevo import

### Commit 3: Extraer inline math (awakening + epigenetic)

- Crear `equations::density_from_qe_volume` en core_physics/
- Crear `equations::gene_expression_threshold` en emergence/
- Crear constantes en constants/emergence.rs
- Actualizar awakening.rs y epigenetic_adaptation.rs
- Tests unitarios de las 2 funciones nuevas

### Commit 4: Cleanup + grep validation

- Verificar: `grep "use crate::simulation" src/rendering/` → solo allowed paths
- Verificar: `grep "terrain_blocks_vision" src/simulation/thermodynamic/physics.rs` → 0
- Cleanup dead imports

---

## Testing

### Capa 1: Unitario

```rust
// blueprint/equations/vision.rs — tests
#[cfg(test)]
mod tests {
    use super::*;

    fn mock_flat_terrain(w: usize, h: usize, alt: f32) -> TerrainField {
        // ... helper que crea terrain plano
    }

    #[test]
    fn same_cell_never_blocks() {
        let t = mock_flat_terrain(10, 10, 0.0);
        assert!(!terrain_blocks_vision(Vec2::ZERO, Vec2::ZERO, &t));
    }

    #[test]
    fn flat_terrain_never_blocks() {
        let t = mock_flat_terrain(20, 20, 5.0);
        assert!(!terrain_blocks_vision(Vec2::new(0.0, 0.0), Vec2::new(19.0, 19.0), &t));
    }

    #[test]
    fn hill_between_blocks() {
        let mut t = mock_flat_terrain(10, 10, 0.0);
        t.set_altitude(5, 5, 100.0);
        assert!(terrain_blocks_vision(Vec2::new(0.0, 0.0), Vec2::new(9.0, 9.0), &t));
    }

    #[test]
    fn out_of_bounds_blocks() {
        let t = mock_flat_terrain(5, 5, 0.0);
        assert!(terrain_blocks_vision(Vec2::new(-100.0, 0.0), Vec2::new(0.0, 0.0), &t));
    }
}

// blueprint/equations/core_physics/ — tests
#[test]
fn density_zero_radius_returns_zero() {
    assert_eq!(density_from_qe_volume(100.0, 0.0), 0.0);
}

#[test]
fn density_unit_values() {
    assert!((density_from_qe_volume(100.0, 10.0) - 1.0).abs() < 1e-6);
}

// blueprint/equations/emergence/ — tests
#[test]
fn gene_expression_threshold_dim_zero() {
    assert!((gene_expression_threshold(0.0) - 0.5).abs() < 1e-6);
}

#[test]
fn gene_expression_threshold_monotonic() {
    let t0 = gene_expression_threshold(0.0);
    let t5 = gene_expression_threshold(5.0);
    assert!(t5 > t0, "Higher dimensions should require more pressure");
}
```

### Capa 2: Integración

```rust
#[test]
fn factor_precision_system_works_without_attention() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // NO AttentionGrid inserted
    app.add_systems(Update, factor_precision_system);
    // Spawn entity with QuantizedPrecision
    // ...
    app.update(); // Should not panic, falls back to camera LOD
}

#[test]
fn factor_precision_system_reads_attention_from_contracts() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(AttentionGrid::default()); // From contracts/
    app.add_systems(Update, factor_precision_system);
    app.update(); // Should read attention values
}
```

### Capa 3: Orquestación

```rust
/// HOF: test que verifica la frontera de pureza de equations/.
fn assert_no_simulation_imports_in_equations() {
    // Este test se implementa como grep check en CI:
    // grep -r "use crate::simulation" src/blueprint/equations/ | wc -l == 0
    // grep -r "use crate::layers::" src/blueprint/equations/ | wc -l == 0 (post DC-1)
}
```

---

## Integración al codebase

### Lo que se CREA
- `blueprint/equations/vision.rs` — terrain_blocks_vision (moved)
- `runtime_platform/contracts/attention.rs` — AttentionGrid (moved)
- `blueprint/equations/core_physics/`: density_from_qe_volume
- `blueprint/equations/emergence/`: gene_expression_threshold
- `blueprint/constants/emergence.rs`: EPIGENETIC_DIM_SENSITIVITY, EPIGENETIC_BASE_THRESHOLD

### Lo que se MUEVE
- `terrain_blocks_vision`: physics.rs → equations/vision.rs
- `raycast_cells_exclusive`: physics.rs → equations/vision.rs
- `AttentionGrid` struct: sensory.rs → contracts/attention.rs

### Lo que NO cambia
- `attention_convergence_system` (solo cambia import path)
- `attention_gating_system` (solo cambia import path)
- `perception_system` (solo cambia import de terrain_blocks_vision)
- `factor_precision_system` (solo cambia import de AttentionGrid)

---

## Scope definido

**Entra:**
- Mover terrain_blocks_vision a equations/vision.rs (DC-4A)
- Mover AttentionGrid a runtime_platform/contracts/ (DC-4B)
- Extraer 2 funciones inline: density + gene_expression_threshold (DC-4C)
- Tests de las 3 capas

**NO entra:**
- Extraer inline math de entity_shape_inference (cubierto por DC-2)
- Refactor de sensory transduction pipeline
- Optimización de raycast algorithm
- Cambiar AttentionGrid API

---

## Criterios de cierre

- [ ] `cargo test` — 0 failures
- [ ] `grep "terrain_blocks_vision" src/simulation/thermodynamic/physics.rs` — 0 resultados
- [ ] `grep "use crate::simulation" src/rendering/quantized_color/systems.rs` — 0 resultados
- [ ] `grep "0.5 + dim" src/simulation/emergence/epigenetic_adaptation.rs` — 0 resultados
- [ ] `grep "qe() / (vol" src/simulation/awakening.rs` — 0 resultados
- [ ] equations/vision.rs tiene 4+ tests
- [ ] density_from_qe_volume tiene 2+ tests
- [ ] gene_expression_threshold tiene 2+ tests
- [ ] Ningún `// DEBT:` introducido
