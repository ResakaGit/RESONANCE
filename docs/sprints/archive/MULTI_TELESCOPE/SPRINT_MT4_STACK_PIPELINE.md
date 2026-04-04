# MT-4: Pipeline del Stack

**Objetivo:** Función orquestadora que ejecuta un ciclo completo del multi-telescopio: fork → ancla simula K₀ ticks → colapso + re-emanación del stack completo. Modo síncrono para tests. Reutiliza el 100% de ADR-015 — solo agrega la capa de stack encima.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Medio (orquestación, no math nueva)
**Bloqueado por:** MT-3 (TelescopeStack, collapse_and_emanate)
**Desbloquea:** MT-5 (activación)

---

## Entregable

### Modificación en `src/batch/telescope/pipeline.rs`

```rust
/// Ejecuta un ciclo completo del multi-telescopio (síncrono).
///
/// 1. Fork: clonar mundo para ancla
/// 2. Ancla simula K₀ ticks (simulación completa, 33 sistemas)
/// 3. Colapso: ancla llega, onda muere, niveles se re-emanan
/// 4. El mundo resultante es el ANCLA (verdad), no la proyección
///
/// Con stack.active_levels=1: comportamiento idéntico a tick_telescope_sync (ADR-015).
pub fn tick_telescope_stack_sync(
    world: &mut SimWorldFlat,
    stack: &mut TelescopeStack,
    cal_config: &CalibrationConfig,
    history: &mut ReconciliationHistory,
    scratch: &mut ScratchPad,
    qe_history: &[f32],
    pop_history: &[f32],
) -> StackTickResult

/// Resultado de un ciclo del stack.
pub struct StackTickResult {
    pub k_anchor: u32,                  // ticks simulados por el ancla
    pub total_reach: u64,               // alcance total del stack (∏ Kᵢ)
    pub collapse: CollapseResult,       // resultado del colapso
    pub metrics: RegimeMetrics,         // métricas al momento del fork
    pub active_levels: u8,              // niveles activos post-colapso
}
```

---

## Flujo del Pipeline

```
tick_telescope_stack_sync(world, stack, ...):

  1. MÉTRICAS: compute_regime_metrics(world, qe_history, pop_history)
     → coherence_length = f(H, ρ₁, λ_max)
     → stack.coherence_length = coherence_length

  2. FORK: anchor = world.clone()

  3. ANCLA SIMULA: K₀ = stack.levels[0].k
     for _ in 0..K₀ { anchor.tick(scratch); }
     // 33 sistemas batch completos. Axiomas respetados. Conservación garantizada.

  4. COLAPSO + RE-EMANACIÓN:
     collapse_result = collapse_and_emanate(stack, &anchor, &metrics, cal_config, history)
     // Todos los niveles destruidos y reconstruidos desde verdad fresca.

  5. COMMIT: *world = anchor
     // La verdad SIEMPRE gana. El telescopio es descartable.

  6. RETURN StackTickResult { k_anchor, total_reach, collapse_result, metrics, active_levels }
```

---

## Contrato

- `world` después de la función contiene el estado del ANCLA (no del telescopio)
- `stack` contiene niveles re-emanados desde la verdad del ancla
- `history` contiene N registros nuevos (uno por nivel activo)
- Conservation: `world.total_qe ≤ world_before.total_qe + tolerancia_irradiance`
- Determinismo: misma semilla + misma config → mismo resultado bit-exacto

---

## Preguntas para tests

### Comportamiento básico
1. `tick_telescope_stack_sync` con mundo vacío → ¿no panic, result.k_anchor > 0?
2. Después de la función: ¿world.tick_id avanzó K₀ ticks?
3. Después de la función: ¿world contiene estado del ANCLA? (no telescopio)
4. `result.total_reach` = ∏ Kᵢ para niveles activos

### Conservation (Axioma 5)
5. 10 entidades, mundo estable: ¿world.total_qe ≤ before? (energía no crece)
6. Después de colapso: ¿stack.levels[N].total_qe ≤ stack.levels[0].total_qe? (monótono por nivel)

### Determinismo
7. Dos ejecuciones con misma semilla → ¿bit-exact en world y stack?
8. Modo stack (active_levels=3) vs modo simple (active_levels=1): ¿world idéntico? (ancla es la misma)

### Compatibilidad ADR-015
9. Con active_levels=1: ¿resultado de tick_telescope_stack_sync ≈ tick_telescope_sync?
10. History recibe registros de todos los niveles

### Convergencia multi-ciclo
11. 20 ciclos consecutivos, mundo estable: ¿active_levels crece progresivamente?
12. 20 ciclos: ¿total_reach crece? (K adaptativo sube en estasis)
13. 20 ciclos: ¿collapse.max_diff_class tiende a Perfect? (calibración mejora)
14. Mundo que transiciona a mitad: ¿active_levels se reduce? ¿K baja?

### Alcance geológico
15. 8 niveles × K=16: ¿total_reach ≈ 4.3×10⁹? (verificar aritmética)

---

## Integración

- **Consume:** MT-3 (TelescopeStack, collapse_and_emanate), ADR-015 (compute_regime_metrics, ReconciliationHistory, CalibrationConfig)
- **Consumido por:** MT-5 (activación, dashboard)
- **Modifica:** `pipeline.rs` (agrega tick_telescope_stack_sync + StackTickResult)
- **No modifica:** tick_telescope_sync (ADR-015 se mantiene intacto para active_levels=1)
