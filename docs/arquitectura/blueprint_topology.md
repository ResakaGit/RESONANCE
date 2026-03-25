# Blueprint: Sustrato Topologico — Geomorfologia Procedural del Mundo

> **Axioma**: La energia necesita un lugar donde existir.
> Antes del campo alquimico, antes de las capas, esta el terreno.
> La topologia es el sustrato fisico que modula como la energia se propaga,
> acumula y materializa.

---

## 1. El Problema

V7 genera el mundo desde nucleos de energia que propagan al `EnergyFieldGrid`.
Pero el grid es **plano** — no tiene altitud, pendiente, rios ni relieve.
Un volcan y un valle tienen la misma "forma" de propagacion. Un rio no existe
como estructura topologica — solo como frecuencia Aqua acumulada.

Esto limita:
- **Variedad de mapas**: sin relieve, todos los mapas se sienten planos.
- **Gameplay espacial**: no hay ventaja de altura, cobertura por terreno, ni rios como barreras.
- **Riqueza de biomas**: sin pendiente ni drenaje, la materializacion depende solo de energia.
- **Realismo emergente**: en la realidad, la topografia MODULA como la energia fluye.

---

## 2. Tres Preocupaciones Ortogonales

El documento `TOPOLOGY_AND_LAYERS.md` identifica correctamente tres ejes independientes:

| Preocupacion | Que modela | Donde vive |
|-------------|-----------|------------|
| **Sustrato topologico** | Altitud, pendiente, rios, deformaciones | `TerrainField` (Resource nuevo) |
| **Campo energetico** | qe, Hz, propagacion, materializacion | `EnergyFieldGrid` (V7 existente) |
| **Inferencia visual** | Color, escala, forma, emision | `EnergyVisual` + `visual_derivation` (existente) |

La topologia NO almacena qe ni Hz. El campo energetico NO almacena altitud.
La visual NO es fuente de verdad. Los tres se leen en `materialization_rules` para producir
el mundo final.

---

## 3. No es una Capa

### Test de DESIGNING.md

| Test | Topologia | Resultado |
|------|----------|-----------|
| **1. Pregunta de energia** | "Como afecta el relieve a la energia?" | Es modulador, no pregunta nueva |
| **2. Dependencia** | No depende de capas ECS — las capas dependen de el | Invierte la relacion |
| **3. Tipo A o B** | Ni A ni B — es un campo espacial del mundo | No es componente de entidad |
| **4. Entropia** | No participa en disipacion directa | Neutral |
| **5. Interferencia** | No es afectado por ondas | No participa |

**Veredicto**: La topologia NO es una capa. Es un **Resource + funciones puras** que existe
DEBAJO del campo energetico. Es el sustrato sobre el que la energia opera.

Es analogo a:
- `EnergyFieldGrid`: campo escalar del mundo (energia)
- `TerrainField`: campo escalar del mundo (geometria)
- `SpatialIndex`: indice espacial del mundo (proximidad)
- `PerceptionCache`: cache derivado del mundo (visibilidad)

---

## 4. Posicion en el Arbol de Dependencias

```text
TerrainField (sustrato fisico)         ← NUEVO: el "planeta"
    │
    ├── modula ──► EnergyNucleus       (emision/decay depende de altitud)
    ├── modula ──► propagation          (difusion depende de pendiente)
    │
    ▼
EnergyFieldGrid (campo alquimico)      ← V7 existente
    │
    ├── lee ────► TerrainField          (topology sample en materialization)
    │
    ▼
materialization_rules                   ← V7 existente
    │  lee: EnergyCell + TerrainSample + Almanac
    │  produce: WorldArchetype + VisualProperties
    │
    ▼
Entidades materializadas (ECS)         ← Capas 0-13 existentes
    │
    ▼
Visual + Render                         ← Presentacion existente
```

**El terreno es la base del stack.** No depende de energia — la energia depende de el.
Esto es correcto fisicamente: el planeta existe antes que la alquimia.

---

## 5. TerrainField Resource

### Estructura

```text
TerrainField (Resource):
  width: u32                            — celdas en X (alineado con EnergyFieldGrid)
  height: u32                           — celdas en Y
  cell_size: f32                        — metros por celda (mismo que EnergyFieldGrid)
  origin: Vec2                          — esquina inferior izquierda
  seed: u64                             — semilla para generacion determinista

  altitude: Vec<f32>                    — metros sobre nivel base, por celda
  slope: Vec<f32>                       — grados (0-90), por celda
  aspect: Vec<f32>                      — direccion de maxima pendiente (0-360), por celda
  drainage: Vec<Vec2>                   — vector de flujo de agua, por celda
  drainage_accumulation: Vec<f32>       — cuanta agua converge aqui, por celda
  terrain_type: Vec<TerrainType>        — clasificacion derivada, por celda

  generation: u32                       — version counter para invalidacion
```

### Alineacion con EnergyFieldGrid

Mismo `width`, `height`, `cell_size`, `origin`. Las dos grillas estan perfectamente alineadas —
`terrain.altitude[idx]` corresponde a `energy.cells[idx]`. Un lookup en una sirve para la otra.

### Generacion

El terrain se genera en **Startup**, antes del warmup de V7:

```text
1. Leer TerrainConfig (RON) — semilla, parametros de ruido, ciclos de erosion
2. Generar heightmap con noise (Perlin/Simplex/FBM)
3. Simular erosion hidraulica (N ciclos)
4. Derivar slope, aspect, drainage de la altitud
5. Clasificar terrain_type por celda
6. Insertar TerrainField como Resource
7. V7 warmup lee TerrainField para modular propagacion
```

---

## 6. Datos por Celda

### Altitude (f32)

Altura en metros sobre el nivel base. Generada por ruido procedural + erosion.
Rango tipico: -50.0 (profundidades) a 200.0 (picos).

### Slope (f32)

Pendiente en grados. Derivada de la diferencia de altitud con los 8 vecinos.
0.0 = plano. 45.0 = empinado. 90.0 = acantilado vertical.

### Aspect (f32)

Direccion de la pendiente mas empinada, en grados (0 = norte, 90 = este).
Util para: exposicion solar (caras sur mas calidas), direccion de escorrentia.

### Drainage (Vec2)

Vector de flujo de agua: hacia donde fluye el agua en esta celda.
Calculado como gradiente descendente del heightmap. Celdas en valles tienen drainage
convergente (muchos vecinos apuntan aqui).

### Drainage Accumulation (f32)

Cuanta agua pasa por esta celda. Celdas en crestas = 0. Celdas en fondos de valle = alto.
Define donde aparecen rios: `accumulation > RIVER_THRESHOLD` → rio implicito.

### TerrainType (enum)

```text
TerrainType:
  Peak              — altitud alta + pendiente alta (crestas, cimas)
  Ridge             — altitud media-alta + pendiente lineal (cordilleras)
  Slope             — pendiente significativa (laderas)
  Valley            — altitud baja + drainage convergente (valles, cañones)
  Plain             — pendiente baja + altitud media (llanuras)
  Riverbed          — drainage_accumulation > umbral (lechos de rio)
  Basin             — altitud baja + pendiente baja + sin drainage (cuencas, lagos)
  Cliff             — pendiente > 60 grados (acantilados, paredes)
  Plateau           — altitud alta + pendiente baja (mesetas)
```

La clasificacion es una funcion pura de (altitude, slope, drainage_accumulation).
No depende de energia — es puramente geometrica.

---

## 7. Como Modula V7

### Emision de Nucleos

La topologia modula los parametros efectivos de cada `EnergyNucleus`:

```text
fn effective_emission_rate(nucleus: &EnergyNucleus, terrain: &TerrainField, pos: Vec2) -> f32:
  let altitude = terrain.altitude_at(pos)
  let slope = terrain.slope_at(pos)

  // Nucleos en valles emiten mas (energia se acumula)
  // Nucleos en crestas emiten menos (energia se dispersa)
  let altitude_factor = 1.0 + (REFERENCE_ALTITUDE - altitude) * ALTITUDE_EMISSION_SCALE

  nucleus.emission_rate * altitude_factor.max(0.1)
```

### Propagacion / Difusion

La energia fluye cuesta abajo mas facil que cuesta arriba:

```text
fn effective_diffusion(base_diffusion: f32, slope: f32, direction: Vec2, aspect: f32) -> f32:
  // Difusion a favor de la pendiente = mas rapida
  // Difusion contra la pendiente = mas lenta
  let slope_alignment = cos(angle_between(direction, aspect_direction))
  let slope_factor = 1.0 + slope_alignment * slope * SLOPE_DIFFUSION_SCALE

  base_diffusion * slope_factor.max(0.1)
```

### Disipacion

La energia se disipa mas rapido en celdas expuestas (crestas) que en celdas protegidas (valles):

```text
fn effective_decay(base_decay: f32, terrain_type: TerrainType) -> f32:
  match terrain_type:
    Peak | Ridge | Cliff => base_decay * 1.5    — exposicion alta, disipacion rapida
    Valley | Basin       => base_decay * 0.7    — protegido, energia se conserva
    Riverbed             => base_decay * 0.8    — agua conserva energia
    _                    => base_decay
```

---

## 8. Como Enriquece la Materializacion

### Arquetipos topograficos

`materialization_rules` recibe AMBOS inputs (energia + terreno) y produce arquetipos mas ricos:

| Energia (freq, state) | Terreno (type) | Arquetipo resultante |
|----------------------|----------------|---------------------|
| Terra + Solid + High density | Peak | Mountain (pico nevado si frio) |
| Terra + Solid + Medium | Slope | Hillside (ladera con vegetacion) |
| Aqua + Liquid | Riverbed | River (rio fluido) |
| Aqua + Liquid | Basin | Lake (lago quieto) |
| Aqua + Solid | Peak | GlacierPeak (glaciar) |
| Ignis + Plasma | Valley | LavaRiver (rio de lava) |
| Ignis + Gas | Peak | VolcanicVent (chimenea volcanica) |
| Umbra + Gas | Valley | MistValley (valle de niebla) |
| Terra + Solid | Cliff | Rockface (pared de roca) |
| Ventus + Gas | Plateau | WindsweptPlateau (meseta ventosa) |

Sin topologia, Terra+Solid solo produce "Rock" o "Mountain" sin distincion.
Con topologia, la FORMA del terreno enriquece el vocabulario visual.

---

## 9. Efectos en Gameplay

### Movimiento

```text
fn traverse_cost_modifier(terrain: TerrainType, entity_state: MatterState) -> f32:
  // Subir cuesta arriba es mas costoso
  // Entidades liquidas fluyen mas facil por rios
  // Entidades gaseosas ignoran terreno
  match entity_state:
    Gas | Plasma => 0.0    — el terreno no afecta gases
    Liquid =>
      match terrain:
        Riverbed | Basin => -0.3  — fluir cuesta abajo es mas facil
        Slope | Cliff    =>  0.2  — liquido no sube bien
        _                =>  0.0
    Solid =>
      match terrain:
        Peak | Cliff     =>  0.5  — muy costoso subir
        Slope            =>  0.2  — costoso
        Valley | Riverbed => -0.1  — cuesta abajo es mas facil
        _                =>  0.0
```

### Linea de vision

Las celdas de terreno alto bloquean vision hacia celdas bajas:

```text
fn terrain_blocks_vision(from: Vec2, to: Vec2, terrain: &TerrainField) -> bool:
  // Raycast sobre el heightmap
  // Si alguna celda intermedia tiene altitud > lerp(from_alt, to_alt), bloquea
```

### Acumulacion de energia

Los valles acumulan energia naturalmente (la difusion fluye cuesta abajo).
Esto crea "cuencas de energia" que generan biomas mas densos y ricos en valles,
y biomas escasos en crestas. Es fisicamente correcto y visualmente atractivo.

---

## 10. Deformaciones Runtime

El terreno puede mutar durante gameplay:

```text
TerrainMutation:
  Crater { center: Vec2, radius: f32, depth: f32 }   — impacto de hechizo
  Uplift { center: Vec2, radius: f32, height: f32 }   — erupcion
  Erosion { cell: (u32, u32), amount: f32 }            — erosion por agua/viento
  Flatten { center: Vec2, radius: f32 }                — construccion
```

Las mutaciones:
1. Modifican `TerrainField.altitude` en las celdas afectadas.
2. Rederivan slope, aspect, drainage en el area afectada (no todo el mapa).
3. Reclasifican terrain_type en el area.
4. Incrementan `TerrainField.generation` → V7 detecta cambio → re-propaga → re-materializa.
5. El mundo se adapta organicamente al cambio topografico.

---

## 11. Generacion Procedural

### Pipeline de generacion (Startup)

```text
Fase 1: HEIGHTMAP BASE
  Noise octaves (FBM/Perlin/Simplex) → altitud por celda
  Parametros: octaves, frequency, amplitude, lacunarity, persistence
  Semilla: determinista desde MapConfig

Fase 2: EROSION HIDRAULICA
  N ciclos de simulacion de agua:
    1. Lluvia uniforme (depositar agua en todas las celdas)
    2. Flujo: agua baja por gradiente de altitud
    3. Erosion: agua arrastra sedimento de celdas con pendiente alta
    4. Deposicion: agua deposita sedimento en celdas de pendiente baja
    5. Evaporacion: reducir agua
  Resultado: heightmap suavizado con valles realistas y rios implicitos

Fase 3: DERIVACION
  Slope: max(abs(altitude[vecino] - altitude[celda])) / cell_size
  Aspect: atan2 del gradiente
  Drainage: flow accumulation algorithm (D8 o D-inf)
  TerrainType: clasificador(altitude, slope, drainage_accumulation)

Fase 4: INSERT RESOURCE
  TerrainField insertado como Resource
  V7 puede iniciar warmup
```

### Determinismo

Misma semilla + mismos parametros = mismo terreno. Garantizado por:
- Noise determinista (no usa rand global)
- Erosion iterativa sin paralelismo no-determinista
- Clasificacion es funcion pura

---

## 12. Pipeline Completo

```text
Startup:
  1. Cargar TerrainConfig (RON)
  2. Generar TerrainField (noise + erosion)
  3. Cargar MapConfig (nucleos)
  4. Inicializar EnergyFieldGrid
  5. V7 warmup (propagar nucleos, modular por terreno)
  6. Materializar mundo (energia + terreno → arquetipos)

Runtime (FixedUpdate):
  Phase::ThermodynamicLayer:
    propagate_nuclei_system     (V7, lee terrain para modulacion)
    dissipate_field_system      (V7, lee terrain para decay)
    derive_cell_state_system    (V7)
    materialization_delta       (V7, arquetipos ricos)
    [eco, índice espacial, containment, motor, … según pipeline.rs]

  Phase::AtomicLayer:
    física / movimiento / colisión
    [terrain traversal u otros según physics.rs]

  Phase::ChemicalLayer / MetabolicLayer / MorphologicalLayer:
    [reacciones, fog/facción, crecimiento — ver blueprint_simulation.md]

Update:
    visual_derivation / shape inference (V7, hints visuales; no FixedUpdate)
```

---

## 13. Organizacion de Codigo

```text
src/
  topology/                          ← NUEVO modulo
    mod.rs                           ← re-exports
    contracts.rs                     ← TerrainSample, TerrainType, DrainageClass
    terrain_field.rs                 ← TerrainField Resource
    generators/
      mod.rs
      noise.rs                       ← heightmap desde noise (FBM/Perlin)
      hydraulics.rs                  ← erosion hidraulica
      slope.rs                       ← slope + aspect desde altitude
      drainage.rs                    ← flow accumulation (D8/D-inf)
      classifier.rs                  ← (altitude, slope, drainage) → TerrainType
    functions.rs                     ← funciones puras: sample_at, modulate_*
    mutations.rs                     ← TerrainMutation, apply_mutation
    constants.rs                     ← umbrales de clasificacion, params de erosion

  simulation/
    worldgen_terrain.rs              ← terrain_generation_system, terrain_effects_system

  assets/
    terrain_config.ron               ← params de noise, erosion, clasificacion
```

---

## 14. Constantes de Tuning

| Constante | Valor default | Rol |
|-----------|---------------|-----|
| `NOISE_OCTAVES` | 6 | Octavas de FBM para heightmap |
| `NOISE_FREQUENCY` | 0.01 | Frecuencia base del noise |
| `NOISE_AMPLITUDE` | 100.0 | Amplitud maxima de altitud |
| `EROSION_CYCLES` | 50 | Ciclos de erosion hidraulica |
| `EROSION_STRENGTH` | 0.3 | Cuanto sedimento arrastra el agua |
| `RIVER_THRESHOLD` | 100.0 | Drainage accumulation minimo para rio |
| `CLIFF_SLOPE_THRESHOLD` | 60.0 | Grados para considerar acantilado |
| `REFERENCE_ALTITUDE` | 50.0 | Altitud de referencia (mar) |
| `ALTITUDE_EMISSION_SCALE` | 0.005 | Como la altitud modula emision |
| `SLOPE_DIFFUSION_SCALE` | 0.3 | Como la pendiente modula difusion |

---

## 15. Trade-offs

| Decision | Valor | Costo |
|----------|-------|-------|
| Grid alineado con EnergyFieldGrid | Lookup O(1), sin interpolacion | Resolucion atada al cell_size de V7 |
| Generacion en Startup (no runtime) | Terreno estable, sin costo per-tick | Delay de carga proporcional a erosion_cycles |
| Erosion hidraulica (no solo noise) | Rios y valles realistas | Complejidad de generacion, mas tiempo de startup |
| TerrainType como clasificacion | Simple, cacheable, determinista | Pierde detalle continuo (cuantizacion) |
| Deformaciones runtime | Terreno dinamico por gameplay | Re-derivacion parcial, incrementa generation |
| Topologia modula V7 (no reemplaza) | V7 sigue siendo la verdad de energia | Mas inputs para propagation (complejidad) |

---

## 16. Riesgos y Mitigacion

| Riesgo | Impacto | Mitigacion |
|--------|---------|------------|
| Erosion lenta en Startup | Loading screen largo | Limitar erosion_cycles, paralelizar con rayon |
| Grid grande = mucha memoria | ~16 bytes/celda × 10000 celdas = 160KB | Aceptable, cache-friendly |
| Deformaciones frecuentes → re-derivacion costosa | Medio | Solo re-derivar area afectada (dirty region) |
| Terreno plano si noise mal configurado | Bajo | Presets probados en terrain_config.ron |
| V7 modulacion introduce bugs sutiles | Medio | Tests: terreno plano → V7 produce mismos resultados que sin topologia |
| Determinismo roto por erosion paralela | Alto | Erosion secuencial, sin rayon en el core |

---

## 17. Plan de Sprints

| Sprint | Entregable | Depende de | Validacion |
|--------|-----------|------------|------------|
| T1 | Contratos: TerrainType, TerrainSample, TerrainField | — | Tipos compilan, serialize |
| T2 | Generacion de altitud (noise) | T1 | Heightmap visual, determinista |
| T3 | Slope + aspect derivation | T2 | Pendientes correctas en 8 vecinos |
| T4 | Drainage + flow accumulation | T2 | Rios convergen en valles |
| T5 | Clasificador TerrainType | T3+T4 | Peaks en crestas, Riverbeds en rios |
| T6 | Modulacion de V7 (propagacion) | T5+V7 | Energia acumula en valles |
| T7 | Enriquecimiento de materializacion | T5+V7 | Arquetipos ricos (River, GlacierPeak) |
| T8 | Efectos en gameplay (movement, vision) | T5 | Traverse cost funciona |
| T9 | Config data-driven (RON) | T1 | Tuneable sin recompilar |
| T10 | Deformaciones runtime | T5 | Crater modifica terreno, mundo se adapta |

---

## 18. Resumen

```text
El sustrato topologico NO cambia que es la energia.
Cambia DONDE la energia fluye con facilidad.

El terreno es el planeta. La energia es la alquimia sobre el planeta.
La topologia modula propagacion, acumulacion y disipacion.
Los valles acumulan. Las crestas dispersan. Los rios canalizan.

Heightmap: noise + erosion → altitud realista.
Derivacion: altitud → slope, drainage, terrain_type.
Modulacion: terrain → emission, diffusion, decay de V7.
Materializacion: energia + terreno → mundo rico.
Gameplay: traverse cost, vision, deformaciones.

Todo procedural. Todo determinista. Todo derivado.
La topologia no inventa — modula.
```
