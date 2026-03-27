# Sprint AC-1 — Interference × Metabolic Extraction

**Módulo:** `src/blueprint/equations/energy_competition/` (nuevo), `src/simulation/metabolic/`
**Tipo:** Ecuaciones puras + extensión de tres sistemas metabólicos existentes
**Eje axiomático:** Axioma 3 × Axioma 8
**Estado:** ⏳ Pendiente
**Oleada:** A (sin dependencias)

---

## Contexto: qué ya existe

**Lo que SÍ existe:**

- `simulation/reactions.rs` — `catalysis_result(projected_qe, interf, multiplier)` aplica
  interferencia en spells. `interference_factor ∈ [-1, 1]` ya funciona en combate.
- `blueprint/equations/core_physics/mod.rs` — `interference(f1, phase1, f2, phase2, t)` puro,
  ecuación exacta del Axioma 8: `cos(2π × Δfreq × t + Δphase)`.
- `simulation/metabolic/photosynthesis.rs` — extrae de `EnergyFieldGrid` basado en tuning.
- `simulation/metabolic/trophic.rs` — depredador extrae de presa.
- `simulation/thermodynamic/osmosis.rs` — difusión entre entidades adyacentes.

**Lo que NO existe:**

1. Ninguno de los tres sistemas metabólicos usa `interference()` para modular la extracción.
2. No existe `metabolic_interference_factor()` — versión clampeada a [0,1] para uso ecológico.
3. No existe constante `METABOLIC_INTERFERENCE_MIN_FACTOR` (extracción mínima garantizada).

---

## Objetivo

Aplicar el Axioma 3+8: "interaction magnitude = base × interference_factor" en los
tres sitios de extracción ecológica. La frecuencia pasa de ser ornamental (identidad visual)
a tener consecuencias físicas en el flujo de energía.

```
extracted_final = extracted_raw × metabolic_interference_factor(extractor, target, t)

donde metabolic_interference_factor ∈ [METABOLIC_FLOOR, 1.0]
    METABOLIC_FLOOR = 0.05   (siempre hay algo de extracción — rozamiento basal)
```

---

## Responsabilidades

### AC-1A: Ecuaciones puras

```rust
// src/blueprint/equations/energy_competition/metabolic_interference.rs  (nuevo)

use crate::blueprint::equations::core_physics;
use crate::blueprint::constants::energy_competition_ec::METABOLIC_INTERFERENCE_FLOOR;

/// Factor de acceso metabólico basado en alineación oscilatoria.
/// Rango: [FLOOR, 1.0] — nunca negativo (extracción metabólica, no daño).
/// Ecuación: cos(2π × Δfreq × t + Δphase).clamp(FLOOR, 1.0)
pub fn metabolic_interference_factor(
    extractor_freq: f32, extractor_phase: f32,
    target_freq:    f32, target_phase:   f32,
    t: f32,
) -> f32 {
    let raw = core_physics::interference(
        extractor_freq, extractor_phase,
        target_freq,    target_phase,
        t,
    );
    raw.clamp(METABOLIC_INTERFERENCE_FLOOR, 1.0)
}

/// Aplica el factor al quantum extraído.
/// extracted_raw viene del sistema de extracción antes de este módulo.
pub fn apply_metabolic_interference(extracted_raw: f32, factor: f32) -> f32 {
    (extracted_raw * factor).max(0.0)
}
```

### AC-1B: Constantes

```rust
// src/blueprint/constants/energy_competition_ec.rs — agregar:

/// Piso de extracción metabólica aun cuando la interferencia es destructiva.
/// Un depredador fuera de banda nunca extrae 0 — hay rozamiento basal.
/// Valor: 5% del bruto calculado.
pub const METABOLIC_INTERFERENCE_FLOOR: f32 = 0.05;
```

### AC-1C: Photosynthesis (extracción del campo de energía ambiental)

```rust
// src/simulation/metabolic/photosynthesis.rs — extensión del sistema existente

// ANTES (pseudocódigo actual):
let extracted = extract_proportional(available, capacity);
pool.inject(extracted);

// DESPUÉS — agregar DESPUÉS del cálculo de extracted_raw:
let grid_freq = field_grid.frequency_at(cell_pos);    // frecuencia del terreno
let grid_phase = field_grid.phase_at(cell_pos);
let factor = metabolic_interference_eq::metabolic_interference_factor(
    osc.frequency_hz, osc.phase,
    grid_freq, grid_phase,
    tick.as_f32(),
);
let extracted = metabolic_interference_eq::apply_metabolic_interference(extracted_raw, factor);
pool.inject(extracted);
```

**Efectos emergentes:**
- Organismos Terra (75 Hz) en terreno Terra → factor ≈ 1.0 → fotosíntesis plena
- Organismos Ignis (450 Hz) en terreno Terra (75 Hz) → factor ≈ 0.05 → hambre crónica
- Ignis en terreno Ignis → plena → presión de migrar a zonas resonantes

### AC-1D: Trophic predation

```rust
// src/simulation/metabolic/trophic.rs — extensión del sistema existente

// DESPUÉS de calcular extracted_raw del pool de la presa:
let factor = metabolic_interference_eq::metabolic_interference_factor(
    predator_osc.frequency_hz, predator_osc.phase,
    prey_osc.frequency_hz,     prey_osc.phase,
    tick.as_f32(),
);
let extracted = metabolic_interference_eq::apply_metabolic_interference(extracted_raw, factor);
```

**Efectos emergentes:**
- Depredador y presa en la misma banda → extracción eficiente → presión de especialización
- Depredador inarmónico → falla al predar → muere o migra
- Coevolución: la presa diverge de frecuencia, el depredador la persigue → danza de Hz

### AC-1E: Osmosis

```rust
// src/simulation/thermodynamic/osmosis.rs — extensión

// DESPUÉS de calcular diffused_qe (la cantidad que difundiría sin interferencia):
let factor = metabolic_interference_eq::metabolic_interference_factor(
    src_osc.frequency_hz, src_osc.phase,
    dst_osc.frequency_hz, dst_osc.phase,
    tick.as_f32(),
);
let actual_diffusion = metabolic_interference_eq::apply_metabolic_interference(diffused_qe, factor);
```

**Efectos emergentes:**
- Entidades de la misma banda difunden rápido → equilibrio de energía dentro de la especie
- Entidades de bandas distintas difunden poco → barrera osmótica implícita
- Una célula Lux dentro de un organismo Terra no puede equilibrar fácilmente su energía

---

## No hace

- No modifica `interference()` en `reactions.rs` (catalysis sigue usando [-1, 1]).
- No crea nuevo componente — es un multiplicador en las funciones de extracción.
- No cambia la estructura de `ExtractionProfile` ni `EnergyPool`.
- No requiere AC-4 (aunque con AC-4, el `tick.t` puede incluir degradación de pureza).

---

## Criterios de aceptación

### AC-1A (Ecuaciones)

```
metabolic_interference_factor(75.0, 0.0, 75.0, 0.0, 0.0)  → 1.0  (misma frec, fase 0)
metabolic_interference_factor(75.0, 0.0, 75.0, PI, 0.0)    → FLOOR  (misma frec, fase opuesta)
metabolic_interference_factor(75.0, 0.0, 450.0, 0.0, t)    → varía con t, mín = FLOOR
apply_metabolic_interference(100.0, 0.0)                    → 0.0
apply_metabolic_interference(100.0, FLOOR)                  → 5.0
apply_metabolic_interference(100.0, 1.0)                    → 100.0
```

### AC-1C/D/E (Sistemas)

Test (MinimalPlugins):
- Terra spawneado en terreno Terra → extrae al menos 90% del máximo disponible.
- Ignis spawneado en terreno Terra → extrae ≤ 10% del máximo (rebota en FLOOR × raw).
- Depredador Terra vs presa Terra → predación eficiente.
- Depredador Terra vs presa Lux → predación ineficiente (FLOOR ≤ extracted ≤ 0.2 × raw).
- Osmosis entre dos Terra → difusión normal.
- Osmosis Terra ↔ Lux → difusión ≤ 10% de la normal.

### General

- `cargo test --lib` sin regresión.
- Los tres sistemas metabólicos pasan sus tests previos (el floor garantiza no-zero extraction).
- Sin HashMap en hot path. Sin allocations nuevas en los tres sistemas modificados.

---

## Dependencias

- `blueprint/equations/core_physics/mod.rs` — `interference()` (ya existe)
- `simulation/metabolic/photosynthesis.rs` — sistema existente (extensión)
- `simulation/metabolic/trophic.rs` — sistema existente (extensión)
- `simulation/thermodynamic/osmosis.rs` — sistema existente (extensión)
- `worldgen/field_grid.rs` — `frequency_at(pos)` y `phase_at(pos)` (verificar que exponen estos)

---

## Referencias

- `src/simulation/reactions.rs` — `catalysis_result()` como referencia de cómo se usa interference en spells
- `src/blueprint/equations/core_physics/mod.rs:46-53` — `interference()` pura
- `docs/design/AXIOMATIC_CLOSURE.md §2` — Tier 1 impact analysis
- Axioma 3: "interaction magnitude = base × interference_factor"
- Axioma 8: "interference_factor = cos(Δfreq × t + Δphase)"
