# ADR-046: Prueba de emergencia (Ax6) vía order parameter de Kuramoto + ablación causal

**Estado:** Aceptado — veredicto del spike cargado (§10), H0 refutada en 6/6 seeds
**Fecha:** 2026-07-09 · veredicto 2026-08-17
**Contexto:** EMERGENT_MEASUREMENTS (Sprint EM-1, ítem EM-1.4)
**ADRs relacionados:** ADR-045 (patrón spike + veredicto) · **ADR-047** (ejecuta
la Fase 4 de §3/§9: el mismo axioma sobre el `entrainment_system` ECS real)

## 1. Contexto y problema

- Módulos afectados (NUEVOS):
  - `src/blueprint/equations/emergence/synchronization.rs` — order parameter + paso mean-field
  - `src/blueprint/constants/synchronization_a6.rs` — umbrales
  - `src/use_cases/experiments/emergence_sync.rs` — experimento Config→Report
  - `tests/axioms/emergence_sync.rs`, `tests/axioms/r10_emergence_gates.rs`

El Axioma 6 ("Emergence at Scale — el comportamiento a escala N es consecuencia de
N−1, sin programación top-down") era el **único axioma sin prueba**. La matriz
`docs/design/AXIOM_LAYER_VALIDATION_MATRIX.md` lo marcaba como guard-rail derivado:
sólo L7/L9 tenían celda Ax6, y era *estructural* ("la capa no viola bottom-up"), no
una medición.

**Problema:** sin una propiedad **agregada** (que no exista a nivel individual) más
una **prueba causal** (que el patrón desaparezca al quitar la regla local), el claim
central del proyecto es narrativa. La infraestructura de entrainment de Kuramoto ya
existía (`equations/emergence/entrainment.rs`, `simulation/emergence/entrainment.rs:28`)
pero sin el order parameter global ni un experimento que lo midiera.

## 2. Experimento diseñado (ítem EM-1.4)

**Hipótesis nula (H0):** *el orden global aparece con o sin la regla local de
acoplamiento* — es decir, la sincronización no es consecuencia de la interacción
N−1. Refutarla prueba Ax6.

| Observable | Métrica | Tolerancia |
|---|---|---|
| Orden acoplado | `r_final = ⟨|(1/N)Σe^{iθ}|⟩` (cola, regla activa) | `> SYNC_R_PASS_MIN` (0.5) |
| Orden ablado | `r_final_ablated` (misma población, `coupling = 0`) | `< SYNC_ABLATED_R_MAX` (0.1) |
| Gap emergente | `r_final − r_final_ablated` | `> SYNC_GAP_MIN` (0.4) |

**Escenario:** N=256 osciladores de fase, `ω ~ N(0, 0.10)`, arranque de fase
uniforme en `[0,2π)`, acoplamiento supercrítico `K = 4·spread`, 2000 ticks, marco
rotante. Determinista (PCG interno, sin `rand`).

**Procedimiento.**
1. `init_state(seed)` genera fases y frecuencias desordenadas.
2. Run A (acoplado): integrar `dθ_i = ω_i + K·R·sin(ψ−θ_i)`; medir `R(t)`.
3. Run B (ablado): **mismo estado inicial**, `K = 0`; medir `R(t)`.
4. Veredicto: `emergence_sync_verdict(r_final, r_final_ablated)`.
5. Gate: `tests/axioms/r10_emergence_gates.rs` valida consistencia de umbrales + boundary.

## 3. Decisión y alternativas

**Decisión:** medir Ax6 con el order parameter de Kuramoto sobre un mean-field de
**fase** headless, con ablación por `coupling = 0`. Alternativas consideradas:

| Opción | Descripción | Contras |
|---|---|---|
| **A (elegida)** | Mean-field de fase puro headless | Requiere módulo nuevo; no reusa el system ECS |
| B | Reusar `entrainment_system` ECS (acopla frecuencia) | Acopla ω no θ; el order parameter de fase no colapsa limpio; menos testeable (necesita App) |
| C | R sobre distribución de frecuencias | Métrica menos canónica; sin transición crítica clara |

Se eligió A: es la transición de sincronización canónica (con `K_c`), headless
(testeable sin Bevy, canon `SPRINT_PRIMER.md:187`), y la ablación es un parámetro
limpio sobre estado inicial idéntico. Un probe ECS de corroboración (opción B) queda
como Fase 4 opcional.

## 4. Justificación

1. **`R` es una propiedad de escala**, no individual: un solo oscilador no tiene
   "sincronización". Mide exactamente lo que Ax6 postula.
2. **La ablación es el contrafáctico N−1→N**: misma población, sólo se apaga la
   interacción entre unidades. Si el orden persiste sin ella, no es emergente.
3. **No hay número mágico**: `SYNC_DT` deriva de `DISSIPATION_GAS` (Ax4); el
   acoplamiento deriva del spread vía `SYNC_SUPERCRITICAL_RATIO`.
4. **Reusa el sustrato existente** (Kuramoto de `entrainment.rs`), no inventa física.

## 5. No viola axiomas

| Ax | Cumplimiento |
|---|---|
| 1 Energy | `R` lee fases, no crea/consume qe; el experimento no inyecta energía |
| 2 Pool | No toca pools; ninguna transferencia |
| 3 Competition | Sin competencia; sólo mide coherencia |
| 4 Dissipation | `SYNC_DT = DISSIPATION_GAS`; el integrador no crea energía |
| 5 Conservation | `R` es función pura de sólo-lectura; conservación intacta |
| 6 **Emergence** | **Eje central.** `R` alto con la regla y `R≈0` ablado = el orden es consecuencia de N−1, no top-down. `R` **observa**, no programa comportamiento |
| 7 Distance | El mean-field es all-to-all por diseño de la prueba; el system ECS real sí decae con distancia (AC-4). `R` no impone campo |
| 8 Oscillatory | `R = |Σe^{iθ}|` usa la misma álgebra `cos/sin` del sustrato oscilatorio (L2) |

## 6. Costos

- **Compilación:** +1 módulo de equations, +1 de constants, +1 experimento, +2 tests. Trivial.
- **Runtime:** experimento headless O(N·ticks) por condición; 256×2000×2 ≈ 10⁶ pasos, ms.
- **Memoria:** dos `Vec<f32>` de tamaño `ticks`. Despreciable.
- **Complejidad:** baja; funciones puras, sin estado global.

## 7. Archivos

| Archivo | Cambio |
|---|---|
| `src/blueprint/equations/emergence/synchronization.rs` | NUEVO — R + step mean-field + verdict |
| `src/blueprint/constants/synchronization_a6.rs` | NUEVO — 6 constantes |
| `src/use_cases/experiments/emergence_sync.rs` | NUEVO — SyncConfig/SyncReport/run |
| `tests/axioms/emergence_sync.rs` | NUEVO — prueba de emergencia + control negativo |
| `tests/axioms/r10_emergence_gates.rs` | NUEVO — gates de umbral + boundary |
| `emergence/mod.rs`, `constants/mod.rs`, `experiments/mod.rs` | + registro de módulos |
| `docs/design/AXIOM_LAYER_VALIDATION_MATRIX.md` | + fila/ficha SYNC, Ax6 medible |

## 8. Tests

- **Unit** (`emergence::synchronization`): R de fases idénticas/antifase/uniformes, invarianza rotacional, ψ, paso mean-field, verdict.
- **Integration** (`--test axioms emergence_sync::`): acoplado sincroniza, ablado no (control negativo causal), gap decisivo, reproducible sobre 5 seeds.
- **Gate** (`--test axioms r10_emergence_gates::`): umbrales en rango + consistencia gap + boundary del verdict.

## 9. Decisión revisable cuando

- ~~El order parameter de fase se corrobore (o refute) con el `entrainment_system` ECS real (Fase 4).~~
  **Fase 4 ejecutada vía ADR-047** (2026-08-17): el motor real acopla ω, no θ, así
  que el probe usa su propia métrica (colapso de σ_ω) y sus propias ablaciones. El
  resultado corrobora — el veredicto ECS vive en ADR-047 §10, no aquí.
- Se quiera medir la **transición crítica** `R(K)` (barrido de acoplamiento) para localizar `K_c` empírico.
- Se añada ruido térmico y se estudie la robustez de la sincronización.

## 10. Veredicto del spike

**PASS — H0 refutada para este sustrato (mean-field de fase analítico).**

Medido con `cargo run --bin measure_emergence -- <seed> 256 2000`
(N=256, `spread = 0.10`, `K = SYNC_SUPERCRITICAL_RATIO × spread = 0.400`,
2000 ticks, `r_final` = media de la cola `SYNC_TAIL_WINDOW_FRAC = 25 %`):

| seed | `r_final` (acoplado) | `r_final_ablated` (K=0) | gap ΔR |
|---:|---:|---:|---:|
| 11 | 0.9617 | 0.0907 | 0.8711 |
| 1 | 0.9674 | 0.0335 | 0.9339 |
| 2 | 0.9574 | 0.0494 | 0.9080 |
| 3 | 0.9608 | 0.0474 | 0.9133 |
| 100 | 0.9605 | 0.0461 | 0.9145 |
| 777 | 0.9633 | 0.0689 | 0.8944 |
| **media** | **0.9619** | **0.0560** | **0.9059** |

Los tres umbrales de §2 se cruzan con margen en **6/6 seeds**:
`r_final > SYNC_R_PASS_MIN (0.5)`, `r_final_ablated < SYNC_ABLATED_R_MAX (0.1)`,
`gap > SYNC_GAP_MIN (0.4)`. El ablado se sienta sobre el piso de N finito
esperado (`1/√256 = 0.0625`; media observada 0.0560) — es decir, **no queda
orden residual** al apagar la regla: lo que sobra es ruido estadístico de
muestra finita, no estructura.

Verificado además por `cargo test --test axioms emergence_sync::` (4/4: acoplado
sincroniza, ablado no, gap decisivo, 5 seeds) y `cargo test --test axioms
r10_emergence_gates::` (16/16, incluye los gates `SYNC_ECS_*` de ADR-047).

**Nota de reproducibilidad:** estos números son **posteriores** al fix del stream
PCG de `init_state` (`use_cases/experiments/emergence_sync.rs`, ver ADR-047 §7):
la fn avanzaba un solo estado por draw gaussiano cuando `gaussian_f32` consume dos
(Box-Muller), de modo que el `u2` del draw `k` reaparecía como el `u1` de la fase
`k+1` — correlación intra-stream ω↔θ. El fix cambia la secuencia de números
aleatorios, no la física; los umbrales no se tocaron y siguen pasando.

**Alcance del claim (importante):** esto prueba que *el modelo de Kuramoto
sincroniza*, con la ablación como contrafáctico limpio. **No** prueba que L2
`OscillatorySignature` ni el motor ECS lo hagan — este experimento no instancia
ninguna entidad. Ese claim, y su medición, viven en ADR-047.
