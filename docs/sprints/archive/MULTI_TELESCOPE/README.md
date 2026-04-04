# Track: MULTI_TELESCOPE — Ejecución Especulativa Jerárquica Cuántica

**Objetivo:** Implementar un stack de N telescopios donde cada nivel proyecta desde el output del inferior, las correcciones operan por colapso + re-emanación (no cascade), y la incertidumbre se cuantifica por la relación de Englert D²+V²≤1. Escala: abiogénesis → modernidad (~4.3×10⁹ ticks) con 8 niveles × K=16.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Bloqueado por:** ADR-015 ✅ (Temporal Telescope implementado, 179 tests)
**Desbloquea:** Simulación a escala geológica, fast-forward al futuro con cono de incertidumbre
**ADR:** `docs/arquitectura/ADR/ADR-016-multi-telescope.md`

---

## Principios de Diseño

### 1. Cuántico: Colapso + Re-Emanación (no cascade)

```
ANTES (ADR-015): Ancla llega → diff → cascade correction → parche incremental
                 Problema: error residual se acumula entre niveles

AHORA (ADR-016): Ancla llega → COLAPSO → niveles anteriores DESTRUIDOS
                 → RE-EMANAR todos los niveles desde verdad fresca
                 Resultado: cero acumulación de error
```

### 2. Stateless puro entre niveles

```
Cada nivel: fn(source_world, metrics, weights, K) → projected_world
Colapso:    fn(anchor, old_stack, history) → (new_stack, updated_weights, records)
Visibilidad: fn(ticks_to_anchor, coherence_length) → f32

Ningún nivel guarda referencia a otro.
Los datos fluyen en una sola dirección: Ancla → Level 0 → Level 1 → ... → Level N.
El aprendizaje fluye hacia abajo: DiffReport → calibrate → weights para el próximo ciclo.
```

### 3. Conservation-Bounded (Axioma 4+5)

```
INVARIANTE: para todo nivel L, projected_qe(L) ≤ source_qe(L-1)
MECANISMO: clamp(base_decay, current_qe) después de cada project_entity
VERIFICACIÓN: property test en cada sprint
```

### 4. Testing: comportamiento real, no mocks

Cada sprint tiene tests que verifican **comportamiento físico**, no implementación:
- "La energía nunca crece" (Axioma 5)
- "La incertidumbre crece con la distancia" (Englert)
- "El colapso produce re-emanación fresca" (cero residuo)
- "Los pesos convergen con las reconciliaciones" (aprendizaje)

---

## Archivos que se CREAN

```
src/batch/telescope/stack.rs              (MT-3: TelescopeStack, collapse_and_emanate)
```

## Archivos que se MODIFICAN

```
src/blueprint/equations/temporal_telescope.rs  (MT-1: speculative_visibility, conservation_bounded, frequency_aware_decay)
src/blueprint/constants/temporal_telescope.rs  (MT-1: MAX_LEVELS, DEFAULT_COHERENCE_LENGTH)
src/batch/telescope/projection.rs              (MT-2: aplicar conservation clamp + frequency-aware decay)
src/batch/telescope/mod.rs                     (MT-3: pub mod stack)
src/batch/telescope/pipeline.rs                (MT-4: tick_telescope_stack_sync usando TelescopeStack)
src/batch/telescope/activation.rs              (MT-5: TelescopeSummary para multi-nivel)
```

## Archivos que NO se modifican

```
src/batch/telescope/diff.rs                    (reutilizado: world_diff como señal de aprendizaje)
src/batch/telescope/calibration_bridge.rs      (reutilizado: calibrate, identify_weak_normalizer)
src/batch/telescope/cascade.rs                 (no usado para inter-nivel; se mantiene para uso intra-nivel)
src/batch/systems/*.rs                         (33 sistemas batch intactos)
src/blueprint/equations/derived_thresholds.rs   (4 constantes fundamentales intactas)
src/batch/arena.rs                             (SimWorldFlat sin cambios)
```

---

## 5 Sprints

| Sprint | Título | Entregable | Dependencias |
|--------|--------|------------|--------------|
| [MT-1](SPRINT_MT1_QUANTUM_EQUATIONS.md) | Ecuaciones cuánticas | speculative_visibility, conservation_bounded_project, frequency_aware_decay_rate + constantes | — |
| [MT-2](SPRINT_MT2_CONSERVATION_PROJECTION.md) | Proyección conservation-bounded | Aplicar clamp + frequency-aware decay en project_entity | MT-1 |
| [MT-3](SPRINT_MT3_TELESCOPE_STACK.md) | Stack de telescopios | TelescopeStack, TelescopeLevel, collapse_and_emanate | MT-1, MT-2 |
| [MT-4](SPRINT_MT4_STACK_PIPELINE.md) | Pipeline del stack | tick_telescope_stack_sync (ciclo completo multi-nivel) | MT-3 |
| [MT-5](SPRINT_MT5_ACTIVATION.md) | Activación y métricas | Dashboard multi-nivel, niveles adaptativos, coherence_length dinámico | MT-4 |

---

## Grafo de dependencias

```
MT-1 (ecuaciones cuánticas) ──→ MT-2 (conservation projection) ──→ MT-3 (stack) ──→ MT-4 (pipeline) ──→ MT-5 (activation)
```

Secuencial. Cada sprint construye sobre el anterior. MT-1 es paralelizable con trabajo en otros tracks.

---

## Criterios de cierre del track

- [ ] speculative_visibility implementada y testeada (Englert D²+V²≤1)
- [ ] Conservation clamp: project_entity.qe ≤ input.qe (Axioma 4+5, property test)
- [ ] Frequency-aware decay: resonancia solar modula disipación (Axioma 8)
- [ ] TelescopeStack con MAX_LEVELS=8, zero-heap
- [ ] collapse_and_emanate: destruye + reconstruye (no parcha)
- [ ] Calibración aprende de cada colapso (weights convergen)
- [ ] active_levels=1 produce comportamiento idéntico a ADR-015
- [ ] 8 niveles × K=16 alcanza 4.3×10⁹ ticks sin error acumulado
- [ ] Cada nivel reporta visibilidad V (0=colapsado, 1=onda pura)
- [ ] `cargo test --lib` verde, 0 warnings en código nuevo
- [ ] 0 `unsafe`, 0 `async`, 0 `Arc<Mutex>`, 0 `unwrap()` en sistemas
