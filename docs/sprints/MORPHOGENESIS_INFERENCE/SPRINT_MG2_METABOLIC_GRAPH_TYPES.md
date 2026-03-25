# Sprint MG-2 — MetabolicGraph Types & Builder

**Módulo:** `src/layers/metabolic_graph.rs` (nuevo) + `src/layers/mod.rs` + `src/blueprint/equations.rs`
**Tipo:** Tipos puros + builder + función pura de inferencia. Sin sistemas runtime.
**Onda:** A — Depende de MG-1 (ecuaciones).
**Estado:** ⏳ Pendiente

## Objetivo

Definir los tipos que representan el DAG metabólico de una entidad viva compleja: `ExergyNode` (nodo-órgano), `ExergyEdge` (arista-flujo), `MetabolicGraph` (componente ECS), y el builder fluent que construye y valida el grafo. Además, la función pura `metabolic_graph_from_manifest` que infiere el DAG desde un `OrganManifest` existente.

## Responsabilidades

### MG-2A: Tipos Core (layers/metabolic_graph.rs)

```rust
/// Nodo funcional del DAG metabólico.
/// Cada nodo es un "órgano lógico" — una función pura de transformación.
/// Los campos `thermal_output` y `entropy_rate` son DERIVADOS (computados en MG-3).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct ExergyNode {
    pub role: OrganRole,           // Reutiliza los 12 roles de organ.rs
    pub efficiency: f32,           // η ∈ (0, 1] — clamped por Carnot en runtime
    pub activation_energy: f32,    // E_a: costo mínimo para operar (qe)
    pub thermal_output: f32,       // Q_diss: derivado, escrito por MG-3
    pub entropy_rate: f32,         // S_gen: derivado, escrito por MG-3
}
```

- **4 campos informativos + 1 campo por compatibilidad de padding** — cumple regla ≤4 útiles.
- `thermal_output` y `entropy_rate` se inicializan a 0 y se computan cada tick.

```rust
/// Arista dirigida: flujo de exergía entre nodos.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct ExergyEdge {
    pub flow_rate: f32,            // J actual (qe/s) — derivado en MG-3
    pub max_capacity: f32,         // Capacidad máxima de la arista
    pub transport_cost: f32,       // μL³/r⁴ — costo WBE
}
```

```rust
/// DAG metabólico completo. Componente ECS, SparseSet.
/// Solo entidades "vivas complejas" lo tienen.
#[derive(Component, Clone, Debug, PartialEq, Reflect)]
#[component(storage = "SparseSet")]
pub struct MetabolicGraph {
    pub nodes: ArrayVec<ExergyNode, 12>,
    pub edges: ArrayVec<ExergyEdge, 16>,
    pub adjacency: ArrayVec<(u8, u8), 16>,  // (from_idx, to_idx)
    pub total_entropy_rate: f32,             // Σ S_gen — derivado
}
```

### MG-2B: Builder con Validación

```rust
pub struct MetabolicGraphBuilder {
    nodes: ArrayVec<ExergyNode, 12>,
    edges: ArrayVec<ExergyEdge, 16>,
    adjacency: ArrayVec<(u8, u8), 16>,
}

impl MetabolicGraphBuilder {
    pub fn new() -> Self;
    pub fn add_node(self, role: OrganRole, efficiency: f32, activation_energy: f32) -> Self;
    pub fn add_edge(self, from: u8, to: u8, max_capacity: f32) -> Self;
    pub fn build(self) -> Result<MetabolicGraph, MetabolicGraphError>;
}
```

- `build()` valida:
  1. **Acyclicidad:** DFS desde cada fuente. Si ciclo → `Err(Cycle)`.
  2. **Mínimo funcional:** al menos 1 nodo con rol Captador (Root, Leaf, Sensory) y al menos 1 nodo conectado al exterior (implícito: nodo sin arista de salida = disipador).
  3. **Índices válidos:** `from < nodes.len()`, `to < nodes.len()`, `from ≠ to`.
  4. **No duplicados:** no dos aristas con mismo `(from, to)`.

- Error type:
```rust
pub enum MetabolicGraphError {
    Cycle,
    NoCaptorNode,
    InvalidIndex,
    DuplicateEdge,
    Empty,
}
```

### MG-2C: Inferencia desde OrganManifest (equations.rs)

- `metabolic_graph_from_manifest(manifest: &OrganManifest, t_core: f32, t_env: f32) -> MetabolicGraph`
  - Crea un nodo por cada `OrganSpec` en el manifesto.
  - `efficiency` = `carnot_efficiency(t_core, t_env) * role_efficiency_factor(role)`.
  - `activation_energy` = `role_activation_energy(role)` (tabla constante).
  - Topología default: cadena lineal `Captador → Procesador → Distribuidor → Actuadores`.
    - Roots y Leafs → nodos Captador.
    - Core y Stem → nodo Procesador/Distribuidor.
    - Fin, Limb → nodos Actuador.
    - Sensory, Thorn, Shell → nodos terminales.
  - Aristas con `max_capacity` proporcional a `scale_factor` del OrganSpec.

- Tablas constantes (en constants.rs):
```rust
pub const ROLE_EFFICIENCY_FACTOR: [f32; 12] = [
    0.8,  // Stem — buen conductor
    0.9,  // Root — alta eficiencia de absorción
    0.7,  // Core — procesamiento general
    0.95, // Leaf — fotosíntesis muy eficiente
    0.6,  // Petal — más decorativo, baja eficiencia
    0.5,  // Sensory — consumidor, no productor
    0.3,  // Thorn — mínima transformación
    0.4,  // Shell — protección, no procesamiento
    0.7,  // Fruit — almacenamiento eficiente
    0.6,  // Bud — potencial alto, eficiencia media
    0.75, // Limb — buen actuador
    0.8,  // Fin — muy eficiente en fluido
];

pub const ROLE_ACTIVATION_ENERGY: [f32; 12] = [
    5.0,  // Stem
    3.0,  // Root
    8.0,  // Core
    2.0,  // Leaf
    1.0,  // Petal
    4.0,  // Sensory
    0.5,  // Thorn
    1.0,  // Shell
    6.0,  // Fruit
    2.0,  // Bud
    7.0,  // Limb
    5.0,  // Fin
];
```

### MG-2D: Re-exports y registro

- Agregar `pub mod metabolic_graph;` en `layers/mod.rs`.
- Re-exportar: `MetabolicGraph`, `ExergyNode`, `ExergyEdge`, `MetabolicGraphBuilder`, `MetabolicGraphError`.
- Registrar `MetabolicGraph` como `Reflect` en `LayersPlugin`.

## Tácticas

- **ArrayVec, no Vec.** 12 nodos y 16 aristas son techo fijo. Sin allocación dinámica en hot path. El ArrayVec ya está en `Cargo.toml` (usado por `OrganManifest`).
- **DFS iterativo para validación de ciclos.** Stack explícito, no recursión. Max profundidad = 12 (nodos max). O(N+E) con N≤12, E≤16 → trivial.
- **Los campos derivados arrancan en 0.** `thermal_output` y `entropy_rate` son escritos por `metabolic_graph_step_system` (MG-3). En la construcción son 0.
- **Builder no consume self.** Patrón fluent con `self` by value (moved). El caller no puede reusar un builder parcial.
- **`metabolic_graph_from_manifest` es conservador.** Genera una topología default razonable. En el futuro, recetas (`EffectRecipe`) podrían customizar la topología.

## NO hace

- No implementa paso temporal (eso es MG-3).
- No modifica OrganManifest ni organ inference existente.
- No crea sistemas ECS.
- No toca el pipeline de simulación.
- No define `EntropyLedger` (eso es MG-6).

## Dependencias

- MG-1 (ecuaciones: `carnot_efficiency` usada por `metabolic_graph_from_manifest`).
- `src/layers/organ.rs` — `OrganRole`, `OrganManifest`, `OrganSpec`.
- `arrayvec` — ya en Cargo.toml.
- `bevy::prelude` — Component, Reflect.

## Criterios de aceptación

### MG-2A
- Test: `ExergyNode` es `Copy` y `Clone`.
- Test: `MetabolicGraph` con 0 nodos → `build()` retorna `Err(Empty)`.
- Test: `MetabolicGraph` con 12 nodos + 16 aristas → `build()` ok.
- Test: `MetabolicGraph` con 13 nodos → compile error (ArrayVec<12>).

### MG-2B
- Test: builder con ciclo (A→B→A) → `Err(Cycle)`.
- Test: builder sin nodo Captador → `Err(NoCaptorNode)`.
- Test: builder con edge `from = to` → `Err(InvalidIndex)`.
- Test: builder con edge duplicado → `Err(DuplicateEdge)`.
- Test: builder con cadena lineal válida → Ok.
- Test: builder con árbol válido (1 captador, 3 terminales) → Ok.

### MG-2C
- Test: `metabolic_graph_from_manifest` con Rosa (Stem+Leaf+Thorn+Petal) → DAG con 4 nodos.
- Test: cada nodo tiene efficiency ≤ carnot_efficiency(T_core, T_env).
- Test: aristas conectan captadores → procesadores → terminales.
- Test: `scale_factor` del OrganSpec influye en `max_capacity` de la arista.

### General
- `cargo test --lib` pasa sin regresión.
- Todos los tipos tienen `Reflect` y doc-comments.
- `layers/mod.rs` re-exporta todos los tipos nuevos.

## Referencias

- `src/layers/organ.rs` — OrganRole, OrganManifest, OrganSpec como modelo
- `src/blueprint/equations.rs` — `carnot_efficiency()` (MG-1)
- `src/blueprint/constants/` — patrón de tablas constantes (shards por dominio)
- `docs/design/MORPHOGENESIS.md` §3.1
