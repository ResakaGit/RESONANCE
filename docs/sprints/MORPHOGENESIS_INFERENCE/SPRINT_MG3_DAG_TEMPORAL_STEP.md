# Sprint MG-3 — Paso Temporal del DAG Metabólico

**Módulo:** `src/simulation/morphogenesis.rs` (nuevo) + `src/blueprint/equations/` + `src/simulation/pipeline.rs`
**Tipo:** Sistemas ECS de una transformación cada uno; orquestación de flujos por tick.
**Onda:** B — Requiere MG-2 (`MetabolicGraph` estable).
**Estado:** ⏳ Pendiente

## Objetivo

Avanzar el estado dinámico del DAG cada `FixedUpdate`: computar flujos `J` por arista, disipación `Q_diss` y `S_gen` por nodo, y garantizar que ningún nodo viole el techo de Carnot cuando cambian `T_core` / `T_env`. Sentar el hook para memoización LOD (`BridgeMetabolicFlowStep`) sin acoplar aún el ledger completo (MG-6).

## Responsabilidades

### MG-3A: Funciones puras en `equations/`

Toda aritmética de reparto/redistribución en funciones puras. El sistema solo orquesta queries.

```rust
/// Propaga flujo J desde un nodo fuente a sus aristas salientes.
/// Reparte `available_exergy` proporcionalmente a `max_capacity` de cada arista.
/// Retorna J asignado por arista; Σ J_out ≤ available_exergy.
pub fn propagate_edge_flows(
    available_exergy: f32,
    edge_capacities: &[(u8, f32)],  // (edge_idx, max_capacity)
) -> ArrayVec<(u8, f32), 16>       // (edge_idx, assigned_flow)

/// Redistribuye violación de eficiencia tras constraint de Carnot.
/// Si η_node > η_carnot, reduce exergía útil del nodo y aumenta Q_diss.
/// Retorna (new_efficiency, additional_heat).
pub fn redistribute_node_violation(
    current_efficiency: f32,
    carnot_limit: f32,
    exergy_in: f32,
    activation_energy: f32,
) -> (f32, f32)
```

- `propagate_edge_flows`: si `edge_capacities` vacío → retorna vacío (nodo terminal, toda exergía se disipa). Si la suma de capacidades = 0 → reparto uniforme.
- `redistribute_node_violation`: si `current_efficiency ≤ carnot_limit` → retorna `(current_efficiency, 0.0)` sin cambio. Guard: `carnot_limit ∈ [0.0, 1.0)`.

### MG-3B: `metabolic_graph_step_system`

```rust
/// Propaga flujos J por aristas y computa Q_diss, S_gen por nodo.
/// Conservación por nodo: Σ J_in = Σ J_out + P_work + Q_diss.
pub fn metabolic_graph_step_system(
    mut query: Query<
        (&mut MetabolicGraph, &BaseEnergy, &AmbientPressure),
        Without<Dead>,
    >,
) {
    for (mut graph, energy, pressure) in &mut query {
        let t_core = equations::equivalent_temperature(energy.qe(), ...);
        let t_env  = /* derivado de AmbientPressure */;
        // 1. Recorrer nodos en orden topológico (ascendente por índice).
        // 2. Para cada nodo: J_in de aristas entrantes → exergy_balance → propagate_edge_flows.
        // 3. Escribir flow_rate en aristas, thermal_output y entropy_rate en nodos.
        // 4. Acumular total_entropy_rate.
    }
}
```

- **Phase:** `Phase::MetabolicLayer`.
- **Query:** 3 tipos — `MetabolicGraph`, `BaseEnergy`, `AmbientPressure`.
- **Escribe:** `MetabolicGraph` in-place (`flow_rate` en aristas, `thermal_output` / `entropy_rate` en nodos, `total_entropy_rate` agregado).
- **Conservación por nodo:** `Σ J_in = Σ J_out + P_work + Q_diss`. Documentar en comentario de sistema qué es `J` en unidades del modelo (qe/tick).
- **`transport_cost` en aristas:** precomputado en builder (MG-2) desde `vascular_transport_cost(viscosity, length, radius)` del `OrganSpec`. No se refresca cada tick (geometría estable intra-tick).
- **Orden:** `.after(nutrient_uptake_system)` y `.after(photosynthesis_system)` — sistemas que proveen intake deben correr primero.
- **Guard change detection:** epsilon configurable `METABOLIC_STEP_EPSILON`. Solo mutar si `(old - new).abs() > METABOLIC_STEP_EPSILON`.

### MG-3C: `entropy_constraint_system`

```rust
/// Reclamp eficiencias de nodos al techo de Carnot por condiciones ambientales actuales.
pub fn entropy_constraint_system(
    mut query: Query<
        (&mut MetabolicGraph, &BaseEnergy, &AmbientPressure),
        Without<Dead>,
    >,
) {
    for (mut graph, energy, pressure) in &mut query {
        let t_core = equations::equivalent_temperature(energy.qe(), ...);
        let t_env  = /* derivado de AmbientPressure */;
        let eta_carnot = equations::carnot_efficiency(t_core, t_env);
        for node in graph.nodes.iter_mut() {
            let (new_eff, extra_heat) =
                equations::redistribute_node_violation(node.efficiency, eta_carnot, ...);
            if (node.efficiency - new_eff).abs() > METABOLIC_STEP_EPSILON {
                node.efficiency = new_eff;
                node.thermal_output += extra_heat;
            }
        }
    }
}
```

- **Phase:** `Phase::MetabolicLayer`.
- **Orden:** `.after(metabolic_graph_step_system)` — un paso constraint por tick.
- **Cadencia:** cada tick (`N = 1`). Justificación: O(nodes) con max 12 nodos = trivial. No justifica skipping.
- **T_core:** derivada de `BaseEnergy.qe()` vía `equivalent_temperature` (misma convención que `thermal_transfer_system`). Documentar mapeo en comentario de módulo.
- **T_env:** derivada de `AmbientPressure.delta_qe_constant` (o convención ya existente en `eco/`). Una sola tabla de mapeo.
- **Sin RNG:** determinismo total.

### MG-3D: Constantes

```rust
// --- Morfogénesis: Paso Temporal ---
pub const METABOLIC_STEP_EPSILON: f32 = 1e-4;      // Umbral de cambio para guard detection
pub const METABOLIC_MIN_FLOW: f32 = 0.01;           // Flujo mínimo por arista (debajo = 0)
pub const METABOLIC_STARVATION_THRESHOLD: f32 = 1.0; // qe mínimo para operar un nodo
```

### MG-3E: Bridge / LOD (hook liviano)

- Registrar `BridgeMetabolicFlowStep` como tipo hermano de `BridgeCache<B>`.
- **Clave cuantizada:** `(graph_hash, qe_quantized, pressure_quantized)` — 3 campos → `u64`.
- **Must:** registrar extensión + test determinista "Far no panic" (entidades lejanas usan valor cacheado sin recomputar).
- **Nice-to-have (no bloqueante):** cuantización completa con política Near/Mid/Far.

### MG-3F: Registro en pipeline

```rust
// simulation/pipeline.rs
app.add_systems(
    FixedUpdate,
    (
        metabolic_graph_step_system
            .after(nutrient_uptake_system)
            .after(photosynthesis_system),
        entropy_constraint_system
            .after(metabolic_graph_step_system),
    ).in_set(Phase::MetabolicLayer),
);
```

## Tácticas

- **Un sistema, una transformación.** `step` propaga flujos; `constraint` reclampea eficiencias. No fusionar — distintos invariantes, distintos tests.
- **Orden topológico por índice ascendente.** Kahn con cola de listos `BinaryHeap<Reverse<u8>>` o simple `ArrayVec` sorted. Garantiza determinismo sin RNG.
- **Conservación numérica.** Tras propagación, assert debug `(Σ J_in - Σ J_out - Q_diss - P_work).abs() < EPSILON` por nodo. En release, el clamp silencia. Documentar tolerancia: `1e-3 qe` por nodo.
- **SparseSet ya viene de MG-2.** Los sistemas filtran `With<MetabolicGraph>` — entidades sin grafo no entran (cero regresión, cero branching en hot path).
- **Backward compatible.** Query vacío si no hay entidades con `MetabolicGraph`. Sin `if-else` por presencia.
- **Inanición gradual.** Si `BaseEnergy.qe() < METABOLIC_STARVATION_THRESHOLD`, los flujos colapsan a 0 en un tick. No es muerte instantánea — el organismo puede recuperarse si recibe intake.
- **Tests primero.** Casos: inanición, flujo estable, sobrecalentamiento (η → 0), cadena lineal, DAG con fork.

## NO hace

- No implementa `organ_transform` / `evaluate_metabolic_chain` / `EntropyLedger` (MG-6).
- No implementa shape/albedo/rugosity solvers (MG-4, MG-5, MG-7).
- No modifica `OrganManifest` ni el builder del MG-2 salvo fixes de contrato bloqueantes.
- No crea componentes nuevos (solo muta `MetabolicGraph` in-place).

## Dependencias

- MG-1 (ecuaciones: `carnot_efficiency`, `entropy_production`, `exergy_balance`, `vascular_transport_cost`).
- MG-2 (`MetabolicGraph`, `ExergyNode`, `ExergyEdge`, topología validada).
- `src/layers/energy.rs` — `BaseEnergy` (L0).
- `src/layers/pressure.rs` — `AmbientPressure` (L6).

## Criterios de aceptación

### MG-3A (Funciones puras)
- Test: `propagate_edge_flows(100.0, &[(0, 60.0), (1, 40.0)])` → edge 0 recibe `60.0`, edge 1 recibe `40.0` (reparto proporcional exacto).
- Test: `propagate_edge_flows(100.0, &[])` → vacío (nodo terminal).
- Test: `propagate_edge_flows(100.0, &[(0, 0.0), (1, 0.0)])` → reparto uniforme: 50.0 cada una.
- Test: `redistribute_node_violation(0.8, 0.6, 100.0, 10.0)` → `(0.6, extra_heat > 0)`.
- Test: `redistribute_node_violation(0.5, 0.6, 100.0, 10.0)` → `(0.5, 0.0)` (no viola, sin cambio).

### MG-3B (Step system — integración)
- Test: grafo de 3 nodos en cadena: Captador(η=0.9, E_a=3) → Procesador(η=0.7, E_a=8) → Actuador(η=0.6, E_a=5). Input: qe=500, T_core=400, T_env=280. Verificar:
  - `J` entre nodos ≤ `max_capacity` de cada arista.
  - `thermal_output` de cada nodo > 0 (todo nodo disipa algo).
  - `total_entropy_rate` = `Σ entropy_production(Q_diss_i, T_core)` dentro de `1e-3`.
- Test: conservación numérica en cadena de 3 nodos: `Σ J_in_raíz = Σ Q_diss_total + exergía_final_último_nodo + Σ E_a`, tolerancia `1e-3 qe`.
- Test: sin `MetabolicGraph` → sistema no ejecuta (query vacío medible, no panic).
- Test: inanición — `BaseEnergy.qe() = 0.5` (< umbral) → todos los `flow_rate = 0.0` tras un tick.
- Test: DAG con fork — 1 captador, 2 aristas salientes con capacidad 60/40 → flujos ~60% / ~40%.

### MG-3C (Constraint system)
- Test: `T_env = 400, T_core = 400` → `carnot = 0` → todas las eficiencias → 0 (o mínimo operativo si se define).
- Test: `T_env = 300, T_core = 500` → `carnot = 0.4`. Nodo con η=0.8 → reclamped a 0.4. Nodo con η=0.3 → sin cambio.
- Test: tras constraint, `∀ node: node.efficiency ≤ carnot_efficiency(T_core, T_env)`.
- Test: constraint es idempotente — aplicar dos veces con mismos inputs no cambia nada.

### MG-3E (Bridge)
- Test: `BridgeMetabolicFlowStep` registrado sin panic.
- Test: lookup con clave conocida retorna valor cacheado; miss retorna None.

### General
- `cargo test --lib` sin regresión.
- Trazabilidad: comentarios en código en español; identificadores en inglés.
- Documentación de T_core/T_env: tabla de mapeo ECS→escalar en comentario de módulo.

## Referencias

- `docs/design/MORPHOGENESIS.md` §3.1, §3.3 (tabla sistemas), §6 MG-3
- `docs/arquitectura/blueprint_morphogenesis_inference.md` §2–§3 (invariantes conservación)
- `src/simulation/pipeline.rs` — orden de fases
- `src/blueprint/equations/` — `equivalent_temperature()`, `carnot_efficiency()`, `entropy_production()`
- `src/layers/energy.rs` — `BaseEnergy` (1 campo: `qe`)
- `src/layers/pressure.rs` — `AmbientPressure` (2 campos: `delta_qe_constant`, `terrain_viscosity`)
