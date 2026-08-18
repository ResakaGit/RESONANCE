# Sprint TU · Consolidación de test binaries (ejecución de ADR-048)

**Estado:** DONE — 2026-08-18
**ADR:** [ADR-048](../../arquitectura/ADR/ADR-048-test-suite-consolidation.md)
**Origen:** `docs/rfc/RFC-002-test-binary-consolidation.md` (stub, Superseded)
**Base:** HEAD `f75362a`

> **Nota de encuadre.** El track TOOL_USE trata de *herramientas emergentes
> in-game* (modify intent, crafting, farming). Este ítem es **tooling de
> desarrollo**, no simulación: se aloja acá porque RFC-002 §1 designó "sprint
> track TU" como destino de la ejecución mecánica. Si el repo crea un track de
> infraestructura/DX, este doc se muda ahí — no comparte backlog con TU-1..TU-4.

---

## Ítem TU-C1 · Consolidar `tests/` de 37 binarios a 6 targets

**Entregable:** 35 archivos no gated agrupados en 4 suites
(`axioms` 14 · `probes` 7 · `pipeline` 7 · `platform` 7) + 2 binarios gated
intactos. `Cargo.toml` sin diff (autodiscovery).

### Definition of Ready (cumplida)

- [x] ADR de respaldo con 3 rondas de crítica adversarial (RFC-002 → ADR-048).
- [x] Gate duro: `git status --porcelain tests/` vacío antes del primer `git mv`.
- [x] Baseline de conteo de tests capturado antes de tocar nada (criterio 2a).

### Ejecución (registro)

| Paso | Acción | Resultado |
|---|---|---|
| 0 | Gate `git status --porcelain tests/` | vacío ✅ |
| 1 | Baseline `cargo test --tests --no-run` + `--list` × 35 | 43.18 s · **272 tests** en 35 archivos |
| 2 | `mkdir` × 4 + `git mv` × 35 + 4 `main.rs` | 35 renames puros `R`; `Cargo.toml` sin diff |
| 3 | Protocolo proptest §2.3 (probe descartable → observar → move → replay) | ruta `tests/proptest-regressions/<name>.txt`; 2 sidecars movidos; replay probado sobre el archivo real |
| 4 | Post-move `cargo test --tests --no-run` + `--list` × 4 | 10.07 s · **272 tests**, diff de conteos **vacío** |
| 5 | Correr las 4 suites | axioms 130+4ign · probes 31 · pipeline 68 · platform 39 — **0 fallos** |
| 6 | Re-grep + actualización de docs | 11 loci (ver ADR-048 §5) |
| 7 | ADR-048 + fila en `ADR/README.md` + stub RFC-002 | hecho |
| 8 | `cargo metadata --no-deps` | **6 test targets** ✅ |

### Acceptance (verificado)

- [x] `cargo metadata`: exactamente 6 test targets; `git diff -- Cargo.toml` vacío.
- [x] Cero tests perdidos: 272 → 272, 35/35 módulos, diff programático vacío.
- [x] Cero cambios en el cuerpo de los archivos movidos (todos `R` en git).
- [x] Protocolo proptest empírico completo, replay confirmado sobre el sidecar real.
- [x] `CLAUDE.md`, matriz, ADR-045/046/047, sprints EM1/AP6, regulatory ×2 actualizados.
- [x] Fila de ADR-048 en `ADR/README.md`.

### Pendiente / abierto

- **Nightly (criterio 5).** `workflow_dispatch` no se dispara localmente. El yml
  quedó actualizado en 4 líneas (78, 86 + los `paths:` triggers 23 y 37, hallazgo
  no previsto por el RFC). **Se valida en el primer run post-push**, vigilando el
  `timeout-minutes: 20` — se dejó sin cambiar por falta de medición de CI.
- **Rutas `tests/*.rs` stale en `docs/regulatory/**`** (~60 refs a
  `tests/property_conservation.rs`): fuera de alcance declarado (ADR-048 §9).
  Deuda documental, sin efecto funcional. Follow-up en su propio PR.

### Lección transferible

El re-grep de comandos (`--test +[a-z0-9_]+`) **no sustituye** al grep de rutas
(`tests/[a-z0-9_]+\.rs`). Mover archivos exige ambos: el primero encuentra
invocaciones, el segundo encuentra triggers de CI, tablas de archivos y citas
`file:line`.
