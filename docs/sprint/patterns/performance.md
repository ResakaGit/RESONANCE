# Patrones de Rendimiento

Estrategias específicas para mantener el frame budget con 32+ systems nuevos.

---

## Presupuesto de Frame

Target: 60 FPS → **16.6ms** por frame en FixedUpdate.

```
Phase             Budget    Systems (current)  Systems (after sprint)
─────────────────────────────────────────────────────────────────────
Input             1.5ms     6                  11 (+5 behavioral)
ThermodynamicLayer 4.0ms    14                 17 (+3 homeostasis/thermo)
AtomicLayer       3.0ms     8                  11 (+3 locomotion)
ChemicalLayer     2.0ms     5                  5  (sin cambio)
MetabolicLayer    3.0ms     6                  10 (+4 trophic/ecology)
MorphologicalLayer 3.0ms    15                 18 (+3 morpho/repro)
─────────────────────────────────────────────────────────────────────
TOTAL             16.5ms    54                 72
```

**Cada system nuevo tiene un budget MÁXIMO de 0.2ms** con 1000 entidades.

---

## Estrategia 1: LOD Temporal (Frequency Scaling)

No todos los systems necesitan correr cada frame:

| Frecuencia | Systems | Implementación |
|------------|---------|---------------|
| **Every frame** | Movement, collision, dissipation | Normal registration |
| **Every 4 frames** | Behavior decision, sensory scan | `run_if(tick % 4 == 0)` |
| **Every 16 frames** | Reproduction, evolution, ecology census | `run_if(tick % 16 == 0)` |
| **Every 60 frames** | Carrying capacity, succession | `run_if(tick % 60 == 0)` |

```rust
fn every_n_ticks(n: u32) -> impl Fn(Res<SimulationClock>) -> bool {
    move |clock: Res<SimulationClock>| clock.tick() % n == 0
}

app.add_systems(FixedUpdate, behavior_decision_system
    .in_set(Phase::Input)
    .run_if(every_n_ticks(4)));
```

---

## Estrategia 2: Spatial Budget Cap

Queries espaciales son O(N) per query. Con M queries per frame:

```
Budget: MAX_SPATIAL_QUERIES_PER_FRAME = 512

Distribución:
  - Predation filter:   64 queries/frame (TROPHIC_SCAN_BUDGET)
  - Sensory awareness:  128 queries/frame
  - Pack coordination:  64 queries/frame
  - Reproduction scan:  32 queries/frame (low priority)
  - Reserve:           224 queries/frame
```

Cada system declara su budget como constante. El total no excede 512.

---

## Estrategia 3: Population Scaling

Los budgets por frame escalan con la población:

```rust
fn adaptive_budget(base: usize, population: usize) -> usize {
    if population < 100 { base }           // Full budget for small worlds
    else if population < 500 { base / 2 }  // Half budget
    else { base / 4 }                       // Quarter budget for large worlds
}
```

---

## Estrategia 4: Change Detection Cascading

Systems que solo necesitan correr cuando inputs cambian:

```rust
// Solo procesa entidades cuya energía O nutrientes cambiaron
pub fn growth_budget_system(
    query: Query<
        (&BaseEnergy, &NutrientProfile, &mut GrowthBudget),
        Or<(Changed<BaseEnergy>, Changed<NutrientProfile>)>,
    >,
) { ... }
```

**Ahorro estimado**: 70-90% de iteraciones en estado estable.

---

## Estrategia 5: Batch Processing con SIMD-Friendly Layout

Para operaciones vectoriales sobre muchas entidades:

```rust
// Collect into aligned buffer, process in batch
pub fn batch_locomotion_system(
    query: Query<(&Transform, &FlowVector, &AlchemicalEngine)>,
    mut results: Local<Vec<(Entity, f32)>>,
) {
    results.clear();
    results.extend(query.iter().map(|(t, f, e)| {
        let cost = equations::locomotion_energy_cost(
            e.max_buffer,
            f.velocity().length(),
            1.0,
        );
        (entity, cost)
    }));
    // Apply in second pass (avoids &mut contention)
}
```

---

## Métricas a Monitorear

```rust
// En DebugPlugin (ya existe bridge_metrics_collect_system):
// Agregar por dominio:
pub struct DomainMetrics {
    pub behavior_decisions_per_frame: u32,
    pub spatial_queries_per_frame: u32,
    pub trophic_transfers_per_frame: u32,
    pub reproductions_per_frame: u32,
    pub deaths_per_frame: u32,
}
```
