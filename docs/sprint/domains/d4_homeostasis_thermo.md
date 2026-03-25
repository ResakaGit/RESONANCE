# D4: Homeostasis & Thermoregulation

**Prioridad**: P1
**Phase**: `Phase::ChemicalLayer`
**Dependencias**: L12 (Homeostasis), L4 (MatterCoherence), morphogenesis equations
**Systems**: 3

---

## Motivación Científica

La homeostasis es la capacidad de un organismo de mantener condiciones internas estables pese a perturbaciones externas. En termorregulación:

- **Ectotermos** (reptiles, peces): T_interna ≈ T_ambiente. Bajo costo, alta dependencia ambiental.
- **Endotermos** (mamíferos, aves): T_interna = constante. Alto costo energético, independencia ambiental.

En Resonance, la "temperatura" es `equivalent_temperature(density)` y la frequency es la "firma molecular". Homeostasis L12 ya tiene los campos: `adapt_rate_hz`, `qe_cost_per_hz`, `stability_band_hz`.

**IMPORTANTE (post-verificación)**: El system `homeostasis_system` YA EXISTE en `structural_runtime.rs:184` y es ACTIVO en Phase::ChemicalLayer. Este system:
- Lee `ContainedIn` + `Homeostasis` + `OscillatorySignature`
- Adapta frecuencia de la entidad hacia la frecuencia del host (pressure container)
- Emite `HomeostasisAdaptEvent`

Por tanto, D4 NO recrea este system. En su lugar, D4 **extiende** la homeostasis existente con 2 systems adicionales de termorregulación que operan DESPUÉS del `homeostasis_system` existente en la cadena de ChemicalLayer.

---

## Ecuaciones (ya existentes + nuevas)

### Existentes (en `equations/field_body/mod.rs`):
- `homeostasis_delta_hz(current_hz, target_hz, adapt_rate, dt) -> f32`
- `homeostasis_qe_cost(delta_hz, cost_per_hz) -> f32`

### Existentes (en `morphogenesis/thermodynamics.rs`):
- `carnot_efficiency(t_core, t_env) -> f32`
- `heat_capacity(qe, specific_heat_factor) -> f32`

### Nuevas (en `equations/homeostasis/mod.rs`):
- `thermoregulation_cost(t_current, t_target, mass, conductivity) -> f32`
```
Q_loss = conductivity × surface_area × |t_current - t_env| / insulation
cost = Q_loss × dt
```
- `ectotherm_temperature(t_env, conductivity) -> f32` — converge a t_env
- `endotherm_temperature(t_target, t_env, insulation, qe_available) -> f32` — mantiene t_target si hay qe

---

## Constantes

```
src/blueprint/constants/homeostasis.rs (NUEVO)
```

```rust
pub const ENDOTHERM_TARGET_TEMP: f32 = 310.0;      // ~37°C en unidades internas
pub const ECTOTHERM_CONVERGENCE_RATE: f32 = 0.1;   // Fracción de convergencia/tick
pub const INSULATION_BASE: f32 = 1.0;               // Base insulation factor
pub const INSULATION_ARMOR_BONUS: f32 = 0.5;        // Shell/Armor adds insulation
pub const THERMOREG_MIN_QE_FRACTION: f32 = 0.1;    // No gastar en thermo si <10% qe
```

---

## Systems (2 nuevos + 1 existente)

### EXISTENTE: `homeostasis_system` (YA ACTIVO — no modificar)
**Phase**: ChemicalLayer (en cadena de reactions.rs:300)
**File**: `structural_runtime.rs:184`
**Reads**: ContainedIn, Homeostasis, OscillatorySignature (host)
**Writes**: OscillatorySignature (adapta freq), emite HomeostasisAdaptEvent
**Status**: ACTIVO. No tocar. Los systems nuevos se encadenan DESPUÉS.

### S1: `thermoregulation_cost_system` (NUEVO — Transformer)
**Phase**: ChemicalLayer, after S1
**Reads**: BaseEnergy, SpatialVolume, MatterCoherence, AmbientPressure (L6)
**Writes**: AlchemicalEngine (drain buffer for thermo cost)
**Logic**:
1. Compute t_core = equivalent_temperature(density)
2. Compute t_env from ambient_pressure or terrain
3. Cost = thermoregulation_cost(t_core, t_env, mass, conductivity)
4. Drain from engine buffer

### S2: `homeostasis_stability_check_system` (NUEVO — Emitter)
**Phase**: ChemicalLayer, .after(thermoregulation_cost_system)
**Reads**: OscillatorySignature, Homeostasis
**Logic**: Si frequency drift > stability_band → flag para morphological adaptation (D8).

---

## Tests

- `homeostasis_adapts_frequency_toward_target`
- `homeostasis_drains_qe_proportional_to_delta`
- `homeostasis_stops_adapting_when_no_qe`
- `thermoreg_endotherm_costs_more_in_cold`
- `thermoreg_ectotherm_converges_to_ambient`
