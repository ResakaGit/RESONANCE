# CT-6: Background Coarsening — Niveles no observados siguen vivos

**Esfuerzo:** M (2–3 sesiones)
**Bloqueado por:** CT-5
**ADR:** ADR-036 §D3

## Objetivo

Los niveles no observados deben seguir evolucionando a tasa reducida. El universo
no se pausa cuando mirás una proteína — los clusters siguen gravitando, las
estrellas siguen envejeciendo.

## Precondiciones

- CT-5 completado (5 niveles conectados)
- Todos los bridges funcionales

## Entregables

### E1: `coarsening.rs` — tick reducido

```rust
// src/cosmic/scales/coarsening.rs

/// Tick coarsened: aplica N ticks en 1, con dissipation proporcional.
///
/// No simula fuerzas individuales — solo agrega:
/// 1. Dissipation acumulada: qe *= (1 - diss_rate)^N
/// 2. Frecuencia drift: freq += freq_drift_rate × N
/// 3. Edad: age += N
pub fn coarse_tick(
    world: &mut SimWorldFlat,
    n_ticks: u64,
    scale: ScaleLevel,
);

/// Determinar cuántos ticks coarsened por cada tick del nivel observado.
pub fn coarsening_ratio(observed: ScaleLevel, target: ScaleLevel) -> u64;
```

**Reglas de coarsening:**

| Distancia al observado | Ratio | Ejemplo (observando S3) |
|------------------------|-------|-------------------------|
| 0 (observado) | 1:1 | S3: cada tick |
| 1 | K | S2: cada K ticks |
| 2 | K² | S1: cada K² ticks |
| 3 | K³ | S0: cada K³ ticks |
| 4 | FROZEN | S4: no corre (no instanciado) |

K = 16 (configurable, debe ser potencia de 2 para alineación con TelescopeStack).

### E2: Conservation enforcement

Después de cada coarse_tick, verificar:
- `total_qe` del nivel ≤ `total_qe` previo (Axiom 5)
- Ninguna entidad con `qe < 0`
- Suma de niveles ≤ qe total del universo (Pool Invariant global)

### E3: Integración en game loop

```rust
/// Sistema que corre en FixedUpdate, tickea niveles background.
pub fn background_coarsening_system(
    mut scale_mgr: ResMut<ScaleManager>,
    clock: Res<SimulationClock>,
) {
    let observed = scale_mgr.observed;
    for instance in &mut scale_mgr.instances {
        if instance.level == observed { continue; }
        let ratio = coarsening_ratio(observed, instance.level);
        if clock.tick_id % ratio == 0 {
            coarse_tick(&mut instance.world, ratio, instance.level);
        }
    }
}
```

## Tasks

- [ ] Crear `src/cosmic/scales/coarsening.rs`
- [ ] `coarse_tick`: dissipation + freq drift + age
- [ ] `coarsening_ratio`: cálculo de ratio por distancia
- [ ] `background_coarsening_system`: integración en FixedUpdate
- [ ] Tests:
  - `coarsening_preserves_conservation` (qe monotone decreasing)
  - `coarsening_ratio_geometric` (K^distance)
  - `frozen_levels_dont_change`
  - `coarsened_dissipation_equivalent_to_fine` (within 1% of N fine ticks)
- [ ] Benchmark: overhead de background coarsening < 1ms por frame
- [ ] 0 warnings, 0 clippy

## Criterios de aceptación

1. Nivel cosmológico sigue evolucionando mientras se mira nivel ecológico
2. Conservation verificada en cada coarse_tick
3. Coarsened dissipation produce resultado dentro de 1% de simulación fina
4. Overhead < 1ms (medido con `std::time::Instant`)
5. Frozen levels no consumen CPU
