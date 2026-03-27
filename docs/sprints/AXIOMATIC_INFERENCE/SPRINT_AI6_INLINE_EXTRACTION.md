# Sprint AI-6 — Extract Inline Magic Numbers + Consolidate Duplicates

**Módulo:** `src/worldgen/systems/nucleus_recycling.rs`, `src/blueprint/equations/awakening.rs`
**Tipo:** Cleanup + consolidación
**Eje axiomático:** N/A (higiene de código)
**Estado:** ⏳ Pendiente
**Bloqueado por:** AI-1
**Esfuerzo:** Bajo (~20min)

---

## Magic numbers inline a extraer

### nucleus_recycling.rs (líneas 84-87)

```rust
ncell.carbon_norm = (ncell.carbon_norm * 0.3).max(0.0);     // ¿por qué 0.3?
ncell.nitrogen_norm = (ncell.nitrogen_norm * 0.3).max(0.0);  // ¿por qué 0.3?
ncell.phosphorus_norm = (ncell.phosphorus_norm * 0.3).max(0.0);
ncell.water_norm = (ncell.water_norm * 0.5).max(0.0);        // ¿por qué 0.5?
```

**Derivación:** El drenaje de nutrientes debería escalar con la eficiencia de conversión
nutriente→energía. `retention = 1.0 - dissipation_rate(Solid)` para minerales (C/N/P),
`retention_water = 1.0 - dissipation_rate(Liquid)` para agua (más volátil).
Con `DISSIPATION_SOLID = 0.005` y `DISSIPATION_LIQUID = 0.02`, los valores derivados son
diferentes a los hardcodeados, pero la relación C/N/P < Water sigue.

Crear en `derived_thresholds`:
```rust
pub fn nutrient_retention_mineral() -> f32 { 1.0 - DISSIPATION_SOLID * 100.0 }  // ~0.5
pub fn nutrient_retention_water() -> f32 { 1.0 - DISSIPATION_LIQUID * 20.0 }    // ~0.6
```

### nucleus_recycling.rs (línea 80)

```rust
.unwrap_or(85.0)  // Fallback frecuencia
```

Reemplazar con `ABIOGENESIS_FLORA_PEAK_HZ` (ya existe en blueprint/constants).

## Duplicados a consolidar

| Constante A | Archivo A | Constante B | Archivo B | Acción |
|------------|-----------|------------|-----------|--------|
| `AWAKENING_MIN_QE` | awakening.rs | `SELF_SUSTAINING_QE_MIN` | axiomatic.rs | Eliminar ambas, usar `derived_thresholds::self_sustaining_qe_min()` |
| `AWAKENING_THRESHOLD` | awakening.rs | `AXIOMATIC_SPAWN_THRESHOLD` | axiomatic.rs | Eliminar ambas, usar `derived_thresholds::spawn_potential_threshold()` |
| `COHERENCE_BANDWIDTH_HZ` | axiomatic.rs | `PRESSURE_FREQUENCY_BANDWIDTH` | radiation_pressure.rs | Consolidar en un solo lugar (derived_thresholds o compartido) |

## Tareas

1. Extraer nutrient drain fractions a `derived_thresholds::nutrient_retention_*()`.
2. Reemplazar `85.0` fallback con `ABIOGENESIS_FLORA_PEAK_HZ`.
3. Eliminar 3 pares de duplicados (6 constantes → 3 funciones en derived_thresholds).
4. Actualizar imports en consumidores.

## Criterio de cierre

- `grep -rn "0\.3\|0\.5" src/worldgen/systems/nucleus_recycling.rs` → 0 matches en nutrient drain
- `grep -rn "85\.0" src/worldgen/systems/nucleus_recycling.rs` → 0 matches
- `AWAKENING_MIN_QE` y `AWAKENING_THRESHOLD` no existen como constantes propias
- `COHERENCE_BANDWIDTH_HZ` y `PRESSURE_FREQUENCY_BANDWIDTH` unificiados
