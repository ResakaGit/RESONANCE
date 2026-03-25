# Sprint Q2 — Magic Numbers → Constantes Nombradas

**Tipo:** Refactor — extraer constantes con documentacion.
**Severidad:** MEDIA-ALTA — imposible tunear sin leer cada linea.
**Onda:** 0 — Sin dependencias.

## Objetivo

Extraer ~60+ constantes hardcodeadas sin nombre a `src/blueprint/constants/` (shard por dominio) o a módulos `constants.rs` locales, con documentación de por qué ese valor.

## Hallazgos por modulo

### layers/ (~25 magic numbers)

| Archivo | Linea | Valor | Debe ser |
|---------|-------|-------|----------|
| `engine.rs` | 34 | `1000.0` | `ENGINE_DEFAULT_MAX_BUFFER` |
| `engine.rs` | 35 | `10.0` | `ENGINE_DEFAULT_INPUT_VALVE` |
| `engine.rs` | 36 | `50.0` | `ENGINE_DEFAULT_OUTPUT_VALVE` |
| `engine.rs` | 84 | `1.5` | `ENGINE_OVERLOAD_MULTIPLIER` |
| `engine.rs` | 126 | `1100.0` | `ENGINE_EFFICIENCY_FREQ_DIVISOR` |
| `engine.rs` | 136 | `0.7` | `ENGINE_EFFICIENCY_FALLOFF` |
| `coherence.rs` | 46 | `5000.0` | `DEFAULT_BOND_ENERGY` |
| `coherence.rs` | 65-77 | `0.0, 5.0, 0.2, 0.5, 1.5, 3.0` | `VELOCITY_LIMIT_*`, `DISSIPATION_MULT_*` por MatterState |
| `energy.rs` | 22 | `100.0` | `DEFAULT_BASE_ENERGY` |
| `flow.rs` | 26 | `5.0` | `DEFAULT_DISSIPATION_RATE` |
| `identity.rs` | 73, 75 | `0.2, -0.2` | `FACTION_ALLY_BONUS`, `FACTION_ENEMY_MALUS` |
| `injector.rs` | 22-24 | `50.0, 1000.0, 0.5` | `INJECTOR_DEFAULT_*` |
| `oscillatory.rs` | 38 | `1000.0` | `DEFAULT_FREQUENCY_HZ` |
| `pressure.rs` | 41-65 | `-5.0, 10.0, -1.0, -2.0...` | `BIOME_VOLCANO_DELTA_QE`, etc. |

### worldgen/ (~20 magic numbers)

| Archivo | Linea | Valor | Debe ser |
|---------|-------|-------|----------|
| `visual_derivation.rs` | 28-29 | `0.7, 0.9` | `SCALE_SOLID_BASE`, `SCALE_SOLID_RANGE` |
| `visual_derivation.rs` | 30 | `1.4, 0.7` | `SCALE_GAS_BASE`, `SCALE_GAS_RANGE` |
| `visual_derivation.rs` | 31 | `1.1, 0.5` | `SCALE_PLASMA_BASE`, `SCALE_PLASMA_RANGE` |
| `visual_derivation.rs` | 48-51 | `0.35, 100.0, 0.6, 300.0` | `EMISSION_PLASMA_*`, `EMISSION_GAS_*` |
| `visual_derivation.rs` | 69-76 | `0.65, 0.25, 0.15, 0.30, 0.55, 0.35` | `OPACITY_*_BASE`, `OPACITY_*_RANGE` |
| `materialization_rules.rs` | 189-204 | `0.7, 1.3, 0.35, 0.75, 0.4, 0.9` | Mismos patrones que visual_derivation |

### blueprint/ (~10 magic numbers)

| Archivo | Linea | Valor | Debe ser |
|---------|-------|-------|----------|
| `equations.rs` | 169+ | `1e-6` (×6) | `DISTANCE_EPSILON` |
| `equations.rs` | 170+ | `1e-4` (×3) | `DIVISION_GUARD_EPSILON` |
| `equations.rs` | 210 | `0.15` | `CONVECTIVE_COEFFICIENT` |
| `equations.rs` | 215-218 | `1.0, 0.3, 0.1` | `CONDUCTION_FACTOR_*` por state |
| `element_id.rs` | 31-32 | `2166136261, 16777619` | `FNV_OFFSET_BASIS`, `FNV_PRIME` |

### simulation/ (~5 magic numbers)

| Archivo | Linea | Valor | Debe ser |
|---------|-------|-------|----------|
| `physics.rs` | 113 | `10.0` | `ACTUATOR_VELOCITY_LIMIT` |
| `containment.rs` | 31, 34 | `0.5` | `IMMERSION_DEPTH_THRESHOLD` |
| `worldgen_materialization.rs` | 243+ | `1000.0` (×4) | `DEFAULT_SPAWN_BOND_ENERGY` |
| `worldgen_materialization.rs` | 272+ | `0.3` | `DEFAULT_SPAWN_DISSIPATION` |

### entities/ (~15 magic numbers en tuples)

| Archivo | Linea | Problema |
|---------|-------|----------|
| `archetypes.rs` | 148-164 | Tuples de 6 valores por BiomeType sin struct |
| `archetypes.rs` | 256-264 | Tuples de 9 valores por HeroClass sin struct |

## Tacticas

- **Agrupar por dominio, no por archivo.** Las constantes de layers van a `src/blueprint/constants/` (shards por dominio). Las de worldgen van a `src/worldgen/constants.rs` (ya existe). Las de entities pueden ir a un nuevo `src/entities/constants.rs` o a presets con struct.
- **Nombrar con patron `DOMINIO_CONTEXTO_CAMPO`.** Ej: `ENGINE_DEFAULT_MAX_BUFFER`, `VISUAL_OPACITY_GAS_BASE`, `BIOME_VOLCANO_DELTA_QE`.
- **Documentar el "por que".** Cada constante debe tener un comentario de 1 linea explicando por que ese valor y no otro. Ej: `/// 1.5x del buffer = zona de overload donde eficiencia cae`.
- **Tuples de biome/hero → structs con campos.** Reemplazar `(0.0, 1.0, 50.0, MatterState::Solid, 0.2, 5000.0)` con `BiomePreset { delta_qe: 0.0, viscosity: 1.0, ... }`.
- **Un commit por modulo.** layers/ primero, luego worldgen/, luego blueprint/, luego simulation/, luego entities/.

## NO hace

- No cambia valores — solo les da nombre.
- No reorganiza modulos.
- No modifica logica.
- No agrega tests (Q6).

## Criterio de aceptacion

- Test: `grep -rn '[0-9]\.[0-9]' src/layers/` no encuentra constantes sin nombre (excepto en tests).
- Test: `src/blueprint/constants/` contiene las constantes de equations (vía facade `mod.rs`).
- Test: `src/worldgen/constants.rs` contiene las constantes visuales.
- Test: BiomeType y HeroClass usan structs con campos nombrados.
- `cargo test` pasa sin cambios en resultados.
- Cada constante tiene comentario de documentacion.

## Estado de implementacion (revision)

| Bloque | Estado |
|--------|--------|
| `blueprint/equations.rs` + epsilones/FNV en `constants.rs` | Hecho |
| `layers/*` (tabla del sprint + drift `OVERLOAD_FACTOR` unificado) | Hecho |
| `worldgen/visual_derivation` + `materialization_rules` + `constants.rs` | Hecho |
| `simulation/physics`, `containment`, `worldgen_materialization` (spawn) | Hecho |
| `entities/archetypes` (`BiomeSpawnPreset`, `HeroSpawnPreset`) | Hecho |
| Heuristica grep en `src/layers/` | Sigue habiendo literales en **comentarios** (`/// T < 0.3`), tests `#[cfg(test)]`, y constantes matematicas locales (`2.0 * PI`); el tuning de gameplay de la tabla Q2 esta nombrado en `blueprint/constants/` o `worldgen/constants.rs`. |
| Otros spawns (`spawn_crystal`, `spawn_lava_knight`, …) | Pendiente opcional (no estaban en la tabla minima Q2). |

**Nota:** El verificador marco riesgo de doble verdad `1.5` vs `OVERLOAD_FACTOR`; `AlchemicalEngine::is_overloaded` ahora usa `OVERLOAD_FACTOR` de `blueprint/constants/`.
