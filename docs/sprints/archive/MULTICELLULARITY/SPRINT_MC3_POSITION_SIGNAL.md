# Sprint MC-3 — Positional Signaling: señal borde/interior

**Módulo:** `src/blueprint/equations/positional_signal.rs` (nuevo)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** MC-2

---

## Objetivo

Función pura que computa un gradiente posicional para cada célula en una colonia.
Células en el borde reciben señal ~1.0. Células en el interior reciben señal ~0.0.
Esta señal es el input para expresión diferencial (MC-4).

## Diseño

### `border_signal(cell_pos, neighbors, colony_center, colony_radius) → f32`

```rust
/// Positional signal ∈ [0, 1]. 1.0 = exposed border. 0.0 = deep interior.
///
/// Axiom 7: signal based on local neighborhood, not global knowledge.
/// Method: count neighbors in 4 cardinal directions. Fewer neighbors = more exposed.
pub fn border_signal(
    neighbor_count: u8,
    max_possible_neighbors: u8,
) -> f32 {
    // Fully surrounded = 0.0 (interior). No neighbors = 1.0 (border).
    1.0 - (neighbor_count as f32 / max_possible_neighbors.max(1) as f32)
}
```

### `positional_gradient(positions, colony_ids, alive_mask) → [f32; MAX_ENTITIES]`

```rust
/// Compute border signal for all entities in all colonies.
/// Pure: (positions, colony_ids, adjacency, alive_mask) → [signal; N]
pub fn positional_gradient(
    positions: &[[f32; 2]; MAX_ENTITIES],
    adjacency: &[[bool; MAX_ENTITIES]; MAX_ENTITIES],
    colony_ids: &[u8; MAX_ENTITIES],
    alive_mask: u64,
) -> [f32; MAX_ENTITIES]
```

## Tests

- `isolated_cell_border_signal_one` — no neighbors → signal = 1.0
- `surrounded_cell_signal_zero` — 4 neighbors → signal = 0.0
- `edge_cell_partial_signal` — 2 neighbors → signal = 0.5
- `non_colony_cell_signal_zero` — colony_id = 0 → signal = 0.0
- `gradient_symmetric` — symmetric colony → symmetric signals
- `deterministic` — same input → same output

## Criterios de aceptación

- `border_signal` es `(u8, u8) → f32`, trivially pure.
- `positional_gradient` is `(...) → [f32; N]`, stateless.
- Axiom 7: only local information (neighbor count), no global colony scan.
- 8+ tests.
