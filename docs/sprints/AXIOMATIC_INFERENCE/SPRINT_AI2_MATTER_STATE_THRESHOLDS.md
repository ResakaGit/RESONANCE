# Sprint AI-2 — Matter State Thresholds from Dissipation Ratios

**Módulo:** `src/blueprint/equations/abiogenesis/axiomatic.rs`
**Tipo:** Refactor ecuaciones puras
**Eje axiomático:** Axiom 4 (dissipation) + Axiom 1 (everything is energy)
**Estado:** ⏳ Pendiente
**Bloqueado por:** AI-1
**Esfuerzo:** Bajo (~30min)

---

## Qué existe hoy

```rust
// En axiomatic.rs — hardcoded:
const PLASMA_DENSITY_THRESHOLD: f32 = 800.0;  // ARBITRARY
const GAS_DENSITY_THRESHOLD: f32 = 300.0;     // ARBITRARY
const LIQUID_DENSITY_THRESHOLD: f32 = 80.0;    // ARBITRARY
```

## Qué debería ser

Las transiciones de estado ocurren donde el régimen de disipación cambia. El ratio entre tasas define el punto de transición:

```
liquid_threshold = (DISSIPATION_LIQUID / DISSIPATION_SOLID) ^ (1/KLEIBER) × DENSITY_SCALE
gas_threshold    = liquid + (DISSIPATION_GAS / DISSIPATION_LIQUID) ^ (1/KLEIBER) × DENSITY_SCALE
plasma_threshold = gas + (DISSIPATION_PLASMA / DISSIPATION_GAS) ^ (1/KLEIBER) × DENSITY_SCALE
```

## Tareas

1. Reemplazar `LIQUID_DENSITY_THRESHOLD`, `GAS_DENSITY_THRESHOLD`, `PLASMA_DENSITY_THRESHOLD` en `axiomatic.rs` con llamadas a `derived_thresholds::liquid_density_threshold()`, etc.
2. Eliminar las constantes hardcodeadas del archivo.
3. Verificar que `matter_state_from_density()` sigue produciendo transiciones correctas.
4. Actualizar tests de `matter_state_from_density` para usar valores derivados.

## Criterio de cierre

- `grep -r "800.0\|300.0\|80.0" src/blueprint/equations/abiogenesis/` → 0 matches en production code
- `matter_state_from_density` tests pasan con thresholds derivados
- Las transiciones siguen siendo `Solid < Liquid < Gas < Plasma` en density
