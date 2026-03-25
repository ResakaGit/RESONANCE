# Sprint SM-3 — Split bridge/presets + bridged_ops

**Módulo:** `src/bridge/`
**Tipo:** Refactor estructural puro. Partir 2 archivos >800 LOC en subdirectorios por dominio.
**Onda:** A — Después de SM-1/SM-2.
**Estado:** ⏳ Pendiente

## Objetivo

Partir `bridge/presets.rs` (1,193 LOC) y `bridge/bridged_ops.rs` (834 LOC) en subdirectorios temáticos. Los 11+ bridge types están todos amontonados en 2 archivos gigantes. Separarlos por dominio físico facilita encontrar y mantener cada bridge.

## Diagnóstico

| Archivo | LOC | Contenido |
|---------|-----|-----------|
| `presets.rs` | **1,193** | Presets de configuración para 11 bridge types (bandas, tolerancias, políticas) |
| `bridged_ops.rs` | **834** | Operaciones bridged (wrappers que usan cache) para múltiples dominios |
| `bridged_physics.rs` | **520** | 3 impls de `Bridgeable` (Density, Temperature, PhaseTransition) |
| `normalize.rs` | **604** | Normalización de inputs para cache keys |
| `context_fill.rs` | **614** | Estado de fase y llenado de caches |
| `metrics.rs` | **646** | Métricas de cache hits/misses |

## Estructura actual

```
bridge/
├── mod.rs
├── cache.rs
├── config.rs
├── constants.rs
├── context_fill.rs        ← 614
├── decorator.rs
├── metrics.rs             ← 646
├── normalize.rs           ← 604
├── presets.rs             ← 1,193 (target)
├── bridged_ops.rs         ← 834 (target)
└── bridged_physics.rs     ← 520
```

## Estructura objetivo

```
bridge/
├── mod.rs                 ← actualizar re-exports
├── cache.rs
├── config.rs
├── constants.rs
├── context_fill.rs
├── decorator.rs
├── metrics.rs
├── normalize.rs
├── presets/                ← NEW: split de presets.rs
│   ├── mod.rs              ← re-exports
│   ├── physics.rs          ← Density, Temperature, PhaseTransition presets
│   ├── combat.rs           ← Collision, Will, Interference presets
│   └── ecosystem.rs        ← Osmosis, Evolution, Catalysis, Dissipation, Drag, Engine presets
├── impls/                  ← NEW: split de bridged_*.rs
│   ├── mod.rs              ← re-exports
│   ├── physics.rs          ← ← bridged_physics.rs (Bridgeable impls)
│   └── ops.rs              ← ← bridged_ops.rs (bridged operation wrappers)
└── (bridged_physics.rs eliminado, contenido en impls/physics.rs)
```

## Pasos de implementación

### SM-3A: Partir `presets.rs` → `presets/`

1. Leer `presets.rs` e identificar bloques por bridge type.
2. Crear `bridge/presets/` con `mod.rs`.
3. Agrupar presets en 3 archivos por dominio:
   - `physics.rs`: `DensityBridge`, `TemperatureBridge`, `PhaseTransitionBridge`
   - `combat.rs`: `CollisionTransferBridge`, `WillBridge`, `InterferenceBridge`
   - `ecosystem.rs`: `OsmosisBridge`, `EvolutionSurrogateBridge`, `CatalysisBridge`, `DissipationBridge`, `DragBridge`, `EngineBridge`
4. `presets/mod.rs` re-exporta todo con `pub use`.
5. Actualizar `bridge/mod.rs`: `pub mod presets` (sin cambio de path externo via re-export).
6. Eliminar `bridge/presets.rs` (reemplazado por directorio).
7. `cargo test --lib` → verde.

### SM-3B: Mover `bridged_*.rs` → `impls/`

1. Crear `bridge/impls/` con `mod.rs`.
2. **Mover** `bridged_physics.rs` → `impls/physics.rs`.
3. **Mover** `bridged_ops.rs` → `impls/ops.rs`.
4. `impls/mod.rs` re-exporta todo.
5. Actualizar `bridge/mod.rs`.
6. `cargo test --lib` → verde.

## Tácticas

- **Agrupar por dominio, no por tipo.** "Physics" agrupa bridges que operan sobre temperatura/densidad/estado. "Combat" agrupa bridges de combate. "Ecosystem" agrupa bridges ecológicos. Cada archivo tiene ~400 LOC — manejable.
- **Re-export transparente.** `use crate::bridge::DensityBridge` sigue funcionando post-split.
- **No renombrar structs ni funciones.** Solo mover.

## NO hace

- No cambia la API de `BridgeCache<B>` ni el trait `Bridgeable`.
- No modifica lógica de normalización, métricas ni context_fill.
- No toca `normalize.rs` (604 LOC — borderline pero cohesivo, no necesita split).
- No introduce la macro `impl_bridgeable!` (eso es SM-5).

## Criterios de aceptación

- `cargo test --lib` pasa sin regresión.
- Ningún archivo en `bridge/` supera 700 LOC.
- `bridge/mod.rs` re-exporta todo — imports externos sin cambio.
- Los 3 archivos de `presets/` tienen ~400 LOC cada uno (±100).

## Referencias

- `src/bridge/mod.rs` — módulo raíz
- `src/bridge/config.rs` — definición de `BridgeKind` (enum con 11 variants)
- `docs/arquitectura/blueprint_layer_bridge_optimizer.md` — contrato de bridge
