# Sprint AI: Autopoiesis Integration — cerrar los 2 gaps del audit

**ADRs:** [ADR-043](../../arquitectura/ADR/ADR-043-species-grid-as-resource.md) · [ADR-044](../../arquitectura/ADR/ADR-044-protocell-to-entity-spawn.md) · [ADR-045](../../arquitectura/ADR/ADR-045-chemistry-canonical-choice.md)
**Esfuerzo:** 2-3 semanas
**Bloqueado por:** AP-6d (calibración ratio gas/liquid — cerrado 2026-04-15)
**Desbloquea:** stack continuo partícula→cosmos; PV-7 (Hordijk-Steel); posible Sprint DL-1 (drug library) sobre base química real

## Contexto

El audit 2026-04-15 reveló dos gaps estructurales:

**Gap #1 — química AP-* vs simulador principal.**  El track AUTOPOIESIS
(`SpeciesGrid` + `ReactionNetwork` + mass-action) vive aislado en
`use_cases/experiments/autopoiesis/`.  El simulador principal usa
`AlchemicalEngine` (L5) + `AlchemicalInjector` (L8) con química qe-based
(Ax 8 resonancia).  Dos universos paralelos, sin puente.

**Gap #2 — protocell vs célula ECS.**  Un blob AP que fisiona es un
**patrón espacial** detectado.  Una `BehavioralAgent` del sim principal
es una **entity ECS** con `BaseEnergy + OscillatorySignature`.  Cuando
un blob cruza `pressure_ratio > 4`, hoy sólo se anota en
`FissionEventRecord` — no nace una célula ECS con identidad persistente.

Esto hace que el claim "stack continuo partícula→cosmos" sea actualmente
falso: el repo tiene dos stacks paralelos (ver `docs/sprints/AUTOPOIESIS/README.md`
§"Gaps estructurales" si se agrega).

Sin este sprint, cada escala del paper principal (PV-1..6) se sostiene
por separado, pero no hay narrativa técnica única que los una.

## Principio

Integración **aditiva y no-destructiva**:

1. `SpeciesGrid` se expone como `Resource` del world Bevy, actualizado
   por un sistema en `ChemicalLayer` del pipeline.  `AlchemicalInjector`
   lee concentraciones y las proyecta como qe a las celdas del
   `EnergyFieldGrid` vía Ax 8 (freq alignment).

2. Cuando `SoupSim` dispara un `FissionEventRecord`, un `Observer` spawn
   una entity con `BaseEnergy = Σ species_qe_blob` y
   `OscillatorySignature = mean_freq(products)`.  La protocell se
   convierte en célula ECS sin perder linaje (se preserva `lineage_id`
   en un nuevo componente `LineageTag`).

3. En un test dedicado, corrim ambos stacks sobre el mismo input
   (food inicial, network) y verificamos equivalencia cualitativa en
   observables clave (persistencia, k_stability, tasa de dissipación).
   Si divergen > tolerancia, ADR-045 nombra al canónico.

## Entregables

1. `SpeciesGridResource` wrappeando `SpeciesGrid` (ADR-043)
2. `species_to_qe_injection_system` en `ChemicalLayer` (ADR-043)
3. `on_fission_spawn_entity` Observer (ADR-044)
4. `LineageTag(u64)` component (ADR-044)
5. `benchmark_chemistry_equivalence` suite + ADR-045 con veredicto

## Ítems

### AI-1 · Puente química · `SpeciesGrid → AlchemicalInjector`

**Type:** feature · **Estimate:** 1 semana

#### 0. Spec
- **What:** Exponer `SpeciesGrid` como `Resource`; sistema en `ChemicalLayer` que traduce `cell.species` a qe injection usando `OscillatorySignature` de la red para alinear frecuencias con `AlchemicalInjector`.
- **Why:** Hoy la química AP-* es invisible al resto del sim.  Sin este puente, el resultado de una fisión autopoiética no afecta al `EnergyFieldGrid`.
- **Acceptance:**
  - [ ] `SpeciesGridResource` registrado en `LayersPlugin`
  - [ ] Sistema corriendo en `ChemicalLayer` phase (FixedUpdate)
  - [ ] Test integración: sembrar food en `SpeciesGrid`, correr 100 ticks, verificar `EnergyFieldGrid.total_qe > 0` en la región del spot
  - [ ] Determinismo: dos corridas misma seed → mismo total_qe inyectado
  - [ ] `cargo test --release --lib` no regresiones (4022 baseline)
- **Out of scope:** Feedback qe → species (inverso); por ahora sólo species → qe.

#### 1. Context
- Existing: `src/layers/species_grid.rs`, `src/layers/alchemical_injector.rs`, `src/simulation/pipeline.rs` (phases).
- `SoupSim` ya es stepeable; hay que wrapper-izarlo o exponer su grid.  Prefiero NO mover `SoupSim` del módulo AP — que siga vivo allí, el resource es una referencia/clone.
- Phase `ChemicalLayer` existe (ver CLAUDE.md §Pipeline).  No crear phase nueva.
- Dep: AI-2 necesita que el grid esté expuesto para poder observar fission desde ECS.

#### 2. Design
No trivial — estrategia **bottom-up**: primero el resource + getter estable, después el sistema de inyección, después el test E2E.

Contratos:
```rust
#[derive(Resource)]
pub struct SpeciesGridResource(pub SpeciesGrid);

pub fn species_to_qe_injection_system(
    grid: Res<SpeciesGridResource>,
    network: Res<ReactionNetworkResource>,
    mut energy: ResMut<EnergyFieldGrid>,
    clock: Res<SimulationClock>,
) { /* ver ADR-043 §4 */ }
```

Alternativa descartada: component-per-cell en lugar de `Resource` — rompe cache locality y NO respeta "SparseSet for transient components" (species grid es denso, no transient).

### AI-2 · Puente protocell→cell · `FissionEvent → spawn Entity`

**Type:** feature · **Estimate:** 1 semana

#### 0. Spec
- **What:** Un `Observer` que reacciona a `FissionEvent` (nuevo event) y spawna 2 entities (una por hijo) con `BaseEnergy + OscillatorySignature + LineageTag + StateScoped`.
- **Why:** Sin esto, la fisión autopoiética no produce una célula trackeable por el resto del simulador.  La protocell queda en el mundo "AP-only".
- **Acceptance:**
  - [ ] `FissionEvent { tick, parent_lineage, children_lineages: [u64; 2], centroid, mean_freq, qe_per_child }` definido y emitido por `SoupSim::step` (vía Resource intermedio por ciclo, no directo del stepper)
  - [ ] Observer `on_fission_spawn_entity` crea 2 entities con `BaseEnergy::new(qe_per_child)`, `OscillatorySignature::new(mean_freq, 0.0)`, `LineageTag(child_lineage)`
  - [ ] Entities aparecen en el `World` tras un tick del ChemicalLayer
  - [ ] Test: escenario formose+spot+calibrado → ≥1 fission → ≥2 entities con LineageTag distintos; `BehavioralAgent`-like queries las ven
  - [ ] `LineageTag` registrado en `Reflect` (debugging)
- **Out of scope:** Geometría 3D de la célula spawneada; usa punto (centroid).  `SpatialVolume` con radius derivado queda para sprint posterior.

#### 1. Context
- Existing: `src/simulation/chemical/fission.rs` (system actual que cuenta `pressure_events`), `src/use_cases/experiments/autopoiesis/soup_sim.rs` (donde vive `FissionEventRecord`).
- Hoy `FissionEventRecord` se registra en `SoupSim.fission_events: Vec<...>`.  Hay que exponerlo como event bus ECS.
- Dep: AI-1 (necesita `SpeciesGridResource` para leer freq promedio por blob).

#### 2. Design
Estrategia **top-down**: definir contrato del event + observer primero, implementar emisión después.

Contratos:
```rust
#[derive(Event, Clone, Debug)]
pub struct FissionEvent {
    pub tick: u64,
    pub parent_lineage: u64,
    pub children_lineages: [u64; 2],
    pub centroid: Vec2,
    pub mean_freq: f32,
    pub qe_per_child: f32,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct LineageTag(pub u64);
```

Observer + spawn en `on_fission_spawn_entity` con `StateScoped(GameState::Playing)`.

Alternativa descartada: spawn directo desde el system AP (sin event bus) — acopla `SoupSim` con `Commands`, rompe "stepper sin Bevy" (ADR-040 §2).

### AI-3 · Calibración · alchemical ≈ mass-action

**Type:** spike (convierte a feature al terminar) · **Estimate:** 3-5 días

#### 0. Spec
- **What:** Benchmark que corre el mismo escenario (food inicial + geometría + ticks) por dos caminos: (a) sólo `AlchemicalInjector` + `AlchemicalEngine` (sin mass-action), (b) sólo `SoupSim` mass-action (sin alchemical injector post-fisión).  Compara observables: `total_dissipated(t)`, `n_closures_final`, `n_fissions`, distribución espacial de qe al final.
- **Why:** Ambas químicas pretenden modelar el mismo fenómeno (transformación + disipación + emergencia).  Si divergen cualitativamente, el claim "un sim, dos químicas complementarias" es falso y hay que elegir canónica.
- **Acceptance:**
  - [ ] `benches/chemistry_equivalence.rs` o test `#[ignore]` que corre ambas paths sobre formose+spot+seed=0 y produce CSV con métricas
  - [ ] Métricas cuantitativas comparadas: (1) `total_dissipated` within ±20 % (tolerancia justificada en ADR-045), (2) misma tendencia monotónica de `n_closures_final` vs seed (Spearman ρ > 0.7), (3) centroides de qe dentro de ±3 celdas
  - [ ] Si pasa: ADR-045 marca "ambas químicas coexisten, mass-action canónica para vida, alchemical canónica para geoplanetario"
  - [ ] Si falla: ADR-045 elige una y marca la otra como deprecated en roadmap
- **Out of scope:** Calibración de constantes fundamentales para que coincidan numéricamente — sólo verificación cualitativa.

#### 1. Context
- Existing: `src/bin/paper_validation.rs` (patrón de comparación multi-path), `src/use_cases/experiments/autopoiesis/mod.rs::run_soup_with_network`, `src/layers/alchemical_{engine,injector}.rs`.
- Gap del audit: no hay benchmark que compare ambos stacks.  AP-5 proptest existe pero sólo valida AP.

#### 2. Design
**Spike** (1-2 días) para producir evidencia, después decidir.  El output del spike ES el contenido técnico del ADR-045.

Pseudo-código:
```rust
fn bench_both_paths(seed: u64) -> (AlchemicalMetrics, MassActionMetrics) {
    let cfg = canonical_cfg(seed);
    let a = run_alchemical_only(&cfg);  // sim principal sin AP
    let m = run_mass_action_only(&cfg); // autopoietic_lab
    (a, m)
}
```

Riesgo: las dos paths pueden requerir distintos inputs para ser "equivalentes".  Si ese mapeo no existe, el spike mismo lo documenta como finding y ADR-045 lo consigna.

## Definition of Done (sprint-wide)

- [ ] 3 ADRs firmados (043, 044, 045)
- [ ] `cargo test --release --lib` pasa (baseline 4022 tests + los agregados en este sprint)
- [ ] `cargo clippy --release --features experimental_bins --bins` sin warnings nuevos en scope
- [ ] `cargo build --release --features experimental_bins --bins` OK
- [ ] Demo ejecutable: `autopoietic_lab --spawn-entities` (nuevo flag) corre formose, fisiona, reporta en JSON las entities ECS spawneadas
- [ ] Actualizar `docs/sprints/AUTOPOIESIS/README.md` agregando sección "Integration (AI)"
- [ ] Actualizar `CLAUDE.md` §"14 ECS layers" si AI-1 expone algo nuevo a nivel de layer
- [ ] Retro con learnings documentados

## Riesgos y mitigaciones

| Riesgo | Prob | Impacto | Mitigación |
|---|---|---|---|
| Spike AI-3 revela que las dos químicas son incompatibles | Medio | Alto | ADR-045 elige canónica explícitamente — no es fracaso, es decisión |
| `SpeciesGrid` como Resource rompe determinismo por scheduler Bevy | Bajo | Medio | Test determinismo en AI-1 lo cataches; worst case usar `FixedUpdate` con phase explícita |
| Observer spawn entities cada fission produce explosión combinatoria | Bajo | Medio | Cap `MAX_ENTITIES_PER_TICK=4` + warning log; test con formose no debería generar >1 fission/tick |
| `LineageTag` componente requiere refactor de `BehavioralAgent` | Bajo | Bajo | `LineageTag` es componente nuevo independiente; `BehavioralAgent` intacto |

## Cierre del arco (post-sprint)

Cuando AI-1, AI-2, AI-3 cierran:

- El stack **partícula→cosmos** es continuo (un solo pipeline, no dos paralelos)
- PV-7 (Hordijk-Steel RAF) se puede agregar validando sobre el simulador principal, no sólo el track AUTOPOIESIS aislado
- Sprint DL-1 (drug library) puede apuntar a `LineageTag` para trackear respuesta por clon
- `docs/sintesis_patron_vida_universo.md` §10 puede referenciar el demo como "la demostración ejecutable del cap. 10" sobre el simulador principal, no sólo AP-*
