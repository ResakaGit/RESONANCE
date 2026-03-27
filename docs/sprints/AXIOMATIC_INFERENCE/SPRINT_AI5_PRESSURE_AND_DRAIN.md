# Sprint AI-5 — Basal Rate + Radiation Pressure from Dissipation

**Módulo:** `src/simulation/metabolic/basal_drain.rs`, `src/blueprint/constants/nucleus_lifecycle.rs`
**Tipo:** Constantes eliminadas → derivación
**Eje axiomático:** Axiom 4 (dissipation scales everything)
**Estado:** ⏳ Pendiente
**Bloqueado por:** AI-2 (density thresholds para pressure)
**Esfuerzo:** Bajo (~20min)

---

## Qué existe hoy

```rust
// basal_drain.rs:
const BASAL_RATE: f32 = 1.0;  // EMPIRICALLY-CALIBRATED

// nucleus_lifecycle.rs:
pub const RADIATION_PRESSURE_THRESHOLD_QE: f32 = 100.0;   // EMPIRICALLY-CALIBRATED
pub const RADIATION_PRESSURE_TRANSFER_RATE: f32 = 0.05;    // EMPIRICALLY-CALIBRATED
```

## Derivación axiomática

- **BASAL_RATE:** El costo de vivir es la disipación del Solid amplificada por el factor de escala.
  `basal_rate = DISSIPATION_SOLID × 200 = 0.005 × 200 = 1.0`. La constante 200 es el amplificador
  que convierte disipación pasiva (lenta) en metabolismo activo (rápido) — derivable de la relación
  entre disipación de campo (qe/s en grid) y consumo de entidad (qe/tick en organismo).

- **PRESSURE_THRESHOLD:** La presión se activa cuando una celda supera la densidad de gas.
  `threshold = gas_density_threshold()`. Por debajo, la difusión pasiva maneja la redistribución.

- **PRESSURE_RATE:** La tasa de transferencia = disipación del Gas (Axiom 4: redistribución
  es un proceso disipativo). `rate = DISSIPATION_GAS = 0.08`.

## Tareas

1. `basal_drain.rs`: reemplazar `BASAL_RATE` con `derived_thresholds::basal_drain_rate()`.
2. `nucleus_lifecycle.rs`: reemplazar `RADIATION_PRESSURE_THRESHOLD_QE` y `RADIATION_PRESSURE_TRANSFER_RATE`
   con re-exports de `derived_thresholds`.
3. `radiation_pressure.rs` (sistema): actualizar imports.

## Criterio de cierre

- `BASAL_RATE` eliminado como constante local
- `RADIATION_PRESSURE_THRESHOLD_QE` y `RADIATION_PRESSURE_TRANSFER_RATE` derivados
- Demo headless produce presión visible (campos no se saturan)
