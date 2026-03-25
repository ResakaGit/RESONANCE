# Track — Migración Estructural: Escalabilidad y Limpieza

**Diagnóstico:** Análisis completo de codebase (295 archivos, ~53.750 LOC) + docs (110 markdowns).
**Motivación:** Archivos >800 LOC, directorios planos con 41 archivos, boilerplate repetido, docs huérfanos.
**Filosofía:** Mover y partir — no reescribir lógica. Cada sprint es un refactor estructural puro sin cambios de comportamiento.
**Riesgo global:** Bajo. Son moves + splits + re-exports. `cargo test --lib` valida cada paso.

---

## Diagnóstico resumido

### Código (`src/`)

| Problema | Evidencia | Impacto |
|----------|-----------|---------|
| **7 archivos >800 LOC** | `materialization.rs` (1851), `presets.rs` (1193), `archetypes.rs` (919), `shape_inference.rs` (849), `materialization_rules.rs` (836), `bridged_ops.rs` (834), `propagation.rs` (810) | Difíciles de navegar, merge conflicts frecuentes |
| **`simulation/` plano** | 41 archivos sin subdirectorios temáticos | No escala al añadir MG-3…MG-8 + sistemas futuros |
| **`worldgen/` monolítico** | 22 archivos, 3 de ellos >800 LOC | Materialización + inferencia + visual mezclados |
| **Bridge boilerplate** | 11+ impls de `Bridgeable` repiten ~30 LOC de normalize/cache_key/into_cached | Mantenimiento tedioso, error-prone |
| **Constants de 1 línea** | 5-10 archivos en `blueprint/constants/` con 1 sola constante | File count innecesario |

### Docs (`docs/`)

| Problema | Evidencia |
|----------|-----------|
| **~~8 docs huérfanos~~** | ~~`VISUAL_QUANTIZATION.md`, `SENSORY_ATTENTION.md`~~ eliminados (2026-03-24). `LAYER15_TACTICAL_INFERENCE.md` retenido como propuesta activa. Restantes indexados en `docs/design/INDEX.md` |
| **Naming inconsistente** | `bluePrints/` (camelCase) vs todo lo demás (snake_case/UPPERCASE) |
| **~~Docs raíz sueltos~~** | Vinculados desde README.md y design/INDEX.md (2026-03-24) |

### Lógica duplicada (hallazgos)

| Hallazgo | Severidad | Veredicto |
|----------|-----------|-----------|
| 5 wrappers de temperature | Baja | **Intencional** — cada uno sirve un contexto distinto (entity, cell, visual, cached, derived) |
| Bridgeable boilerplate | Media | **Extraer macro** `impl_bridgeable!` |
| Constants de 1 línea | Baja | **Consolidar** en archivos por dominio |
| Inline math en systems | Muy baja | **No encontrada** — DRY bien aplicado |

---

## Grafo de dependencias

```
SM-1 (Split worldgen/materialization)     ── Onda 0 (más LOC, más impacto)
SM-2 (Subdirectorios simulation/)         ── Onda 0 (paralelo con SM-1)
     │
     ├──► SM-3 (Split bridge/)            ── Onda A
     ├──► SM-4 (Split entities/archetypes)── Onda A (paralelo con SM-3)
     │
     └──► SM-5 (Macro impl_bridgeable!)   ── Onda B (requiere SM-3)
          │
          └──► SM-6 (Consolidar constants)── Onda B (paralelo con SM-5)

SM-7 (Docs: rename + archive + limpiar)  ── Onda C (independiente de código)
```

## Ondas de ejecución

| Onda | Sprints | Qué habilita |
|------|---------|-------------|
| **0** | SM-1, SM-2 | Romper los dos monolitos más grandes; habilita MG-3+ sin conflicto |
| **A** | SM-3, SM-4 | Bridge y entities escalables |
| **B** | SM-5, SM-6 | Eliminar boilerplate y fragmentación de constants |
| **C** | SM-7 | Docs limpios y consistentes |

## Índice de sprints

| Sprint | Archivo | Módulo principal | Onda | Estado |
|--------|---------|-----------------|------|--------|
| [SM-1](SPRINT_SM1_SPLIT_WORLDGEN.md) | Split worldgen/materialization | `src/worldgen/` | 0 | ✅ Cerrado (2026-03-25) — `worldgen/materialization/` subdirectorio ya existe |
| [SM-2](SPRINT_SM2_SIMULATION_SUBDIRS.md) | Subdirectorios simulation | `src/simulation/` | 0 | ✅ Cerrado (2026-03-25) |
| [SM-3](SPRINT_SM3_SPLIT_BRIDGE.md) | Split bridge presets+ops | `src/bridge/` | A | ✅ Cerrado (2026-03-25) |
| [SM-4](SPRINT_SM4_SPLIT_ARCHETYPES.md) | Split entities/archetypes | `src/entities/` | A | ✅ Cerrado (2026-03-25) |
| [SM-5](SPRINT_SM5_BRIDGE_MACRO.md) | Macro impl_bridgeable! | `src/bridge/` | B | ✅ Cerrado (2026-03-25) — `macros.rs` + `impls/` ya implementados; `CompetitionNormBridge` wired manualmente |
| [SM-6](SPRINT_SM6_CONSOLIDATE_CONSTANTS.md) | Consolidar constants 1-línea | `src/blueprint/constants/` | B | ✅ Cerrado (2026-03-25) — constantes ya registradas en shards originales; micro-archivos eliminados |
| [SM-7](SPRINT_SM7_DOCS_CLEANUP.md) | Docs rename + archive | `docs/` | C | ✅ Cerrado (2026-03-25) |

## Invariantes del track

1. **Zero cambio de comportamiento.** Ningún sprint modifica lógica. Solo mueve, parte y re-exporta.
2. **`cargo test --lib` verde en cada sprint.** Criterio de aceptación universal.
3. **Re-exports preservan API pública.** `pub use` en cada `mod.rs` nuevo garantiza que imports externos no rompan.
4. **Un commit por split.** Cada archivo movido es un commit atómico para facilitar git blame.
5. **No tocar archivos <300 LOC.** Solo se parten archivos que lo necesitan.

## Referencias

- `docs/sprints/MIGRATION/README.md` — Track previo M1–M5 (cerrado)
- `docs/sprints/CODE_QUALITY/README.md` — Q5 plugin split (relacionado)
- `CLAUDE.md` — Reglas de coding, max 4 fields, one system one transformation
- `DESIGNING.md` — Filosofía de layers y axioma energético
