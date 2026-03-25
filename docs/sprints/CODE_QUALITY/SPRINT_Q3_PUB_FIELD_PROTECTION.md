# Sprint Q3 — Proteger Campos Pub con Accessors

**Tipo:** Refactor — encapsulacion.
**Severidad:** MEDIA — cualquier sistema puede corromper estado saltando invariantes.
**Onda:** 0 — Sin dependencias.

## Objetivo

Convertir campos `pub` de structs criticos a `pub(crate)` con metodos de acceso que validan invariantes. Esto evita que codigo externo mute estado sin pasar por las ecuaciones del blueprint.

## Hallazgos — structs con campos pub peligrosos

### Prioridad 1: Afectan ecuaciones directamente

| Struct | Archivo | Campo | Riesgo | Accessor sugerido |
|--------|---------|-------|--------|-------------------|
| `BaseEnergy` | `layers/energy.rs:15` | `pub qe` | Bypass de `inject()`/`drain()` que validan | `pub fn qe(&self) -> f32` (read), mantener `inject/drain` para write |
| `MatterCoherence` | `layers/coherence.rs:31` | `pub state`, `pub bond_energy_eb`, `pub thermal_conductivity` | Transicion de fase sin ecuaciones | `pub fn state(&self) -> MatterState`, `pub fn set_state(&mut self, s: MatterState)` |
| `FlowVector` | `layers/flow.rs:14` | `pub velocity` | Puede exceder velocity_limit | `pub fn velocity(&self) -> Vec2`, `pub fn set_velocity(&mut self, v: Vec2)` con clamp |
| `OscillatorySignature` | `layers/oscillatory.rs:23` | `pub frequency_hz`, `pub phase` | Phase puede exceder 2π, freq puede ser NaN | `pub fn frequency_hz(&self) -> f32`, setter con sanitize |

### Prioridad 2: Componentes de gameplay

| Struct | Archivo | Campo | Riesgo | Accessor sugerido |
|--------|---------|-------|--------|-------------------|
| `AlchemicalEngine` | `layers/engine.rs:16` | `pub current_buffer`, `pub max_buffer`, `pub input_valve`, `pub output_valve` | Mutacion sin validar limites | Getters + setters con clamp |
| `WillActuator` | `layers/will.rs:13` | `pub movement_intent`, `pub channeling_ability`, `pub active_slot` | Estado conflictivo posible | Getters + metodos de transicion |
| `Grimoire` | `layers/will.rs:110` | `pub abilities: Vec<AbilitySlot>` | Vec crece sin limite | `pub fn push_ability()` con max slots |
| `MobaIdentity` | `layers/identity.rs:32` | `pub critical_multiplier` | Puede ser negativo | Getter + setter con clamp >= 0 |

### Prioridad 3: Worldgen

| Struct | Archivo | Campo | Riesgo |
|--------|---------|-------|--------|
| `EnergyNucleus` | `worldgen/nucleus.rs` | Todos los campos pub | Bypass del `new()` que sanitiza NaN/Inf |
| `FrequencyContribution` | `worldgen/contracts.rs` | Campos pub | Bypass del `new()` que sanitiza |

### NO tocar (pub es correcto)

- `Scoreboard` — DTO simple, no tiene invariantes.
- `SpatialEntry` — value type, no tiene estado mutable.
- Marker components (`DespawnOnContact`, `SpellMarker`, etc.) — sin campos.
- Config structs (`PhysicsConfig`, etc.) — builder inputs.

## Tacticas

- **`pub(crate)` no `private`.** Los sistemas de simulacion dentro del crate necesitan acceso directo. Solo el codigo externo al crate debe pasar por accessors. `pub(crate)` es el balance correcto.
- **Getters son `#[inline]`**. Los accessors de lectura (`fn qe(&self) -> f32`) son triviales y deben ser inline para zero overhead.
- **Setters validan.** `set_velocity` clampea a velocity_limit. `set_state` no valida (la transicion es responsabilidad de equations). Pero `inject` y `drain` en BaseEnergy SI validan (ya existen).
- **Migrar callers incrementalmente.** Buscar todos los usos de `.qe` directo, reemplazar por `.qe()`. Buscar asignaciones `.qe = x`, reemplazar por `inject`/`drain` o nuevo setter si necesario.
- **Un commit por struct.** BaseEnergy primero (mas critico), luego MatterCoherence, luego el resto.

## NO hace

- No cambia logica de ecuaciones.
- No agrega validacion de negocio en setters (solo sanitizacion basica: NaN, clamp).
- No modifica tests existentes (excepto para usar accessors).
- No mueve archivos.

## Dependencias

- Ninguna (Sprint 0).

## Estado (2026-03)

**Implementado en código:** campos `pub` → `pub(crate)` + getters/setters en los structs de prioridad 1–3 de la tabla; callers migrados en `src/`. `Grimoire::push_ability` retorna `bool` y respeta `MAX_GRIMOIRE_ABILITIES` (64). Motor: API pública `buffer_level` / `buffer_cap` / `valve_in_rate` / `valve_out_rate` + `try_subtract_buffer`.

**Tests:** `tests/q3_pub_field_api.rs` (crate de integración) ejercita accessors públicos; ahí no compila acceso directo a campos encapsulados.

**Pendiente / fuera de alcance inmediato:** `AlchemicalForge`, `AbilitySlot`, `EnergyCell` y otros DTOs con `pub` no listados en la tabla Q3; evaluar en Q7 u otro sprint.

## Criterio de aceptacion

- Test: `BaseEnergy.qe` no es accesible directamente fuera del crate (compile error).
- Test: `energy.qe()` retorna el valor.
- Test: `MatterCoherence.state` no es accesible directamente fuera del crate.
- Test: `FlowVector.velocity` no es accesible directamente fuera del crate.
- Test: `OscillatorySignature.frequency_hz` no es accesible directamente fuera del crate.
- Test: `EnergyNucleus` campos no accesibles directamente (solo via `new()` + getters).
- `cargo test` pasa.
- `cargo build` pasa (ningun caller externo usa campos directamente).

**Nota:** el criterio “compile error” se valida intentando usar campos en `tests/*.rs`: el harness de integración es otro crate y solo ve ítems `pub`. Los tests positivos viven en `tests/q3_pub_field_api.rs`. Para snippets `compile_fail` explícitos se puede añadir `trybuild` más adelante.
