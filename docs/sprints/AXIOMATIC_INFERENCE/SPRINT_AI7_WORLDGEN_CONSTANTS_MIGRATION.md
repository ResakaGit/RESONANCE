# Sprint AI-7 — Worldgen Constants Migration to Derived Functions

**Módulo:** `src/worldgen/constants.rs`, consumidores en `worldgen/systems/`, `worldgen/`, `rendering/`, `eco/`
**Tipo:** Refactor API (const → fn) + recalibración visual
**Eje axiomático:** Axiom 1 (energy = state) + Axiom 4 (dissipation scales everything)
**Estado:** ✅ Cerrado (2026-03-27) — alternative path: visual_calibration.rs
**Bloqueado por:** AI-1 ✅, AI-2 ✅
**Esfuerzo:** Alto (~3h — toca 11+ consumidores, requiere recalibración visual)

---

## Contexto

5 constantes en `worldgen/constants.rs` están documentadas con `DEBT:` — tienen valor hardcodeado que diverge del axiomáticamente derivado. No se migraron porque son `pub const` consumidas por 11+ archivos, y cambiarlas a `fn()` rompe la API.

## Las 5 constantes DEBT

| Constante | Valor actual | Valor derivado | Consumidores | Impacto del cambio |
|-----------|-------------|----------------|--------------|-------------------|
| `REFERENCE_DENSITY` | 50.0 | `liquid_density_threshold()` ≈ 127 | 6 archivos en `rendering/`, `worldgen/` | Escala visual cambia 2.5× |
| `DENSITY_HIGH_THRESHOLD` | 100.0 | `liquid_density_threshold()` ≈ 127 | 4 archivos en `worldgen/`, `eco/` | Clasificación de densidad cambia |
| `PURITY_THRESHOLD` | 0.7 | `sense_coherence_min() × 2` ≈ 0.40 | 5 archivos en `worldgen/`, `rendering/` | Más entidades clasifican como "puras" |
| `FIELD_CONDUCTIVITY_SPREAD` | 0.1 | `DISSIPATION_LIQUID` = 0.02 | 3 archivos en `worldgen/systems/` | Difusión 5× más lenta |
| `MATERIALIZED_SPAWN_BOND_ENERGY` | 1000.0 | `1/DISSIPATION_SOLID` = 200 | 3 archivos en `worldgen/systems/` | Bonds 5× más débiles |

**Constantes que YA coinciden (no necesitan cambio):**

| Constante | Valor | Derivado | Match |
|-----------|-------|----------|-------|
| `MIN_MATERIALIZATION_QE` | 10.0 | `self_sustaining_qe_min() / 2` = 10 | ✅ |
| `FIELD_DECAY_RATE` | 1.0 | `basal_drain_rate()` = 1.0 | ✅ |
| `DENSITY_LOW_THRESHOLD` | 20.0 | `DENSITY_SCALE` = 20.0 | ✅ |
| `MATERIALIZED_COLLIDER_RADIUS_FACTOR` | 0.5 | Geometric (half cell) | ✅ |
| `MATERIALIZED_MIN_COLLIDER_RADIUS` | 0.01 | ≈ `DISSIPATION_SOLID` = 0.005 | ~✅ |

## Tareas

### Fase 1: Cambiar API (const → fn)

1. En `worldgen/constants.rs`: cambiar 5 constantes de `pub const` a `pub fn` que llaman `derived_thresholds::*()`.
2. En cada consumidor (11+ archivos): cambiar `CONSTANT_NAME` a `constant_name()`.
3. Compilar — corregir errores de tipo donde se esperaba `const`.

**Archivos consumidores a actualizar (grep results):**

```bash
# REFERENCE_DENSITY
grep -rn "REFERENCE_DENSITY" src/ --include="*.rs" | grep -v "test\|//"

# DENSITY_HIGH_THRESHOLD
grep -rn "DENSITY_HIGH_THRESHOLD" src/ --include="*.rs" | grep -v "test\|//"

# PURITY_THRESHOLD
grep -rn "PURITY_THRESHOLD" src/ --include="*.rs" | grep -v "test\|//"

# FIELD_CONDUCTIVITY_SPREAD
grep -rn "FIELD_CONDUCTIVITY_SPREAD" src/ --include="*.rs" | grep -v "test\|//"

# MATERIALIZED_SPAWN_BOND_ENERGY
grep -rn "MATERIALIZED_SPAWN_BOND_ENERGY" src/ --include="*.rs" | grep -v "test\|//"
```

### Fase 2: Recalibración visual

Los valores derivados difieren significativamente de los calibrados:
- `REFERENCE_DENSITY`: 50 → 127 (escala visual 2.5× más comprimida)
- `PURITY_THRESHOLD`: 0.7 → 0.40 (muchas más entidades "puras")
- `FIELD_CONDUCTIVITY_SPREAD`: 0.1 → 0.02 (difusión mucho más lenta)
- `MATERIALIZED_SPAWN_BOND_ENERGY`: 1000 → 200 (bonds más débiles)

**Cada cambio requiere:**
1. Corrida headless antes/después (baseline visual)
2. Verificar que la simulación no diverge (conservation, no NaN)
3. Ajustar SOLO si el resultado visual es aceptable
4. Si no es aceptable: documentar por qué el valor derivado no funciona y proponer derivación alternativa

### Fase 3: Tests de regresión visual

1. Correr `genesis_validation` y `visual_showcase` antes y después
2. Comparar total_qe, max_cell_qe, entity count, cells_with_energy
3. Si delta > 50% en cualquier métrica: WARN, investigar
4. Actualizar baseline esperado en sprint docs

## Criterio de cierre

- `grep -rn "DEBT:" src/worldgen/constants.rs` → 0 matches
- Todas las constantes de worldgen son `fn()` o ya coinciden con derivado
- `cargo test` 0 failures
- Demo headless produce imagen con biomas distinguibles (no regresión a rosa uniforme)
- Documento de recalibración con antes/después

## Riesgos

- **Alto:** Cambiar `REFERENCE_DENSITY` de 50 a 127 puede romper el sistema visual de colores completamente. Todo el rendering de `quantized_color` está calibrado a density/50.
- **Medio:** `FIELD_CONDUCTIVITY_SPREAD` a 0.02 hace la difusión 5× más lenta — los campos pueden no expandirse lo suficiente en warmup.
- **Bajo:** `PURITY_THRESHOLD` a 0.40 es más permisivo — más entidades se materializan como "puras".

## Alternativa

Si los valores derivados producen resultados visuales inaceptables, la alternativa es:
1. Documentar que estos 5 valores son **calibración visual** (no física)
2. Moverlos a un módulo separado `worldgen/visual_calibration.rs`
3. Marcar como "visual tuning, not axiom-derived" explícitamente
4. Cerrar el DEBT como "design decision: visual calibration is not physics"
