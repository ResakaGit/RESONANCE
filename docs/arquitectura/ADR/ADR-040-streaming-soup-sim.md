# ADR-040: Streaming `SoupSim` — incremental stepper vs batch `run_soup`

**Estado:** Propuesto
**Fecha:** 2026-04-14
**Contexto:** AUTOPOIESIS track, sprint AP-6c (viz Bevy + egui)
**ADRs relacionados:** ADR-037 (reaction substrate), ADR-039 (fission), pendiente ADR-041

## 1. Contexto y problema

- Módulos afectados:
  - `src/use_cases/experiments/autopoiesis.rs:236-372` (`run_soup`, `run_soup_with_network`)
  - `src/bin/autopoietic_lab.rs:128-137` (callsite headless único)
  - Consumidor futuro: el binario Bevy de AP-6c (no existe aún).

- Evidencia de la fricción:
  - `run_soup_with_network` (autopoiesis.rs:253) es **monolítico**: abre `for tick in 0..config.ticks { … }` en l.286 y no retorna hasta agregarlo todo en `SoupReport` (l.367). No hay forma de observar el grid en tick T sin re-correr desde 0.
  - El estado mutable de la simulación vive como variables locales de la función: `grid`, `mask`, `tracks`, `total_dissipated`, `last_final_hashes`, `scratch_cells`, `damp_field`, `strength_field` (autopoiesis.rs:262-284). **No hay struct** que lo encapsule.
  - Los tests de AP-5 (`property_autopoiesis.rs`) sólo consumen `SoupReport` final → el patrón batch está baked en el contrato público.
  - La CLI headless ya existente (autopoietic_lab.rs:128-137) encaja perfecta con batch — no hay que romperla.

- Restricciones del stack (CLAUDE.md):
  - Hard Block #3: NO `async`/`await`. Stepping tiene que ser síncrono, loop puro.
  - Hard Block #4: NO `Arc<Mutex<T>>`. Una viz Bevy que lea el estado debe hacerlo con `Res<SoupSim>` o ticking dentro de un sistema, no compartiendo punteros.
  - Hard Block #17: gameplay en `FixedUpdate` + Phase — el stepping de la sopa en la viz es conceptualmente `ChemicalLayer`, debe ir ahí.

- Decisiones previas que condicionan:
  - ADR-037: `species: [f32; 32]` por celda, SoA contiguo.
  - ADR-039: fission es un evento puntual; `apply_fission(&mut grid, …)` necesita `&mut` sobre el grid (fission.rs:202).

**Problema:** AP-6c necesita pause/play/speed y un heatmap que se actualice en vivo. Eso requiere llamar la lógica por tick desde un sistema Bevy. `run_soup_with_network` está escrito como batch no reentrante; envolverlo en un hilo o romperlo con canales viola Hard Block #3/#4. Sin un stepper, la viz o re-ejecuta desde cero cada frame (absurdo) o duplica la lógica (code smell garantizado).

**Fuerzas:**
- La viz quiere **stepping** + lectura del estado intermedio.
- AP-5 proptest y la CLI headless quieren **batch** + `SoupReport` byte-estable.
- El harness actual es ~120 líneas de orquestación y no debería reescribirse; refactorear mal rompe AP-5.
- Determinismo: seed + red + config ⇒ reporte idéntico antes y después del refactor (invariante del track, README.md:116).

## 2. Decisión

**Extraemos el estado mutable del harness a una struct `SoupSim` con un método `step()` explícito.** `run_soup_with_network` se reimplementa como thin wrapper: `SoupSim::new(cfg, net).run_to_end()`.

```rust
// src/use_cases/experiments/autopoiesis/soup_sim.rs  (módulo nuevo, hermano del file actual)

pub struct SoupSim {
    // --- inputs inmutables ---
    cfg: SoupConfig,
    net: ReactionNetwork,
    food: Vec<SpeciesId>,
    bw: f32,
    // --- estado mutable (hot path) ---
    grid: SpeciesGrid,
    mask: ClosureMembraneMask,
    scratch_cells: Vec<SpeciesCell>,
    damp_field: Vec<f32>,
    strength_field: Vec<f32>,
    // --- estado agregado (cold) ---
    tick: u64,
    total_dissipated: f32,
    tracks: Vec<FateTrack>,
    n_closures_initial: u32,
    initial_snapshot_taken: bool,
    last_final_hashes: Vec<u64>,
}

impl SoupSim {
    pub fn new(cfg: SoupConfig, net: ReactionNetwork) -> Self { … }
    pub fn step(&mut self);                  // avanza 1 tick
    pub fn tick(&self) -> u64 { self.tick }
    pub fn grid(&self) -> &SpeciesGrid { &self.grid }
    pub fn mask(&self) -> &ClosureMembraneMask { &self.mask }
    pub fn finish(self) -> SoupReport { … }  // consume + agrega
    pub fn run_to_end(mut self) -> SoupReport { while self.tick < self.cfg.ticks { self.step(); } self.finish() }
}

pub fn run_soup_with_network(cfg: &SoupConfig, net: ReactionNetwork) -> SoupReport {
    SoupSim::new(cfg.clone(), net).run_to_end()
}
```

**Alcance in-scope:**
- Extracción mecánica del loop `for tick` de autopoiesis.rs:286-352 a `SoupSim::step()`.
- Wrapper backward-compatible: `run_soup` + `run_soup_with_network` mantienen firma y comportamiento byte-exactos.
- Getters `&` para grid/mask/tick — lectura sólo, la viz no muta.

**Alcance out-of-scope:**
- Lineage/fission edges (→ ADR-041).
- Bevy wiring (→ ADR-042 y código AP-6c.1+).
- Serde sobre `SoupSim` (el struct no cruza disco; sólo cruza módulos).
- Paralelización intra-tick.

## 3. Alternativas consideradas

### A · Dejar batch y correr la simulación en un thread de fondo
- **Cómo encajaría:** `std::thread::spawn` en el bin Bevy, canal `mpsc` enviando snapshots del grid cada tick.
- Pros: cero refactor del harness. AP-5 intocable.
- Contras: introduce concurrencia real — se serializa `SpeciesGrid` por canal (copia O(w·h·32) por tick) o se comparte con `Arc<Mutex<_>>` (**viola Hard Block #4**). Pause/play requiere señalización bidireccional → complejidad desproporcionada. Rompe "Bevy schedule only" de CLAUDE.md.
- **Descarte:** viola hard blocks y suma complejidad mayor que el refactor.

### B · Re-correr desde seed hasta tick T cada vez que la viz quiere leer
- **Cómo encajaría:** cero refactor. Cada frame, `run_soup_with_network(cfg con ticks=T_viz)`.
- Pros: tests intactos. Cero riesgo de divergencia.
- Contras: O(T²) acumulado. A 60fps con T=5000 el cost es absurdo. Pause incoherente — "pause" congelaría la lectura pero la sim seguiría re-ejecutándose. No satisface AC de formose.ron renderizando closure <5000 ticks a ≥30fps (SPRINT_AP6_AUTOPOIETIC_LAB.md:45,48).
- **Descarte:** no-starter por performance.

### C · Stepper global vía Resource + system en `FixedUpdate`
- **Cómo encajaría:** `#[derive(Resource)] pub struct SoupSim { … }` directo; sistema `step_soup_system` en Phase::ChemicalLayer.
- Pros: encaja 100% con el modelo Bevy. Pause = `app.insert_resource(SimPaused(true))`.
- Contras: **Resource implica Bevy como dep del módulo** — autopoiesis.rs hoy es `#![no_bevy]` de facto (no importa nada de `bevy::*`, grep confirma). Atar la lógica a Bevy contamina AP-5 proptest (headless, sin Bevy).
- **Descarte por ubicación, no por idea.** Se adopta la idea pero manteniendo `SoupSim` **Bevy-free** (plain struct). El wrapper `#[derive(Resource)]` va en el bin o en un módulo `viz/` separado.

### **Elegida — C' · `SoupSim` Bevy-free + wrapper `Resource` en el bin**
Combina lo mejor de A/C sin sus costos: la struct vive en `use_cases/experiments/`, el bin Bevy la envuelve con `#[derive(Resource)]` localmente. AP-5 sigue usando `run_to_end()` sin cambios.

## 4. Consecuencias

**Positivas:**
- Viz puede leer `soup.grid()` cada frame y pintar heatmap sin copias.
- Pause/play = no invocar `step()` ese frame. Speed = llamar N veces por frame.
- Facilita AP-6c.4 (lineage panel) — el Vec de fission events puede consumirse incrementalmente.
- Abre la puerta a **tests de invariante paso-a-paso** (Ax 2/5 cada tick, no sólo al final) — ganancia para AP-5 si se quiere extender.

**Negativas:**
- ~120 líneas reorganizadas. Riesgo de introducir bug de timing off-by-one (por ejemplo: `if tick % detection_every != 0 { continue; }` convertido a `if self.tick % self.cfg.detection_every != 0 { self.tick += 1; return; }` es sutil).
- Superficie pública crece: struct + métodos vs 1 función.
- `SoupSim` es `!Copy` y pesado (~grid_size × 32 × f32 + scratch). No debe moverse en hot path.

**A vigilar:**
- Divergencia determinística: el test golden debe bloquear merge si un solo byte del `SoupReport` cambia.
- Lifetime de `ReactionNetwork` dentro de `SoupSim` — hoy `run_soup_with_network` toma `net` por valor (autopoiesis.rs:253); `SoupSim` la guardará también por valor. Sin lifetimes externos, todo owned.

**Deuda técnica asumida:**
- Duplicación temporal de ensamblaje de grid entre `SoupSim::new` y lo que hoy está inline en autopoiesis.rs:261-269 — desaparece al mover el código, no se agrega.

## 5. Plan de implementación

1. **Escribir test de equivalencia** (golden) antes de tocar nada:
   ```rust
   // tests/property_autopoiesis.rs o nuevo tests/soup_sim_equivalence.rs
   #[test] fn soup_sim_run_to_end_matches_legacy_harness() {
       for seed in [42, 1337, 2026] {
           let cfg = SoupConfig { seed, … };
           let net = random_reaction_network(seed, …);
           let legacy = run_soup_with_network_legacy(&cfg, net.clone());
           let streamed = SoupSim::new(cfg.clone(), net).run_to_end();
           assert_eq!(serde_json::to_string(&legacy).unwrap(),
                      serde_json::to_string(&streamed).unwrap());
       }
   }
   ```
   (Copiar el cuerpo actual a `run_soup_with_network_legacy` temporalmente; borrar al cerrar AP-6c.0.)

2. **Crear `src/use_cases/experiments/autopoiesis/soup_sim.rs`** con `SoupSim` y mover la lógica del loop.

3. **Reescribir `run_soup_with_network`** como wrapper de 1 línea.

4. **Correr `cargo test -p resonance autopoiesis`** — tier B + property. Todo verde + golden test verde.

5. **Borrar `run_soup_with_network_legacy`** y el golden test una vez mergeado (deuda explícita con TODO con referencia a este ADR).

**Verificación:** JSON byte-exacto legacy vs streamed en 3 seeds diferentes + `cargo test` mantiene ~3,166 tests verdes.

**Riesgos de migración:**
- `tracks` mutations dentro de branches condicionales (autopoiesis.rs:306-317, 339-349): el orden de inicialización importa. Extraerlas como métodos privados `self.maybe_take_initial_snapshot(&closures, tick)` + `self.update_fates(&closures, &alive_hashes, tick, pressure_crossed)` reduce el blast radius de cada cambio.
- `last_final_hashes` se asigna al final de cada detection tick (autopoiesis.rs:351) pero **sólo se usa** en `finish()` (l.358). Preservar esa semántica; no convertirlo en "cada tick".

## 6. Referencias

- Harness actual: `src/use_cases/experiments/autopoiesis.rs:236-372`
- Callsite único: `src/bin/autopoietic_lab.rs:128-137`
- CLAUDE.md hard blocks: §"Hard Blocks" #3/#4/#17
- Sprint: `docs/sprints/AUTOPOIESIS/SPRINT_AP6_AUTOPOIETIC_LAB.md` (AC §43-52)
- Invariantes del track: `docs/sprints/AUTOPOIESIS/README.md:108-117`

## 7. Notas para el revisor humano (STRIP BEFORE MERGE)

- **Exploración realizada:**
  - Leí `autopoiesis.rs` completo (líneas citadas son exactas).
  - Grep `run_soup` sólo tiene 1 callsite externo (autopoietic_lab.rs).
  - Verifiqué que `autopoiesis.rs` no importa `bevy::*` — es puro stdlib + serde + crate internos.
  - Confirmé que `SpeciesGrid` / `ClosureMembraneMask` son data-only (sin Resource/Component).

- **Suposiciones que requieren validación:**
  - `SoupReport` es JSON-byte-estable entre runs idénticos — no verifiqué con `serde_json::to_string` dos veces sobre el mismo struct. **[ASSUMPTION]** creo que sí por Vec orden-preservado, pero hay que confirmar antes de escribir el golden.
  - El `last_window_ticks` averaging (autopoiesis.rs:354-365) corre **dentro** de `finish()` — intocable en `step()`.

- **Preguntas abiertas:**
  - ¿Exponer `SoupSim::grid_mut()` para que la viz inyecte perturbaciones (click-to-seed)? Propongo dejarlo OUT — la viz es sólo observadora en AP-6c. Sub-sprint futuro si se quiere "sandbox".
  - ¿`SoupSim` debería implementar `Clone`? Cost alto (grid copy) y no lo requiere ningún caso de uso identificado. Marcar explícitamente `// no Clone intended`.
