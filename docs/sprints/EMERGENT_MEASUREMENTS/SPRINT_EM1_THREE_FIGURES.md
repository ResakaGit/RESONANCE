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

## Cierre del arco

Cuando este sprint cierra (incluso parcialmente con algún null result):

- Hay **al menos una figura publicable** con datos reales del simulador.
- El harness de medición queda reutilizable para PV-7 (RAF benchmark) y sprints emergentes futuros (PP, NS).
- El proyecto pasa de "framework interesante con 3.166 tests" a "framework con resultado medido". Ese es el umbral paper.
