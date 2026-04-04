# MT-1: Ecuaciones Cuánticas

**Objetivo:** Funciones puras que implementan los tres principios cuánticos: visibilidad especulativa (Englert), proyección conservation-bounded (Axioma 4+5), y tasa de decay con frecuencia (Axioma 8). Más constantes derivadas para el stack multi-nivel.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Bajo (math pura, sin ECS, extiende temporal_telescope.rs existente)
**Bloqueado por:** —
**Desbloquea:** MT-2, MT-3

---

## Entregables

### 1. En `src/blueprint/equations/temporal_telescope.rs` (agregar al archivo existente)

```rust
/// Visibilidad especulativa (coherencia) del nivel. Englert: D²+V²≤1.
/// V=0: colapsado (certeza). V=1: onda pura (máxima incertidumbre).
///
/// D = e^{-ticks_to_anchor / coherence_length} (confianza por proximidad)
/// V = sqrt(1 - D²) (incertidumbre complementaria)
pub fn speculative_visibility(ticks_to_anchor: u64, coherence_length: f32) -> f32

/// Proyección conservation-bounded. Axioma 4+5.
/// Clamp: base_decay ≤ resultado ≤ current_qe.
/// La disipación siempre reduce. La proyección nunca supera el input.
pub fn conservation_bounded_project(
    current_qe: f32,
    base_decay: f32,
    projected: f32,
) -> f32

/// Tasa de decay efectiva modulada por resonancia solar. Axioma 8.
/// Entidades resonantes con el sol decaen menos (fotosíntesis compensa).
/// Entidades disonantes decaen más.
pub fn frequency_aware_decay_rate(
    base_dissipation: f32,
    entity_freq: f32,
    solar_freq: f32,
    solar_bandwidth: f32,
    photosynthesis_efficiency: f32,
) -> f32
```

### 2. En `src/blueprint/constants/temporal_telescope.rs` (agregar al archivo existente)

```rust
/// Niveles máximos del stack. 16⁸ ≈ 4.3×10⁹ ticks alcanzables.
pub const MAX_LEVELS: usize = 8;

/// Longitud de coherencia por defecto (ticks). Calibración.
/// Determina qué tan rápido decae la visibilidad especulativa con la distancia al ancla.
pub const DEFAULT_COHERENCE_LENGTH: f32 = 100.0;
```

---

## Contrato stateless

Todas las funciones reciben valores escalares y retornan f32. Sin estado. Sin side effects. Sin ECS. Sin Bevy.

---

## Preguntas para tests

### speculative_visibility
1. `speculative_visibility(0, 100.0)` → ¿V ≈ 0.0? (en el ancla, certeza total)
2. `speculative_visibility(1000000, 100.0)` → ¿V ≈ 1.0? (muy lejos, onda pura)
3. `speculative_visibility(100, 100.0)` → ¿V entre 0.3 y 0.9? (punto medio)
4. V siempre en [0.0, 1.0] para cualquier input
5. D² + V² ≤ 1.0 (invariante de Englert) para inputs variados
6. `speculative_visibility(50, 0.0)` → ¿V = 1.0? (coherence_length=0 → nada confiable)

### conservation_bounded_project
7. `conservation_bounded_project(100.0, 90.0, 95.0)` → ¿95.0? (dentro de rango, sin clamp)
8. `conservation_bounded_project(100.0, 90.0, 120.0)` → ¿100.0? (clamped arriba: Axioma 5)
9. `conservation_bounded_project(100.0, 90.0, 80.0)` → ¿90.0? (clamped abajo: mínimo es base_decay)
10. Nunca retorna > current_qe (property test con 1000 inputs aleatorios)
11. Nunca retorna < base_decay (property test)
12. Con base_decay > current_qe → ¿retorna current_qe? (edge case: decay excesivo)

### frequency_aware_decay_rate
13. Entidad con freq = SOLAR_FREQUENCY → ¿effective_dissipation < base? (resonante, subsidiada)
14. Entidad con freq lejos del sol → ¿effective_dissipation ≈ base? (sin subsidio)
15. Resultado siempre ≥ 0.0 (nunca negativo)
16. Resultado siempre ≤ base_dissipation (fotosíntesis solo reduce dissipation, no la invierte)
17. `photosynthesis_efficiency=0` → ¿resultado = base? (sin fotosíntesis)
18. Resonancia = 1.0, efficiency = 1.0 → ¿resultado = 0.0? (perfect subsidy)

---

## Integración

- **Consume:** `gaussian_frequency_alignment` (de `determinism.rs`, existente), constantes solares (de `batch/constants.rs`)
- **Consumido por:** MT-2 (projection), MT-3 (stack — visibility), MT-5 (dashboard — visibility display)
- **Modifica:** `temporal_telescope.rs` (agrega funciones), `constants/temporal_telescope.rs` (agrega constantes)
- **No modifica:** Ningún otro archivo
