# Sprint EC-8 — Integration Demo: Escenario Competitivo End-to-End

**Módulo:** `src/entities/`, `assets/maps/`, `tests/`
**Tipo:** Integración, EntityBuilder, demo visual, benchmarks.
**Onda:** E — Requiere todos los sprints anteriores.
**Estado:** ⏳ Pendiente

## Objetivo

Integrar todo el track EC en un escenario jugable: múltiples pools con hijos de distintos tipos de extracción, demostrando emergencia de Lotka-Volterra, colapso de host, homeostasis regulada, y jerarquía multi-nivel. Incluye EntityBuilder API, mapa RON, y benchmarks.

## Responsabilidades

### EC-8A: EntityBuilder — Pool Support

Extender `EntityBuilder` con métodos fluent para pools y links:

```rust
impl EntityBuilder {
    /// Añade un EnergyPool a la entidad.
    pub fn with_energy_pool(
        self,
        pool: f32,
        capacity: f32,
        intake_rate: f32,
        dissipation_rate: f32,
    ) -> Self;

    /// Añade un PoolParentLink a la entidad.
    pub fn with_pool_parent(
        self,
        parent: Entity,
        extraction_type: ExtractionType,
        primary_param: f32,
    ) -> Self;
}
```

- Fluent API consistente con builder existente.
- Validación en constructores de EC-2 (clamps).

### EC-8B: Arquetipos de Pool

En `entities/archetypes.rs` (o submódulo):

```rust
/// Spawn de zona ambiental: pool puro que representa un bioma/región.
pub fn spawn_environment_pool(
    commands: &mut Commands,
    pool: f32,
    capacity: f32,
    intake_rate: f32,
    position: Vec3,
) -> Entity;

/// Spawn de organismo competitivo: entidad con BaseEnergy + PoolParentLink.
pub fn spawn_competitor(
    commands: &mut Commands,
    parent: Entity,
    extraction_type: ExtractionType,
    primary_param: f32,
    initial_qe: f32,
    position: Vec3,
) -> Entity;

/// Spawn de pool intermedio: pool que es hijo de otro pool (Matryoshka).
pub fn spawn_sub_pool(
    commands: &mut Commands,
    parent: Entity,
    extraction_type: ExtractionType,
    fitness: f32,
    pool_capacity: f32,
    intake_rate: f32,
    position: Vec3,
) -> Entity;
```

### EC-8C: Mapa Demo `competition_arena.ron`

```ron
// assets/maps/competition_arena.ron
MapConfig(
    name: "competition_arena",
    grid_size: (32, 32),
    nuclei: [
        // Pool 1: Bosque (alta intake, muchos competidores)
        Nucleus(position: (8, 8), element: Terra, radius: 6, intensity: 800),
        // Pool 2: Desierto (baja intake, pocos competidores)
        Nucleus(position: (24, 8), element: Ignis, radius: 4, intensity: 300),
        // Pool 3: Océano (media intake, competencia intensa)
        Nucleus(position: (16, 24), element: Aqua, radius: 8, intensity: 600),
    ],
)
```

- `RESONANCE_MAP=competition_arena cargo run`.
- Startup system que spawna pools y competidores sobre los núcleos.

### EC-8D: Escenario de Demo (startup system)

```rust
pub fn spawn_competition_demo(mut commands: Commands) {
    // Pool Bosque: alta intake
    let forest = spawn_environment_pool(&mut commands, 5000.0, 10000.0, 100.0, ...);

    // 3 árboles Type III (competitive), fitness variado
    spawn_competitor(&mut commands, forest, Competitive, 0.6, 300.0, ...);
    spawn_competitor(&mut commands, forest, Competitive, 0.3, 200.0, ...);
    spawn_competitor(&mut commands, forest, Competitive, 0.1, 100.0, ...);

    // 1 parásito Type IV (aggressive)
    spawn_competitor(&mut commands, forest, Aggressive, 0.4, 50.0, ...);

    // 1 regulador Type V (homeostatic)
    spawn_competitor(&mut commands, forest, Regulated, 80.0, 150.0, ...);

    // Pool Océano con sub-pool (Matryoshka)
    let ocean = spawn_environment_pool(&mut commands, 8000.0, 15000.0, 80.0, ...);
    let reef = spawn_sub_pool(&mut commands, ocean, Competitive, 0.5, 3000.0, 40.0, ...);
    spawn_competitor(&mut commands, reef, Greedy, 200.0, 100.0, ...);
    spawn_competitor(&mut commands, reef, Greedy, 150.0, 80.0, ...);
}
```

### EC-8E: Tests de Integración

En `tests/energy_competition_integration.rs`:

1. **Lotka-Volterra emergente:** Pool con Type III + Type V hijos. Correr 500 ticks. Verificar que las extracciones oscilan (no convergen a steady state ni divergen a colapso).

2. **Host collapse:** Pool con Type IV hijo, aggression alto. Correr hasta colapso. Verificar:
   - Pool capacity degrada monotónicamente.
   - Pool llega a 0.
   - Todos los hijos pierden intake post-colapso.

3. **Homeostasis:** Pool con solo Type V hijos. Correr 200 ticks. Verificar:
   - Pool se estabiliza (delta < epsilon).
   - `PoolDiagnostic.health_status = Healthy`.

4. **Multi-level Matryoshka:** Pool-raíz → sub-pool → hijos-hoja. Correr 100 ticks. Verificar:
   - Fitness de sub-pool refleja eficiencia de hijos-hoja.
   - Pool-raíz distribuye según fitness inferido.

5. **Conservación estricta:** Cualquier escenario, 1000 ticks. Verificar:
   - `conservation_error < EPSILON` en cada tick para cada pool.
   - `Sigma extracted <= available` en cada tick.

6. **Determinismo:** Mismo escenario 2 veces → mismos resultados bit a bit.

### EC-8F: Benchmark

En `benches/energy_competition_bench.rs`:

```rust
/// Benchmark: 100 pools × 10 hijos cada uno.
/// Medir: tiempo por tick del pipeline EC completo.
fn bench_pool_distribution(c: &mut Criterion);

/// Benchmark: competition_matrix para N=16.
fn bench_competition_matrix(c: &mut Criterion);
```

- Target: < 1ms para 100 pools × 10 hijos en un tick.

### EC-8G: Registro en Plugin

En `plugins/simulation_plugin.rs` (o donde corresponda):

- Registrar componentes: `EnergyPool`, `PoolParentLink`, `PoolConservationLedger`, `PoolDiagnostic`.
- Registrar sistemas en `Phase::MetabolicLayer` con ordering correcto.
- Registrar startup system condicional por mapa.

## Tácticas

- **Demo como proof-of-concept.** El escenario demuestra los 5 tipos de extracción en acción. No es contenido final de juego — es validación del framework.
- **EntityBuilder preserva API existente.** Los nuevos métodos son adiciones, no breaking changes.
- **Tests de integración son los acceptance tests del track entero.** Si los 6 escenarios pasan, el framework funciona.
- **Benchmark establece baseline.** Si el performance degrada en PRs futuros, el bench lo detecta.

## NO hace

- No implementa UI/HUD para visualizar pools (extensión visual futura).
- No implementa balance de gameplay (tuning de constantes es post-v1).
- No implementa migración/re-parenting (extensión post-v1).
- No implementa información asimétrica (todos los hijos conocen pool_ratio del padre en v1).
- No implementa resolución secuencial (v1 es simultáneo). Extensión post-v1.

## Criterios de aceptación

### EC-8A (EntityBuilder)
- Test: `EntityBuilder::new().with_energy_pool(1000, 2000, 50, 0.01)` compila y spawna correctamente.
- Test: `EntityBuilder::new().with_pool_parent(parent, Competitive, 0.5)` compila.

### EC-8B (Arquetipos)
- Test: `spawn_environment_pool` retorna Entity con `EnergyPool` + `Transform`.
- Test: `spawn_competitor` retorna Entity con `BaseEnergy` + `PoolParentLink` + `Transform`.
- Test: `spawn_sub_pool` retorna Entity con `EnergyPool` + `PoolParentLink`.

### EC-8C/D (Demo)
- Test: `RESONANCE_MAP=competition_arena cargo run` no crashea en 10 segundos.
- Test: visual: pools visibles, competidores posicionados.

### EC-8E (Integración)
- 6 escenarios pasan (detallados arriba).

### EC-8F (Benchmark)
- `cargo bench -- energy_competition` corre sin error.
- Resultado < 1ms para 100×10 escenario.

### General
- `cargo test` (lib + integration) sin regresión.
- `cargo build` sin warnings nuevos.

## Referencias

- Blueprint Energy Competition Layer §3.5 (Lotka-Volterra Emergence), §4 (Scale Invariance)
- `src/entities/builder.rs` — EntityBuilder API existente
- `src/entities/archetypes.rs` — `spawn_*` funciones existentes
- `assets/maps/` — Mapas RON existentes
- `benches/` — Benchmarks existentes
- Todos los sprints EC-1 a EC-7
