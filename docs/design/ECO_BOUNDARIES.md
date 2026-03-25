# BLUEPRINT — Eco-Boundaries: Topologia de Fronteras y Evaluacion Diferida

---

## 1. Objetivo

Crear un sistema topologico basado en "Fronteras de Contexto" que gestiona variables ambientales
(presion, viscosidad, temperatura base) mediante evaluacion diferida (lazy evaluation).

```text
El 90% del espacio esta en equilibrio. Computar contexto ahi es redundante.
Solo las fronteras importan. Solo ahi se calcula.
```

Eco-Boundaries clasifica el espacio del EnergyFieldGrid (V7) en zonas estables y fronteras,
cacheando contexto por zona y computando gradientes solo donde las zonas transicionan.

---

## 2. Herencia obligatoria

Eco-Boundaries hereda sin excepciones:

- Pipeline: `Input -> PrePhysics -> Physics -> Reactions -> PostPhysics`
- Ecuaciones puras en `src/blueprint/equations.rs` — NO se modifican.
- Arquitectura por capas ortogonales (Capas 0-13) — NO se agrega capa.
- AmbientPressure (Capa 6) sigue existiendo como componente Type B en biomas.
- EnergyFieldGrid (V7) es la fuente de verdad — Eco-Boundaries es un derivado.

Eco-Boundaries NO es una capa. Es un **Resource + SystemParam** que lee el campo energetico
y lo proyecta como contexto optimizado. Analogico a `PerceptionCache` o `SpatialIndex`.

---

## 3. Principios de diseno

1. **Lazy-first**: el `EcoBoundaryField` solo se recalcula cuando el `EnergyFieldGrid` cambia (generation counter). Si nada cambio, zero costo.
2. **Interior = O(1)**: una entidad en el centro de una zona lee valores cacheados. Un lookup, sin matematica.
3. **Frontera = computo real**: solo las entidades en fronteras ejecutan interpolacion de gradientes. Es el unico caso donde el costo es proporcional a la complejidad.
4. **Void = skip**: zonas sin energia (alta atmosfera, vacio) desactivan reglas enteras (catalysis, conveccion). Ahorro por omision.
5. **Emergente**: las zonas no se definen manualmente — emergen del estado del campo energetico. Cambian naturalmente con propagacion, disipacion, y clima.
6. **Stateless en interfaz**: `ContextLookup` es un SystemParam sin estado. Recibe posicion, retorna contexto.
7. **Climate como offset**: las estaciones modifican los valores base de zona, no las reglas. Es un multiplicador, no una logica nueva.

---

## 4. Tabla de modulos

| # | Modulo | Tipo | Responsabilidad | Entradas | Salidas |
|---|--------|------|-----------------|----------|---------|
| 01 | `eco/contracts` | Tipos puros | ZoneClass, BoundaryMarker, TransitionType, ContextResponse | — | tipos compartidos |
| 02 | `eco/zone_classifier` | Stateless | Funciones puras: cell state -> ZoneClass | EnergyCell | ZoneClass |
| 03 | `eco/boundary_detector` | Stateless | Funciones puras: cell + vecinos -> BoundaryMarker | EnergyCell, neighbors | BoundaryMarker |
| 04 | `eco/boundary_field` | Resource | EcoBoundaryField: markers + zone contexts cacheados | EnergyFieldGrid | zonas clasificadas |
| 05 | `eco/context_lookup` | SystemParam | ContextLookup: posicion -> contexto (cached o interpolado) | pos, EcoBoundaryField | ContextResponse |
| 06 | `eco/climate` | Resource | ClimateState: estacion, offsets, transiciones | tick count | modificadores globales |
| 07 | `eco/systems` (`eco_boundaries_system`) | Sistema ECS | Actualiza EcoBoundaryField cuando el grid cambia | EnergyFieldGrid | mut EcoBoundaryField |

---

## 5. Tipos nuevos

### 5.1 ZoneClass (enum)

Clasificacion de zona derivada del estado de la celda:

| Zona | Condicion | Contexto tipico |
|------|-----------|----------------|
| HighAtmosphere | qe < MIN_QE, altura > techo | presion 0, viscosidad 0 |
| Surface | qe >= MIN_QE, rango habitable | presion normal, viscosidad variable |
| Subaquatic | estado Liquid, densidad > umbral | presion alta, viscosidad alta |
| Subterranean | densidad > umbral, estado Solid | presion muy alta, viscosidad extrema |
| Volcanic | temp > GAS_TRANSITION, Ignis | presion alta, viscosidad baja, reactivity alta |
| Frozen | temp < SOLID_TRANSITION, Aqua/Terra | presion normal, viscosidad media |
| Void | qe ~ 0, sin materializacion | todo en 0 — reglas desactivadas |

### 5.2 BoundaryMarker (enum)

```text
Interior(zone_id: u16)              — centro de zona estable
Boundary {
  zone_a, zone_b: ZoneClass,        — las dos zonas adyacentes
  gradient_factor: f32,              — 0.0 = zona_a, 1.0 = zona_b
  transition_type: TransitionType,   — que tipo de frontera
}
```

### 5.3 TransitionType (enum)

PhaseBoundary (Solid-Liquid, etc.), DensityGradient, ElementFrontier (Ignis-Aqua, etc.), ThermalShock.

### 5.4 ContextResponse (struct)

```text
pressure: f32
viscosity: f32
temperature_base: f32
dissipation_mod: f32
reactivity_mod: f32
is_boundary: bool
zone: ZoneClass
```

### 5.5 EcoBoundaryField (Resource)

Grid de BoundaryMarkers + tabla de ZoneContext cacheados. Mismas dimensiones que EnergyFieldGrid. Se recalcula solo cuando `grid.generation` cambia.

### 5.6 ClimateState (Resource)

Estacion actual, progreso del ciclo, offsets de temperatura/precipitacion/viento. Modifica los valores base de zona gradualmente.

---

## 6. Pipeline

### 6.1 Evaluacion de contexto

```text
Entidad pide contexto → ContextLookup.context_at(pos)
  │
  ├─ pos → cell_idx (grid discretization)
  ├─ cell_idx → BoundaryMarker
  │
  ├─ Interior(zone_id)?
  │    └─ return zone_contexts[zone_id]     ← O(1) lookup
  │
  └─ Boundary { zone_a, zone_b, gradient }?
       ├─ ctx_a = zone_contexts[zone_a]
       ├─ ctx_b = zone_contexts[zone_b]
       └─ return lerp(ctx_a, ctx_b, gradient) ← interpolacion
```

### 6.2 Recomputacion del campo

```text
eco_boundaries_system (PrePhysics, despues de V7 derive_cell_state):
  if grid.generation == boundaries.last_generation:
    return                              ← skip, nada cambio

  for cell in grid.cells:
    zone = classify(cell)               ← funcion pura
    marker = detect_boundary(cell, neighbors) ← funcion pura
  aggregate zone contexts               ← promedios por zona
  apply climate modifiers               ← ClimateState offsets
  boundaries.last_generation = grid.generation
```

---

## 7. Zonas y clima

| Mecanismo | Como funciona | Resultado |
|-----------|---------------|-----------|
| Estacion | ClimateState.temperature_offset sube/baja | Zonas Frozen se expanden/contraen |
| Precipitacion | ClimateState.precipitation_factor sube | viscosidad de Surface aumenta |
| Viento | ClimateState.wind_intensity sube | disipacion en HighAtmosphere sube |
| Transicion | Lerp suave en los ultimos 20% de cada estacion | Sin cambios abruptos |
| Nucleos cambian | V7 altera EnergyFieldGrid | Zonas se reclasifican naturalmente |

---

## 8. Integracion con sistemas existentes

| Sistema | Antes | Despues |
|---------|-------|---------|
| `dissipation_system` | `rate *= matter.dissipation_multiplier()` | `rate *= matter.dissipation_multiplier() * context.dissipation_mod` |
| `movement_will_drag_system` | viscosity del bioma cercano | `context.viscosity` de ContextLookup |
| `catalysis_scan_system` | ejecuta siempre | `if context.reactivity_mod > 0` (Void skip) |
| `state_transitions_system` | temp directa | `temp + context.temperature_base` |
| `materialization_delta` | archetype de celda | archetype + BoundaryMarker para fronteras |

---

## 9. Posicion arquitectonica

- **No es una capa**: no responde pregunta nueva sobre energia. Es observador espacial.
- **No reemplaza Capa 6**: `AmbientPressure` sigue como fuente de inyeccion de energia.
- **Es Resource derivado**: como `PerceptionCache`. Descartable, recalculable.
- **Es stateless en interfaz**: `ContextLookup` recibe posicion, retorna contexto.
- **Respeta el pipeline**: se inserta en PrePhysics despues de V7, antes de Physics.
- **Integrable con Bridge Optimizer**: `ContextLookup` se puede envolver como bridge con `BridgeCache<ContextBridge>`.

---

## 10. Organizacion de codigo

```text
src/
  eco/                              ← NUEVO modulo
    mod.rs                          ← re-exports
    contracts.rs                    ← ZoneClass, BoundaryMarker, TransitionType, ContextResponse
    zone_classifier.rs              ← cell state → ZoneClass (funciones puras)
    boundary_detector.rs            ← cell + neighbors → BoundaryMarker (funciones puras)
    boundary_field.rs               ← EcoBoundaryField resource
    context_lookup.rs               ← ContextLookup SystemParam
    climate.rs                      ← ClimateState, Season
    constants.rs                    ← umbrales zonales

  simulation/
    eco_boundaries_system.rs        ← sistema que actualiza EcoBoundaryField

  assets/
    climate_config.ron              ← duracion de estaciones, offsets
    zone_config.ron                 ← umbrales de clasificacion
```

---

## 11. Trade-offs

| Decision | Valor | Costo |
|----------|-------|-------|
| Zonas emergentes (no manuales) | No hay contenido que mantener, todo derivado | Requiere field grid funcional (V7) |
| Lazy recompute por generation | Zero costo cuando grid no cambia | Delay de 1 tick cuando cambia |
| Interpolacion en fronteras | Transiciones suaves, sin pop-in | Costo proporcional a entidades en frontera |
| Void skip | Ahorro masivo en zonas vacias | Mas branching en sistemas |
| Clima como offset | Simple, predecible | No soporta fenomenos climaticos locales |

---

## 12. Riesgos y mitigacion

| Riesgo | Impacto | Mitigacion |
|--------|---------|------------|
| Grid cambia mucho (combate) → recomputo constante | Medio | Cooldown de N ticks entre recomputos |
| Fronteras demasiado estrechas (1 celda) → interpolacion brusca | Medio | Expandir fronteras a 2-3 celdas con gradiente suave |
| Void demasiado agresivo (desactiva features) | Bajo | Umbral configurable, Void parcial con reactividad reducida |
| Clima afecta gameplay PvP de forma desequilibrada | Medio | Desactivar clima en modos competitivos |

---

## 13. Sprints

Ver `docs/sprints/ECO_BOUNDARIES/README.md` para el plan completo de implementacion.

| Sprint | Entregable | Onda |
|--------|-----------|------|
| E1 | Contratos: ZoneClass, BoundaryMarker, ContextResponse | 0 |
| E2 | Clasificador zonal + EcoBoundaryField Resource | A |
| E3 | ContextLookup SystemParam | B |
| E4 | ClimateState + estaciones | A |
| E5 | Integracion con simulacion (physics, reactions) | C |
| E6 | Integracion con materializacion (arquetipos de frontera) | C |

---

## 14. Referencias

- `docs/design/BLUEPRINT.md` (modelo de capas)
- `DESIGNING.md` (filosofia de capas y 5-test)
- `docs/arquitectura/blueprint_v7.md` (EnergyFieldGrid, materializacion)
- `docs/arquitectura/blueprint_layer_bridge_optimizer.md` (patron de cache)
- `docs/design/V7.md` (V7 worldgen)
- `src/layers/pressure.rs` (Capa 6 actual)
- `src/layers/containment.rs` (ContainedIn, ContactType)
