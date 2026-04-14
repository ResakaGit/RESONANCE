# CT-8: Scale-Aware Rendering — La experiencia del dios observando

**Esfuerzo:** L (3–5 sesiones)
**Bloqueado por:** CT-6
**ADR:** ADR-036 §D6

## Objetivo

Binario `cosmic_telescope` con visualización 3D que permite navegar desde el
Big Bang hasta nivel molecular con transiciones suaves entre escalas.

## Precondiciones

- CT-6 completado (5 niveles con background)
- Bevy 0.15 rendering pipeline
- earth_telescope como referencia de HUD (`bin/earth_telescope.rs`)

## Entregables

### E1: `cosmic_telescope.rs` — binario principal

```rust
// src/bin/cosmic_telescope.rs
//
// Controls:
//   Click entity → zoom in (colapso observacional)
//   Escape       → zoom out (agregación)
//   Scroll       → camera zoom (dentro del nivel)
//   Space        → pause/resume
//   1-5          → saltar a escala S0-S4 (si instanciada)
//   Tab          → cycle seed (multiverso)
```

### E2: Render por escala

| Escala | Render style | Entities like |
|--------|-------------|---------------|
| S0 Cosmológico | Puntos luminosos + halos de energía | Galaxias |
| S1 Estelar | Esferas brillantes + discos | Estrellas |
| S2 Planetario | Esferas con textura de energía | Planetas (earth_telescope) |
| S3 Ecológico | Criaturas con mesh inferido | Juego actual |
| S4 Molecular | Esferas + sticks (ball-and-stick) | Proteínas |

### E3: Transición de escala (zoom animation)

1. Click en entidad → highlight
2. Cámara se acerca (lerp 0.5s)
3. Fade out entidades del nivel actual (alpha → 0)
4. Spawn entidades del nivel inferior (scale 0 → 1, 0.3s)
5. Fade in nuevo nivel
6. HUD actualiza: escala, qe, edad, seed, regime

### E4: HUD multi-escala

```
┌─────────────────────────────────────────────┐
│ COSMIC TELESCOPE                            │
│ Scale: Ecological (S3)    Seed: 42          │
│ Universe age: 1.2M ticks  Regime: Stasis    │
│ Total qe: 8,421           Entities: 347     │
│                                              │
│ Breadcrumb: S0:Cluster_7 > S1:Star_12 >     │
│             S2:Planet_3 > S3 (here)          │
│                                              │
│ [1]S0 ◉  [2]S1 ◉  [3]S2 ◉  [4]S3 ●  [5]S4 ○│
│ ◉=active  ●=observed  ○=not instantiated    │
└─────────────────────────────────────────────┘
```

### E5: Breadcrumb trail

El HUD muestra el path de zoom: qué cluster → qué estrella → qué planeta → aquí.
Permite click en cualquier breadcrumb para zoom-out directo a ese nivel.

## Tasks

- [ ] Crear `src/bin/cosmic_telescope.rs`
- [ ] Render por escala (5 visual modes)
- [ ] Zoom animation (lerp + fade + spawn)
- [ ] HUD con breadcrumb trail
- [ ] Input: click → zoom, escape → out, scroll → camera, tab → seed
- [ ] Transición suave entre escalas (no flash/pop)
- [ ] Tests manuales:
  - Zoom S0→S1→S2→S3→S4 sin crash
  - Zoom S4→S3→S2→S1→S0 sin crash
  - Tab cambia seed → re-colapsa nivel actual
  - Pause/resume funciona en cualquier escala
- [ ] 0 warnings, 0 clippy

## Criterios de aceptación

1. Transición visual suave (no teleport, no flash)
2. HUD muestra breadcrumb path completo
3. Funciona a ≥30 FPS en release
4. Cada escala tiene look visualmente distinto
5. Click en breadcrumb navega directamente a esa escala
