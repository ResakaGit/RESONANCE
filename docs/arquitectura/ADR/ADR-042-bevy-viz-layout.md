# ADR-042: Layout del código Bevy de `autopoietic_lab` — bin-local vs `src/viz/`

**Estado:** Propuesto (reduced form)
**Fecha:** 2026-04-14
**Contexto:** AUTOPOIESIS track, sprint AP-6c tareas #1-#4

## 1. Contexto

- Sprint AP-6c (SPRINT_AP6_AUTOPOIETIC_LAB.md:33-40) propone archivos nuevos:
  - `src/viz/autopoietic_view.rs` (heatmap)
  - `src/viz/autopoietic_ui.rs` (egui controls)
  - `src/viz/lineage_tree.rs` (tree)
- Estado del repo: **`src/viz/` no existe.** Ningún módulo de viz vive fuera de `src/bin/`.
- Convención observada:
  - `src/bin/lab.rs` — 1807 LOC monolítico con Bevy + egui + 15 experimentos + 25 maps.
  - `src/bin/sim_viewer.rs` — 206 LOC monolítico.
  - `src/bin/museum.rs` — 149 LOC monolítico.
  - `src/bin/particle_lab.rs` — 80 LOC monolítico.
  - `src/bin/autopoietic_lab.rs` — 297 LOC (headless puro, sin Bevy).
  - `src/rendering/` existe pero tiene 3 archivos pequeños (`frame_buffer.rs`, `pixel_window.rs`, `terminal.rs`) — es infra CPU-rendering shared, no un home para plugins Bevy.
  - `src/viewer/` también existe con contenido similar (snapshot dumping), no Bevy plugins.
- Dependencias aprobadas (`Cargo.toml`): `bevy 0.15 (features=serialize)`, `bevy_egui 0.31`, `egui_plot 0.29`.
- `CLAUDE.md` § "Coding Rules" #2: max 4 fields per component; #3: un sistema, una transformación; #7: Phase assignment; #17 Hard Block: gameplay sólo en `FixedUpdate` + Phase.

**Problema:** El sprint sugiere crear un módulo top-level `src/viz/` nuevo. Hacerlo introduce una convención organizacional que no existe y duplica el rol semántico de `src/rendering/` y `src/viewer/`. Pero meter todo en un monolito `autopoietic_lab.rs` tipo `lab.rs` (1807 LOC) es hostil a mantenimiento.

## 2. Decisión

**Los plugins Bevy viven como módulos hermanos del binario, dentro de `src/bin/autopoietic_lab/`**. El entry `src/bin/autopoietic_lab.rs` es thin (parse CLI + dispatch) y declara `mod view; mod ui; mod lineage; mod headless;`.

```
src/bin/
  autopoietic_lab.rs            # thin entry: parse CLI, --headless → headless::run(), else viz::run()
  autopoietic_lab/
    mod.rs                      # (opcional; si Cargo acepta submódulos dir-del-bin sin él, mejor)
    headless.rs                 # mueve el código actual autopoietic_lab.rs:111-155
    view.rs                     # SpeciesHeatmapPlugin + MembraneOverlayPlugin
    ui.rs                       # ControlsPlugin (egui: pause/play/speed/seed/network selector)
    lineage.rs                  # LineageTreePlugin (egui_plot DAG)
    sim_resource.rs             # #[derive(Resource)] wrapper sobre SoupSim (ADR-040)
```

**Condición de escape:** si cualquier módulo supera ~500 LOC o es reutilizado por un segundo binario, se promueve a `src/viz/autopoietic/` con un ADR de promoción. Mientras sea consumido sólo por este bin, queda local.

**Alcance in-scope:** layout de archivos AP-6c. Reglas de phase/system dentro de esos módulos.

**Alcance out-of-scope:** re-layout de bins existentes (`lab.rs` 1807 LOC). Convención general para _todos_ los bins futuros — este ADR sienta precedente pero no manda.

## 3. Alternativa descartada

**Crear `src/viz/autopoietic_*.rs` top-level ahora.**
- Pro: matchea literalmente el sprint.
- Contra 1: `src/viz/` no existe — inventar un top-level es una decisión transversal, no de este sprint. Requeriría definir qué va en `viz/` vs `rendering/` vs `viewer/` — fuera de scope.
- Contra 2: código consumido por **un único bin** no pertenece a un módulo top-level. YAGNI.
- Contra 3: rompe el patrón de todos los bins existentes sin beneficio medible en el corto plazo.

**Monolito `autopoietic_lab.rs` > 1000 LOC al estilo `lab.rs`.**
- Pro: matchea el patrón más común del repo.
- Contra: `lab.rs` es reconocidamente el bin menos mantenible. Reproducir eso sin necesidad es code smell intencional.

## 4. Consecuencias

**Positivas:**
- Un solo lugar para mirar AP-6c: `src/bin/autopoietic_lab/`.
- Cada submódulo ~100-200 LOC, responsabilidad única (view / ui / lineage / headless).
- Si AP-6c se corta a medio camino, el blast radius queda contenido a un directorio; nada que purgar en `src/viz/`.
- Mantiene invariante CLAUDE.md: plugins en `FixedUpdate` + Phase asignada por sistema (view/ui usan `Update`; la integración con `SoupSim::step()` va en Phase::ChemicalLayer).

**Negativas:**
- Divergencia de texto literal del sprint doc (SPRINT_AP6_AUTOPOIETIC_LAB.md:33-40 menciona `src/viz/…`). Hay que actualizar el sprint o poner nota que la decisión final vive acá.
- Si AP-7/AP-8 quieren reutilizar el heatmap, habrá que promover. Costo aceptable (es un git mv + update de `use`).
- Cargo requiere sintaxis precisa para "submódulos de un binario". Si no compila tal cual, fallback: `src/bin/autopoietic_lab_view.rs` + `_ui.rs` + `_lineage.rs` hermanos directos (no en subdir). La semántica es la misma.

## 5. Referencias

- Sprint: `docs/sprints/AUTOPOIESIS/SPRINT_AP6_AUTOPOIETIC_LAB.md:33-40`
- Bin actual headless: `src/bin/autopoietic_lab.rs:1-297`
- Monolito contraejemplo: `src/bin/lab.rs` (1807 LOC)
- Otros bins viewer: `src/bin/sim_viewer.rs`, `src/bin/museum.rs`, `src/bin/particle_lab.rs`
- Bevy egui pattern ya usado: `src/bin/lab.rs:15-16,330`
- ADR-040 (streaming stepper) — precondición de `sim_resource.rs`.
- ADR-041 (lineage in report) — consumido por `lineage.rs`.

## 6. Notas para el revisor humano (STRIP BEFORE MERGE)

- **Exploración realizada:**
  - Listé `src/` top-level y confirmé ausencia de `src/viz/`.
  - Conté LOC de bins existentes (`wc -l`): rango 80-1807, con un outlier (`lab.rs`) y mediana ~200.
  - `Cargo.toml` confirmado: `bevy 0.15 + bevy_egui 0.31 + egui_plot 0.29` disponibles.
  - No verifiqué que Cargo acepte la sintaxis `src/bin/<name>/mod.rs` sin friction. **[ASSUMPTION]** creo que sí porque las guidelines oficiales lo permiten, pero hay que probar — si rompe, el fallback `autopoietic_lab_view.rs` hermano es equivalente y no modifica la decisión.

- **Suposiciones que requieren validación:**
  - **[ASSUMPTION]** 500 LOC es el umbral de promoción — arbitrario. Alternativas: "cuando un segundo bin importe algún módulo" (mejor criterio funcional, elegido como condición OR).

- **Preguntas abiertas:**
  - ¿Actualizar `SPRINT_AP6_AUTOPOIETIC_LAB.md` para referir este ADR en la columna _Archivo_? Sí, cerrar la inconsistencia.
  - ¿Vale la pena un `lib.rs` intermedio? No para AP-6c — YAGNI hasta que haya 2 consumidores.
