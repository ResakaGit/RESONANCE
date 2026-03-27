# Sprint GS-7 — Visual Contract: Mapeo Inyectivo Físico→Visual

**Modulo:** `src/rendering/visual_contract.rs` (nuevo), `src/blueprint/equations/visual_contract.rs` (nuevo), `src/worldgen/visual_derivation.rs` (extensión readonly)
**Tipo:** Ecuaciones puras + sistema read-only + Resource de hints visuales.
**Onda:** B — Requiere GS-5 (VictoryNucleus) + `quantized_color` existente.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe

**Lo que SÍ existe:**

- `rendering/quantized_color/` — `QuantizedColorPlugin`, paleta de frecuencias → colores. El mapping frecuencia→color ya existe.
- `worldgen/visual_derivation.rs` — `visual_derivation_system`: derivación de color y tamaño desde física. Corre en `Update`.
- `layers/base_energy.rs::BaseEnergy` — `qe` visible.
- `layers/oscillatory.rs::OscillatorySignature` — frecuencia → color.
- `layers/coherence.rs::MatterCoherence` — `structural_damage` → opacidad/saturación.
- `layers/flow.rs::FlowVector` — velocidad → traza de movimiento.
- `layers/identity.rs::Faction` — equipo → tono de color base.

**Lo que NO existe:**

1. **Contrato formal del mapping.** No hay documentación/código que garantice injectividad. Dos estados físicos distintos pueden producir la misma señal visual.
2. **Resource de VisualHints.** El visual no se acumula en un lugar canónico — está disperso en sistemas.
3. **qe→brillo como función explícita.** Existe implícitamente en `visual_derivation` pero sin ecuación nombrada.
4. **structural_damage→saturación.** La degradación visual por daño no existe como función separada.
5. **Velocidad→alpha trail.** No hay trail de movimiento formalmente definido.
6. **Test de injectividad.** No se verifica que el contrato sea biyectivo sobre el espacio de estados relevantes.

---

## Objetivo

Formalizar el contrato visual: cada dimensión física relevante (qe, frecuencia, daño estructural, velocidad) mapea a una dimensión visual ortogonal (brillo, matiz, saturación, trail). El mapping es inyectivo por construcción — no hay ambigüedad táctica. El renderer consume `VisualHints`, no ECS.

```
frecuencia     → matiz     (quantized_color — ya existe)
qe             → brillo    (luminance)   ← GS-7 formaliza
structural_dmg → saturación (saturation decay) ← GS-7 formaliza
velocidad      → trail_alpha              ← GS-7 formaliza
faction        → tono base (hue offset)  ← GS-7 documenta
```

---

## Responsabilidades

### GS-7A: Ecuaciones del contrato visual

```rust
// src/blueprint/equations/visual_contract.rs

/// qe → brillo normalizado. qe_max = umbral de existencia máximo esperado.
/// Retorna [0,1]. 0 = muerto/casi muerto, 1 = máxima energía.
pub fn qe_to_luminance(qe: f32, qe_max: f32) -> f32 {
    if qe_max <= 0.0 { return 0.0; }
    (qe / qe_max).clamp(0.0, 1.0).sqrt()  // sqrt: perceptual linearity
}

/// structural_damage → multiplicador de saturación.
/// damage ∈ [0,1]. 0 = intacto (saturación completa), 1 = destruido (gris).
pub fn damage_to_saturation(structural_damage: f32) -> f32 {
    1.0 - structural_damage.clamp(0.0, 1.0) * SATURATION_DECAY_FACTOR
}

/// velocidad → alpha del trail de movimiento.
/// speed: magnitud de FlowVector. max_speed: umbral de velocidad máxima esperada.
pub fn speed_to_trail_alpha(speed: f32, max_speed: f32) -> f32 {
    if max_speed <= 0.0 { return 0.0; }
    (speed / max_speed).clamp(0.0, 1.0).powi(2)  // cuadrático: trails sólo en alta velocidad
}

/// VictoryNucleus → pulso de brillo periódico para señalizar objetivo.
/// tick_id para efecto de pulsación. Retorna factor [0.8,1.2].
pub fn nucleus_pulse_factor(tick_id: u64, pulse_period_ticks: u32) -> f32 {
    use std::f32::consts::TAU;
    let phase = (tick_id % pulse_period_ticks as u64) as f32 / pulse_period_ticks as f32;
    1.0 + 0.2 * (phase * TAU).sin()
}

/// ¿Es el mapping inyectivo? Verifica que dos estados físicos distintos producen
/// señales visuales distintas. Test invariant — no llames en hot path.
#[cfg(test)]
pub fn is_injective_sample(
    qe_a: f32, freq_a: f32, dmg_a: f32,
    qe_b: f32, freq_b: f32, dmg_b: f32,
    qe_max: f32,
) -> bool {
    let lum_a = qe_to_luminance(qe_a, qe_max);
    let lum_b = qe_to_luminance(qe_b, qe_max);
    let sat_a = damage_to_saturation(dmg_a);
    let sat_b = damage_to_saturation(dmg_b);
    // Distinto (qe, dmg) → distinto (lum, sat) en al menos una dimensión
    (lum_a - lum_b).abs() > 0.01 || (sat_a - sat_b).abs() > 0.01 ||
    (freq_a - freq_b).abs() > 1.0
}
```

### GS-7B: Resource VisualHints

```rust
// src/rendering/visual_contract.rs

/// Hint visual calculado por el sistema de contrato. Read-only para el renderer.
/// Derivado en Phase::MorphologicalLayer, consumido en Update.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityVisualHint {
    pub entity_id: u32,     // WorldEntityId — no Entity Bevy (INV-5)
    pub hue: f32,           // [0,1] — de quantized_color
    pub luminance: f32,     // [0,1] — de qe
    pub saturation: f32,    // [0,1] — de structural_damage
    pub trail_alpha: f32,   // [0,1] — de velocidad
    pub is_nucleus: bool,   // pulso visual especial
    pub pulse_factor: f32,  // [0.8,1.2] si is_nucleus
}

/// Resource canónico de hints visuales. El renderer consume SOLO esto.
/// Actualizado una vez por tick en Phase::MorphologicalLayer.
#[derive(Resource, Default, Debug)]
pub struct VisualHints {
    pub entities: Vec<EntityVisualHint>,  // ordenado por entity_id
}

impl VisualHints {
    pub fn clear(&mut self) { self.entities.clear(); }
    pub fn push(&mut self, hint: EntityVisualHint) { self.entities.push(hint); }
    pub fn sorted(&mut self) { self.entities.sort_unstable_by_key(|h| h.entity_id); }
}
```

### GS-7C: Sistema del contrato visual

```rust
/// Deriva VisualHints desde estado físico. Read-only sobre ECS.
/// Phase::MorphologicalLayer — último sistema antes de Update.
/// INV-5: nunca escribe componentes ECS. Sólo escribe VisualHints Resource.
pub fn visual_contract_sync_system(
    entities: Query<(
        &WorldEntityId,
        &BaseEnergy,
        &OscillatorySignature,
        Option<&MatterCoherence>,
        Option<&FlowVector>,
        Option<&VictoryNucleus>,
    )>,
    color_palette: Res<QuantizedColorPalette>,
    clock: Res<SimulationClock>,
    mut hints: ResMut<VisualHints>,
    config: Res<VisualContractConfig>,
) {
    hints.clear();
    for (id, energy, osc, coherence, flow, nucleus) in &entities {
        let hue = color_palette.frequency_to_hue(osc.frequency_hz());
        let luminance = visual_contract_eq::qe_to_luminance(energy.qe(), config.qe_max);
        let saturation = coherence.map_or(1.0, |c| {
            visual_contract_eq::damage_to_saturation(c.structural_damage())
        });
        let speed = flow.map_or(0.0, |f| f.velocity().length());
        let trail_alpha = visual_contract_eq::speed_to_trail_alpha(speed, config.max_speed);
        let is_nucleus = nucleus.is_some();
        let pulse_factor = if is_nucleus {
            visual_contract_eq::nucleus_pulse_factor(clock.tick_id, config.nucleus_pulse_period)
        } else {
            1.0
        };

        hints.push(EntityVisualHint {
            entity_id: id.0,
            hue, luminance, saturation, trail_alpha, is_nucleus, pulse_factor,
        });
    }
    hints.sorted();  // canónico: ordenado por entity_id
}
```

### GS-7D: Constantes

```rust
// src/blueprint/constants/visual_contract.rs

/// Máximo qe esperado para normalizar luminancia (entidad de referencia).
pub const QE_MAX_REFERENCE: f32 = 1000.0;
/// Factor de decay de saturación por daño estructural. 1.0 = gris total a damage=1.
pub const SATURATION_DECAY_FACTOR: f32 = 0.85;
/// Velocidad máxima de referencia para normalizar trail.
pub const MAX_REFERENCE_SPEED: f32 = 15.0;
/// Período de pulsación del núcleo de victoria en ticks.
pub const NUCLEUS_PULSE_PERIOD_TICKS: u32 = 30;

#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct VisualContractConfig {
    pub qe_max: f32,
    pub max_speed: f32,
    pub nucleus_pulse_period: u32,
}
```

---

## Tacticas

- **Read-only absoluto sobre ECS.** `visual_contract_sync_system` tiene `Query<(&...), ()>` — sin `mut`. INV-5 aplicado por el compilador.
- **VisualHints como buffer.** El renderer lee `VisualHints` en `Update` — asíncronicamente del tick de simulación. Patrón snapshot ya existente en IWG.
- **Cuatro dimensiones ortogonales.** Hue (frecuencia), luminance (qe), saturation (daño), trail (velocidad). No hay overlap — el contrato es inyectivo por construcción.
- **nucleus_pulse desde tick_id.** La pulsación usa `tick_id` como reloj — determinista, sin `std::time`. INV-8 compliant.

---

## NO hace

- No cambia el sistema de rendering de Bevy (sprites, meshes, shaders) — eso es resonance-app.
- No define UI/HUD — eso es `runtime_platform/hud`.
- No genera colores para el minimapa — eso es derivación de `VisualHints` por la app.
- No implementa animaciones de muerte o spawn — efectos especiales son responsabilidad del renderer.

---

## Dependencias

- `rendering/quantized_color/` — `QuantizedColorPalette`, mapeo frecuencia→color.
- GS-5 — `VictoryNucleus` marker (para pulse).
- `layers/coherence.rs::MatterCoherence` — `structural_damage`.
- `layers/flow.rs::FlowVector` — velocidad.
- `blueprint/ids/types.rs::WorldEntityId` — ID canónico (no `Entity` Bevy).
- `src/sim_world.rs` — `WorldSnapshot` patrón (VisualHints es análogo).

---

## Criterios de aceptacion

### GS-7A (Ecuaciones)
- `qe_to_luminance(0.0, 1000.0)` → `0.0`.
- `qe_to_luminance(1000.0, 1000.0)` → `1.0`.
- `qe_to_luminance(250.0, 1000.0)` → `0.5` (sqrt(0.25)).
- `damage_to_saturation(0.0)` → `1.0`.
- `damage_to_saturation(1.0)` → menor que `0.5`.
- `speed_to_trail_alpha(0.0, 15.0)` → `0.0`.
- `speed_to_trail_alpha(15.0, 15.0)` → `1.0`.
- `nucleus_pulse_factor(0, 30)` → `1.0` (fase 0 → sin().cos = 0).
- Test injectividad: dos entidades con mismo qe pero distinta frecuencia → hints distintos.

### GS-7B/C (Resource + Sistema)
- Test: `visual_contract_sync_system` no modifica ningún componente ECS.
- Test: `VisualHints.entities` ordenado por `entity_id` después del sync.
- Test: entidad con `VictoryNucleus` → `is_nucleus = true`, `pulse_factor ≠ 1.0`.
- Test: entidad sin `MatterCoherence` → `saturation = 1.0` (default sin daño).
- Test: `cargo test --lib` sin regresión.

### General
- Invariante INV-5 verificable: grep `mut` en `visual_contract_sync_system` → cero queries mutables.

---

## Referencias

- `src/rendering/quantized_color/` — paleta existente
- `src/worldgen/visual_derivation.rs` — sistema de derivación existente (GS-7 lo formaliza)
- `src/sim_world.rs` — INV-5 (renderer read-only), patrón WorldSnapshot
- Blueprint §7: "Visual Legibility Contract", "Injective State→Signal Mapping"
- `docs/arquitectura/blueprint_quantized_color.md` — color pipeline
- `docs/arquitectura/blueprint_visual_quantization.md` — cuantización visual
