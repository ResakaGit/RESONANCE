# CT-7: Temporal Integration — TelescopeStack por escala

**Esfuerzo:** M (2–3 sesiones)
**Bloqueado por:** CT-6
**ADR:** ADR-036 §D5, ADR-015, ADR-016

## Objetivo

Integrar el Temporal Telescope existente (ADR-015/016) dentro de cada escala
espacial. Cada nivel tiene su propio TelescopeStack que proyecta el futuro
dentro de esa escala.

## Precondiciones

- CT-6 completado (5 niveles con background coarsening)
- TelescopeStack funcional (`batch/telescope/stack.rs`)

## Entregables

### E1: TelescopeStack per-ScaleInstance

Extender `ScaleInstance` para incluir su propio stack temporal:

```rust
pub struct ScaleInstance {
    // ... campos existentes de CT-0 ...
    pub telescope: Option<TelescopeStack>,  // None si frozen
}
```

### E2: K adaptativo por escala

Cada escala tiene K diferente porque los fenómenos tienen timescales distintos:

| Escala | K_min | K_max | Rationale |
|--------|-------|-------|-----------|
| S0 Cosmológico | 256 | 16384 | Fenómenos lentos, predicción fácil |
| S1 Estelar | 64 | 4096 | Nucleosíntesis es gradual |
| S2 Planetario | 16 | 1024 | Estaciones, ciclos geológicos |
| S3 Ecológico | 4 | 256 | Vida es impredecible, K bajo |
| S4 Molecular | 2 | 64 | MD es caótico, Lyapunov alto |

### E3: Cross-scale projection hints

El estado del nivel padre informa la proyección del hijo:
- Si S0 está en régimen `Stasis`, S1 puede usar K alto
- Si S0 está en `Chaos` (colisión de clusters), S1 baja K

```rust
pub fn parent_regime_hint(parent: &ScaleInstance) -> RegimeHint;
pub fn adjust_k_from_parent(child_k: u32, hint: RegimeHint) -> u32;
```

## Tasks

- [ ] Agregar `telescope: Option<TelescopeStack>` a `ScaleInstance`
- [ ] Configurar K ranges por ScaleLevel
- [ ] `parent_regime_hint`: extraer regime del padre
- [ ] `adjust_k_from_parent`: modular K del hijo
- [ ] Tests:
  - `telescope_per_scale_independent`
  - `molecular_scale_low_k` (Lyapunov alto → K bajo)
  - `cosmological_scale_high_k` (predicción fácil → K alto)
  - `parent_chaos_reduces_child_k`
- [ ] 0 warnings, 0 clippy

## Criterios de aceptación

1. Cada escala activa tiene su propio TelescopeStack
2. K adaptativo respeta los ranges por escala
3. Régimen del padre influye en K del hijo
4. Niveles frozen no tienen TelescopeStack (ahorro de RAM)
