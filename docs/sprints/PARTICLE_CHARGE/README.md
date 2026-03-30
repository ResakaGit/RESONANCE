# Track: PARTICLE_CHARGE — Átomos emergentes de axiomas termodinámicos

Partículas con carga +/- que se atraen/repelen por Coulomb.
"Átomos" y "elementos" emergen de la combinatoria de cargas × frecuencias.
Pre-requisito: scale (>64 entities) + spatial acceleration + continuous forces.

**Invariante:** Ningún elemento se programa. Emergen de carga + frecuencia + energía mínima.

---

## Pre-requisitos (infraestructura)

| Sprint | Nombre | Esfuerzo | Entregable |
|--------|--------|----------|------------|
| [PC-0](SPRINT_PC0_SCALE.md) | Entity Scale | 2 sem | alive_mask u64→bitset, MAX_ENTITIES 64→1024 |
| [PC-1](SPRINT_PC1_SPATIAL_TREE.md) | Spatial Acceleration | 2 sem | Barnes-Hut O(N log N) en `equations/spatial_tree.rs` |
| [PC-2](SPRINT_PC2_CONTINUOUS_FORCES.md) | Continuous Forces | 1 sem | Force accumulator per entity, applied cada tick |

## Core (partículas)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [PC-3](SPRINT_PC3_CHARGE_LAYER.md) | Charge Layer | 1 sem | PC-0,1,2 | L-1: `ParticleCharge { charge: f32, mass: f32 }` |
| [PC-4](SPRINT_PC4_COULOMB.md) | Coulomb Force | 1 sem | PC-3 | `coulomb_force(q1, q2, r)` + Lennard-Jones |
| [PC-5](SPRINT_PC5_BONDING.md) | Emergent Bonding | 2 sem | PC-4 | Pares estables = "átomos". Tripletes = "moléculas". |
| [PC-6](SPRINT_PC6_ELEMENT_EMERGENCE.md) | Element Emergence | 2 sem | PC-5 | Observar cuántos "elementos" distintos emergen |

## Dependency chain

```
PC-0 (scale) ─┐
PC-1 (spatial) ┤→ PC-3 (charge) → PC-4 (Coulomb) → PC-5 (bonding) → PC-6 (elements)
PC-2 (forces) ─┘
```

## Arquitectura de archivos

```
src/blueprint/
├── equations/
│   ├── spatial_tree.rs          ← PC-1: Barnes-Hut tree (pure math, no Bevy)
│   ├── coulomb.rs               ← PC-4: coulomb_force, lennard_jones, force_accumulate
│   └── emergent_bonding.rs      ← PC-5: bond_energy, stable_pair_detection
├── constants/
│   └── particle_charge.rs       ← PC-3: charge constants derivadas de 4 fundamentals
src/batch/
├── arena.rs                     ← PC-0: MAX_ENTITIES → 1024, alive_mask → bitset
├── systems/
│   ├── particle_forces.rs       ← PC-2/4: continuous force application
│   └── bonding.rs               ← PC-5: stable pair detection + bond formation
src/layers/
│   └── particle_charge.rs       ← PC-3: ParticleCharge component (Bevy, L-1)
```

## Axiomas

| Axioma | Cómo aplica |
|--------|-------------|
| 1 | Carga ES energía. `charge = qe_polarity`. |
| 2 | Σ charge conservada (par neutro = charge_total = 0). |
| 3 | Cargas opuestas compiten por pairing. |
| 4 | Bonding disipa energía (bond_energy < sum of free particles). |
| 5 | Charge nunca se crea. Solo se redistribuye. |
| 6 | Elementos emergen. Zero tabla periódica hardcoded. |
| 7 | Coulomb: `F ∝ 1/r²`. Distancia atenuación estricta. |
| 8 | Charge × frequency = "identidad elemental". Interference determina compatibilidad. |

## Constantes derivadas

| Constante | Derivación |
|-----------|-----------|
| `COULOMB_SCALE` | `1.0 / DENSITY_SCALE` — normalización de fuerza al grid |
| `BOND_ENERGY_THRESHOLD` | `DISSIPATION_SOLID × 200` — cuándo un par es estable |
| `LENNARD_JONES_SIGMA` | `1.0 / DENSITY_SCALE` — tamaño de partícula |
| `LENNARD_JONES_EPSILON` | `DISSIPATION_SOLID × 100` — profundidad del pozo |

## Esfuerzo total: ~11 semanas, ~2000 LOC, ~150 tests
