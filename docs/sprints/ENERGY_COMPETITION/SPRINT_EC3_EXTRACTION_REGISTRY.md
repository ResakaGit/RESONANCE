# Sprint EC-3 — Extraction Registry: Composición Funcional de Extracción

**Módulo:** `src/blueprint/equations/energy_competition/extraction.rs`
**Tipo:** Funciones puras + enum composable. Higher-Order Function layer sin ECS.
**Onda:** A — Requiere EC-1 (funciones base). Paralelo con EC-2.
**Estado:** ⏳ Pendiente

## Objetivo

Definir el **registro de funciones de extracción** como un sistema composable: las 5 funciones primitivas (EC-1B) se combinan con **modificadores** para producir fenotipos de comportamiento. La composición es mediante enum stack, no closures ni trait objects. Evaluable como función pura: `evaluate_extraction(ctx) -> f32`.

## Principio

> El fenotipo no se almacena — es la función compuesta evaluada. `proportional + stress_response = opportunistic_generalist`. La composición es funcional; el resultado es un número.

## Responsabilidades

### EC-3A: Contexto de Extracción

```rust
/// Contexto inmutable pasado a toda función de extracción.
/// Contiene todo lo necesario para decidir cuánto extraer.
#[derive(Clone, Copy, Debug)]
pub struct ExtractionContext {
    /// Energía disponible en el pool padre post-disipación.
    pub available: f32,
    /// Ratio pool/capacity del padre.
    pub pool_ratio: f32,
    /// Número de hermanos (incluyéndose).
    pub n_siblings: u32,
    /// Fitness total de todos los hermanos.
    pub total_fitness: f32,
}
```

- Stack-only, `Copy`, sin referencias.
- El sistema (EC-4) construye este contexto por cada pool padre, una vez por tick.

### EC-3B: Modificadores de Extracción

```rust
/// Modificadores que alteran el resultado de la función base.
/// Composición por stack de hasta 4 modificadores (DOD: max 4).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum ExtractionModifier {
    /// Bajo estrés (pool_ratio < threshold), extrae más.
    StressResponse { threshold: f32, multiplier: f32 },
    /// Si pool_ratio < min_viable, no extrae nada.
    ThresholdGated { min_viable: f32 },
    /// Escala la extracción por un factor fijo.
    ScaleFactor { factor: f32 },
    /// Clamp máximo de extracción por tick.
    CapPerTick { max_per_tick: f32 },
}
```

- Enum cerrado. 4 variantes — extensible a más si el diseño lo pide (exhaustive match).
- Parámetros inline en la variante. Sin allocations.
- NO closures, NO `Box<dyn Fn>`, NO trait objects.

### EC-3C: Extraction Profile (composición)

```rust
/// Perfil completo de extracción: base + hasta MAX_MODIFIERS modificadores.
/// Es un "fenotipo funcional" evaluable como pura.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct ExtractionProfile {
    /// Función base (una de las 5 primitivas).
    pub base: ExtractionType,
    /// Parámetro primario de la función base.
    pub primary_param: f32,
    /// Stack de modificadores aplicados en orden.
    pub modifiers: [Option<ExtractionModifier>; MAX_EXTRACTION_MODIFIERS],
}
```

- `MAX_EXTRACTION_MODIFIERS = 4` (constante en `constants/`).
- `[Option<ExtractionModifier>; 4]` — stack-only, no `ArrayVec` necesario.
- `Copy` derivable porque `ExtractionModifier` es `Copy`.

### EC-3D: Función de Evaluación

```rust
/// Evalúa la extracción completa: base + modificadores, en orden.
pub fn evaluate_extraction(profile: &ExtractionProfile, ctx: &ExtractionContext) -> f32
```

1. Evaluar función base según `profile.base`:
   ```
   Proportional → extract_proportional(ctx.available, ctx.n_siblings)
   Greedy       → extract_greedy(ctx.available, profile.primary_param)
   Competitive  → extract_competitive(ctx.available, profile.primary_param, ctx.total_fitness)
   Aggressive   → extract_aggressive(ctx.available, profile.primary_param, DAMAGE_RATE_DEFAULT).0
   Regulated    → extract_regulated(ctx.available, ctx.pool_ratio, profile.primary_param,
                                     REGULATED_THRESHOLD_LOW_DEFAULT, REGULATED_THRESHOLD_HIGH_DEFAULT)
   ```
2. Aplicar modificadores en orden (fold):
   ```
   for modifier in profile.modifiers.iter().flatten() {
       result = apply_modifier(result, modifier, ctx);
   }
   ```
3. Clamp final: `result.clamp(0.0, ctx.available)`.

```rust
/// Aplica un modificador a un resultado de extracción.
fn apply_modifier(base_result: f32, modifier: &ExtractionModifier, ctx: &ExtractionContext) -> f32
```

- `StressResponse`: `if ctx.pool_ratio < threshold { result * multiplier } else { result }`.
- `ThresholdGated`: `if ctx.pool_ratio < min_viable { 0.0 } else { result }`.
- `ScaleFactor`: `result * factor.max(0.0)`.
- `CapPerTick`: `result.min(max_per_tick.max(0.0))`.

### EC-3E: Aggressive Extraction — Pool Damage

```rust
/// Evalúa extracción agresiva con su componente de daño al pool.
/// Retorna (taken, pool_damage). El sistema aplica pool_damage a la capacidad del padre.
pub fn evaluate_aggressive_extraction(
    profile: &ExtractionProfile,
    ctx: &ExtractionContext,
    damage_rate: f32,
) -> (f32, f32)
```

- Solo aplica cuando `profile.base == ExtractionType::Aggressive`.
- `taken` pasa por modificadores normalmente.
- `pool_damage = taken * damage_rate.clamp(0.0, 1.0)`.
- Los modificadores NO afectan `pool_damage` — solo `taken`.

### EC-3F: Fenotipos Predefinidos (factories)

```rust
/// Generalista oportunista: proporcional + stress response.
pub fn opportunistic_generalist() -> ExtractionProfile;

/// Especialista conservador: greedy + threshold gated.
pub fn conservative_specialist(capacity: f32, min_viable: f32) -> ExtractionProfile;

/// Parásito adaptativo: aggressive + cap per tick.
pub fn adaptive_parasite(aggression: f32, max_drain: f32) -> ExtractionProfile;

/// Homeostático resiliente: regulated + stress response.
pub fn resilient_homeostatic(base_rate: f32) -> ExtractionProfile;

/// Depredador apex: greedy + scale factor alto, sin daño al pool.
pub fn apex_predator(capacity: f32) -> ExtractionProfile;
```

- Factories — retornan `ExtractionProfile` stack-allocated.
- Solo combinan primitivas existentes. No añaden lógica nueva.
- Documentadas con `///` que explica el fenotipo emergente.

### EC-3G: Constantes

```rust
pub const MAX_EXTRACTION_MODIFIERS: usize = 4;
```

## Tácticas

- **Enum dispatch, no dynamic dispatch.** `match profile.base { ... }` en `evaluate_extraction`. El compilador genera un jump table. Sin vtable, sin indirección.
- **Fold determinista.** Modificadores se aplican en orden del array. Mismo perfil + mismo contexto = mismo resultado, siempre.
- **No almacenar `ExtractionProfile` como componente (v1).** El sistema (EC-4) construye el perfil desde `ExtractionType` + `primary_param` de `PoolParentLink`. Los modificadores vienen de constantes o de un componente auxiliar futuro. Esto mantiene EC-2 limpio.
- **La función de daño es separada.** `evaluate_extraction` retorna solo `f32`. El daño al pool es un side-channel que el sistema maneja aparte. Mantiene la función principal pura y simple.

## NO hace

- No crea componentes ECS (eso es EC-2).
- No crea sistemas (eso es EC-4).
- No implementa "memoria" (history-based modifiers). Eso es extensión post-v1.
- No implementa migración (re-parenting). Extensión post-v1.
- No modifica funciones de EC-1 — las consume.

## Criterios de aceptación

### EC-3A (Contexto)
- Test: `ExtractionContext` es `Copy`.
- Test: `size_of::<ExtractionContext>()` = 16 bytes (4 × f32).

### EC-3B (Modificadores)
- Test: `ExtractionModifier` es `Copy`.
- Test: todas las variantes distinguibles por `PartialEq`.

### EC-3C (Profile)
- Test: `ExtractionProfile` es `Copy`.
- Test: `size_of::<ExtractionProfile>()` razonable (< 128 bytes).
- Test: perfil con 0 modificadores evalúa igual que función base.
- Test: perfil con 4 modificadores evalúa correctamente en orden.

### EC-3D (evaluate_extraction)
- Test: `Proportional` puro: `evaluate_extraction(proportional_profile, ctx_4siblings)` = `available / 4`.
- Test: `Greedy` + `CapPerTick(200)`: nunca excede 200 sin importar available.
- Test: `Competitive` + `StressResponse(0.3, 1.5)`: bajo estrés extrae 50% más.
- Test: `Regulated` + `ThresholdGated(0.1)`: pool_ratio < 0.1 → extracción = 0.
- Test: resultado siempre en `[0, available]`.
- Test: determinismo — 100 evaluaciones idénticas.

### EC-3E (Aggressive damage)
- Test: `evaluate_aggressive_extraction(aggressive_profile, ctx, 0.1)` → `(taken, taken*0.1)`.
- Test: modificadores afectan `taken` pero no `pool_damage` rate.

### EC-3F (Fenotipos)
- Test: cada factory retorna un `ExtractionProfile` válido.
- Test: `opportunistic_generalist` bajo estrés extrae más que sin estrés.
- Test: `conservative_specialist` con pool_ratio < min_viable → extrae 0.
- Test: `apex_predator` extrae más que `conservative_specialist` con misma available.

### General
- `cargo test --lib` sin regresión.
- >=25 tests unitarios.
- Todas las funciones puras, sin ECS, sin side effects.

## Referencias

- Blueprint Energy Competition Layer §2 (Extraction Functions), §5 (Behavioral Composition)
- `src/blueprint/equations/metabolic_graph/writer_monad.rs` — Precedente de composición pura
- EC-1 (funciones base que esta sprint compone)
- EC-2 (`ExtractionType` enum que esta sprint consume)
