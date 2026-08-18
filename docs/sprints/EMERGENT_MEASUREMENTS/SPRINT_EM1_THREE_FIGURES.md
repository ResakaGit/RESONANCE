# Sprint EM-1: Tres figuras emergentes — Kleiber · Lotka-Volterra · Linaje

**ADR:** — (posible ADR-043 si el harness de medición se reutiliza)
**Esfuerzo:** 1 semana tiempo-silla (2–3 calendar)
**Bloqueado por:** AP-6c (solo para item 3)
**Desbloquea:** paper cross-scale, PV-7 (Hordijk RAF), candidate figuras para `docs/sintesis_patron_vida_universo.md`

## Contexto

El stack Resonance tiene 7 escalas vivas y 3.166 tests verdes. Lo que **no tiene** es una figura publicable que muestre un axioma haciendo trabajo real, medido en ensemble, con error bars. Este sprint produce exactamente tres figuras medibles a partir de binarios que ya corren — no es trabajo de framework, es trabajo de **experimento**.

Las tres son independientes en ejecución, ordenables por costo creciente:

1. **Kleiber 3/4** — barato, baja varianza, casi seguro sale (cache ya existe).
2. **Lotka-Volterra emergente** — medio; si sale, el Axioma 8 está haciendo trabajo real.
3. **Deriva de linaje post-fisión** — bloqueado por AP-6c; valida que autopoiesis produce diversidad, no copia xerox.

Si las 3 salen: primer resultado cross-scale publicable. Si alguna **no** sale, es un hallazgo (el axioma correspondiente no basta y hay que revisar bridges). En ambos casos el sprint es útil.

## Definition of Done (sprint-wide)

- `cargo test` verde en toda la workspace.
- 3 binarios `measure_<name>` headless, deterministas por seed.
- 3 CSVs + 3 PNGs checked-in en `docs/figures/em1/`.
- Un `README.md` por figura con: comando exacto, seed, métrica, esperado vs observado, interpretación 3 líneas.
- Sin `unwrap`/`expect` en runtime de medición.
- Ninguna constante nueva fuera de `blueprint/constants/` o `{module}/constants.rs`.

---

## EM-1.1 — Kleiber 3/4 en ensemble

**Type:** experiment · **Estimate:** 3–5h (S)

### 0. Spec
- **What:** Binario headless que spawnea N=500 entidades con masa log-uniform en 3 décadas, corre hasta estado estacionario, mide `qe/tick` consumido por entidad, ajusta ley de potencia y exporta CSV + PNG log-log.
- **Why:** Validar que el `KleiberCache` + constante `KLEIBER_EXPONENT=0.75` producen el exponente correcto **sin tuning** en ensemble. Si sale, es la primera figura publicable del proyecto.
- **Acceptance:**
  - `cargo run --release --bin measure_kleiber -- --n 500 --seed 42 --ticks 5000 --out docs/figures/em1/kleiber` completa en < 60s.
  - CSV con columnas `entity,mass,metabolic_rate_qe_per_tick`.
  - Fitted exponent β vía regresión log-log: **0.73 ≤ β ≤ 0.77** (±0.02 de 0.75).
  - R² ≥ 0.95 sobre ≥ 3 décadas de masa.
  - PNG log-log con puntos + recta ajustada + β y R² en el título.
- **Out of scope:** Multi-seed ensemble (si sale clean con 1 seed, ya sirve). Inter-especies (solo 1 tipo de entidad). Otros exponentes (0.67 Rubner).

### 1. Contexto
- `src/batch/cache/kleiber_cache.rs` ya cachea `mass^0.75`.
- `src/blueprint/equations/` tiene `kleiber_exact`.
- Patrón de headless: copiar estructura de `src/bin/headless_sim.rs` + `src/bin/autopoietic_lab.rs` (modo `--headless`).
- Plotting: `plotters` ya está en deps (verificar en `Cargo.toml`); si no, **no agregar crate** — exportar CSV + script Python opcional en `scripts/plot_kleiber.py`.

### 2. Diseño
- **Estrategia:** bottom-up, función pura `measure_kleiber(world, ticks) -> Vec<(mass, rate)>` + binario thin wrapper.
- **Contrato:**
  ```rust
  pub struct KleiberSample { pub entity: u32, pub mass: f32, pub rate: f32 }
  pub fn run_kleiber_ensemble(seed: u64, n: usize, ticks: u32) -> Vec<KleiberSample>;
  pub fn fit_power_law(samples: &[KleiberSample]) -> (f32 /*β*/, f32 /*R²*/);
  ```
- **Alternativa descartada:** reutilizar `lab` bin con flag. Rechazado: contamina un binario de exploración con lógica de medición reproducible.

---

## EM-1.2 — Lotka-Volterra emergente

**Type:** experiment · **Estimate:** 2–4h si emerge; 3–5 días si requiere tuning de bridges (M–L)

### 0. Spec
- **What:** Binario headless que spawnea N_prey entidades con frecuencia f_A y N_pred con f_B resonante (|f_A − f_B| < `COHERENCE_BANDWIDTH/2`), corre 10⁵ ticks, loggea poblaciones por tick y exporta serie temporal + espacio de fases.
- **Why:** Test crítico del Axioma 8 a nivel ecosistema. Si las oscilaciones predador-presa **emergen solas** sin ecuaciones LV escritas, Resonance es un modelo vivo. Si no emergen, el diagnóstico vale tanto como el éxito.
- **Acceptance:**
  - `cargo run --release --bin measure_lotka_volterra -- --seed 42 --ticks 100000 --out docs/figures/em1/lv` completa en < 5 min.
  - CSV con `tick,prey_count,pred_count`.
  - **Criterio de emergencia:** FFT de `prey_count(t)` tiene un pico dominante con potencia > 10× el ruido de fondo, y el pico de `pred_count(t)` está desfasado π/2 ± π/8 respecto al de prey. Si este criterio falla → el item entrega el CSV + un `DIAGNOSIS.md` con la hipótesis de por qué no emergió (→ insumo para un futuro sprint de tuning).
  - PNG: panel superior población vs tiempo; panel inferior espacio de fases (prey, pred).
- **Out of scope:** Ajuste de parámetros LV clásicos (α, β, δ, γ) — no los tenemos escritos, solo los observamos emerger o no. Multi-especie (>2).

### 1. Contexto
- Spawn pattern: similar a `src/bin/versus.rs` y `src/bin/cambrian.rs`.
- Frecuencias controladas vía `L2 OscillatorySignature`.
- Transferencia trófica: `bridge/collision_transfer` + `bridge/competition_norm`.
- Riesgo real: si el acoplamiento `L8 AlchemicalInjector → L0 BaseEnergy` no discrimina fuertemente por Δf, los dos pools convergen a equilibrio plano sin oscilación.

### 2. Diseño
- **Estrategia:** top-down desde escenario; sin código nuevo de simulación, solo un harness de medición.
- **Contrato:**
  ```rust
  pub struct LvConfig { pub seed: u64, pub n_prey: usize, pub n_pred: usize, pub freq_prey: f32, pub freq_pred: f32, pub ticks: u32 }
  pub struct LvSeries { pub tick: Vec<u32>, pub prey: Vec<u32>, pub pred: Vec<u32> }
  pub fn run_lv(cfg: LvConfig) -> LvSeries;
  pub fn detect_oscillation(series: &LvSeries) -> OscillationVerdict;  // { Emerged { freq, phase_lag }, Flat, Chaotic }
  ```
- **Alternativa descartada:** correr `ecosystem_music` y analizar el WAV. Rechazado: añade capa de conversión a audio que no necesitamos; mejor CSV directo.

---

## EM-1.3 — Deriva de linaje post-fisión

**Type:** experiment · **Estimate:** 1–2 días post AP-6c (M)

### 0. Spec
- **What:** Usando `autopoietic_lab --headless` (AP-6b) + `lineage_grid` (AP-6c), correr sopa con red `formose.ron` hasta ≥5 generaciones, exportar árbol + composición química por nodo, medir **distancia Jensen-Shannon** entre distribuciones de especies madre→hija→nieta.
- **Why:** Autopoiesis sin deriva es xerox, no vida. Esta figura responde: ¿las hijas son químicamente distintas de la madre? ¿Cuánto? ¿La deriva es estable (varianza acotada) o se va al caos?
- **Acceptance:**
  - `cargo run --release --bin measure_lineage_drift -- --network formose --seed 42 --min-generations 5 --out docs/figures/em1/lineage` completa en < 3 min.
  - CSV con `node_id,parent_id,generation,jsd_to_parent,species_vector_hash`.
  - JSD madre→hija promedio en rango **[0.02, 0.35]** (no-cero = hay deriva; no-caos = no totalmente random).
  - Varianza de JSD por generación **no creciente monotónicamente** (señal de que la deriva no explota).
  - PNG: árbol genealógico con color de nodo ∝ JSD a la madre.
- **Out of scope:** Selección natural, presión ambiental, múltiples redes. Solo `formose.ron` como primer caso.

### 1. Contexto
- **Bloqueado por AP-6c** (ADR-040 `SoupSim` stepper + ADR-041 `lineage_grid` en reporte). Puede arrancar el scaffolding en paralelo, pero la métrica depende de que `SoupReport` incluya `LineageNode { id, parent, generation, species_composition: Vec<(SpeciesId, f32)> }`.
- Asset: `assets/reactions/formose.ron` ya existe (AP-6b2).

### 2. Diseño
- **Estrategia:** top-down post-reporte; el binario consume `SoupReport` serializado y computa JSD offline.
- **Contrato:**
  ```rust
  pub fn jensen_shannon(p: &[(SpeciesId, f32)], q: &[(SpeciesId, f32)]) -> f32;
  pub fn compute_drift_table(report: &SoupReport) -> Vec<DriftRow>;
  ```
- **Alternativa descartada:** métrica Hamming sobre hash de composición. Rechazado: discreto, no captura gradiente. JSD es la métrica estándar en filogenia bioinformática.

---

## Plan de ejecución sugerido

| Orden | Item | Paralelizable con | Razón |
|------:|------|-------------------|-------|
| 1 | EM-1.1 Kleiber | — | Barato, calibra el pipeline de medición (CSV + PNG + README). |
| 2 | EM-1.2 Lotka-Volterra | AP-6c (otra persona) | Medio riesgo; empezar temprano por si hay tuning. |
| 3 | AP-6c cierra | — | Desbloquea 1.3. |
| 4 | EM-1.3 Linaje | — | Última; aprovecha todo el pipeline ya validado. |

## Red flags explícitos

- Si EM-1.1 da β ∉ [0.73, 0.77] → **no** es bug del script; es un hallazgo físico. Abrir ADR antes de tocar `KLEIBER_EXPONENT`.
- Si EM-1.2 falla el criterio de emergencia → **no** forzar parámetros para que oscile. Escribir `DIAGNOSIS.md` y cerrar el item con status "null result documented".
- Si EM-1.3 da JSD ≈ 0 → la fisión está clonando; revisar `fission.rs` y el mixing de sustrato pre-división.

## EM-1.4 — Emergencia de sincronización (Axioma 6) + ablación causal

**Type:** experiment · **Estimate:** 4–6h (S) · **Status:** DONE — veredicto cargado 2026-08-17 · **ADR:** ADR-046 §10

### 0. Spec
- **What:** Experimento headless que corre N osciladores de fase acoplados (mean-field de Kuramoto) desde arranque desordenado, mide el order parameter global `R = |(1/N)Σe^{iθ}|`, y compara la condición acoplada vs **ablada** (`coupling = 0`) sobre el mismo estado inicial.
- **Why:** Dar a Ax6 una operacionalización medible con contrafáctico causal. `R` es una propiedad de escala (no existe a nivel individual); la ablación es el contrafáctico N−1→N. Es el gemelo temático de EM-1.2 (Ax8 haciendo trabajo real).
- **Alcance real del claim (corregido 2026-08-17):** este ítem prueba que **el modelo de Kuramoto sincroniza** — mean-field analítico, all-to-all, sin ECS. **No** prueba que L2 `OscillatorySignature` ni el motor lo hagan: el experimento no instancia ninguna entidad. La versión anterior de este ítem decía "cerrar la única prueba faltante del core" y su §4 se atribuía "el primer par (Capa, Ax6) medible (fila SYNC)" — ambas cosas eran **sobreclaims**, corregidos por EM-1.5 / ADR-047.
- **Acceptance (verificado):**
  - `cargo run --bin measure_emergence -- <seed> 256 2000` imprime VERDICT PASS. ✅
  - `R_final > 0.5`, `R_ablated < 0.1`, gap `> 0.4` en 6/6 seeds (11, 1, 2, 3, 100, 777). ✅ Medido: `R_final = 0.9619`, `R_ablated = 0.0560`, gap `0.9059` (medias); el ablado se sienta sobre el piso de N finito `1/√256 = 0.0625`.
  - Reproducible sobre 5 seeds (`tests/emergence_sync.rs`, 4/4). ✅
  - Gate de umbrales consistente (`tests/r10_emergence_gates.rs`, 16/16). ✅
- **Out of scope:** transición crítica `R(K)` (barrido de acoplamiento); ruido térmico; corroboración con el `entrainment_system` ECS (Fase 4 opcional del plan → **ejecutada como ítem EM-1.5**, no retro-expandida aquí).

### 1. Contexto
- Kuramoto ya existía como fn pura (`blueprint/equations/emergence/entrainment.rs`) y system (`simulation/emergence/entrainment.rs:28`, acopla frecuencia).
- Faltaba el order parameter de fase — implementado en `blueprint/equations/emergence/synchronization.rs`.

### 2. Diseño
- Mean-field de fase headless puro (no reusa el system ECS, que acopla ω no θ). Ver ADR-046 §3.
- Contrato: `run_sync_experiment(&SyncConfig) -> SyncReport` corre ambas condiciones; `kuramoto_order_parameter(&[f32]) -> f32`; `emergence_sync_verdict(r_final, r_ablated) -> bool`.
- Constantes en `blueprint/constants/synchronization_a6.rs` (SYNC_DT derivada de DISSIPATION_GAS).

### 3. How tested
- Unit: `cargo test --lib blueprint::equations::emergence::synchronization` (10 tests de este ítem: R, paso mean-field, verdict; el módulo hoy corre 25 — los 15 restantes son de EM-1.5).
- Integration: `cargo test --test emergence_sync` (acoplado sincroniza, ablado no, gap, multi-seed).
- Gate: `cargo test --test r10_emergence_gates`.

### 4. Entrega adicional (fuera de la DoD original del sprint)
- Fila `SYNC-analytic` en `docs/design/AXIOM_LAYER_VALIDATION_MATRIX.md` — declarada explícitamente como **referencia, no motor**. (La redacción original, "cierra el primer par (Capa, Ax6) medible", era un sobreclaim: ese par lo cierra EM-1.5.)
- ADR-046 con hipótesis nula + veredicto del spike (§10).

## EM-1.5 — Ax6 sobre el motor real: probe del `entrainment_system` ECS

**Type:** experiment · **Estimate:** 4–6h (S) · **Status:** DONE (2026-08-17) — PASS · **ADR:** ADR-047 (ex RFC-001, 3 rondas de crítica adversarial)

### 0. Spec
- **What:** Test de integración headless que corre **el system real** — `entrainment_system`, la misma fn que registra `AtomicPlugin` — sobre N=64 entidades con `OscillatorySignature` y `SpatialIndex` reales, y mide el colapso de la dispersión de frecuencias `S = 1 − σ_ω(T)/σ_ω(0)` contra **dos** ablaciones causales.
- **Why:** EM-1.4 probó un resultado de 1975 (que Kuramoto sincroniza), no una propiedad de RESONANCE — su experimento no instancia entidades y **L2 no participa**. Hasta medir esto, el par (L2, Ax6) era `GAP` disfrazado de `PASS` en la matriz. Este ítem cierra el sobreclaim y da la primera medición de Ax6 sobre el motor.
- **Las dos ablaciones (por qué dos):**
  - **A1 (regla):** el system no se agrega al schedule → contrafáctico N−1→N puro.
  - **A2 (alcance):** system **ACTIVO**, población a `3 × ENTRAINMENT_SCAN_RADIUS` → el broadphase devuelve vecindad vacía y la regla corre en el vacío. Prueba que el orden exige *interacción efectiva*, no la mera presencia del código en el schedule. El supresor es el **cutoff duro** del radio de scan, no la atenuación continua (con el decay solo, `K_eff(36) ≈ 0.0075` y el sistema aún convergería → el control fallaría).
- **Acceptance (verificado):**
  - `cargo test --test emergence_ecs` PASS en **1.20 s**, sin GPU, un solo test target. ✅
  - Acoplado `S ≥ SYNC_ECS_S_PASS_MIN (0.5)` en **5/5 seeds**. ✅ Medido: 0.8595 / 0.8739 / 0.8475 / 0.7881 / 0.8667 (media 0.847, mín 0.788).
  - A1 y A2: identidad **bit a bit** de ω(0) vs ω(T) vía `hash_f32_slice` → `S == 0.0` exacto en 5/5 seeds. ✅
  - Determinismo bit a bit entre corridas repetidas. ✅
  - `cargo test --test r10_emergence_gates` verde con los 9 gates `SYNC_ECS_*` nuevos (16/16). ✅
- **Out of scope:** pipeline completo (`AtomicPlugin` + fases + run-conditions); transición crítica `R(K)`; atenuación **continua** de Ax7 (barrido S(spacing) intra-rango); régimen de truncación de vecinos (>8 candidatos y su sesgo por spawn index — el spacing 8 se elige justamente para excluirlo).

### 1. Contexto
- El motor difiere del experimento analítico en los tres ejes que importan: acopla **ω** (no θ), régimen **lineal** (`sin x ≈ x`), topología de **vecinos ≤ 8** con decay por distancia (AC-4, Ax7).
- Por eso el order parameter de fase `R` **no aplica** (las fases quedan con offsets constantes) — es el mismo contra que ADR-046 §3 le puso a su opción B. La métrica honesta para *frequency entrainment* es el colapso de σ_ω.
- Sustrato ya existente: `simulation/emergence/entrainment.rs:28` (system), `equations/emergence/entrainment.rs` (fns puras), `world/space.rs` (`SpatialIndex`).
- **Bug pre-existente encontrado y corregido:** `emergence_sync.rs::init_state` avanzaba un solo estado PCG por draw gaussiano cuando `gaussian_f32` consume dos (Box-Muller) → el `u2` del draw `k` reaparecía como el `u1` de la fase `k+1` (correlación intra-stream ω↔θ). Corregido; los umbrales de EM-1.4 no se tocaron y siguen pasando.

### 2. Diseño
- **Estrategia:** system aislado en schedule `Update` mínimo (opción A de ADR-047 §3). Rechazado B (probe sobre `AtomicPlugin` completo): cualquier system de la cadena física puede tocar ω o mover entidades → la ablación deja de ser un contrafáctico limpio.
- **Métrica como fn pura, no inline en el test** (regla 12): `frequency_std` (σ muestral n−1), `frequency_collapse_s` (bordes σ₀≤0→0, no-finitos→0, clamp [0,1]) y `sync_ecs_verdict` en `blueprint/equations/emergence/synchronization.rs`, con unit tests por borde. El test de integración **llama** la métrica.
- **Escenario:** grilla 8×8, `spacing = 8.0` (⅔ del scan radius → king-graph con exactamente 8 vecinos por nodo interior, **sin truncación**), `ω ~ N(75, 8)` Hz, φ=0 (don't-care: el system no lee fase), `SpatialEntry` radio 0.5, `SpatialIndex` estático (las entidades no se mueven), 400 ticks, 5 seeds, 3 Apps por seed desde un builder compartido.
- **Trampas evitadas por diseño:** el offset de 75 Hz es obligatorio (`gaussian_f32` centra en 0 y L2 clampea a `max(0.0)` → sin él la gaussiana queda rectificada y el experimento se corrompe en silencio); el schedule es `Update` y **nunca** `FixedUpdate` (con `MinimalPlugins` corre 0..k veces por `update()` según wall-clock); `query_radius` devuelve al propio nodo, así que el guard estructural cuenta con `filter(entity != self)`.
- **Constantes:** 6 `SYNC_ECS_*` en `blueprint/constants/synchronization_a6.rs`, con `SYNC_ECS_SPREAD_HZ = SYNC_ECS_SPREAD_RATIO × KURAMOTO_LOCK_THRESHOLD_HZ` (derivada de la dinámica bajo prueba) y ambas cotas del ratio gateadas en r10.

### 3. How tested
- Unit: `cargo test --lib blueprint::equations::emergence::synchronization` — 25/25 (15 nuevos: σ n−1 vs n, no-finitos, invarianza a traslación, valor conocido; bordes de `frequency_collapse_s`; boundary del verdict).
- Integration: `cargo test --test emergence_ecs` — 8/8 en 1.20 s (3 guards estructurales + acoplado 5 seeds + A1 + A2 + determinismo).
- Gate: `cargo test --test r10_emergence_gates` — 16/16.
- Referencia analítica: `cargo test --test emergence_sync` — 4/4 (post-fix del stream PCG).

### 4. Resultado y lectura
- **PASS.** `S` acoplado 0.788–0.874; ablados `S == 0` exacto en ambas condiciones y las 5 seeds. Detalle por seed en ADR-047 §10.
- **La predicción del diseño se cumplió con su mecanismo:** el estado final llega a `lock fraction = 1.0000`, es decir la **escalera congelada** que el ADR anticipó como piso teórico `S ≈ 0.75`. σ(T) ≈ 1.1–1.8 Hz ≈ 1–2 × `KURAMOTO_LOCK_THRESHOLD_HZ`. Por eso el umbral se fijó en 0.5 y **no hubo que ajustar nada** post-hoc.
- **Línea de base del lock:** en A1 (población intacta) 4.3–8.1 % de los pares vecinos ya nacen dentro de 1 Hz por azar — contra ese piso se lee el 100 % del acoplado. En A2 la fracción es 0.0000 por ausencia de pares (denominador 0), no por ausencia de lock.

## Cierre del arco

Cuando este sprint cierra (incluso parcialmente con algún null result):

- Hay **al menos una figura publicable** con datos reales del simulador.
- El harness de medición queda reutilizable para PV-7 (RAF benchmark) y sprints emergentes futuros (PP, NS).
- El proyecto pasa de "framework interesante con 3.166 tests" a "framework con resultado medido". Ese es el umbral paper.
