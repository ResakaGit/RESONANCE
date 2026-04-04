# MT-3: Stack de Telescopios

**Objetivo:** Implementar `TelescopeStack` (array fijo de 8 niveles, zero-heap) con el algoritmo de colapso + re-emanación cuántico. Cuando el ancla llega, los niveles se destruyen y se reconstruyen frescos — cero acumulación de error.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Medio (struct + algoritmo de colapso + tests de convergencia)
**Bloqueado por:** MT-1 (speculative_visibility), MT-2 (conservation-bounded projection)
**Desbloquea:** MT-4 (pipeline del stack)

---

## Entregable

### `src/batch/telescope/stack.rs` (archivo nuevo)

```rust
/// Nivel individual del telescopio.
pub struct TelescopeLevel {
    pub state: TelescopeState,          // de ADR-015
    pub projected_world: SimWorldFlat,  // estado proyectado
    pub k: u32,                         // K de este nivel
    pub visibility: f32,                // V de Englert [0=colapsado, 1=onda]
}

/// Stack de telescopios. Array fijo, zero-heap. MAX_LEVELS=8.
pub struct TelescopeStack {
    pub levels: [TelescopeLevel; MAX_LEVELS],
    pub active_levels: u8,
    pub coherence_length: f32,
}

/// Resultado del colapso + re-emanación.
pub struct CollapseResult {
    pub records: [ReconciliationRecord; MAX_LEVELS],
    pub records_count: u8,
    pub max_diff_class: DiffClass,      // peor diff encontrado
    pub levels_rebuilt: u8,             // cuántos niveles re-emanados
}

/// Colapso cuántico + re-emanación de todos los niveles.
/// Ancla llega → destruir ondas → calibrar desde diff → reconstruir frescos.
/// Stateless respecto al ancla: ancla es read-only.
pub fn collapse_and_emanate(
    stack: &mut TelescopeStack,
    anchor: &SimWorldFlat,
    metrics: &RegimeMetrics,
    cal_config: &CalibrationConfig,
    history: &mut ReconciliationHistory,
) -> CollapseResult

/// Re-emana un nivel desde su fuente. Pura: (source, metrics, weights, K) → world.
fn emanate_level(
    source: &SimWorldFlat,
    metrics: &RegimeMetrics,
    weights: &NormalizerWeights,
    k: u32,
) -> SimWorldFlat

/// Decide si agregar un nivel. Pura: (stack, metrics) → bool.
pub fn should_add_level(stack: &TelescopeStack, metrics: &RegimeMetrics) -> bool

/// Decide si remover un nivel. Pura: (stack) → bool.
pub fn should_remove_level(stack: &TelescopeStack) -> bool

/// Alcance total del stack en ticks: ∏ Kᵢ para niveles activos.
pub fn total_reach(stack: &TelescopeStack) -> u64
```

---

## Algoritmo de Colapso

```
collapse_and_emanate(stack, anchor, metrics, cal_config, history):

  // Fase 1: MEDIR — comparar cada nivel con su verdad
  for level in 0..stack.active_levels:
    let truth = if level == 0 { anchor } else { &stack[level-1].projected_world }
    let diff = world_diff(truth, &stack[level].projected_world, DIFF_THRESHOLD_PCT)
    records[level] = ReconciliationRecord { diff.class, ... }

  // Fase 2: CALIBRAR — aprender de la medición (pesos actualizados)
  for level in 0..stack.active_levels:
    stack[level].state.weights = calibrate(&records[level], &weights, history, cal_config)
    history.push(records[level])

  // Fase 3: COLAPSAR + RE-EMANAR — destruir ondas, reconstruir frescos
  stack[0].projected_world = anchor.clone()  // Level 0 = verdad colapsada
  stack[0].visibility = 0.0

  for level in 1..stack.active_levels:
    let source = &stack[level-1].projected_world
    let k = optimal_k(&metrics, &stack[level].state.weights, K_MIN, K_MAX)
    stack[level].projected_world = project_world(source, &metrics, &weights, k)
    stack[level].k = k
    let ticks = total_reach_up_to(stack, level)
    stack[level].visibility = speculative_visibility(ticks, stack.coherence_length)

  // Fase 4: ADAPTAR — crecer o reducir
  if should_add_level(stack, metrics) && stack.active_levels < MAX_LEVELS:
    stack.active_levels += 1
    emanate new level from previous
  if should_remove_level(stack) && stack.active_levels > 1:
    stack.active_levels -= 1
```

---

## Contrato stateless

`collapse_and_emanate` muta `stack` y `history` (owned por el caller). `anchor` es read-only. Cada `emanate_level` es pura: `(source, metrics, weights, K) → SimWorldFlat`. No hay estado compartido entre niveles excepto el flujo descendente source → projected_world.

---

## Preguntas para tests

### Colapso + Re-Emanación
1. Stack con 1 nivel activo: ¿collapse produce nivel 0 = anchor? (identity)
2. Stack con 3 niveles: ¿todos reconstruidos desde verdad fresca? (no residuo)
3. Después de collapse: ¿level[0].visibility = 0.0? (colapsado)
4. Después de collapse: ¿level[N].visibility > level[0].visibility? (crece con distancia)
5. Después de collapse: ¿level[N].projected_world.total_qe ≤ anchor.total_qe? (Axioma 5 en cascada)

### Conservation a través de niveles
6. 8 niveles activos: ¿qe monótonamente decreciente level 0 → level 7? (property test)
7. K=16 por nivel, mundo estable: ¿total_reach = 16⁸ ≈ 4.3×10⁹?
8. Ningún nivel tiene entidad con qe > qe del nivel anterior (property test)

### Englert D²+V²≤1
9. Para cada nivel: D² + V² ≤ 1.0 (invariante cuántico)
10. Visibilidad crece monótonamente con el nivel (más lejos = más incertidumbre)

### Niveles Adaptativos
11. En estasis prolongada (10 colapsos PERFECT): ¿active_levels crece?
12. En transición (3 colapsos SYSTEMIC): ¿active_levels decrece?
13. active_levels nunca < 1 (mínimo: un telescopio)
14. active_levels nunca > MAX_LEVELS (máximo: 8)

### Compatibilidad ADR-015
15. Con active_levels=1: ¿comportamiento idéntico a tick_telescope_sync? (regresión)
16. ReconciliationHistory recibe records de TODOS los niveles (no solo del primero)

### Calibración Convergente
17. 50 colapsos con mundo estable → ¿weights convergen? (no divergen)
18. Records contienen métricas de CADA nivel (no promediados)

---

## Integración

- **Consume:** MT-1 (speculative_visibility), MT-2 (project_world conservation-bounded), ADR-015 (diff, calibrate, optimal_k, project_world, TelescopeState, NormalizerWeights)
- **Consumido por:** MT-4 (pipeline usa collapse_and_emanate), MT-5 (dashboard lee visibility)
- **Crea:** `stack.rs`
- **Modifica:** `mod.rs` (1 línea: `pub mod stack;`)
