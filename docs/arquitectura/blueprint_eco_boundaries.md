# Blueprint: Eco-Boundaries — Topologia de Fronteras y Evaluacion Diferida

> **Axioma**: La inmensa mayoria del espacio ambiental esta en equilibrio.
> Computar presion, temperatura y densidad en cada punto del mundo es redundante.
> Solo las fronteras importan.

---

## 1. El Problema

V7 introduce `EnergyFieldGrid`: una grilla 2D donde cada celda acumula energia de nucleos,
deriva temperatura, estado de materia y pureza. Los sistemas de simulacion (physics, reactions)
necesitan **contexto ambiental** (presion, viscosidad, temperatura base) para cada entidad.

Actualmente hay dos fuentes de contexto:
- **Capa 6 (AmbientPressure)**: componente estatico en biomas. Un volcan tiene `delta_qe: -5.0, viscosity: 2.0`. No cambia.
- **V7 EnergyFieldGrid**: estado dinamico por celda. Cambia cada tick por propagacion/disipacion.

**El problema**: si cada entidad consulta el grid cada tick para obtener contexto, estamos
computando ~200 entidades x 60 ticks = 12,000 lecturas de contexto por segundo. Pero la mayoria
de entidades estan en zonas estables — un arbol en un bosque lee el mismo contexto durante toda
la partida.

**Observacion clave**: el mundo se divide naturalmente en **zonas amplias** donde las variables
ambientales son uniformes (un bosque, un lago, una montania) y **fronteras estrechas** donde
las variables cambian rapidamente (la orilla del lago, la entrada de una cueva, el borde del volcan).

---

## 2. La Solucion: Clasificacion Zonal + Lazy Evaluation

### Dos regimenes de computo

```text
INTERIOR DE ZONA (90%+ del espacio):
  Entidad consulta contexto → lookup de zona → valor cacheado → O(1)
  Sin matematicas. Sin lecturas de grid. Un lookup.

FRONTERA ENTRE ZONAS (10%- del espacio):
  Entidad se acerca al borde → detectar transicion → interpolar gradiente → O(N vecinos)
  Matematica real, pero solo para las entidades que estan en la frontera.
```

### Pipeline

```text
EnergyFieldGrid (V7, cada tick)
    │
    ▼
EcoBoundaryField (Resource, recomputa solo si el grid cambio)
    │  Clasifica celdas: Interior | Frontera
    │  Agrupa en zonas contiguas
    │  Cachea valores de contexto por zona
    │
    ▼
ContextLookup (SystemParam, stateless)
    │  Entidad pide contexto por posicion
    │  Interior? → return zona.cached_context
    │  Frontera? → interpolar entre zonas adyacentes
    │
    ▼
Sistemas de simulacion (physics, reactions)
    Reciben ContextResponse con presion, viscosidad, temperatura base
```

---

## 3. No es una Capa

### Test de DESIGNING.md

| Test | Eco-Boundaries | Resultado |
|------|---------------|-----------|
| **1. Pregunta de energia** | "Donde cambia el caracter de la energia?" | Es una proyeccion, no una pregunta nueva |
| **2. Dependencia** | Lee Capas 0,1,2,4 via EnergyFieldGrid | Valida pero no bidireccional |
| **3. Tipo A o B** | Ni A ni B — es un Resource | No es componente de entidad |
| **4. Entropia** | No crea/destruye energia | Neutral |
| **5. Interferencia** | Definida POR interferencia, no participa EN ella | Observador |

**Veredicto**: Eco-Boundaries NO es una capa. Es un **Resource + SystemParam** que lee el estado
del campo energetico y lo proyecta como contexto optimizado para la simulacion.

Es analogico a:
- `PerceptionCache`: lee capas 0+2, cachea visibilidad
- `SpatialIndex`: lee transforms, cachea proximidad
- `EnergyFieldGrid`: lee nucleos, cachea estado de campo

### Relacion con Capa 6

`AmbientPressure` (Capa 6) sigue existiendo. Es un componente Type B en biomas que inyecta/drena
energia directamente. Eco-Boundaries no la reemplaza — la **subsume para lectura de contexto**:

```text
ANTES: sistema lee AmbientPressure del bioma mas cercano (query espacial cada tick)
DESPUES: sistema lee ContextLookup (zona cacheada, O(1) en interior)
```

`AmbientPressure` sigue siendo la fuente de verdad para inyeccion de energia (`delta_qe`).
`ContextLookup` es la interfaz optimizada para lectura de condiciones ambientales.

---

## 4. Clasificacion de Zonas

### ZoneClass (enum)

Las zonas emergen del estado del EnergyFieldGrid. No se definen manualmente:

```text
ZoneClass:
  HighAtmosphere    — qe < MIN_MATERIALIZATION_QE, temp < SOLID_TRANSITION
  Surface           — qe >= MIN_QE, temp en rango habitable, estado Solid o Liquid
  Subaquatic        — estado Liquid, densidad > umbral acuatico
  Subterranean      — densidad > umbral subterraneo, estado Solid
  Volcanic          — temp > GAS_TRANSITION, alta pureza de Ignis
  Frozen            — temp < SOLID_TRANSITION, alta pureza de Aqua/Terra
  Void              — qe ~ 0, sin materialización — reglas desactivadas
```

Cada zona tiene **valores de contexto base** derivados del promedio de sus celdas:

```text
ZoneContext:
  pressure: f32           — presion efectiva (derivada de densidad promedio)
  viscosity: f32          — viscosidad del medio (derivada de estado de materia)
  temperature_base: f32   — temperatura base (promedio de la zona)
  dissipation_mod: f32    — multiplicador de disipacion
  reactivity_mod: f32     — multiplicador de reactividad quimica
```

### Techo Atmosferico

El `Void` es un caso especial: por encima del techo atmosferico, las reglas ambientales se
simplifican drasticamente:
- Presion = 0
- Viscosidad = 0 (sin resistencia)
- Reactividad = 0 (sin oxidacion)
- Disipacion = minima (la energia se conserva en el vacio)

Esto ahorra computo: entidades en el Void no necesitan calculos de drag, conveccion, ni catálisis.

---

## 5. Deteccion de Fronteras

### Algoritmo

Una celda es **frontera** si al menos uno de sus 8 vecinos tiene una `ZoneClass` distinta:

```text
Para cada celda (x, y):
  mi_zona = classify(cell[x][y])
  para cada vecino (nx, ny) en los 8 adyacentes:
    vecino_zona = classify(cell[nx][ny])
    si mi_zona != vecino_zona:
      marcar como Frontera(mi_zona, vecino_zona)
      break
  si ningún vecino difiere:
    marcar como Interior(mi_zona)
```

### BoundaryMarker (enum)

```text
BoundaryMarker:
  Interior(zone_id: u16)                     — en el centro de una zona estable
  Boundary {
    zone_a: ZoneClass,                       — zona de este lado
    zone_b: ZoneClass,                       — zona del otro lado
    gradient_factor: f32,                    — 0.0 = puramente zona_a, 1.0 = puramente zona_b
    transition_type: TransitionType,         — Phase, Density, Element, Thermal
  }
```

### TransitionType

```text
TransitionType:
  PhaseBoundary      — cambio de estado de materia (Solid-Liquid, Liquid-Gas, etc.)
  DensityGradient    — cambio de clase de densidad (Low-Medium, Medium-High)
  ElementFrontier    — cambio de elemento dominante (Ignis-Aqua, Terra-Umbra)
  ThermalShock       — gradiente termico extremo (>50 grados entre vecinos)
```

---

## 6. ContextLookup SystemParam

### Diseno

`ContextLookup` es un SystemParam stateless que los sistemas usan en lugar de consultar
AmbientPressure directamente:

```text
ContextLookup (SystemParam):
  grid: Res<EnergyFieldGrid>
  boundaries: Res<EcoBoundaryField>
  climate: Res<ClimateState>

  fn context_at(&self, pos: Vec2) -> ContextResponse:
    cell_idx = grid.world_to_cell(pos)
    marker = boundaries.markers[cell_idx]

    match marker:
      Interior(zone_id):
        return boundaries.zone_contexts[zone_id]  ← O(1) lookup

      Boundary { zone_a, zone_b, gradient_factor, .. }:
        ctx_a = boundaries.zone_contexts[zone_a]
        ctx_b = boundaries.zone_contexts[zone_b]
        return lerp(ctx_a, ctx_b, gradient_factor)  ← interpolacion
```

### ContextResponse

```text
ContextResponse:
  pressure: f32
  viscosity: f32
  temperature_base: f32
  dissipation_mod: f32
  reactivity_mod: f32
  is_boundary: bool            — para sistemas que necesitan saber
  zone: ZoneClass              — la zona actual (o la dominante en frontera)
```

### Integracion con Bridge Optimizer

`ContextLookup` se puede envolver con el Bridge Optimizer:

```text
BridgedContextLookup (SystemParam):
  inner: ContextLookup
  config: Res<BridgeConfig<ContextBridge>>
  cache: ResMut<BridgeCache<ContextBridge>>

  fn context_at(&self, pos: Vec2) -> ContextResponse:
    // Normalizar posicion a celda (ya es discreto por el grid)
    // Lookup en cache por cell_idx
    // Miss? → inner.context_at(pos) → cache insert
```

Dado que el grid ya discretiza el espacio, el cache hit rate sera ~99% para entidades
que no se mueven y ~90%+ para entidades moviles (muchas celdas son Interior).

---

## 7. ClimateState — Ciclos y Estaciones

### Resource global

```text
ClimateState (Resource):
  season: Season                    — Spring, Summer, Autumn, Winter
  cycle_progress: f32               — 0.0-1.0 dentro de la estacion actual
  temperature_offset: f32           — modificador global de temperatura
  precipitation_factor: f32         — afecta viscosidad en Surface
  wind_intensity: f32               — afecta disipacion en HighAtmosphere
  season_duration_ticks: u32        — ticks por estacion
  current_tick: u32
```

### Como afecta las zonas

`ClimateState` modifica los `ZoneContext` base de cada zona:

```text
zona.temperature_base += climate.temperature_offset
zona.viscosity *= climate.precipitation_factor      (lluvia = mas viscosidad)
zona.dissipation_mod *= climate.wind_intensity      (viento = mas disipacion)
```

Estos modificadores se aplican cuando se recalcula el `EcoBoundaryField`.
No se aplican per-entity ni per-tick — son parte del cache de zona.

### Transicion entre estaciones

La transicion es gradual (lerp entre estaciones) para evitar cambios abruptos:

```text
if estamos en transicion (cycle_progress > 0.8):
  factor = (cycle_progress - 0.8) / 0.2    — 0.0 a 1.0 en el ultimo 20%
  offsets = lerp(current_season_offsets, next_season_offsets, factor)
```

---

## 8. Integracion con V7 Materializacion

### Arquetipos de frontera

Las fronteras pueden materializar arquetipos especiales:

| Tipo de frontera | Arquetipo visual | Efecto |
|-----------------|-----------------|--------|
| PhaseBoundary (Solid-Liquid) | Orilla, costa, charco | Particulas de transicion |
| PhaseBoundary (Liquid-Gas) | Vapor, niebla, fumarola | Emisión visual |
| ElementFrontier (Ignis-Aqua) | Obsidiana, vapor corrosivo | Zona de inestabilidad |
| ElementFrontier (Terra-Umbra) | Tierra corrupta, raices oscuras | Mezcla visual |
| ThermalShock | Grietas, distorsion de calor | VFX de gradiente |
| DensityGradient (High-Low) | Pendiente, acantilado | Geometria emergente |

### Integracion con materialization_delta_system

```text
Al materializar celda en frontera:
  1. Derivar archetype base normal (V7 reglas existentes)
  2. Si BoundaryMarker != Interior:
     a. Modificar archetype segun TransitionType
     b. Agregar componente BoundaryVFX con tipo de transicion
     c. El render system usa BoundaryVFX para efectos visuales
```

---

## 9. Integracion con Sistemas Existentes

### dissipation_system (physics.rs)

```text
ANTES: rate *= matter.dissipation_multiplier()
DESPUES: rate *= matter.dissipation_multiplier() * context.dissipation_mod
```

### movement_will_drag_system (physics.rs)

```text
ANTES: drag = equations::drag_force(viscosity, density, velocity)
       donde viscosity viene del bioma mas cercano
DESPUES: drag = equations::drag_force(context.viscosity, density, velocity)
         donde context viene de ContextLookup.context_at(pos)
```

### catalysis_scan_system (reactions.rs)

```text
ANTES: ejecuta catálisis siempre
DESPUES: if context.reactivity_mod > 0.0: ejecuta catálisis
         En Void (reactivity = 0), skip completo → ahorro significativo
```

### state_transitions_system (reactions.rs)

```text
ANTES: temp = PhysicsOps.temperature(entity)
DESPUES: temp = PhysicsOps.temperature(entity) + context.temperature_base
         Esto permite que la zona modifique el punto de equilibrio termico
```

---

## 10. Pipeline Completo

```text
Phase::ThermodynamicLayer (cadena worldgen en prephysics.rs, antes de containment):
  1. propagate_nuclei_system      (V7 — actualiza EnergyFieldGrid)
  2. dissipate_field_system       (V7 — aplica Segunda Ley)
  3. derive_cell_state_system     (V7 — temp, state, purity por celda)
  4. eco_boundaries_system        (clasifica zonas, detecta fronteras)
  5. … materialization_delta_system, flush visual, etc.
  6. perception_system            (otra subcadena en pipeline.rs, tras motor/overlays)

Phase::AtomicLayer:
  dissipation, movement, drag pueden consumir contexto vía ContextLookup / eco

Phase::ChemicalLayer:
  catalysis usa context.reactivity_mod como gate
  state_transitions usa context.temperature_base como offset

Update (visual derivation):
  visual_derivation / shape inference pueden usar ZoneClass para color (no es PostPhysics)
```

---

## 11. Recomputacion Lazy del EcoBoundaryField

El EcoBoundaryField NO se recalcula cada tick. Se recalcula solo cuando el EnergyFieldGrid
cambio significativamente:

```text
eco_boundaries_system:
  if grid.generation == boundaries.last_generation:
    return    ← nada cambio, skip completo

  // Recomputar zonas y fronteras
  for cell in grid.cells:
    classify cell → ZoneClass
    detect neighbors → BoundaryMarker
  aggregate zone contexts (promedios)
  apply climate modifiers
  boundaries.last_generation = grid.generation
```

El `generation` counter del grid se incrementa solo cuando `propagate_nuclei_system` o
`dissipate_field_system` modifican celdas. Si ningun nucleo cambio y la disipacion es gradual,
el generation puede no cambiar durante muchos ticks.

**Warmup**: durante los primeros ticks (V7 warmup), el grid cambia constantemente.
Las fronteras se recalculan cada tick. Despues del warmup, se estabilizan.

---

## 12. Posicion Arquitectonica

### No es una capa

No responde una pregunta nueva sobre la energia. Es un observador espacial del campo.

### No reemplaza Capa 6

`AmbientPressure` sigue existiendo como componente Type B en biomas. Eco-Boundaries subsume
la lectura de contexto ambiental pero no la inyeccion de energia (`delta_qe`).

### Es un Resource derivado

Como `PerceptionCache`, `SpatialIndex`, `EnergyFieldGrid`. Estado descartable, recalculable,
no de gameplay.

### Es stateless en interfaz

`ContextLookup` es un SystemParam que recibe posicion → retorna contexto. No tiene estado propio.
La cache es el `EcoBoundaryField` resource.

### Respeta el pipeline

Se inserta en la cadena `Phase::ThermodynamicLayer` después de la derivación de celdas V7 y antes de materialización delta (`prephysics.rs`). No rompe el `.chain()` existente.

---

## 13. Organizacion de Codigo

```text
src/
  eco/                              ← NUEVO modulo
    mod.rs                          ← re-exports
    contracts.rs                    ← ZoneClass, BoundaryMarker, TransitionType, ContextResponse
    zone_classifier.rs              ← funciones puras: cell state → ZoneClass
    boundary_detector.rs            ← funciones puras: cell + neighbors → BoundaryMarker
    boundary_field.rs               ← EcoBoundaryField resource
    context_lookup.rs               ← ContextLookup SystemParam
    climate.rs                      ← ClimateState resource, Season enum
    constants.rs                    ← umbrales de clasificacion zonal

  simulation/
    eco_boundaries_system.rs        ← sistema que actualiza EcoBoundaryField

  assets/
    climate_config.ron              ← duracion de estaciones, offsets por estacion
    zone_config.ron                 ← umbrales de clasificacion zonal (data-driven)
```

---

## 14. Constantes de Tuning

| Constante | Valor default | Rol |
|-----------|---------------|-----|
| `SUBAQUATIC_DENSITY_THRESHOLD` | 60.0 | Densidad minima para zona subacuatica |
| `SUBTERRANEAN_DENSITY_THRESHOLD` | 120.0 | Densidad minima para zona subterranea |
| `THERMAL_SHOCK_GRADIENT` | 50.0 | Diferencia de temp para ThermalShock |
| `VOID_QE_THRESHOLD` | 1.0 | qe maxima para zona Void |
| `ATMOSPHERE_CEILING_HEIGHT` | 200.0 | Altura maxima de la atmosfera |
| `SEASON_DURATION_TICKS` | 36000 | Ticks por estacion (10 minutos a 60fps) |
| `CLIMATE_TRANSITION_WINDOW` | 0.2 | Fraccion de la estacion usada para transicion |
| `BOUNDARY_RECOMPUTE_COOLDOWN` | 10 | Ticks minimos entre recomputos de fronteras |

---

## 15. Plan de Sprints

| Sprint | Entregable | Depende de | Validacion |
|--------|-----------|------------|------------|
| E1 | Contratos: ZoneClass, BoundaryMarker, ContextResponse | — | Test: tipos compilan, serialize/deserialize |
| E2 | Clasificador zonal + EcoBoundaryField Resource | E1, V7 | Test: grid → zonas correctas, fronteras detectadas |
| E3 | ContextLookup SystemParam | E2 | Test: interior → cached O(1), frontera → interpolacion |
| E4 | ClimateState + estaciones | E1 | Test: modificadores cambian gradualmente, transicion suave |
| E5 | Integracion con simulacion | E3 | Test: dissipation, drag, catalysis usan contexto, Void skip |
| E6 | Integracion con materializacion | E2 | Test: arquetipos de frontera, VFX en transiciones |

---

## 16. Estimacion de Impacto

### Escenario: 200 entidades, 60 ticks/s, grid 100x100

**Sin Eco-Boundaries**:
- 200 entidades × 60 ticks × query espacial para contexto = 12,000 queries/s
- Cada query: buscar bioma cercano + leer AmbientPressure + derivar contexto

**Con Eco-Boundaries**:
- EcoBoundaryField recomputo: cada ~10 ticks (solo si grid cambio) = 6 recomputos/s × 10,000 celdas
- 200 entidades × 60 ticks × lookup de zona = 12,000 lookups/s (O(1) cada uno)
- ~10% entidades en frontera × interpolacion = 1,200 interpolaciones/s
- **Reduccion: ~90% del costo de contexto ambiental**

### Escenario: Void skip

- Entidades en HighAtmosphere/Void: skip completo de catalysis + conveccion
- Si 20% de entidades estan en Void → ahorro de ~20% en reactions

---

## 17. Resumen

```text
Eco-Boundaries NO cambia que se simula.
Cambia DONDE se simula con detalle.

Interior de zona: valores cacheados. O(1). Sin matematica.
Frontera: interpolacion entre zonas. Solo donde importa.
Void: reglas desactivadas. Skip completo.
Clima: modificadores globales que shiftan zonas gradualmente.

El mundo se auto-clasifica desde la energia.
Las fronteras emergen de los gradientes.
Las estaciones shiftan los gradientes.
Todo lazy. Todo derivado. Todo observable.
```
