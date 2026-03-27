# Sprint EC-4 — Pool Distribution System: El Tick de Extracción Competitiva

**Módulo:** `src/simulation/metabolic/pool_distribution.rs`
**Tipo:** Sistema ECS — conecta funciones puras con el mundo Bevy.
**Onda:** B — Requiere EC-2 (componentes) + EC-3 (evaluación de extracción).
**Estado:** ⏳ Pendiente

## Objetivo

Crear el sistema central que, cada tick: (1) aplica intake a cada pool, (2) recolecta hijos por padre, (3) evalúa funciones de extracción de cada hijo, (4) escala extracciones al available, (5) distribuye energía, (6) aplica disipación, (7) aplica daño al pool (Type IV). Determinista, sin allocations en hot path.

## Responsabilidades

### EC-4A: Sistema `pool_intake_system`

```rust
/// Aplica intake al pool. Ejecuta antes de distribución.
pub fn pool_intake_system(
    mut pools: Query<&mut EnergyPool>,
) {
    for mut pool in &mut pools {
        let intake = pool.intake_rate();
        let new_pool = (pool.pool() + intake).min(pool.capacity());
        if pool.pool() != new_pool {
            pool.set_pool(new_pool);
        }
    }
}
```

- **Phase:** `Phase::MetabolicLayer`.
- **Orden:** primero en la cadena EC. `.before(pool_distribution_system)`.

### EC-4B: Sistema `pool_distribution_system`

```rust
/// Distribuye energía de pools padre a hijos según funciones de extracción.
/// Invariante: Sigma extracted <= available_for_extraction(pool).
pub fn pool_distribution_system(
    mut pools: Query<&mut EnergyPool>,
    children: Query<(Entity, &PoolParentLink, &mut BaseEnergy), Without<Dead>>,
)
```

**Algoritmo (por pool padre):**

1. **Agrupar hijos por padre.** Iterar `children`, agrupar por `link.parent`. Buffer stack-local: `ArrayVec<(Entity, ExtractionType, f32), MAX_CHILDREN_PER_POOL>`.

2. **Por cada pool padre:**
   a. Calcular `available = available_for_extraction(pool, intake=0, dissipation_rate)` (intake ya aplicado).
   b. Calcular `total_fitness = sum(primary_param)` para hijos Type III.
   c. Construir `ExtractionContext { available, pool_ratio, n_siblings, total_fitness }`.
   d. Por cada hijo: `claimed[i] = evaluate_extraction(profile_from_link, &ctx)`.
   e. `scale_extractions_to_available(&mut claimed, available)` — enforce pool invariant.
   f. Por cada hijo: `child.energy.inject(claimed[i])`.
   g. `pool.set_pool(pool.pool() - sum(claimed) - dissipation_loss(pool))`.

3. **Daño al pool (Type IV):**
   a. Para cada hijo con `ExtractionType::Aggressive`:
      `(_, damage) = evaluate_aggressive_extraction(profile, &ctx, DAMAGE_RATE_DEFAULT)`.
   b. `pool.degrade_capacity(damage)`.

4. **Limpieza de links huérfanos:**
   a. Si `pool.get(link.parent)` falla → el padre murió. Marcar link para remoción.
   b. Usar `Commands` para remover `PoolParentLink` de huérfanos.

**Orden de extracción determinista:** hijos se procesan por `Entity` index ascendente (determinista en Bevy). No hay ventaja posicional en v1 (simultaneous model). El scaling global via `scale_extractions_to_available` hace el orden irrelevante para el resultado.

### EC-4C: Sistema `pool_dissipation_system`

```rust
/// Aplica disipación obligatoria a pools sin hijos.
/// Pools con hijos ya disiparon en pool_distribution_system.
pub fn pool_dissipation_system(
    mut pools: Query<&mut EnergyPool>,
    children: Query<&PoolParentLink>,
)
```

- Para pools sin ningún hijo apuntando: aplicar `dissipation_loss` directamente.
- Pools con hijos: ya disiparon en EC-4B (paso 2g).
- **Phase:** `Phase::MetabolicLayer`, `.after(pool_distribution_system)`.

### EC-4D: Constantes

```rust
/// Máximo de hijos por pool (stack buffer).
pub const MAX_CHILDREN_PER_POOL: usize = 64;
```

### EC-4E: Registro en Pipeline

En `simulation/pipeline.rs`:

```rust
// Energy Competition — Pool Distribution
app.add_systems(FixedUpdate, (
    pool_intake_system,
    pool_distribution_system.after(pool_intake_system),
    pool_dissipation_system.after(pool_distribution_system),
).in_set(Phase::MetabolicLayer));
```

## Tácticas

- **Sin HashMap.** Agrupar hijos por padre usando sorted `ArrayVec` o iteración doble. Con `MAX_CHILDREN_PER_POOL = 64`, un buffer stack es suficiente. Si un pool tiene más hijos que el buffer, los extra se procesan en el siguiente tick (graceful degradation, no crash).
- **Una iteración, dos queries.** El sistema itera `children` para recolectar, luego itera por pool. No nested queries — Bevy no permite `Query<&mut X>` accedida dos veces.
- **`scale_extractions_to_available` es el enforcement.** No importa qué funciones de extracción usen los hijos ni cuánto pidan: el scaling garantiza el invariante. La lógica es trivial y probada en EC-1.
- **Daño al pool separado del extraction loop.** El daño se aplica después de la distribución para no afectar el `available` del tick actual. Efecto se manifiesta en el tick siguiente.
- **Guard orphan links.** Padre muerto → link inválido. Remover con `Commands`. No panic, no unwrap.

## NO hace

- No computa competencia entre pools (eso es EC-5).
- No verifica conservación global (eso es EC-6).
- No infiere fitness del padre (eso es EC-7).
- No crea pools ni links — eso es responsabilidad de spawn (EntityBuilder / archetypes).
- No modifica `BaseEnergy` directamente excepto via `inject` para hijos.

## Criterios de aceptación

### EC-4A (Intake)
- Test: pool con intake_rate=50, pool=1000, capacity=2000 → pool=1050 post-intake.
- Test: pool con intake_rate=50, pool=1990, capacity=2000 → pool=2000 (clamped).

### EC-4B (Distribution)
- Test: 1 padre (pool=1000), 3 hijos Type I (Proportional) → cada hijo recibe ~333.
- Test: 1 padre (pool=1000), 2 hijos Type III (fitness 0.7 y 0.3) → reciben ~700 y ~300.
- Test: 1 padre (pool=100), 2 hijos Type II (capacity=80 cada uno) → scaling: reciben 50 cada uno.
- Test: Type IV hijo → pool capacity degrada en siguiente tick.
- Test: padre muere (despawn) → hijos pierden link, sin panic.
- Test: pool invariant: `sum(extracted) <= available` post-distribución. Verificar con 10 combinaciones.

### EC-4C (Dissipation)
- Test: pool sin hijos, pool=1000, dissipation=0.01 → pool=990.
- Test: pool con hijos: disipación ya aplicada en distribution (no doble-dip).

### EC-4E (Pipeline)
- Test: app mínima con `MinimalPlugins` + sistemas EC → corre un update sin crash.
- Test: orden: intake → distribution → dissipation (verificar con Bevy schedule).

### General
- `cargo test --lib` sin regresión.
- Pool invariant nunca violado en 1000 ticks con escenario de stress (10 pools, 50 hijos).
- Sin allocations en hot path (no `Vec`, no `HashMap`).

## Referencias

- Blueprint Energy Competition Layer §1.2 (Pool Conservation), §7 (Invariants)
- `src/simulation/metabolic/trophic.rs` — Precedente de sistema con multiple queries y energy transfer
- `src/layers/energy.rs` — `BaseEnergy::inject()`, `EnergyOps`
- EC-1 (`pool_next_tick`, `available_for_extraction`, `scale_extractions_to_available`)
- EC-2 (`EnergyPool`, `PoolParentLink`, `ExtractionType`)
- EC-3 (`evaluate_extraction`, `ExtractionContext`)
