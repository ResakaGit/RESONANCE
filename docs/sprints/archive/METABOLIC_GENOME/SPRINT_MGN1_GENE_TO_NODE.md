# Sprint MGN-1 — Gene → ExergyNode Mapping

**Módulo:** `src/blueprint/equations/metabolic_genome.rs` (nuevo)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente

---

## Objetivo

Función pura que convierte un gen del `VariableGenome` en un `ExergyNode`.
El OrganRole se infiere de la posición y valor del gen — no se asigna.

---

## Diseño

### Inferencia de OrganRole desde posición génica

Los 12 OrganRoles se mapean a los 4 core biases × 3 tiers:

```
gene_index % 4 → core dimension:
  0 = growth    → Tier captor:  Root  → Tier process: Core  → Tier actuator: Fruit
  1 = mobility  → Tier captor:  Fin   → Tier process: Limb  → Tier actuator: Tentacle*
  2 = branching → Tier captor:  Leaf  → Tier process: Stem  → Tier actuator: Petal
  3 = resilience→ Tier captor:  Shell → Tier process: Thorn → Tier actuator: Bud

Tier from gene distance to core:
  distance 0 (gene 0-3): core biases — no nodo metabólico (son el InferenceProfile)
  distance 1 (gene 4-7): captor tier (in-degree 0, reciben del ambiente)
  distance 2 (gene 8-11): process tier (transforman energía)
  distance 3+ (gene 12+): actuator tier (gastan energía para función)
```

### `gene_to_exergy_node(gene_value, gene_index) → ExergyNode`

```rust
pub fn infer_role_from_gene(gene_index: usize) -> OrganRole {
    let dimension = gene_index % 4;
    let tier = (gene_index.saturating_sub(MIN_GENES)) / 4;
    // ROLE_MAP[dimension][tier.min(2)] → OrganRole
    // Deterministic, no RNG, pure from position.
}

pub fn gene_to_exergy_node(gene_value: f32, gene_index: usize) -> ExergyNode {
    let role = infer_role_from_gene(gene_index);
    ExergyNode {
        role,
        efficiency: ROLE_EFFICIENCY_FACTOR[role] * gene_value,  // gene modula η
        activation_energy: ROLE_ACTIVATION_ENERGY[role] * (1.0 - gene_value), // alto gen → baja barrera
        thermal_output: 0.0,  // computed by step_system, not at init
        entropy_rate: 0.0,    // computed by step_system
    }
}
```

**Axiom 4:** `efficiency < 1.0` siempre (ROLE_EFFICIENCY_FACTOR max = 0.95).
**Axiom 6:** Role emerge de posición, no se asigna.
**Axiom 7:** activation_energy varía con gene_value — proximity in gene space = lower barrier.

---

## Tests (TDD, escribir antes de implementar)

### Contrato
- `gene_to_node_core_genes_not_mapped` — genes 0-3 no deberían generar nodos (son core biases)
- `gene_to_node_valid_role` — todo gen ≥ 4 produce un OrganRole válido
- `gene_to_node_efficiency_bounded` — η ∈ [0, ROLE_EFFICIENCY_FACTOR[role]]

### Lógica
- `infer_role_dimension_cycles` — gene 4,8,12 todos mapeados al mismo dimension (0=growth)
- `infer_role_tier_increases` — gene 4 = tier 0 (captor), gene 8 = tier 1 (process)
- `gene_value_modulates_efficiency` — gene=1.0 → max η; gene=0.0 → η=0
- `gene_value_modulates_activation` — gene=1.0 → low E_a; gene=0.0 → high E_a
- `deterministic` — mismo input → mismo output

### Errores
- `gene_index_zero_no_panic` — gene 0 handled gracefully
- `gene_value_nan_safe` — NaN gene → clamped to 0
- `gene_value_out_of_range_clamped` — gene=-1 or 2.0 → clamped [0,1]

---

## Criterios de aceptación

- `gene_to_exergy_node` es fn pura: `(f32, usize) → ExergyNode`.
- No toca MetabolicGraph, MetabolicGraphBuilder, ni ningún system.
- Zero imports de Bevy (solo blueprint types).
- 12+ tests.
- `cargo test --lib metabolic_genome` sin regresión.

---

## Referencias

- `src/layers/metabolic_graph.rs` — ExergyNode struct
- `src/layers/organ.rs` — OrganRole enum (12 variants)
- `src/blueprint/constants/metabolic_graph_mg2.rs` — ROLE_EFFICIENCY_FACTOR, ROLE_ACTIVATION_ENERGY
- `src/blueprint/equations/variable_genome.rs` — VariableGenome, MIN_GENES, MAX_GENES
