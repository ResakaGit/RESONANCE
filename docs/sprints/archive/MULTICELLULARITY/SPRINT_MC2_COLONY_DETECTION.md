# Sprint MC-2 — Colony Detection: cluster de células enlazadas

**Módulo:** `src/blueprint/equations/colony_detection.rs` (nuevo)
**Tipo:** Pure math (graph traversal), stateless, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** MC-1

---

## Objetivo

Función pura que detecta colonias (clusters de entidades conectadas por StructuralLink).
Union-Find en fixed-size arrays. No heap.

## Diseño

### `detect_colonies(adjacency, alive_mask, max_entities) → ColonyMap`

```rust
/// Colony assignment per entity. Stack-allocated Union-Find.
pub struct ColonyMap {
    pub colony_id: [u8; MAX_ENTITIES],  // entity → colony ID (0 = no colony)
    pub colony_count: u8,
    pub max_colony_size: u8,
}

/// Detect connected components in the entity graph.
/// adjacency[i][j] = true if StructuralLink between i and j.
pub fn detect_colonies(
    adjacency: &[[bool; MAX_ENTITIES]; MAX_ENTITIES],
    alive_mask: u64,
) -> ColonyMap
```

### `colony_stats(map, entities) → ColonyStats`

```rust
pub struct ColonyStats {
    pub count: u8,           // number of colonies
    pub mean_size: f32,      // average cells per colony
    pub max_size: u8,        // largest colony
    pub multicellular_rate: f32, // fraction of entities in colonies ≥ MIN_COLONY_SIZE
}
```

## Tests

- `no_links_no_colonies` — isolated entities → colony_count = 0
- `pair_linked_one_colony` — 2 linked entities → 1 colony of size 2
- `chain_of_three_one_colony` — A-B-C → 1 colony of size 3
- `two_separate_pairs_two_colonies` — A-B + C-D → 2 colonies
- `min_colony_size_respected` — pairs (size 2) not counted if MIN=3
- `dead_entities_excluded` — alive_mask filters
- `deterministic` — same input → same output
- `max_entities_no_panic` — full 64 entities, all linked

## Criterios de aceptación

- Union-Find en `[u8; MAX_ENTITIES]`, zero heap.
- `detect_colonies` es `(&adjacency, u64) → ColonyMap`, stateless.
- 8+ tests.
