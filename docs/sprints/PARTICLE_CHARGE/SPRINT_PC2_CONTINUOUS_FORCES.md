# PC-2: Continuous Forces (force accumulator pattern)

**Track:** PARTICLE_CHARGE
**Esfuerzo:** 1 semana
**Bloqueado por:** Nada (paralelo a PC-0 y PC-1)
**Desbloquea:** PC-3 (Charge Layer)
**ADR:** ADR-020

---

## Objetivo

Establecer el patron de acumulacion de fuerzas como standard del batch simulator:
primero acumular todas las fuerzas, despues aplicar. Eliminar la aplicacion inline
que rompe simetria de Newton 3.

## Motivacion

Hoy `particle_forces` (particle_forces.rs:21) extrae particulas, acumula fuerzas
con `coulomb::accumulate_forces`, y aplica inline iterando `alive_mask`. El problema:

1. La acumulacion en `coulomb.rs` ya es correcta (Newton 3 simetrico)
2. Pero la aplicacion itera `alive_mask` bits con trailing_zeros — el mapping de
   indice de bit a indice de particula puede desfasarse si hay gaps
3. No hay validacion de que `idx < count` sea coherente con el bit position

El fix correcto es:
- Acumular fuerzas en buffer `[f32; 2] × N` indexado por SLOT (no por particula)
- Aplicar en segunda pasada: para cada slot alive, `v += (F/m) * dt`

## Caso de uso

"Quiero que la simulacion de particulas sea fisicamente correcta: la fuerza sobre
A por B es exactamente igual y opuesta a la fuerza sobre B por A, independiente
del orden de evaluacion."

## Entregables

### 1. Force buffer slot-indexed

```rust
// batch/systems/particle_forces.rs

/// Force buffer indexed by entity SLOT (not particle index).
/// Slot i corresponds to entities[i]. Zero for non-alive or non-charged.
type ForceBuffer = [[f32; 2]; MAX_ENTITIES];

pub fn particle_forces(world: &mut SimWorldFlat, strategy: ForceStrategy, dt: f32) {
    if strategy == ForceStrategy::Disabled { return; }

    // Pass 1: accumulate forces into slot-indexed buffer
    let mut forces: ForceBuffer = [[0.0; 2]; MAX_ENTITIES];
    accumulate_slot_forces(world, strategy, &mut forces);

    // Pass 2: apply F/m → Δv for each alive entity
    for slot in world.alive_mask.iter_set() {
        let mass = world.entities[slot].particle_mass.max(0.01);
        world.entities[slot].velocity[0] += (forces[slot][0] / mass) * dt;
        world.entities[slot].velocity[1] += (forces[slot][1] / mass) * dt;
    }
}
```

### 2. accumulate_slot_forces (dispatch tree vs brute)

```rust
fn accumulate_slot_forces(world: &SimWorldFlat, strategy: ForceStrategy, out: &mut ForceBuffer) {
    let (particles, slot_map, count) = extract_particles_with_slots(world);
    if count < 2 { return; }

    let forces = if count >= BRUTE_FORCE_THRESHOLD {
        spatial_tree::accumulate_forces_adaptive(&particles, count, TREE_THETA)
    } else {
        coulomb::accumulate_forces(&particles, count)
    };

    // Map particle-indexed forces back to slot-indexed buffer
    for (particle_idx, &slot) in slot_map.iter().enumerate().take(count) {
        out[slot] = forces[particle_idx];
    }
}
```

### 3. extract_particles_with_slots

```rust
/// Extract ChargedParticle array + slot mapping from world.
/// Returns (particles, slot_indices, count).
/// slot_indices[particle_idx] = entity slot in world.entities[].
fn extract_particles_with_slots(world: &SimWorldFlat) -> (
    [ChargedParticle; MAX_ENTITIES],
    [usize; MAX_ENTITIES],
    usize,
) { ... }
```

### 4. Tests

| Test | Assert |
|------|--------|
| `newton3_symmetry` | Spawn 2 charges, acumular. `F_on_A == -F_on_B` exacto |
| `newton3_three_body` | 3 charges: sum(forces) == [0, 0] |
| `force_order_invariant` | Shuffle entity order, same forces |
| `slot_mapping_with_gaps` | Alive slots [0, 5, 10, 100]: forces mapped correctamente |
| `disabled_strategy_noop` | ForceStrategy::Disabled no modifica velocidades |
| `zero_charge_zero_force` | Entities sin charge no reciben fuerza |

## Criterio de aceptacion

- [x] Fuerzas acumuladas en buffer slot-indexed antes de aplicar
- [x] Newton 3 exacto: sum(forces) = [0, 0] para cualquier configuracion
- [x] Orden de evaluacion no afecta resultado
- [x] dispatch tree/brute-force por threshold
- [x] Zero `unsafe`
- [x] Compatible con PC-0 (bitset) y PC-1 (tree)

## Axiomas respetados

| Axioma | Verificacion |
|--------|-------------|
| 5 (Conservation) | Newton 3 enforced por acumulacion simetrica + test |
| 7 (Distance) | Fuerzas decaen con distancia (Coulomb 1/r^2, LJ 1/r^7) |
