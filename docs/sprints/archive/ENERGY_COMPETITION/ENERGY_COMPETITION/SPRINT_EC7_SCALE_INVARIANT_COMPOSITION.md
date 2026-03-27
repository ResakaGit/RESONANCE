# Sprint EC-7 — Scale-Invariant Composition: Fitness Inferido y Matrioska

**Módulo:** `src/blueprint/equations/energy_competition/scale.rs` + `src/simulation/metabolic/scale_composition.rs`
**Tipo:** Funciones puras + sistema ECS.
**Onda:** D — Requiere EC-5 (dinámica) + EC-6 (contabilidad).
**Estado:** ⏳ Pendiente

## Objetivo

Implementar la propiedad Matryoshka: el comportamiento de una entidad padre se **infiere** desde sus hijos. Un pool cuyas criaturas internas son eficientes tiene alto fitness a su nivel. Un pool con competencia destructiva interna tiene fitness bajo. La escala es transparente: célula → órgano → organismo → población usan el mismo modelo.

## Principio

> fitness(parent) = f(Sigma retained(children), efficiency). El padre no tiene fitness propio programado — emerge de la competencia abajo.

## Responsabilidades

### EC-7A: Fitness Inferido (funciones puras)

```rust
/// Infiere el fitness de un pool-padre desde el desempeño de sus hijos.
/// Fitness alto = hijos eficientes, poca energía desperdiciada.
/// Fitness bajo = competencia destructiva, alta disipación relativa.
pub fn infer_pool_fitness(
    total_retained: f32,
    total_dissipated: f32,
    total_extracted: f32,
    structural_complexity: f32,
) -> f32
```

- `efficiency = total_retained / max(total_extracted, EPSILON)`.
- `complexity_bonus = (structural_complexity * COMPLEXITY_FITNESS_WEIGHT).min(COMPLEXITY_CAP)`.
- `fitness = efficiency * (1.0 + complexity_bonus)`.
- Clamp: `[0.0, FITNESS_MAX]`.
- `total_retained = sum(child_qe_gained)` (energía que los hijos efectivamente retuvieron).
- `structural_complexity = active_children as f32` (proxy v1; medidas más sofisticadas post-v1).

```rust
/// Infiere la tasa de intake de un pool a partir de la eficiencia de su pipeline interno.
/// Un organismo con órganos eficientes procesa más energía del ambiente.
pub fn infer_intake_rate(
    base_intake: f32,
    internal_efficiency: f32,
) -> f32
```

- `effective_intake = base_intake * internal_efficiency.clamp(0.0, 1.0)`.
- Un padre con hijos destructivos (efficiency baja) tiene menos intake efectivo.
- Feedback loop: ineficiencia interna → menor intake → menos para distribuir → más competencia.

### EC-7B: Propagación Cross-Scale

```rust
/// Propaga el fitness inferido de un pool a su PoolParentLink si el pool es hijo de otro pool.
/// Habilita la jerarquía multi-nivel: célula → órgano → organismo → población.
pub fn propagate_fitness_to_link(
    inferred_fitness: f32,
    current_primary_param: f32,
    blend_rate: f32,
) -> f32
```

- `new_param = lerp(current_primary_param, inferred_fitness, blend_rate.clamp(0.0, 1.0))`.
- `blend_rate` controla qué tan rápido el fitness inferido reemplaza al fitness fijo.
- `blend_rate = 0.0` → fitness fijo (no se infiere). `blend_rate = 1.0` → fitness 100% inferido.
- Permite transición gradual sin saltos.

```rust
/// Clasifica el régimen competitivo de un pool desde sus métricas.
pub fn classify_competitive_regime(
    competition_intensity: f32,
    health_status: PoolHealthStatus,
    active_children: u16,
) -> CompetitiveRegime
```

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum CompetitiveRegime {
    /// Recursos abundantes, poca competencia.
    Abundance,
    /// Competencia moderada, equilibrio posible.
    Contested,
    /// Competencia intensa, dominancia emergiendo.
    Dominance,
    /// Recursos insuficientes, colapso inminente.
    Scarcity,
}
```

- Basado en `competition_intensity` (EC-5) + `health_status`.
- Exhaustive match.

### EC-7C: Sistema `scale_composition_system`

```rust
/// Infiere fitness de pools-padre y propaga a links jerárquicos.
pub fn scale_composition_system(
    pools: Query<(Entity, &EnergyPool, &PoolConservationLedger), Without<Dead>>,
    children: Query<(&PoolParentLink, &BaseEnergy)>,
    mut parent_links: Query<&mut PoolParentLink>,
    diagnostics: Query<&PoolDiagnostic>,
)
```

**Algoritmo por pool:**

1. Leer `PoolConservationLedger` → obtener `total_extracted`, `total_dissipated`, `active_children`.
2. Computar `total_retained` = sum de qe ganado por hijos este tick.
3. `fitness = infer_pool_fitness(retained, dissipated, extracted, complexity)`.
4. Si esta entidad pool tiene su propio `PoolParentLink` (es hija de otro pool):
   a. `new_param = propagate_fitness_to_link(fitness, link.primary_param, FITNESS_BLEND_RATE)`.
   b. Guard: `if link.primary_param != new_param { link.primary_param = new_param; }`.
5. Opcionalmente: escribir `CompetitiveRegime` en `PoolDiagnostic` (si se extiende) o en componente separado.

- **Phase:** `Phase::MetabolicLayer`, `.after(pool_conservation_system)`.
- No modifica pools ni BaseEnergy. Solo propaga fitness inferido hacia arriba.

### EC-7D: Constantes

```rust
pub const COMPLEXITY_FITNESS_WEIGHT: f32 = 0.05;      // Bonus por complejidad (por hijo)
pub const COMPLEXITY_CAP: f32 = 0.3;                   // Máximo bonus de complejidad
pub const FITNESS_MAX: f32 = 2.0;                      // Fitness máximo absoluto
pub const FITNESS_BLEND_RATE: f32 = 0.1;               // Rate de blend por tick
```

## Tácticas

- **Bottom-up, no top-down.** El fitness se calcula desde abajo y se propaga hacia arriba. Nunca al revés. Un padre no "decide" ser eficiente — sus hijos determinan eso.
- **Blend rate evita oscilaciones.** Sin blend, el fitness cambia bruscamente cada tick → oscilaciones. Con blend = 0.1, el fitness se actualiza 10% por tick → convergencia suave.
- **v1: un solo nivel.** El sistema procesa pools de un nivel. Multi-nivel funciona naturalmente porque el sistema corre en iteración sobre todos los pools: los pools "hoja" se procesan primero (no tienen PoolConservationLedger), y los pools "intermedios" leen el fitness ya inferido de sus hijos-pool. Orden determinista por Entity index.
- **No requiere schedule especial.** Un solo paso por tick es suficiente para propagar un nivel. Multi-nivel converge en N ticks (N = profundidad de la jerarquía). Para jerarquías de 3-4 niveles, convergencia en 3-4 ticks ≈ imperceptible.

## NO hace

- No modifica la distribución de energía (eso es EC-4).
- No modifica pools directamente (solo links vía propagación).
- No implementa migración/re-parenting entre niveles.
- No implementa feedback de fitness a comportamiento (eso sería un sistema de gameplay/behavior).

## Criterios de aceptación

### EC-7A (Fitness inferido)
- Test: `infer_pool_fitness(800, 100, 1000, 3.0)` → efficiency=0.8, complexity=0.15, fitness=0.92.
- Test: `infer_pool_fitness(0, 500, 500, 2.0)` → fitness=0.0 (todo disipado).
- Test: `infer_pool_fitness(900, 50, 1000, 0.0)` → fitness=0.9 (sin complexity bonus).
- Test: fitness siempre en `[0, FITNESS_MAX]`.
- Test: `infer_intake_rate(100.0, 0.8)` = 80.0.
- Test: `infer_intake_rate(100.0, 0.0)` = 0.0.

### EC-7B (Propagación)
- Test: `propagate_fitness_to_link(0.9, 0.5, 0.1)` = `0.5 + 0.1 * (0.9 - 0.5) = 0.54`.
- Test: `propagate_fitness_to_link(0.9, 0.5, 0.0)` = 0.5 (sin blend).
- Test: `propagate_fitness_to_link(0.9, 0.5, 1.0)` = 0.9 (blend total).
- Test: `classify_competitive_regime(0.1, Healthy, 5)` = `Abundance`.
- Test: `classify_competitive_regime(0.8, Stressed, 10)` = `Dominance`.
- Test: `classify_competitive_regime(0.5, Collapsing, 8)` = `Scarcity`.

### EC-7C (Sistema)
- Test: app mínima, pool con 3 hijos → `PoolParentLink.primary_param` actualizado.
- Test: pool sin PoolParentLink propio → no se propaga (es pool raíz).
- Test: blend: 10 ticks → primary_param converge hacia fitness inferido.
- Test: guard: si fitness no cambia, link no muta.

### Escala
- Test: jerarquía 2 niveles (pool-raíz → pool-intermedio → hijos-hoja). Tras 20 ticks, pool-raíz refleja eficiencia de hijos-hoja.
- Test: determinismo — misma jerarquía, mismos inputs, mismos resultados.

### General
- `cargo test --lib` sin regresión.
- >=15 tests unitarios.

## Referencias

- Blueprint Energy Competition Layer §4 (Scale Invariance), §4.2–§4.3
- `src/blueprint/equations/ecology/` — `evolution_aggregate_fitness()` como precedente
- `src/simulation/metabolic/morphogenesis.rs` — Patrón de sistema que infiere propiedades
- EC-5 (`competition_intensity`, `PoolHealthStatus`, `PoolDiagnostic`)
- EC-6 (`PoolConservationLedger` — datos de entrada para inferencia)
