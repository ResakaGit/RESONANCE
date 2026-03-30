# Sprint MGN-7 — Internal Catalysis: Nodos reducen activation_energy de vecinos

**Módulo:** `src/blueprint/equations/metabolic_genome.rs` (extensión)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** MGN-3

---

## Contexto

Hoy cada ExergyNode tiene `activation_energy` fija. Un nodo necesita E_a para activarse
independientemente de sus vecinos. En biología, las enzimas bajan la energía de activación
de reacciones adyacentes — un output cataliza otro proceso.

---

## Objetivo

Ecuación pura donde el `thermal_output` de un nodo reduce la `activation_energy`
de nodos conectados por edges directas. Proto-enzima: output de A → input facilitado de B.

**Axioma 3** (Competition): catalysis es interference constructiva entre nodos.
**Axioma 4** (Dissipation): catalizar cuesta — el heat usado para catalysis no está disponible como output.
**Axioma 8** (Oscillatory): catalysis es más efectiva cuando nodos tienen frecuencias similares (resonance).

---

## Diseño

### `catalytic_activation_reduction(nodes, edges, freq_alignment) → [f32; MAX_NODES]`

```rust
/// Compute catalytic reduction of activation_energy for each node.
///
/// For each edge A→B:
///   reduction_B += A.thermal_output × CATALYSIS_EFFICIENCY × freq_alignment(A, B)
///
/// A.thermal_output used for catalysis is NOT double-counted — it's consumed.
/// Axiom 4: catalysis_cost = thermal_output × CATALYSIS_FRACTION (can't catalyze for free).
/// Axiom 8: freq_alignment = exp(-|f_A - f_B|² / (2 × bandwidth²))
///
/// Returns: (effective_activation_energies[], total_catalysis_cost)
pub fn catalytic_activation_reduction(
    nodes: &[ExergyNode],
    edges: &[ExergyEdge],
    node_count: usize,
    edge_count: usize,
    node_frequencies: &[f32],  // frequency per node (from gene position)
) -> ([f32; METABOLIC_GRAPH_MAX_NODES], f32) {
    // 1. For each node, start with base activation_energy
    // 2. For each edge A→B where A.thermal_output > 0:
    //    - freq_align = gaussian(f_A, f_B, COHERENCE_BANDWIDTH)
    //    - reduction = A.thermal_output × CATALYSIS_EFFICIENCY × freq_align
    //    - effective_E_a[B] = (base_E_a[B] - reduction).max(CATALYSIS_MIN_ACTIVATION)
    // 3. Total cost = Σ (reduction × CATALYSIS_FRACTION)
}
```

### Constantes (derivadas de axiomas)

```rust
/// Fraction of thermal_output that can catalyze neighbors. Axiom 4: not free.
/// Derived: DISSIPATION_LIQUID / DISSIPATION_SOLID = 4.0 → inverse = 0.25.
pub const CATALYSIS_EFFICIENCY: f32 = DISSIPATION_SOLID / DISSIPATION_LIQUID; // 0.25

/// Fraction of catalytic benefit that is consumed (cost). Axiom 4.
/// Derived: DISSIPATION_SOLID × 4 = 0.02.
pub const CATALYSIS_COST_FRACTION: f32 = DISSIPATION_SOLID * 4.0;

/// Minimum activation energy — catalysis can't reduce to zero (thermodynamic floor).
/// Derived: DISSIPATION_SOLID × 100 = 0.5 qe.
pub const CATALYSIS_MIN_ACTIVATION: f32 = DISSIPATION_SOLID * 100.0;
```

### Propiedad: DAG preserved

La catálisis NO cambia la topología. Solo modifica `activation_energy` (un scalar por nodo).
El DAG sigue siendo acíclico. Los flows siguen siendo unidireccionales.
La catálisis es un **modifier** sobre el nodo, no un nuevo edge.

### Frequency alignment (Axiom 8)

```rust
/// Gaussian frequency alignment between two nodes.
/// Uses COHERENCE_BANDWIDTH from the 4 fundamental constants.
fn catalytic_freq_alignment(f_a: f32, f_b: f32, bandwidth: f32) -> f32 {
    let delta = (f_a - f_b).abs();
    (-delta * delta / (2.0 * bandwidth * bandwidth)).exp()
}
```

Nodos con frecuencias similares se catalizan mejor. Esto crea "vías metabólicas especializadas"
donde nodos del mismo frequency band cooperan internamente.

---

## Tests

### Contrato — Conservation (Axiom 4)
- `catalysis_has_cost` — total_cost > 0 when catalysis occurs
- `no_catalysis_no_cost` — 0 thermal_output → 0 cost
- `cost_proportional_to_reduction` — más catalysis → más cost
- `effective_ea_never_below_minimum` — E_a ≥ CATALYSIS_MIN_ACTIVATION siempre

### Lógica — Catalysis (Axiom 3)
- `thermal_output_reduces_neighbor_ea` — nodo con heat baja E_a del vecino directo
- `no_edge_no_catalysis` — nodos sin conexión directa no se catalizan
- `higher_thermal_stronger_catalysis` — más heat → más reducción
- `zero_thermal_no_reduction` — nodo frío no cataliza nada

### Lógica — Frequency alignment (Axiom 8)
- `same_frequency_max_catalysis` — f_A = f_B → alignment = 1.0
- `different_frequency_reduced_catalysis` — |Δf| = bandwidth → alignment ≈ 0.6
- `very_different_frequency_no_catalysis` — |Δf| >> bandwidth → alignment ≈ 0.0
- `frequency_alignment_symmetric` — align(A,B) = align(B,A)

### Emergencia (Axiom 6)
- `specialized_pathway_emerges` — nodos con similar freq catalizan entre sí → vía eficiente
- `mixed_frequencies_weak_catalysis` — nodos con freq dispersas → poca catalysis
- `catalysis_chain_amplifies` — A cataliza B, B cataliza C → cascada

### Determinismo
- `deterministic` — same inputs → same outputs

### Edge cases
- `single_node_no_catalysis` — 1 nodo → sin vecinos → sin reducción
- `all_zero_thermal_no_change` — graph frío → E_a sin cambio
- `nan_frequency_safe` — NaN freq → alignment = 0

---

## Criterios de aceptación

- `catalytic_activation_reduction()` es fn pura: no modifica el graph, retorna new E_a values.
- `E_a_effective ≥ CATALYSIS_MIN_ACTIVATION` siempre (thermodynamic floor).
- Cost > 0 cuando catalysis occurs (Axiom 4).
- Frequency alignment usa `COHERENCE_BANDWIDTH` (4 fundamental constants).
- DAG no se modifica — solo scalar adjustment.
- 16+ tests.
- Zero Bevy, zero heap, zero side effects.

---

## Nota arquitectónica: por qué NO feedback loops

La catálisis es **feedforward** (A→B reduce E_a de B). NO es feedback (B→A).
El DAG sigue siendo acíclico. Si queremos feedback (B produce algo que
afecta a A), eso requiere un **segundo tick** — A cataliza B en tick T,
B produce output que llega a A en tick T+1 via flujo normal.

Esto es correcto biológicamente: las cascadas enzimáticas son secuenciales
en cada ciclo metabólico, con feedback loop emergente entre ciclos.

---

## Referencias

- `src/blueprint/equations/derived_thresholds.rs` — DISSIPATION_SOLID, DISSIPATION_LIQUID, COHERENCE_BANDWIDTH
- `src/layers/metabolic_graph.rs` — ExergyNode.activation_energy, ExergyEdge
- `src/simulation/metabolic/morphogenesis.rs:step_dag` — donde se usa E_a para balance
- Michaelis-Menten (1913): enzyme kinetics — rate ∝ [S] / (K_m + [S])
