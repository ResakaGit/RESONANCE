# TT-10: Activación y Dashboard

**Objetivo:** Conectar los sistemas dormidos (GeologicalLOD, MultiscaleSignalGrid) al Telescopio. Exponer métricas al dashboard. Este sprint no crea lógica nueva — conecta piezas existentes.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Bajo (wiring, no math nueva)
**Bloqueado por:** TT-9 (pipeline dual)
**Desbloquea:** Casos de uso CU-1 a CU-5

---

## Entregables

### 1. Wire GeologicalLOD → Telescope K

**Archivo:** `src/simulation/emergence/geological_lod.rs` (modificación)

Actualmente `GeologicalLOD` tiene niveles estáticos: `[1, 10, 100, 1000]`. Conectar al K dinámico del Telescopio:

```rust
// Antes: LOD level seleccionado por entity count vs budget
// Después: LOD level = telescope.current_k (clamped a [1, 10, 100, 1000])
// El Telescopio ya calcula K óptimo — GeologicalLOD lo consume directamente.
```

### 2. Wire MultiscaleSignalGrid → Normalizers

**Archivo:** `src/simulation/emergence/multiscale.rs` (modificación)

MultiscaleSignalGrid ya agrega energía en 3 niveles (local → regional → global). Alimentar el nivel regional como input para Fisher information y Shannon entropy:

```rust
// regional_signals[64] → fisher_information(current, previous, dt)
// regional_signals[64] → shannon_entropy(regional_signals)
// Estos valores alimentan RegimeMetrics del Telescopio.
```

### 3. Dashboard metrics

**Archivo:** `src/runtime_platform/dashboard_bridge.rs` (modificación)

Agregar a `SimTickSummary` o crear `TelescopeSummary`:

```rust
pub struct TelescopeSummary {
    pub phase: TelescopePhase,
    pub current_k: u32,
    pub projection_accuracy: f32,     // media de últimas 10 reconciliaciones
    pub correction_frequency: f32,    // LOCAL+SYSTEMIC / total reconciliations
    pub hurst: f32,
    pub autocorrelation: f32,
    pub fisher: f32,
    pub lambda_max: f32,
    pub regime_label: &'static str,   // "STASIS" / "PRE-TRANS" / "TRANSITION" / "POST-TRANS"
}
```

### 4. Registrar sistemas en plugins

Registrar `GeologicalLOD` y `MultiscaleSignalGrid` en el plugin correspondiente si no están registrados.

---

## Preguntas para tests

1. GeologicalLOD con telescope.current_k=64 → ¿LOD level = 10? (nearest bucket)
2. GeologicalLOD con telescope.current_k=1 → ¿LOD level = 1? (full detail)
3. MultiscaleSignalGrid regional_signals alimenta fisher_information correctamente
4. TelescopeSummary.projection_accuracy = mean(last 10 reconciliation accuracies)
5. TelescopeSummary.correction_frequency = (local+systemic) / total
6. TelescopeSummary.regime_label = "STASIS" cuando σ² < threshold y ρ₁ < 0.8
7. Dashboard se actualiza cada tick sin overhead significativo (< 1% CPU)
8. GeologicalLOD desconectado (no telescope) → ¿comportamiento legacy intacto?
9. MultiscaleSignalGrid desconectado → ¿Fisher/entropy usan fallback (EnergyFieldGrid directo)?
10. Todo funciona con telescope deshabilitado (TelescopePhase::Idle)

---

## Integración

- **Consume:** TT-9 (pipeline), TT-1 (statistics), TT-3 (metrics)
- **Modifica:**
  - `simulation/emergence/geological_lod.rs` (wire to K)
  - `simulation/emergence/multiscale.rs` (feed to normalizers)
  - `runtime_platform/dashboard_bridge.rs` (expose metrics)
- **No modifica:** Ningún sistema batch, ningún layer, ninguna constante fundamental
