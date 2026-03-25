# BLUEPRINT — Sustrato Topologico: Geomorfologia Procedural del Mundo

---

## 1. Objetivo

La energia necesita un lugar donde existir. Antes del campo alquimico, antes de las capas, esta el terreno.

```text
El planeta existe antes que la alquimia.
La topologia define DONDE la energia fluye con facilidad.
Los valles acumulan. Las crestas dispersan. Los rios canalizan.
```

El sustrato topologico introduce un heightmap procedural con erosion hidraulica que:
- Da relieve al mundo: altitud, pendiente, rios, acantilados, mesetas.
- Modula V7: emision, difusion y disipacion dependen de la forma del terreno.
- Enriquece la materializacion: energia + terreno → arquetipos ricos (River, GlacierPeak, LavaRiver).
- Habilita gameplay espacial: ventaja de altura, cobertura, barreras naturales, coste de travesia.
- Genera variedad de mapas: misma energia, distinto terreno = mundo diferente.

---

## 2. Herencia obligatoria

El sustrato topologico hereda sin excepciones:

- Pipeline: `Input → PrePhysics → Physics → Reactions → PostPhysics`
- Ecuaciones puras en `src/blueprint/equations.rs` — NO se modifican.
- Arquitectura por capas ortogonales (Capas 0-13) — NO se agrega capa.
- Determinismo operativo: misma semilla + mismos parametros = mismo terreno, bit-a-bit.
- Contratos explicitos por modulo.
- V7 worldgen existente — se extiende, no se reemplaza.

La topologia agrega un Resource y funciones puras. No modifica capas existentes ni ecuaciones.

---

## 3. Principios de diseno

1. **Stateless-first**: la generacion, derivacion y clasificacion van en funciones puras sin dependencia de ECS. Los sistemas son solo wiring.
2. **El terreno es un Resource, no una capa**: `TerrainField` es estado del mundo (como `EnergyFieldGrid` o `SpatialIndex`), no una propiedad de entidad.
3. **Grid alineado con V7**: mismo `width`, `height`, `cell_size`, `origin` que `EnergyFieldGrid`. Lookup O(1) por indice compartido.
4. **La topologia modula, no reemplaza**: V7 sigue siendo la verdad de energia. El terreno solo modifica parametros efectivos de emision, difusion y disipacion.
5. **Determinismo garantizado**: noise determinista (semilla explicita), erosion secuencial (sin paralelismo no-determinista), clasificacion es funcion pura.
6. **Generacion en Startup**: el terreno se genera antes del warmup de V7. El mundo arranca con relieve completo.
7. **Data-driven**: parametros de noise, erosion y clasificacion se definen en RON, no hardcodeados.
8. **Deformaciones runtime**: el terreno puede mutar durante gameplay (crateres, erupciones), con re-derivacion parcial eficiente.

---

## 4. Tabla de modulos

| # | Modulo | Tipo | Responsabilidad | Entradas | Salidas |
|---|--------|------|-----------------|----------|---------|
| 01 | `topology/contracts` | Tipos puros | TerrainSample, TerrainType, DrainageClass, constantes | — | tipos compartidos |
| 02 | `topology/terrain_field` | Resource | TerrainField storage (altitude, slope, aspect, drainage) | — | estado del terreno |
| 03 | `topology/generators/noise` | Stateless | Heightmap desde noise (FBM/Perlin/Simplex) | seed, params | altitude[] |
| 04 | `topology/generators/hydraulics` | Stateless | Erosion hidraulica iterativa | altitude[], params | altitude[] refinado |
| 05 | `topology/generators/slope` | Stateless | Derivar slope + aspect desde altitude | altitude[], cell_size | slope[], aspect[] |
| 06 | `topology/generators/drainage` | Stateless | Flow accumulation (D8/D-inf) | altitude[] | drainage[], accumulation[] |
| 07 | `topology/generators/classifier` | Stateless | (altitude, slope, accumulation) → TerrainType | campos derivados | terrain_type[] |
| 08 | `topology/functions` | Stateless | Funciones puras: sample_at, modulate_emission, modulate_diffusion, modulate_decay | TerrainField + pos | valores modulados |
| 09 | `topology/mutations` | Stateless | TerrainMutation enum, apply_mutation, dirty region | mutation + field | field modificado |
| 10 | `topology/constants` | Data | Umbrales de clasificacion, params de erosion | — | constantes |
| 11 | `worldgen/systems/terrain` | Sistema ECS | terrain_generation_system, terrain_effects_system | Config → TerrainField | Resource insertado |
| 12 | `assets/terrain_config.ron` | Data-driven | Config de noise, erosion, clasificacion | archivo RON | config parseada |

---

## 5. Tipos nuevos

### 5.1 TerrainField (Resource)

Grid 2D alineado con `EnergyFieldGrid`. Almacena por celda:
- `altitude: f32` — metros sobre nivel base (-50.0 a 200.0)
- `slope: f32` — grados (0-90)
- `aspect: f32` — direccion de maxima pendiente (0-360)
- `drainage: Vec2` — vector de flujo de agua
- `drainage_accumulation: f32` — cuanta agua converge aqui
- `terrain_type: TerrainType` — clasificacion derivada

Metadatos: `width`, `height`, `cell_size`, `origin`, `seed`, `generation` (version counter).

### 5.2 TerrainType (enum)

Clasificacion geometrica derivada de (altitude, slope, drainage_accumulation):

```text
Peak       — altitud alta + pendiente alta (cimas)
Ridge      — altitud media-alta + pendiente lineal (cordilleras)
Slope      — pendiente significativa (laderas)
Valley     — altitud baja + drainage convergente (valles)
Plain      — pendiente baja + altitud media (llanuras)
Riverbed   — drainage_accumulation > umbral (lechos de rio)
Basin      — altitud baja + pendiente baja + sin drainage (cuencas, lagos)
Cliff      — pendiente > 60 grados (acantilados)
Plateau    — altitud alta + pendiente baja (mesetas)
```

NO depende de energia — es puramente geometrica.

### 5.3 TerrainSample (struct)

Snapshot de una celda para lectura por otros sistemas:

```text
TerrainSample:
  altitude: f32
  slope: f32
  aspect: f32
  drainage: Vec2
  drainage_accumulation: f32
  terrain_type: TerrainType
```

### 5.4 DrainageClass (enum)

Clasificacion de cuanta agua pasa por la celda:

```text
Dry         — accumulation < 10
Moist       — accumulation 10-50
Wet         — accumulation 50-100
River       — accumulation > RIVER_THRESHOLD
```

### 5.5 TerrainMutation (enum)

Modificaciones al terreno en runtime:

```text
Crater { center, radius, depth }      — impacto de hechizo
Uplift { center, radius, height }     — erupcion
Erosion { cell, amount }               — erosion localizada
Flatten { center, radius }             — construccion
```

### 5.6 TerrainConfig (asset)

Parametros de generacion cargados desde RON:

```text
TerrainConfig:
  seed: u64
  noise: NoiseParams { octaves, frequency, amplitude, lacunarity, persistence }
  erosion: ErosionParams { cycles, strength, deposition_rate, evaporation }
  classification: ClassificationThresholds { cliff_slope, river_accumulation, peak_altitude, ... }
  enabled: bool
```

---

## 6. Pipeline de generacion

### 6.1 Startup (antes del warmup V7)

```text
1. Cargar TerrainConfig (RON)
2. Generar heightmap base (noise FBM)
3. Simular erosion hidraulica (N ciclos)
4. Derivar slope + aspect
5. Calcular drainage + flow accumulation
6. Clasificar terrain_type por celda
7. Insertar TerrainField como Resource
8. V7 warmup lee TerrainField para modular propagacion
```

### 6.2 Runtime (FixedUpdate)

```text
PrePhysics:
  propagate_nuclei_system     (V7, lee terrain para modulacion)
  dissipate_field_system      (V7, lee terrain para decay)
  [sistemas existentes]

Physics:
  terrain_effects_system      (NUEVO: aplica traverse_cost, friction)
  [sistemas existentes]

PostPhysics:
  materialization_delta       (V7, lee terrain para arquetipos ricos)
  visual_derivation           (V7, lee terrain para shape hints)
```

### 6.3 Mutacion (gameplay)

```text
1. Aplicar TerrainMutation a altitude[] (area afectada)
2. Re-derivar slope, aspect, drainage en dirty region
3. Re-clasificar terrain_type en dirty region
4. Incrementar generation → V7 detecta cambio → re-propaga → re-materializa
```

---

## 7. Como modula V7

### 7.1 Emision de nucleos

La altitud modula la tasa de emision efectiva:

```text
fn effective_emission_rate(nucleus, terrain, pos) -> f32:
  let altitude = terrain.altitude_at(pos)
  let altitude_factor = 1.0 + (REFERENCE_ALTITUDE - altitude) * ALTITUDE_EMISSION_SCALE
  nucleus.emission_rate * altitude_factor.max(0.1)
```

Valles emiten mas (energia se acumula). Crestas emiten menos (energia se dispersa).

### 7.2 Difusion / Propagacion

La energia fluye cuesta abajo mas facil que cuesta arriba:

```text
fn effective_diffusion(base, slope, direction, aspect) -> f32:
  let alignment = cos(angle_between(direction, aspect_direction))
  let slope_factor = 1.0 + alignment * slope * SLOPE_DIFFUSION_SCALE
  base * slope_factor.max(0.1)
```

### 7.3 Disipacion

Terreno expuesto disipa mas rapido, terreno protegido conserva:

```text
fn effective_decay(base_decay, terrain_type) -> f32:
  match terrain_type:
    Peak | Ridge | Cliff => base_decay * 1.5
    Valley | Basin       => base_decay * 0.7
    Riverbed             => base_decay * 0.8
    _                    => base_decay
```

---

## 8. Como enriquece la materializacion

`materialization_rules` recibe AMBOS inputs (energia + terreno) y produce arquetipos mas ricos:

| Energia (freq, state) | Terreno (type) | Arquetipo resultante |
|----------------------|----------------|---------------------|
| Terra + Solid + High | Peak | Mountain (pico nevado si frio) |
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

## 9. Efectos en gameplay

### 9.1 Coste de travesia

```text
fn traverse_cost_modifier(terrain_type, entity_state) -> f32:
  match entity_state:
    Gas | Plasma => 0.0       — terreno no afecta gases
    Liquid =>
      Riverbed | Basin => -0.3  — fluir cuesta abajo
      Slope | Cliff    =>  0.2  — liquido no sube bien
    Solid =>
      Peak | Cliff     =>  0.5  — muy costoso
      Slope            =>  0.2  — costoso
      Valley | Riverbed => -0.1  — cuesta abajo
```

### 9.2 Linea de vision

Terreno alto bloquea vision a celdas bajas. Raycast sobre heightmap.

### 9.3 Acumulacion natural

Los valles acumulan energia (difusion fluye cuesta abajo) → biomas mas densos y ricos en valles, escasos en crestas. Fisicamente correcto y visualmente atractivo.

---

## 10. Deformaciones runtime

```text
TerrainMutation:
  Crater { center, radius, depth }   — impacto de hechizo
  Uplift { center, radius, height }  — erupcion
  Erosion { cell, amount }           — erosion localizada
  Flatten { center, radius }         — construccion
```

Pipeline de mutacion:
1. Modificar `altitude[]` en celdas afectadas.
2. Re-derivar slope, aspect, drainage en dirty region (no todo el mapa).
3. Re-clasificar terrain_type en dirty region.
4. Incrementar `generation` → V7 detecta cambio → re-propaga → re-materializa.

---

## 11. Posicion arquitectonica

- **No es una capa**: no responde una pregunta sobre la energia. Falla el 5-test de DESIGNING.md.
- **Es un Resource**: `TerrainField` como `EnergyFieldGrid`, `SpatialIndex`, `PerceptionCache`.
- **Esta DEBAJO del campo energetico**: el planeta existe antes que la alquimia. V7 lee terreno, no al reves.
- **No modifica ecuaciones**: `equations.rs` permanece intacto.
- **Es determinista**: misma semilla → mismo terreno.
- **Es stateless en funciones**: generacion, derivacion y clasificacion son funciones puras.
- **Es data-driven**: parametros en RON, no hardcodeados.

---

## 12. Organizacion de codigo

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

## 13. Constantes de tuning

| Constante | Valor default | Rol |
|-----------|---------------|-----|
| `NOISE_OCTAVES` | 6 | Octavas de FBM para heightmap |
| `NOISE_FREQUENCY` | 0.01 | Frecuencia base del noise |
| `NOISE_AMPLITUDE` | 100.0 | Amplitud maxima de altitud |
| `EROSION_CYCLES` | 50 | Ciclos de erosion hidraulica |
| `EROSION_STRENGTH` | 0.3 | Cuanto sedimento arrastra el agua |
| `RIVER_THRESHOLD` | 100.0 | Drainage accumulation minimo para rio |
| `CLIFF_SLOPE_THRESHOLD` | 60.0 | Grados para considerar acantilado |
| `REFERENCE_ALTITUDE` | 50.0 | Altitud de referencia |
| `ALTITUDE_EMISSION_SCALE` | 0.005 | Como la altitud modula emision |
| `SLOPE_DIFFUSION_SCALE` | 0.3 | Como la pendiente modula difusion |

---

## 14. Trade-offs

| Decision | Valor | Costo |
|----------|-------|-------|
| Grid alineado con EnergyFieldGrid | Lookup O(1), sin interpolacion | Resolucion atada al cell_size de V7 |
| Generacion en Startup (no runtime) | Terreno estable, sin costo per-tick | Delay de carga proporcional a erosion_cycles |
| Erosion hidraulica (no solo noise) | Rios y valles realistas | Complejidad de generacion, mas tiempo de startup |
| TerrainType como clasificacion | Simple, cacheable, determinista | Pierde detalle continuo (cuantizacion) |
| Deformaciones runtime | Terreno dinamico por gameplay | Re-derivacion parcial, incrementa generation |
| Topologia modula V7 (no reemplaza) | V7 sigue siendo la verdad de energia | Mas inputs para propagation |

---

## 15. Riesgos y mitigacion

| Riesgo | Impacto | Mitigacion |
|--------|---------|------------|
| Erosion lenta en Startup | Loading screen largo | Limitar erosion_cycles, paralelizar con rayon |
| Grid grande = mucha memoria | ~16 bytes/celda × 10000 = 160KB | Aceptable, cache-friendly |
| Deformaciones frecuentes → re-derivacion costosa | Medio | Dirty region: solo re-derivar area afectada |
| Terreno plano si noise mal configurado | Bajo | Presets probados en terrain_config.ron |
| V7 modulacion introduce bugs sutiles | Medio | Test: terreno plano → mismos resultados que sin topologia |
| Determinismo roto por erosion paralela | Alto | Erosion secuencial, sin rayon en core |

---

## 16. Sprints

Ver `docs/sprints/TOPOLOGY/README.md` para el plan completo de implementacion.

| Sprint | Entregable | Onda |
|--------|-----------|------|
| T1 | Contratos: TerrainType, TerrainSample, TerrainField | 0 |
| T2 | Generacion de altitud (noise FBM) | A |
| T3 | Slope + aspect derivation | B |
| T4 | Drainage + flow accumulation | B |
| T5 | Clasificador TerrainType | C |
| T6 | Modulacion de V7 (propagacion) | D |
| T7 | Enriquecimiento de materializacion | D |
| T8 | Efectos en gameplay (movement, vision) | D |
| T9 | Config data-driven (RON) | A |
| T10 | Deformaciones runtime | E |

---

## 17. Referencias

- `docs/design/BLUEPRINT.md` (modelo de capas)
- `DESIGNING.md` (filosofia de capas y tests)
- `TOPOLOGY_AND_LAYERS.md` (documento de analisis original)
- `docs/arquitectura/blueprint_topology.md` (blueprint de arquitectura detallado)
- `docs/design/V7.md` (V7, worldgen que se extiende)
- `src/worldgen/field_grid.rs` (EnergyFieldGrid, grid hermano)
