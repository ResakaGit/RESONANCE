# Sprint SM-6 — Consolidar constants de 1 línea

**Módulo:** `src/blueprint/constants/`
**Tipo:** Refactor estructural. Fusionar micro-archivos en archivos por dominio.
**Onda:** B — Paralelo con SM-5.
**Estado:** ⏳ Pendiente

## Objetivo

El directorio `blueprint/constants/` tiene 48 archivos. Al menos 10 contienen solo 1-3 constantes (4-15 LOC). Consolidar estos micro-archivos en archivos de dominio más grandes reduce file count y overhead de navegación sin perder organización semántica.

## Diagnóstico

### Micro-archivos (1-3 constantes, <20 LOC)

| Archivo | Constantes | LOC | Dominio lógico |
|---------|-----------|-----|---------------|
| `layer00_base_energy.rs` | 1 | ~4 | layer defaults |
| `layer02_oscillation.rs` | 1 | ~4 | layer defaults |
| `layer03_friction_drag.rs` | 1 | ~6 | layer defaults |
| `layer05_engine_overload.rs` | 1 | ~4 | layer defaults |
| `layer13_structural_link.rs` | 1 | ~4 | layer defaults |
| `element_id_fnv.rs` | 2 | ~6 | ids |
| `numeric_math.rs` | 3 | ~8 | math |
| `layer09_moba_crit.rs` | 2 | ~6 | layer defaults |
| `layer08_injection.rs` | 1-2 | ~6 | layer defaults |
| `layer10_resonance_link.rs` | 1-2 | ~6 | layer defaults |

### Archivos que ya están bien (>5 constantes, cohesivos)

| Archivo | Constantes | Dejar |
|---------|-----------|-------|
| `layer04_photosynthesis.rs` | 11 | ✅ |
| `layer04_phase_transition.rs` | 6 | ✅ |
| `thermal_transfer.rs` | 8+ | ✅ |
| `organ_role_visual_li6.rs` | 12+ (arrays) | ✅ |
| `morphogenesis_track/` | 14+ | ✅ |

## Propuesta de consolidación

### Fusionar micro-archivos de layers en `layer_defaults.rs`

```rust
// NEW: src/blueprint/constants/layer_defaults.rs
// Constantes default por capa que no justifican archivo propio.

// L0: BaseEnergy
pub const BASE_ENERGY_MIN: f32 = 0.001;

// L2: Oscillation
pub const DEFAULT_OSCILLATION_PHASE: f32 = 0.0;

// L3: Friction / Drag
pub const FRICTION_STATIC_THRESHOLD: f32 = 0.1;

// L5: Engine
pub const ENGINE_OVERLOAD_MULTIPLIER: f32 = 2.0;

// L8: Injection
pub const INJECTION_DEFAULT_RADIUS: f32 = 5.0;

// L9: Moba
pub const CRIT_BASE_MULTIPLIER: f32 = 1.5;

// L10: ResonanceLink
pub const LINK_DECAY_RATE: f32 = 0.01;

// L13: StructuralLink
pub const STRUCTURAL_SPRING_DAMPING: f32 = 0.1;
```

**Resultado:** 8 archivos → 1 archivo (~40 LOC). Cada constante mantiene su comentario de capa.

### Fusionar `element_id_fnv.rs` + `numeric_math.rs` en `math_and_ids.rs`

```rust
// NEW: src/blueprint/constants/math_and_ids.rs

// Numeric safety
pub const DISTANCE_EPSILON: f32 = 1e-6;
pub const DIVISION_GUARD_EPSILON: f32 = 1e-8;
pub const DRAG_SPEED_EPSILON: f32 = 1e-4;

// Element ID hashing
pub const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
pub const FNV_PRIME: u64 = 1099511628211;
```

**Resultado:** 2 archivos → 1 archivo (~15 LOC).

## Pasos de implementación

### SM-6A: Crear archivos consolidados

1. Crear `blueprint/constants/layer_defaults.rs`.
2. Copiar constantes de los 8 micro-archivos de layers.
3. Crear `blueprint/constants/math_and_ids.rs`.
4. Copiar constantes de `numeric_math.rs` y `element_id_fnv.rs`.

### SM-6B: Actualizar mod.rs

1. En `blueprint/constants/mod.rs`:
   - Quitar los 10 `pub mod` de micro-archivos.
   - Añadir `pub mod layer_defaults` y `pub mod math_and_ids`.
   - Actualizar `pub use` para mantener los mismos nombres exportados.
2. Verificar que `pub use layer_defaults::*` y `pub use math_and_ids::*` cubren todo.

### SM-6C: Eliminar micro-archivos

1. Eliminar los 10 archivos vaciados.
2. `cargo test --lib` → verde.
3. `cargo build` → sin warnings.

### SM-6D: Verificar imports externos

1. Buscar `use crate::blueprint::constants::layer00_base_energy` etc.
2. Todos deben funcionar via `pub use layer_defaults::*` en `mod.rs`.
3. Si algún import usa el nombre del módulo directamente, actualizar.

## Tácticas

- **No tocar archivos >5 constantes.** Solo consolidar micro-archivos. Los archivos grandes están bien organizados.
- **Mantener comentario de capa.** Cada constante en `layer_defaults.rs` lleva `// L{N}: {nombre}` para que sea escaneable.
- **pub use * en mod.rs.** Los consumidores usan `constants::BASE_ENERGY_MIN`, no `constants::layer_defaults::BASE_ENERGY_MIN`. El re-export transparente preserva esto.
- **48 archivos → ~38 archivos.** Reducción de ~20% en file count. Cada archivo restante tiene ≥5 constantes o es un dominio cohesivo.

## NO hace

- No cambia valores de constantes.
- No renombra constantes.
- No toca archivos de constantes con >5 definiciones.
- No modifica `morphogenesis_track/` (ya bien organizado como subdirectorio).
- No mueve constantes entre dominios (ej. no mover thermal a layer_defaults).

## Criterios de aceptación

- `cargo test --lib` pasa sin regresión.
- File count en `blueprint/constants/` se reduce de 48 a ≤40.
- Ninguna constante pública cambió de nombre.
- `blueprint/constants/mod.rs` exporta exactamente los mismos símbolos que antes.
- `layer_defaults.rs` tiene comentarios por capa (`// L0:`, `// L3:`, etc.).

## Referencias

- `src/blueprint/constants/mod.rs` — re-export hub actual (102 LOC)
- `CLAUDE.md` — "Constants in constants" rule
- `docs/sprints/CODE_QUALITY/SPRINT_Q2_MAGIC_NUMBERS.md` — track de constantes nombradas
