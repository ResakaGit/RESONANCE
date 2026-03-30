# BS-2: Bug Fixes — CompetitionNormBridge, Hot Reload, expect()

**Objetivo:** Corregir 3 bugs confirmados en el Bridge Optimizer antes de extenderlo.

**Estado:** PENDIENTE
**Esfuerzo:** S (~80 LOC)
**Bloqueado por:** —
**Desbloquea:** — (hacer primero, sin dependencias)

---

## Bug 1: CompetitionNormBridge sin lifecycle orchestration

### Problema

`CompetitionNormBridge` está definido en `config.rs:65`, tiene preset en `presets/ecosystem.rs`, y se aplica en `apply_bridge_config_asset()`. **Pero NO está en:**

1. `context_fill.rs` → `bridge_caches_max_fill_ratio()` macro `scan!()` — no se escanea su fill ratio
2. `context_fill.rs` → `apply_bridge_phase_side_effects()` macro `each!()` — no se toggle enabled/eviction
3. `context_fill.rs` → `clear_all_bridge_caches()` macro `clear!()` — no se limpia en reset
4. `metrics.rs` → `bridge_metrics_collect_all()` — no se recolectan métricas
5. `metrics.rs` → `rebuild_bridge_metrics_summary()` — no aparece en reporte

### Consecuencia

- Cache de CompetitionNormBridge **nunca transiciona** de Warmup→Filling→Active
- Eviction **siempre habilitada** (default del constructor), incluso durante Filling
- Métricas **invisibles** — hit rate desconocido

### Fix

Agregar `CompetitionNormBridge` a las 5 macro invocaciones. Patrón: grep cada macro y verificar que los 12 bridges activos (Density, Temperature, PhaseTransition, Interference, Dissipation, Drag, Engine, Will, Catalysis, CollisionTransfer, Osmosis, CompetitionNorm) estén presentes.

### Test

```
competition_norm_bridge_enabled_toggles_with_phase_transition
competition_norm_bridge_appears_in_metrics_summary
competition_norm_bridge_fill_ratio_scanned_in_filling_phase
```

---

## Bug 2: Hot reload no resetea BridgePhaseState

### Problema

`bridge_config_hot_reload_system()` en `presets/mod.rs:443-477`:
- Aplica nuevas configs correctamente
- Limpia caches si `had_built_before`
- **NO toca `BridgePhaseState`**

### Escenario de falla

```
Tick 100: Warmup → Filling (auto-transition)
Tick 110: Usuario edita bridge_config.ron (bandas más anchas)
Tick 111: Hot reload dispara; configs actualizadas, caches limpias
          BridgePhaseState.phase == Filling, ticks_in_filling == 10
          fill_ratio recalculada con caches vacías → 0.0
          → OK por ahora, pero si las bandas nuevas generan pocos buckets:
Tick 115: fill_ratio = 0.85 (con bandas anchas llena rápido)
          → Filling→Active prematuramente con solo 5 ticks de warm-up
          → Cache semi-poblada, hit rate pobre
```

### Fix

En `bridge_config_hot_reload_system`, después de aplicar configs:

```rust
if clear_caches {
    let current_phase = world.resource::<BridgePhaseState>().phase;
    if current_phase != BridgePhase::Active {
        // Reset al inicio del ciclo para re-calibrar con nuevos parámetros
        bridge_phase_reset(world);
    }
    // Si ya en Active, solo clear caches (apply_bridge_config_asset ya lo hace)
}
```

### Test

```
hot_reload_in_filling_resets_to_warmup
hot_reload_in_active_keeps_active_but_clears_caches
hot_reload_in_warmup_resets_counters
```

---

## ~~Bug 3: expect() en hot path de ops.rs~~ — FALSO POSITIVO

### Análisis

`bridge/impls/ops.rs:796,802` — los `expect()` están dentro de `#[cfg(test)]`, son código de test exclusivamente. No hay `expect()` ni `unwrap()` en funciones no-test de ops.rs.

**Acción:** Ninguna.

---

## Archivos tocados

| Archivo | Cambio |
|---------|--------|
| `src/bridge/context_fill.rs` | + CompetitionNormBridge en 3 macros |
| `src/bridge/metrics.rs` | + CompetitionNormBridge en 2 macros |
| `src/bridge/presets/mod.rs` | Reset phase en hot reload |
| `src/bridge/impls/ops.rs` | expect → let-else |

---

## Checklist pre-merge

- [ ] `CompetitionNormBridge` en las 5 invocaciones macro (audit grep)
- [ ] Todos los 12 bridges activos presentes en cada macro (audit exhaustivo)
- [ ] Hot reload resetea a Warmup si no en Active
- [ ] Zero `expect()` en funciones no-test de `impls/ops.rs`
- [ ] `cargo test --lib` verde
- [ ] Tests nuevos cubren los 3 bugs
