# PC-0: Entity Scale (64→1024, bitset)

**Track:** PARTICLE_CHARGE
**Esfuerzo:** 2 semanas
**Bloqueado por:** Nada
**Desbloquea:** PC-3 (Charge Layer)
**ADR:** ADR-020

---

## Objetivo

Escalar MAX_ENTITIES de 128 a 1024. Reemplazar `alive_mask: u128` por bitset
de `[u64; 16]`. Cambiar `entity_count: u8 → u16`.

## Motivacion

128 entidades no alcanzan para atomos emergentes. Un "atomo" necesita al menos
2 particulas (proton + electron). Una "molecula" simple necesita 6-10. Para
observar emergencia de "elementos" necesitamos ~200-500 particulas por mundo.
1024 da margen para complejidad + overhead ecologico.

## Caso de uso

"Quiero correr `particle_lab` con 500 particulas cargadas y observar cuantos
tipos de moleculas estables emergen en 100 generaciones."

## Entregables

### 1. Bitset type `AliveMask`

```rust
// batch/arena.rs
const ALIVE_WORDS: usize = MAX_ENTITIES / 64; // 16

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AliveMask {
    words: [u64; ALIVE_WORDS],
}

impl AliveMask {
    pub fn set(&mut self, idx: usize) { ... }
    pub fn clear(&mut self, idx: usize) { ... }
    pub fn is_set(&self, idx: usize) -> bool { ... }
    pub fn count(&self) -> u16 { ... }           // popcount
    pub fn iter_set(&self) -> AliveIter { ... }  // yields indices
    pub fn is_empty(&self) -> bool { ... }
}
```

### 2. SimWorldFlat cambios

```rust
pub struct SimWorldFlat {
    pub entity_count: u16,        // was u8
    pub alive_mask: AliveMask,    // was u128
    pub entities: [EntitySlot; MAX_ENTITIES],  // 1024
    pub genomes: [VariableGenome; MAX_ENTITIES],
    pub codon_genomes: [CodonGenome; MAX_ENTITIES],
    pub codon_tables: [CodonTable; MAX_ENTITIES],
    // ... rest unchanged
}
```

### 3. Migrar todos los consumidores de alive_mask

Cada lugar que hace `while mask != 0 { let i = mask.trailing_zeros(); mask &= mask - 1; }`
pasa a usar `alive_mask.iter_set()`.

### 4. Tests

| Test | Assert |
|------|--------|
| `bitset_set_clear_1024` | Set/clear en posiciones 0, 63, 64, 127, 128, 1023 |
| `bitset_count_matches_alive` | popcount == entity_count siempre |
| `bitset_iter_yields_all_set` | iter_set devuelve exactamente los indices set |
| `spawn_kill_consistency_1024` | spawn 500, kill 200, alive_mask coherente |
| `world_clone_preserves_bitset` | Clone de SimWorldFlat preserva bitset |

## Criterio de aceptacion

- [x] `MAX_ENTITIES = 1024`
- [x] `alive_mask` es bitset, no u128
- [x] `entity_count` es u16
- [x] Todos los tests existentes pasan (adaptados al nuevo API)
- [x] `cargo bench --bench batch_benchmark` no regresiona >10% para N<128
- [x] Zero `unsafe`
