# Sprint MG-6 — Writer Monad + EntropyLedger (Matrioska fase 1)

**Módulo:** `src/blueprint/equations/` + `src/layers/metabolic_graph.rs` + `src/simulation/morphogenesis.rs`
**Tipo:** Tipos `Copy` + funciones puras + sistema ECS que materializa ledger derivado.
**Onda:** C — Paralelo con MG-4 y MG-5; requiere MG-2 (no exige MG-3 en el grafo de README, pero en runtime el ledger debe leer un grafo coherente → ordenar después del step cuando MG-3 exista).
**Estado:** ⏳ Pendiente

## Objetivo

Formalizar el patrón **Writer termodinámico**: cada nodo es `organ_transform` → `OrganOutput` (masa/exergía útil + desechos W + calor Q). Componer la cadena con `evaluate_metabolic_chain` y volcar el resumen en `EntropyLedger` (componente SparseSet, **recomputado cada tick**, no estado persistente lógico).

### Contrato MG-3 ↔ MG-6 (anti-drift)

- **Regla por defecto (recomendada):** `EntropyLedger` y `per_node_heat` salen **únicamente** de `evaluate_metabolic_chain` + `organ_transform`. Los campos `thermal_output` / `entropy_rate` en `ExergyNode` (escritos por MG-3) deben **igualar** `per_node_heat[i]` y `entropy_production(per_node_heat[i], T_core)` para el mismo índice `i` en el mismo tick, **o** MG-3 deja de escribir Q en nodos cuando el ledger está activo — elegir una variante en el PR y testear identidad algebraica.
- **Prohibido:** dos definiciones de ΣQ divergentes consumidas por MG-5/MG-7.

## Responsabilidades

### MG-6A: Tipos en `equations/`

```rust
/// Salida de un órgano lógico: útil + desechos (Writer).
/// Invariantes: mass_in = mass_out + waste_mass,
///              exergy_in = exergy_out + heat_dissipated + activation_energy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrganOutput {
    pub mass_out: f32,        // Masa útil transferida al siguiente nodo
    pub exergy_out: f32,      // Exergía útil transferida
    pub waste_mass: f32,      // Desechos de masa (W)
    pub heat_dissipated: f32, // Calor disipado (Q_diss)
}

/// Resultado de evaluar todo el DAG en orden topológico.
/// per_node_heat[i] = heat_dissipated del nodo i (no orden de visita).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChainOutput {
    pub final_exergy: f32,         // Exergía útil al final de la cadena
    pub total_heat: f32,           // Σ Q_diss de todos los nodos
    pub total_waste: f32,          // Σ W de todos los nodos
    pub per_node_heat: [f32; 12],  // Q_diss por nodo (índice = posición en graph.nodes)
}
```

### MG-6B: Funciones puras

#### `organ_transform`

```rust
/// Transforma masa y exergía de entrada en un órgano lógico.
/// Conservación: mass_in = mass_out + waste_mass,
///               exergy_in = exergy_out + heat_dissipated + activation_energy.
pub fn organ_transform(
    mass_in: f32,
    exergy_in: f32,
    efficiency: f32,
    activation_energy: f32,
) -> OrganOutput
```

- `exergy_out = exergy_balance(exergy_in, efficiency, activation_energy)` (MG-1).
- `heat_dissipated = exergy_in - exergy_out - activation_energy`. Si < 0 → clamp 0 (activación consume todo).
- `waste_mass = mass_in * (1.0 - efficiency)`. Proporcional a ineficiencia.
- `mass_out = mass_in - waste_mass`.
- Guards: `mass_in < 0` → clamp 0. `efficiency` clamped a `[0.0, 1.0]` por Carnot upstream.
- **Assert debug:** `(mass_in - mass_out - waste_mass).abs() < 1e-4` y `(exergy_in - exergy_out - heat_dissipated - activation_energy).abs() < 1e-4`.

#### `evaluate_metabolic_chain`

```rust
/// Evalúa el DAG metabólico completo en orden topológico.
/// Retorna ChainOutput con totales y heat por nodo.
pub fn evaluate_metabolic_chain(
    graph: &MetabolicGraph,
    initial_mass: f32,
    initial_exergy: f32,
) -> ChainOutput
```

- **Orden topológico determinista:** Kahn con cola de listos ordenada por índice ascendente. Mismo grafo → mismo orden → mismo resultado.
- **Join (varios padres):** sumar `mass_out` / `exergy_out` de predecesores ya evaluados como entradas del nodo hijo. Flujo por arista proporcional a `max_capacity` (mismo reparto que `propagate_edge_flows` de MG-3).
- **Fork (varios hijos):** repartir masa/exergía saliente proporcionalmente a `max_capacity` de aristas salientes. Función pura `distribute_to_children(exergy, mass, edge_capacities) -> ArrayVec<(f32, f32), 16>` en `equations/`.
- **Acumula:** `total_heat += heat_dissipated`, `total_waste += waste_mass`, `per_node_heat[i] = heat_dissipated` por cada nodo `i`.
- **Final:** `final_exergy` = suma de `exergy_out` de nodos terminales (sin aristas salientes).
- **Landauer (blueprint §3.2.1):** clamp inferior `LANDAUER_MIN_HEAT = 0.001` en `constants.rs` por nodo. Opcional v1; si se omite, documentar "defer post-v1".

### MG-6C: Componente `EntropyLedger`

```rust
/// Libro contable termodinámico. Recomputado cada tick, no estado persistente.
/// Solo entidades con MetabolicGraph lo reciben.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct EntropyLedger {
    /// Σ Q_diss de organ_transform por nodo (qe/tick).
    pub total_heat_generated: f32,
    /// Σ waste_mass de organ_transform por nodo (qe/tick).
    pub total_waste_generated: f32,
    /// S_gen = entropy_production(total_heat, T_core) (MG-1).
    pub entropy_rate: f32,
    /// η_total = final_exergy / max(initial_exergy, EPSILON).
    pub exergy_efficiency: f32,
}
```

- 4 campos — cumple regla ECS.
- `entropy_rate` = `entropy_production(total_heat_generated, t_core)` (MG-1). No duplicar fórmula inline.
- `exergy_efficiency`: si `initial_exergy ≤ EPSILON` → `0.0` (prohibido NaN).
- Registrar en `LayersPlugin` con `Reflect`.

### MG-6D: `entropy_ledger_system`

```rust
/// Materializa EntropyLedger desde evaluate_metabolic_chain cada tick.
pub fn entropy_ledger_system(
    mut commands: Commands,
    query: Query<
        (Entity, &MetabolicGraph, &BaseEnergy),
        Without<Dead>,
    >,
    mut ledger_query: Query<&mut EntropyLedger>,
) {
    for (entity, graph, energy) in &query {
        let t_core = equations::equivalent_temperature(energy.qe(), ...);
        let initial_mass   = energy.qe();  // proxy masa = qe en modelo
        let initial_exergy = energy.qe();  // exergía inicial = qe disponible

        let chain = equations::evaluate_metabolic_chain(graph, initial_mass, initial_exergy);

        let s_gen = equations::entropy_production(chain.total_heat, t_core);
        let eta   = if initial_exergy > EPSILON {
            chain.final_exergy / initial_exergy
        } else {
            0.0
        };

        let new_ledger = EntropyLedger {
            total_heat_generated: chain.total_heat,
            total_waste_generated: chain.total_waste,
            entropy_rate: s_gen,
            exergy_efficiency: eta,
        };

        if let Ok(mut existing) = ledger_query.get_mut(entity) {
            if *existing != new_ledger {
                *existing = new_ledger;
            }
        } else {
            commands.entity(entity).insert(new_ledger);
        }
    }
}
```

- **Phase:** `Phase::MetabolicLayer`.
- **Query:** 3 tipos — `MetabolicGraph`, `BaseEnergy`, `Entity`.
- **Orden:** `.after(entropy_constraint_system)` — lee `MetabolicGraph` después del reclamp de η para usar eficiencias corregidas.
- **Guard:** comparación por `PartialEq` del struct completo (4 campos f32). Si iguales, no muta.

### MG-6E: Constantes

```rust
// --- Morfogénesis: Writer Monad ---
pub const LANDAUER_MIN_HEAT: f32 = 0.001;           // Cota inferior Q por nodo (Landauer)
pub const CHAIN_CONSERVATION_EPSILON: f32 = 1e-3;   // Tolerancia de conservación en debug assert
```

### MG-6F: Función auxiliar `distribute_to_children`

```rust
/// Reparte masa y exergía entre aristas salientes proporcional a max_capacity.
/// Retorna (mass_i, exergy_i) por arista. Σ mass_i = total_mass, Σ exergy_i = total_exergy.
pub fn distribute_to_children(
    total_mass: f32,
    total_exergy: f32,
    edge_capacities: &[(u8, f32)],  // (edge_idx, max_capacity)
) -> ArrayVec<(u8, f32, f32), 16>   // (edge_idx, mass, exergy)
```

- Si capacidades vacías → retorna vacío (nodo terminal).
- Si Σ capacidades = 0 → reparto uniforme.
- Conservación: `Σ mass_i = total_mass`, `Σ exergy_i = total_exergy` dentro de `CHAIN_CONSERVATION_EPSILON`.

## Tácticas

- **Stack-only.** `OrganOutput`, `ChainOutput` y acumuladores en stack. `per_node_heat` es `[f32; 12]` fijo. Sin `Vec` en hot path.
- **Desync prohibido.** Nunca "leer ledger del frame anterior" para lógica de gameplay; siempre recomputar desde grafo actual.
- **Downstream.** MG-5 y MG-7 consumen `total_heat_generated` como Q metabólico agregado — es la **única** fuente canónica.
- **Idempotencia.** Si el grafo y las condiciones ambientales no cambian entre ticks, el ledger no cambia (guard por `PartialEq`).
- **Debug asserts.** En modo debug, activar verificación de conservación por nodo y por cadena. En release, solo clamps silenciosos.
- **Kahn determinista.** Cola de listos: `ArrayVec<u8, 12>` sorted ascendente tras cada inserción. O(N log N) con N ≤ 12 = trivial.

## NO hace

- No implementa optimización de forma ni albedo (MG-4, MG-5).
- No implementa rugosity (MG-7).
- No introduce RNG.
- No modifica `MetabolicGraph` — solo lo lee.

## Dependencias

- MG-1 (`entropy_production`, `exergy_balance`, guards numéricos).
- MG-2 (topología y tipos de grafo: `MetabolicGraph`, `ExergyNode`, `ExergyEdge`).
- `src/layers/energy.rs` — `BaseEnergy` (1 campo: `qe`).

## Criterios de aceptación

### MG-6A (Tipos)
- Test: `OrganOutput` es `Copy`.
- Test: `ChainOutput` es `Copy`.
- Test: `size_of::<ChainOutput>()` = `3 * 4 + 12 * 4 = 60 bytes` (stack-friendly).

### MG-6B (organ_transform)
- Test: `organ_transform(100.0, 500.0, 0.7, 10.0)`:
  - `mass_out = 70.0`, `waste_mass = 30.0` (100 - 70).
  - `exergy_out = exergy_balance(500, 0.7, 10) = 340.0`.
  - `heat_dissipated = 500 - 340 - 10 = 150.0`.
  - Verificar `mass_in = mass_out + waste_mass` exacto.
  - Verificar `exergy_in = exergy_out + heat + E_a` dentro de `1e-4`.
- Test: `organ_transform(0.0, 0.0, 0.7, 10.0)` → todo 0 (sin input, nodo apagado).
- Test: `organ_transform(100.0, 5.0, 0.7, 10.0)` → `exergy_out = 0` (activación > exergía disponible × η).
- Test: conservación con 20 combinaciones aleatorias (table-driven): siempre `M_in = M_out + W` y `E_in = E_out + Q + E_a` dentro de ε.

### MG-6B (evaluate_metabolic_chain)
- Test: cadena lineal 3 nodos — Captador(η=0.9, E_a=3) → Procesador(η=0.7, E_a=8) → Actuador(η=0.6, E_a=5), M=100, E=500:
  - Nodo 0: `organ_transform(100, 500, 0.9, 3)` → mass_out=90, exergy_out=447, heat=50, waste=10.
  - Nodo 1: `organ_transform(90, 447, 0.7, 8)` → mass_out=63, exergy_out=304.9, heat=134.1, waste=27.
  - Nodo 2: `organ_transform(63, 304.9, 0.6, 5)` → mass_out=37.8, exergy_out=177.94, heat=121.96, waste=25.2.
  - `total_heat ≈ 50 + 134.1 + 121.96 = 306.06`.
  - `total_waste ≈ 10 + 27 + 25.2 = 62.2`.
  - `final_exergy ≈ 177.94`.
  - `per_node_heat = [50.0, 134.1, 121.96, 0, 0, ..., 0]`.
- Test: DAG con **fork** (1 captador → 2 procesadores, capacidades 60/40):
  - Reparto proporcional: 60% y 40% de masa/exergía del captador.
  - Conservación: `Σ mass_out_hijos + Σ waste = mass_in_captador`.
- Test: DAG con **join** (2 captadores → 1 procesador):
  - El procesador recibe `Σ mass_out` y `Σ exergy_out` de ambos padres.
  - `per_node_heat` del procesador refleja el calor de procesar la suma.
- Test: determinismo — mismo grafo, mismos inputs, 100 evaluaciones → resultados idénticos bit a bit.
- Test: grafo de 1 nodo → `final_exergy = exergy_out` del único nodo.

### MG-6C (EntropyLedger)
- Test: 4 campos, `SparseSet`, `Reflect`, `Copy`.
- Test: `exergy_efficiency` con `initial_exergy = 0.0` → `0.0` (no NaN).
- Test: `entropy_rate = entropy_production(total_heat, T_core)` — verificar consistencia con MG-1.

### MG-6D (Sistema — integración)
- Test: app mínima con 1 entidad (3 nodos, cadena lineal) → `EntropyLedger` insertado con valores consistentes con `evaluate_metabolic_chain` manual.
- Test: idempotente — si grafo no cambia entre ticks, `EntropyLedger` no muta (change detection no dispara).
- Test: entidad sin `MetabolicGraph` → no recibe `EntropyLedger`.

### MG-6F (distribute_to_children)
- Test: `distribute_to_children(100.0, 500.0, &[(0, 60.0), (1, 40.0)])` → edge 0: (60, 300), edge 1: (40, 200).
- Test: `distribute_to_children(100.0, 500.0, &[])` → vacío.
- Test: conservación: `Σ mass = 100`, `Σ exergy = 500` dentro de ε.

### General
- ≥20 tests unitarios de conservación total.
- `cargo test --lib` sin regresión.

## Referencias

- `docs/design/MORPHOGENESIS.md` §3.2.1–3.2.3, §6 MG-6
- `docs/arquitectura/blueprint_morphogenesis_inference.md` §2–§3 (Writer, ledger derivado)
- `src/blueprint/equations/` — `exergy_balance()`, `entropy_production()` (MG-1)
- `src/layers/energy.rs` — `BaseEnergy` (1 campo: `qe`)
- `docs/sprints/MORPHOGENESIS_INFERENCE/SPRINT_MG5_ALBEDO_INFERENCE.md` — consumo de Q
