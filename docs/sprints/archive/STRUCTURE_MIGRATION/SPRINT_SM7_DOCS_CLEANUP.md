# Sprint SM-7 — Docs: Rename, Archive, Limpiar

**Módulo:** `docs/` + archivos raíz
**Tipo:** Reorganización de documentación. Naming consistente, archivar huérfanos, mover sueltos.
**Onda:** C — Independiente del código.
**Estado:** ⏳ Pendiente

## Objetivo

Resolver 3 problemas de la documentación: naming inconsistente (`bluePrints/` camelCase), 8 docs huérfanos sin referencias, y archivos sueltos en `docs/` raíz. El resultado es una estructura limpia, consistente y navegable.

## Diagnóstico

### Naming inconsistente

| Carpeta actual | Problema | Propuesta |
|---------------|---------|-----------|
| `docs/bluePrints/` | camelCase, resto es snake_case/UPPERCASE | `docs/design/` |
| `docs/VerifyWay/` | PascalCase, inconsistente | `docs/verify/` |
| `docs/arquitectura/` | Español en path | `docs/architecture/` (opcional, ver nota) |

**Nota:** `arquitectura/` es aceptable según la convención del proyecto ("Documentación puede usar español narrativo"). El rename es opcional. `bluePrints/` y `VerifyWay/` sí rompen consistencia.

### Docs huérfanos (0 referencias entrantes)

| Archivo | LOC | Problema |
|---------|-----|---------|
| `bluePrints/BLUEPRINT_LAYER15_TACTICAL_INFERENCE.md` | ~100 | Propone Layer 15 que no existe en DESIGNING.md ni en código |
| `bluePrints/BLUEPRINT_VISUAL_QUANTIZATION.md` | **12** | Stub vacío |
| `bluePrints/BLUEPRINT_SENSORY_ATTENTION.md` | ~50 | Reemplazado por `arquitectura/blueprint_sensory_lod.md` |
| `arquitectura/blueprint_sensory_lod.md` | ~80 | No indexado en `arquitectura/README.md` |
| `docs/PLANE_COMPOSITION.md` | ~60 | Aislado, sin refs |
| `docs/DEMO_FLOW.md` | ~40 | Solo ref en root README |
| `PLANT_SIMULATION.md` (raíz) | 88 | Flora MVP — ¿aún relevante? |
| `TOPOLOGY_AND_LAYERS.md` (raíz) | 148 | Parcialmente cubierto por blueprint_topology + blueprint_v7 |

### Archivos sueltos en docs/ raíz

```
docs/
├── DEMO_FLOW.md           ← suelto
├── PLANE_COMPOSITION.md   ← suelto, huérfano
```

## Estructura objetivo

```
docs/
├── design/                         ← RENAME de bluePrints/
│   ├── BLUEPRINT.md
│   ├── BLUEPRINT_BRIDGE_OPTIMIZER.md
│   ├── BLUEPRINT_ECO_BOUNDARIES.md
│   ├── BLUEPRINT_EMERGENT_FLORA.md
│   ├── BLUEPRINT_FOLDER_STRUCTURE.md
│   ├── BLUEPRINT_GAMEDEV_IMPLEMENTATION.md
│   ├── BLUEPRINT_GAMEDEV_PATTERNS.md
│   ├── BLUEPRINT_CHEMICAL_REFACTOR.md
│   ├── BLUEPRINT_GEOMETRY_DEFORMATION_ENGINE.md
│   ├── BLUEPRINT_MACRO_STEPPING.md
│   ├── BLUEPRINT_MORPHOGENESIS.md
│   ├── BLUEPRINT_QUANTIZED_COLOR_ENGINE.md
│   ├── BLUEPRINT_SIM_LAYERS.md
│   ├── BLUEPRINT_THERMODYNAMIC_LADDER.md
│   ├── BLUEPRINT_TOPOLOGY.md
│   ├── BLUEPRINT_V6.md
│   ├── BLUEPRINT_V7.md
│   └── archive/                    ← NEW: huérfanos + históricos
│       ├── BLUEPRINT_LAYER15_TACTICAL_INFERENCE.md
│       ├── BLUEPRINT_VISUAL_QUANTIZATION.md
│       ├── BLUEPRINT_SENSORY_ATTENTION.md
│       ├── BLUEPRINT_V2.md
│       ├── BLUEPRINT_V3.md
│       ├── BLUEPRINT_V4.md
│       └── BLUEPRINT_V5.md
├── arquitectura/                   ← sin rename (español es convención)
│   ├── README.md                   ← ACTUALIZAR: añadir blueprint_sensory_lod
│   └── (sin cambio en archivos)
├── sprints/                        ← sin cambio
├── verify/                         ← RENAME de VerifyWay/
│   └── VERIFY_WAVE_2026-03-19.md
└── guides/                         ← NEW: docs sueltos
    ├── DEMO_FLOW.md
    └── PLANE_COMPOSITION.md
```

**Root (opcionales):**
```
PLANT_SIMULATION.md     → mover a docs/guides/ si equipo confirma que no es entry point
TOPOLOGY_AND_LAYERS.md  → mover a docs/guides/ si equipo confirma
```

## Pasos de implementación

### SM-7A: Rename `bluePrints/` → `design/`

1. `git mv docs/bluePrints docs/design`.
2. **Actualizar TODAS las referencias** — buscar `bluePrints` en todo el repo:
   - `docs/arquitectura/README.md`
   - `docs/arquitectura/blueprint_morphogenesis_inference.md`
   - `docs/arquitectura/*.md` (links a blueprints)
   - `docs/sprints/*/README.md`
   - `CLAUDE.md`
   - `DESIGNING.md` (si aplica)
3. Usar buscar-y-reemplazar: `bluePrints` → `design` en paths de links markdown.
4. Verificar que ningún enlace roto queda: `grep -r "bluePrints" docs/`.

### SM-7B: Crear `design/archive/`

1. Crear `docs/design/archive/`.
2. Mover archivos huérfanos:
   - `BLUEPRINT_LAYER15_TACTICAL_INFERENCE.md` → `archive/`
   - `BLUEPRINT_VISUAL_QUANTIZATION.md` → `archive/`
   - `BLUEPRINT_SENSORY_ATTENTION.md` → `archive/`
3. Mover blueprints de versiones cerradas (V2-V5):
   - `BLUEPRINT_V2.md` → `archive/`
   - `BLUEPRINT_V3.md` → `archive/`
   - `BLUEPRINT_V4.md` → `archive/`
   - `BLUEPRINT_V5.md` → `archive/`
4. Crear `archive/README.md` mínimo:
   ```markdown
   # Archived Blueprints
   Docs huérfanos o históricos. No referenciados activamente.
   ```

### SM-7C: Rename `VerifyWay/` → `verify/`

1. `git mv docs/VerifyWay docs/verify`.
2. Actualizar refs si existen (actualmente 0 — doc aislado).

### SM-7D: Crear `docs/guides/`

1. Crear `docs/guides/`.
2. `git mv docs/DEMO_FLOW.md docs/guides/`.
3. `git mv docs/PLANE_COMPOSITION.md docs/guides/`.
4. Actualizar ref en `README.md` raíz si apunta a `docs/DEMO_FLOW.md`.

### SM-7E: Actualizar índices

1. **`docs/arquitectura/README.md`:** Añadir `blueprint_sensory_lod.md` al índice.
2. **`docs/design/` (ex-bluePrints):** Verificar que no hay refs internas rotas tras mover archive.
3. **`CLAUDE.md`:** Actualizar paths de blueprints si cambiaron.
4. **`docs/sprints/README.md`:** Actualizar refs a design/ si aplica.

### SM-7F: Root docs (decisión del equipo)

1. **`PLANT_SIMULATION.md`:** ¿Sigue siendo entry point? Si no → `git mv` a `docs/guides/`.
2. **`TOPOLOGY_AND_LAYERS.md`:** ¿Sigue siendo entry point? Si no → `git mv` a `docs/guides/`.
3. **No mover sin confirmación.** Estos archivos pueden ser entry points para nuevos contributors.

## Tácticas

- **`git mv` siempre.** Preserva historia. Nunca copiar+eliminar.
- **Buscar y reemplazar masivo.** `grep -r "bluePrints" .` antes y después para verificar 0 refs rotas.
- **No editar contenido de docs.** Solo mover y actualizar links. Si un doc tiene errores de contenido, es otro sprint.
- **archive/ no se borra.** Los docs archivados siguen accesibles. Solo dejan de estar en el índice principal.

## NO hace

- No edita contenido de ningún blueprint (solo links/paths).
- No elimina documentación — solo archiva.
- No toca `docs/sprints/` (ya bien organizado).
- No renombra `docs/arquitectura/` (español es convención aceptada).
- No modifica código fuente.

## Criterios de aceptación

- `grep -r "bluePrints" docs/` → 0 resultados (todas las refs actualizadas).
- `docs/design/` existe con todos los blueprints activos.
- `docs/design/archive/` contiene 7 docs históricos/huérfanos.
- `docs/verify/` existe (renombrado de VerifyWay).
- `docs/guides/` existe con DEMO_FLOW.md y PLANE_COMPOSITION.md.
- `docs/arquitectura/README.md` lista `blueprint_sensory_lod.md`.
- CLAUDE.md paths actualizados si cambió alguno.
- Ningún link markdown roto en docs/ (verificar con grep de `](` + path que no exista).

## Referencias

- `docs/arquitectura/README.md` — índice de arquitectura
- `docs/arquitectura/00_contratos_glosario.md` — contrato editorial
- `CLAUDE.md` — paths a blueprints
- `docs/sprints/MIGRATION/README.md` — precedente de migración
