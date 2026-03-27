# Simulation Probes — Validación de Simulación por Clase de Entidad

> La simulación debe probarse donde duele: en la composición.
> Las ecuaciones puras ya tienen 2150+ unit tests. Lo que falta es verificar que
> cuando spawneás una célula con un engine y homeostasis, **el ciclo metabólico ocurre**.

---

## 0. El problema

Las demos actuales (`demo_celula`, `demo_planta`, `demo_animal`, `demo_virus`) son visuales.
Spawnean entidades y esperan que el observador humano confirme que "algo pasa".
No hay test automatizado que verifique que la simulación produce la dinámica esperada
cuando el pipeline completo (6 fases, `FixedUpdate`) corre con entidades reales.

**Lo que existe:** unit tests en `blueprint/equations/` + integration tests por subsistema aislado.

**Lo que falta:** tests que corran el pipeline completo con arquetipos reales y verifiquen
que las propiedades emergentes ocurren — el metabolismo cicla, la planta crece,
el animal caza, el virus drena.

---

## 1. Patrón: Simulation Probe

Un **probe** es un test headless que:

1. Arma un `App` mínimo con los sistemas de simulación reales (sin render, sin window)
2. Spawnea entidades usando los mismos arquetipos que las demos (`spawn_celula`, `spawn_planta_demo`, etc.)
3. Corre N ticks via `app.update()`
4. Aserta propiedades observables sobre los componentes

```rust
fn probe_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Registrar los sistemas de simulación relevantes al dominio bajo test.
    // NO registrar render, window, camera.
    app
}

fn run_ticks(app: &mut App, n: u32) {
    for _ in 0..n { app.update(); }
}
```

**Principio:** cada probe testea **una clase de entidad** contra **un dominio de simulación**.
No mezcla. Si falla, sabés exactamente qué se rompió.

---

## 2. Probes por clase de entidad

### Célula — ciclo metabólico + homeostasis

**Qué ejercita:** `AlchemicalEngine` (L5), `Homeostasis` (L12), `GrowthBudget`, `TrophicState`.

**Sistemas bajo test:**
- `engine_processing_system` (ThermodynamicLayer)
- `dissipation_system` (AtomicLayer)
- `growth_budget_system` (MetabolicLayer)
- `trophic_satiation_decay_system` (MetabolicLayer)

**Assertions (200 ticks):**

| Tick | Propiedad | Condición |
|------|-----------|-----------|
| 0 | `BaseEnergy.qe` | `== CELULA_QE` (150.0) |
| 0 | `AlchemicalEngine.buffer_level` | `== CELULA_BUF_INIT` (60.0) |
| 50 | `buffer_level` | `> 0` — engine procesando |
| 200 | `BaseEnergy.qe` | `> 0` — célula sobrevive |
| 200 | `TrophicState.satiation` | cambió respecto a tick 0 |
| * | `qe` | nunca NaN, nunca negativo |

**Qué NO verifica:** rendering, HUD, posición visual.

---

### Planta — fotosíntesis + crecimiento

**Qué ejercita:** `IrradianceReceiver`, `NutrientProfile`, `GrowthBudget`, `OrganManifest`, `MorphogenesisShapeParams`.

**Sistemas bajo test:**
- `photosynthetic_contribution_system` (ChemicalLayer)
- `growth_budget_system` (MetabolicLayer)
- `allometric_growth_system` (MorphologicalLayer)
- `lifecycle_stage_inference_system` (MorphologicalLayer)

**Assertions (300 ticks):**

| Tick | Propiedad | Condición |
|------|-----------|-----------|
| 0 | `SpatialVolume.radius` | `== PLANTA_RADIUS` (0.25) |
| 0 | `GrowthBudget.accumulated` | `== 0` |
| 100 | `GrowthBudget.accumulated` | `> 0` — fotosíntesis alimenta growth |
| 300 | `SpatialVolume.radius` | `> PLANTA_RADIUS` — creció |
| * | `qe` | nunca NaN, nunca negativo |

**Condición previa:** la planta necesita `IrradianceReceiver.photon_density > 0` para que
`photosynthetic_contribution_system` genere qe. El probe debe inyectar irradiancia
(sea vía `NutrientFieldGrid` seed o escribiendo el componente directamente).

---

### Animal — comportamiento + cadena trófica

**Qué ejercita:** `BehavioralAgent`, `WillActuator` (L7), `TrophicConsumer`, `TrophicState`, `Homeostasis` (L12).

**Sistemas bajo test:**
- `behavioral_assessment_system`, `behavioral_decision_system` (Input)
- `will_to_velocity_system` (AtomicLayer)
- `trophic_satiation_decay_system` (MetabolicLayer)
- `trophic_herbivore_forage_system` (MetabolicLayer)

**Assertions (200 ticks):**

| Tick | Propiedad | Condición |
|------|-----------|-----------|
| 0 | `TrophicState.satiation` | `== ANIMAL_SATIATION` (0.3) |
| 50 | `satiation` | `< 0.3` — hambre crece |
| 200 | `BehaviorIntent` | no es default — AI decidió algo |
| * | `qe` | `> 0` — animal sobrevive |
| * | `qe` | nunca NaN |

**Condición previa:** spawnar plantas cerca como fuente trófica (mismo patrón que `demo_animal`).

---

### Virus — parasitismo energético

**Qué ejercita:** `AlchemicalInjector` (L8), interferencia de frecuencia, drenaje de host.

**Sistemas bajo test:**
- `catalysis_spatial_filter_system` (ChemicalLayer)
- `catalysis_math_strategy_system` (ChemicalLayer)
- `dissipation_system` (AtomicLayer)

**Assertions (100 ticks):**

| Tick | Propiedad | Condición |
|------|-----------|-----------|
| 0 | `host.qe` | `== CELULA_QE` (150.0) |
| 0 | `virus.qe` | `== VIRUS_QE` (25.0) |
| 100 | `host.qe` | `< CELULA_QE` — virus drenó |
| * | `qe` (todos) | nunca NaN, nunca negativo |

**Condición previa:** virus y host deben estar dentro del `influence_radius` del `AlchemicalInjector`.

---

### Pool — competencia energética

**Qué ejercita:** `EnergyPool`, `PoolParentLink`, `ExtractionType`, `PoolConservationLedger`.

**Sistemas bajo test:**
- `pool_intake_system`, `pool_distribution_system`, `pool_dissipation_system`
- `pool_conservation_system`, `competition_dynamics_system`
- `scale_composition_system`

**Assertions (500 ticks):**

| Tick | Propiedad | Condición |
|------|-----------|-----------|
| 0 | `pool.pool()` | `== initial` |
| 500 | `pool.pool()` | `>= 0` — no colapsa |
| 500 | `ledger.active_children()` | `> 0` — hijos siguen vivos |
| * | conservation error | `< POOL_CONSERVATION_EPSILON` |

**Nota:** estos probes ya existen en `tests/energy_competition_integration.rs`. Son el modelo
a seguir para las otras clases de entidad.

---

### Morfogénesis — fenotipos emergentes

**Qué ejercita:** `MetabolicGraph`, `MorphogenesisShapeParams`, `InferredAlbedo`, `MorphogenesisSurface`.

**Sistemas bajo test:**
- `metabolic_graph_step_system`, `entropy_constraint_system`
- `shape_optimization_system`, `albedo_inference_system`, `surface_rugosity_system`

**Assertions (13 ticks):**

| Bioma | Propiedad | Condición |
|-------|-----------|-----------|
| Acuático | `fineness_ratio` | `> 2.5` (fusiforme) |
| Acuático | `albedo` | `< 0.4` (oscuro) |
| Desierto | `albedo` | `> 0.7` (brillante) |
| Desierto | `rugosity` | `> 2.0` (radiadores) |
| Bosque | `albedo` | `∈ [0.25, 0.55]` (neutro) |

**Nota:** estos probes ya existen en `tests/morphogenesis_integration.rs` y en
`entities/archetypes/morphogenesis.rs`. Son completos.

---

## 3. Invariantes transversales

Cada probe debe verificar estos invariantes independientemente de la clase de entidad:

1. **No-NaN:** ningún `BaseEnergy.qe`, `OscillatorySignature.frequency_hz`, `SpatialVolume.radius` es NaN o Inf
2. **No-negativo:** `BaseEnergy.qe >= 0` siempre
3. **Conservation:** `total_qe(t+1) <= total_qe(t) + ε` (solo disipación puede reducir; no hay creación espontánea)

Implementar como función helper reutilizable:

```rust
fn assert_invariants(app: &App) {
    let mut q = app.world().query::<&BaseEnergy>();
    for energy in q.iter(app.world()) {
        assert!(!energy.qe().is_nan(), "NaN detected in BaseEnergy");
        assert!(energy.qe() >= 0.0, "negative qe: {}", energy.qe());
    }
}
```

---

## 4. Qué NO son los probes

- **No son tests de rendering.** No verifican meshes, colores, cámaras.
- **No son benchmarks.** No miden performance.
- **No son tests de UI/HUD.** El `DemoMetricsHud` queda fuera.
- **No son tests de input.** No simulan clicks ni pathfinding del jugador.
- **No replican las demos.** Las demos siguen existiendo para observación visual. Los probes son su versión automatizada y verificable.

---

## 5. Estado actual

| Clase | Probe | Tests | Qué verifica |
|-------|-------|-------|-------------|
| Célula | `probe_celula.rs` | 5 | satiation decay, supervivencia, engine buffer, no-NaN, 3 células |
| Planta | `probe_planta.rs` | 6 | photosynthesis +qe, growth +radius, zero irradiance, no budget, no-NaN, 2 plantas |
| Animal | `probe_animal.rs` | 5 | satiation decay, supervivencia, plantas fuente, no-NaN, BehaviorIntent |
| Virus | `probe_virus.rs` | 6 | injector composition, host/virus survival, satiation, no-NaN, escena completa |
| Pool | `energy_competition_integration.rs` | 6 | Lotka-Volterra, host collapse, conservation, Matryoshka, determinism |
| Morfogénesis | `morphogenesis_integration.rs` | 6 | fineness, albedo, rugosity por bioma |

**Total probes:** 34 tests cubriendo las 6 clases de entidad.

### Hallazgos del primer pase

1. **`allometric_growth_system` requiere `GrowthIntent`** — componente transitorio generado por
   `growth_budget_system`. Sin él, la planta no crece. El probe inyecta `GrowthIntent` manualmente
   para aislar el crecimiento de la inferencia de biomasa.
2. **Virus no drena hosts en este probe** — `catalysis_spatial_filter_system` requiere `SpellMarker`,
   que el virus no tiene. El drenaje parasítico depende de `collision_interference_system` +
   `SpatialIndex` (no registrado en este probe). El probe documenta esta dependencia.
3. **Fotosíntesis funciona con irradiancia manual** — `photon_density > 0` + `absorbed_fraction > 0`
   produce incremento de qe verificable. En producción, `irradiance_update_system` calcula estos valores
   desde nuclei + almanac.

---

## 6. Estructura de archivos

```
tests/
  probe_celula.rs          ← engine + homeostasis + satiation
  probe_planta.rs          ← photosynthesis + growth + lifecycle
  probe_animal.rs          ← behavior + trophic + will
  probe_virus.rs           ← injection + host drain
  energy_competition_integration.rs  ← (ya existe)
  morphogenesis_integration.rs       ← (ya existe)
```

Cada archivo es autocontenido: arma su `App`, spawnea sus entidades, corre sus ticks, aserta.
Sin shared harness complejo. Misma estética que `energy_competition_integration.rs`.
