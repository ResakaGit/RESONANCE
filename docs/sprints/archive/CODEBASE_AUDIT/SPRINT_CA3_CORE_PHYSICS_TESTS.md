# Sprint CA-3 — Tests para core_physics

**Módulo:** `src/blueprint/equations/core_physics/mod.rs`
**Tipo:** Tests puros — zero cambio de producción.
**Onda:** 1 — Requiere CA-1 (compilación verde). Paralelo con CA-2.
**Estado:** ✅ Cerrado (2026-03-25) — 40 tests escritos, 0 fallos

## Objetivo

Cubrir las 13 funciones públicas de `core_physics` con tests unitarios. Este módulo implementa la matemática fundacional de toda la simulación (volumen, densidad, interferencia, disipación, drag, integración de velocidad, temperatura, transiciones de estado) y tiene **0 tests**.

## Diagnóstico

| Función | Línea | Capas que la usan | Tests |
|---------|-------|--------------------|-------|
| `sphere_volume(radius)` | 13 | L1 SpatialVolume | 0 |
| `projected_circle_area(radius)` | 22 | Rendering, LOD | 0 |
| `sphere_surface_area(radius)` | 28 | Disipación, morfogénesis | 0 |
| `density(qe, radius)` | 33 | L1→L4 (temperatura, estado) | 0 |
| `interference(freq_a, freq_b, phase_a, phase_b)` | 46 | L2→L8 (catálisis, daño) | 0 |
| `is_constructive(interference)` | 56 | L2 filtros | 0 |
| `is_destructive(interference)` | 61 | L2 filtros | 0 |
| `is_critical(interference)` | 66 | L9 MOBA crit | 0 |
| `effective_dissipation(base_rate, velocity, friction)` | 76 | L3 disipación | 0 |
| `drag_force(viscosity, density, velocity)` | 82 | L3→L6 física | 0 |
| `integrate_velocity(velocity, force, qe, dt)` | 93 | L3 movimiento | 0 |
| `equivalent_temperature(density)` | 106 | L4 transiciones | 0 |
| `state_from_temperature(temp, bond_energy)` | 115 | L4 MatterState | 0 |

## Pasos de implementación

### CA-3A: Tests de geometría (L1)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ── sphere_volume ──
    #[test]
    fn sphere_volume_unit_radius() { ... }        // 4π/3 ≈ 4.189
    #[test]
    fn sphere_volume_zero_radius_is_zero() { ... }
    #[test]
    fn sphere_volume_negative_radius_treats_as_positive_or_zero() { ... }

    // ── projected_circle_area ──
    #[test]
    fn projected_circle_area_unit_radius() { ... }  // π ≈ 3.14159
    #[test]
    fn projected_circle_area_zero_is_zero() { ... }

    // ── sphere_surface_area ──
    #[test]
    fn sphere_surface_area_unit_radius() { ... }   // 4π ≈ 12.566
}
```

### CA-3B: Tests de densidad y temperatura (L1→L4)

```rust
    // ── density ──
    #[test]
    fn density_unit_sphere_100qe() { ... }         // 100 / 4.189 ≈ 23.87
    #[test]
    fn density_zero_qe_is_zero() { ... }
    #[test]
    fn density_zero_radius_returns_f32_max() { ... } // comportamiento documentado

    // ── equivalent_temperature ──
    #[test]
    fn equivalent_temperature_positive_density() { ... }
    #[test]
    fn equivalent_temperature_zero_density_is_zero() { ... }

    // ── state_from_temperature ──
    #[test]
    fn state_solid_below_threshold() { ... }
    #[test]
    fn state_liquid_in_range() { ... }
    #[test]
    fn state_gas_in_range() { ... }
    #[test]
    fn state_plasma_above_threshold() { ... }
    #[test]
    fn state_boundary_solid_liquid() { ... }         // exactamente en el borde
```

### CA-3C: Tests de interferencia (L2)

```rust
    // ── interference ──
    #[test]
    fn interference_same_freq_same_phase_is_constructive() { ... }
    #[test]
    fn interference_same_freq_opposite_phase_is_destructive() { ... }
    #[test]
    fn interference_range_minus_one_to_one() { ... }

    // ── is_constructive / is_destructive / is_critical ──
    #[test]
    fn is_constructive_positive_interference() { ... }
    #[test]
    fn is_destructive_negative_interference() { ... }
    #[test]
    fn is_critical_above_threshold() { ... }
    #[test]
    fn is_critical_below_threshold_false() { ... }
```

### CA-3D: Tests de física (L3)

```rust
    // ── effective_dissipation ──
    #[test]
    fn effective_dissipation_zero_velocity_equals_base() { ... }
    #[test]
    fn effective_dissipation_increases_with_speed() { ... }
    #[test]
    fn effective_dissipation_non_negative() { ... }

    // ── drag_force ──
    #[test]
    fn drag_zero_velocity_is_zero() { ... }
    #[test]
    fn drag_opposes_velocity_direction() { ... }
    #[test]
    fn drag_scales_with_viscosity() { ... }

    // ── integrate_velocity ──
    #[test]
    fn integrate_velocity_zero_force_unchanged() { ... }
    #[test]
    fn integrate_velocity_positive_force_accelerates() { ... }
    #[test]
    fn integrate_velocity_zero_qe_unchanged() { ... }  // division guard
    #[test]
    fn integrate_velocity_finite_result() { ... }
```

## Naming

Formato: `<function>_<condition>_<expected>` — e.g. `sphere_volume_zero_radius_is_zero`.

## Tácticas

- **Tests de propiedad.** Además de valores concretos, verificar invariantes:
  - `sphere_volume(r) >= 0` para todo r >= 0.
  - `interference ∈ [-1, 1]`.
  - `drag_force` siempre opuesto a velocity.
  - `state_from_temperature` cubre los 4 estados exhaustivamente.
- **Leer las constantes** (`SOLID_TRANSITION`, `LIQUID_TRANSITION`, `GAS_TRANSITION`, `GAME_BOLTZMANN`, `CRITICAL_THRESHOLD`) antes de escribir tests para usar valores correctos en boundaries.
- **No mockear nada.** Son funciones puras — input directo, output directo.

## NO hace

- No modifica las funciones de `core_physics`.
- No agrega funciones nuevas.
- No toca otros módulos de equations.

## DoD

- `#[cfg(test)] mod tests` en `core_physics/mod.rs` con ≥30 tests.
- Cobertura: todas las 13 funciones públicas con al menos 2 tests cada una.
- Edge cases cubiertos: zero, negative, boundary, NaN/Inf inputs.
- `cargo test --lib -- core_physics` verde.

## Referencias

- `src/blueprint/equations/core_physics/mod.rs` — las 13 funciones
- `src/blueprint/constants/` — constantes usadas por las funciones
- `CLAUDE.md` — Test naming: `<function>_<condition>_<expected>`
