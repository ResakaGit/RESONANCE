# PC-1: Spatial Acceleration (Barnes-Hut O(N log N))

**Track:** PARTICLE_CHARGE
**Esfuerzo:** 2 semanas
**Bloqueado por:** Nada (paralelo a PC-0 y PC-2)
**Desbloquea:** PC-3 (Charge Layer)
**ADR:** ADR-020

---

## Objetivo

Implementar Barnes-Hut quadtree para fuerzas Coulomb en O(N log N). Pure math
en `blueprint/equations/spatial_tree.rs`. Sin Bevy dependency.

## Motivacion

`accumulate_forces` actual es O(N^2). Con N=1024 son ~500K pares por tick a 60 Hz.
Barnes-Hut reduce a ~10K operaciones con theta=0.25.

Para N<60 el overhead del tree no compensa — se usa brute force directo. El
threshold `BRUTE_FORCE_THRESHOLD` se deriva de `DENSITY_SCALE * 3 = 60`.

## Caso de uso

"Quiero 1024 particulas cargadas a 60 Hz sin que el frame time exceda 16ms
por mundo en un solo core."

## Entregables

### 1. QuadTree (stack-allocated)

```rust
// blueprint/equations/spatial_tree.rs

const MAX_TREE_NODES: usize = 4096; // 4× MAX_ENTITIES (worst case)

pub struct QuadNode {
    pub center_of_charge: [f32; 2],  // charge-weighted centroid
    pub total_charge: f32,
    pub total_mass: f32,
    pub bounds: [f32; 4],            // [min_x, min_y, max_x, max_y]
    pub particle_idx: Option<u16>,   // leaf: Some(idx), internal: None
    pub children: [u16; 4],          // NW, NE, SW, SE (0 = empty)
}

pub struct QuadTree {
    nodes: [QuadNode; MAX_TREE_NODES],
    node_count: u16,
}
```

### 2. API publica

```rust
/// Build tree from particle positions + charges. O(N log N).
pub fn build(particles: &[ChargedParticle], count: usize, arena: [f32; 4]) -> QuadTree;

/// Compute net force on particle `idx` using tree. O(log N) per query.
/// theta: opening angle. Smaller = more precise, slower.
pub fn force_on(&self, idx: usize, particles: &[ChargedParticle], theta: f32) -> [f32; 2];

/// Compute all forces. Dispatches tree vs brute-force by count.
pub fn accumulate_forces_adaptive(
    particles: &[ChargedParticle],
    count: usize,
    theta: f32,
) -> Vec<[f32; 2]>;
```

**Nota:** `accumulate_forces_adaptive` retorna `Vec` porque el caller necesita
array de tamanho variable. Esto es aceptable — NO es un componente, es un buffer
temporal en stack del batch tick.

### 3. Constantes derivadas

```rust
// blueprint/constants/particle_charge.rs

/// Barnes-Hut opening angle. Derived: 1 - KLEIBER_EXPONENT = 0.25.
/// Conservative: only groups distant nodes (>4× distance).
pub const TREE_THETA: f32 = 1.0 - KLEIBER_EXPONENT; // 0.25

/// Below this N, use brute-force O(N^2). Derived: DENSITY_SCALE × 3 = 60.
pub const BRUTE_FORCE_THRESHOLD: usize = (DENSITY_SCALE * 3.0) as usize;
```

### 4. Tests

| Test | Assert |
|------|--------|
| `tree_build_empty` | 0 particles → empty tree |
| `tree_build_one` | 1 particle → single leaf |
| `tree_force_two_charges` | 2 opposite charges: tree force == brute force (exact) |
| `tree_force_vs_brute_100` | 100 random particles: tree forces within 5% of brute force |
| `tree_force_vs_brute_500` | 500 random: within 5% (theta=0.25) |
| `tree_conservation` | Sum of all forces ~= [0, 0] (Newton 3) |
| `tree_adaptive_dispatch` | N<60 uses brute force, N>=60 uses tree |
| `tree_deterministic` | Same input → same output (no floating point non-determinism) |

### 5. Bench

```rust
#[bench]
fn bench_forces_brute_128() { ... }    // baseline
#[bench]
fn bench_forces_tree_128() { ... }     // should be ~same or slower (overhead)
#[bench]
fn bench_forces_tree_512() { ... }     // should be ~3x faster than brute
#[bench]
fn bench_forces_tree_1024() { ... }    // should be ~8x faster than brute
```

## Criterio de aceptacion

- [x] `spatial_tree.rs` en `blueprint/equations/` (pure math, no Bevy)
- [x] Tree force within 5% of brute force para theta=0.25
- [x] Newton 3: `sum(forces) < epsilon` para cualquier configuracion
- [x] Determinista: mismo input → mismo output
- [x] Bench: tree 1024 < brute 1024 por al menos 3x
- [x] Zero `unsafe`, zero heap en QuadTree (stack-allocated)
- [x] Constantes derivadas de 4 fundamentales

## Axiomas respetados

| Axioma | Verificacion |
|--------|-------------|
| 5 (Conservation) | Newton 3 test: sum forces = 0 |
| 7 (Distance) | Tree preserva 1/r^2 con error < theta |
| 8 (Oscillatory) | Frequency alignment no afectada por tree (se calcula en bond, no en force) |
