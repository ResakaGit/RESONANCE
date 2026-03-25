# Sprint MG-1 — Ecuaciones Termodinámicas Fundamentales

**Módulo:** `src/blueprint/equations/` + `src/blueprint/constants/`
**Tipo:** Funciones puras sin ECS. Fundamento matemático de todo el track.
**Onda:** 0 — Bloqueante para todos los demás sprints.
**Estado:** ⏳ Pendiente

## Objetivo

Implementar las 8 ecuaciones puras que formalizan los límites termodinámicos (Carnot, entropía, exergía), el costo de forma (Constructal/Myring), el albedo inferido (Stefan-Boltzmann + Newton), la rugosidad de superficie (ley cuadrático-cúbica) y el costo de transporte vascular (Hagen-Poiseuille). Todas van en `equations.rs` como funciones `pub fn` sin dependencias ECS.

## Responsabilidades

### MG-1A: Ecuaciones de Límite Termodinámico

- `carnot_efficiency(t_core: f32, t_env: f32) -> f32`
  - `η_max = 1.0 - t_env / t_core`
  - Guard: si `t_core <= t_env` → retorna `0.0` (no puede extraer trabajo).
  - Guard: `t_core = max(t_core, EPSILON)` para evitar div/0.
  - Rango retorno: `[0.0, 1.0)`.

- `entropy_production(q_diss: f32, t_core: f32) -> f32`
  - `S_gen = q_diss / max(t_core, EPSILON)`
  - Unidades: qe/K equivalente (no SI estricto — consistente con `equivalent_temperature`).
  - Guard: `q_diss < 0` → clamp a 0 (la entropía no se destruye).

- `exergy_balance(j_in: f32, efficiency: f32, activation_energy: f32) -> f32`
  - `Ex = j_in * efficiency - activation_energy`
  - Si resultado < 0 → retorna 0 (nodo no puede operar, se apaga).
  - `efficiency` ya clamped por Carnot antes de llamar.

- `heat_capacity(qe: f32, specific_heat_factor: f32) -> f32`
  - `C_v = qe * specific_heat_factor`
  - Permite `dT = dQ / C_v`. Sin C_v, un quantum de calor cambiaría T arbitrariamente.
  - `specific_heat_factor` ∈ `constants.rs`.

### MG-1B: Ecuaciones de Forma y Transporte

- `shape_cost(medium_density: f32, velocity: f32, drag_coeff: f32, projected_area: f32, vascular_cost: f32) -> f32`
  - `C = 0.5 * ρ * v² * C_D * A_proj + C_vasc`
  - Constructal: el organismo minimiza C cambiando forma.
  - Guard: `velocity = 0` → C = C_vasc (sin arrastre, solo costo interno).

- `vascular_transport_cost(viscosity: f32, length: f32, radius: f32) -> f32`
  - `C_t = viscosity * length³ / max(radius⁴, EPSILON)`
  - Hagen-Poiseuille simplificado. Justifica branching fractal.
  - Guard: `radius → 0` causa costo → ∞ (clamped por EPSILON en r⁴).

- `inferred_drag_coefficient(length: f32, max_diameter: f32) -> f32`
  - Myring body: `fineness = length / max(max_diameter, EPSILON)`.
  - `C_D = DRAG_COEFF_BASE / (1.0 + DRAG_FINENESS_SCALE * fineness²)`.
  - Fineness alto (largo y delgado) → C_D bajo → forma eficiente.
  - Fineness 1 (esfera) → C_D alto → arrastre máximo.
  - Rango: `[DRAG_COEFF_MIN, DRAG_COEFF_BASE]`.

### MG-1C: Ecuación de Albedo

- `inferred_albedo(q_metabolic: f32, solar_irradiance: f32, proj_area: f32, emissivity: f32, t_core: f32, t_env: f32, surf_area: f32, convection_coeff: f32) -> f32`
  - Balance radiativo de superficie:
    `Q_met + (1 - α) * I * A_proj = ε * σ * (T⁴_core - T⁴_env) * A_surf + h * (T_core - T_env) * A_surf`
  - Despejando α:
    `α = 1.0 - (Q_dissipable - Q_met) / max(I * A_proj, EPSILON)`
    donde `Q_dissipable = ε * σ * (T⁴_core - T⁴_env) * A_surf + h * ΔT * A_surf`
  - Clamp: `α ∈ [ALBEDO_MIN, ALBEDO_MAX]` (0.05, 0.95).
  - Si `I * A_proj ≈ 0` (sin sol) → α = 0.5 (fallback neutral).

### MG-1D: Ecuación de Rugosidad

- `inferred_surface_rugosity(q_total: f32, volume: f32, t_core: f32, t_env: f32, convection_coeff: f32) -> f32`
  - Necesidad de superficie: `A_needed = q_total / max(h * (t_core - t_env), EPSILON)`.
  - Superficie de esfera equivalente: `A_sphere = 4π * (3V / 4π)^(2/3)`.
  - `rugosity = (A_needed / max(A_sphere, EPSILON)).clamp(RUGOSITY_MIN, RUGOSITY_MAX)`.
  - `rugosity = 1.0` → esfera lisa. `rugosity = 4.0` → superficie 4× mayor que esfera.
  - Fenotipos: 1.0–1.5 = liso; 1.5–2.5 = pliegues; 2.5–4.0 = aletas/radiadores.

### MG-1E: Constantes (constants.rs)

```rust
// --- Morfogénesis: Límites Termodinámicos ---
pub const STEFAN_BOLTZMANN: f32 = 5.67e-8;          // σ (escalado al modelo)
pub const DEFAULT_EMISSIVITY: f32 = 0.9;             // ε (cuerpo gris)
pub const DEFAULT_CONVECTION_COEFF: f32 = 10.0;      // h (convección natural)
pub const SPECIFIC_HEAT_FACTOR: f32 = 0.01;          // C_v por unidad de qe

// --- Morfogénesis: Forma ---
pub const DRAG_COEFF_BASE: f32 = 0.47;              // C_D de esfera
pub const DRAG_COEFF_MIN: f32 = 0.04;               // C_D mínimo (torpedo)
pub const DRAG_FINENESS_SCALE: f32 = 0.15;          // Sensibilidad al fineness
pub const SHAPE_OPTIMIZER_DAMPING: f32 = 0.3;        // Factor de amortiguación
pub const SHAPE_OPTIMIZER_MAX_ITER: u32 = 3;         // Iteraciones por frame

// --- Morfogénesis: Albedo ---
pub const ALBEDO_MIN: f32 = 0.05;                    // Negro casi absoluto
pub const ALBEDO_MAX: f32 = 0.95;                    // Blanco casi absoluto
pub const ALBEDO_FALLBACK: f32 = 0.5;                // Sin sol → neutro

// --- Morfogénesis: Rugosidad ---
pub const RUGOSITY_MIN: f32 = 1.0;                   // Esfera lisa
pub const RUGOSITY_MAX: f32 = 4.0;                   // Máxima superficie extra
```

## Tácticas

- **Reusar `equivalent_temperature` como referencia.** Las ecuaciones siguen el mismo patrón: escalar, puro, clamped, documentado.
- **STEFAN_BOLTZMANN escalado.** En el mundo de Resonance, las temperaturas son equivalentes (`density / k_boltzmann`). σ se escala para que el balance radiativo produzca valores razonables con T ∈ [100, 5000].
- **No optimizar prematuramente.** Estas funciones son O(1) aritmética. No necesitan BridgeCache. El cache se añade en MG-4/MG-5 cuando se llaman N veces por frame.
- **Test-driven.** Escribir los tests ANTES de la implementación. Los rangos físicos son el contrato.

## NO hace

- No crea componentes ECS (eso es MG-2).
- No crea sistemas (eso es MG-3+).
- No modifica ecuaciones existentes (`thermal_transfer`, `drag_force`, etc. permanecen intactas).
- No toca el pipeline de simulación.
- No introduce dependencias de crates nuevos.

## Criterios de aceptación

### MG-1A
- Test: `carnot_efficiency(500.0, 300.0) ≈ 0.4`.
- Test: `carnot_efficiency(300.0, 300.0) = 0.0` (sin gradiente).
- Test: `carnot_efficiency(300.0, 500.0) = 0.0` (T_env > T_core).
- Test: `entropy_production(100.0, 500.0) = 0.2`.
- Test: `entropy_production(-5.0, 500.0) = 0.0` (clamp negativo).
- Test: `exergy_balance(100.0, 0.5, 10.0) = 40.0`.
- Test: `exergy_balance(10.0, 0.5, 10.0) = 0.0` (no puede operar).

### MG-1B
- Test: `shape_cost` con velocity=0 → solo vascular_cost.
- Test: `shape_cost` crece cuadráticamente con velocity.
- Test: `vascular_transport_cost` crece con length³, decrece con radius⁴.
- Test: `inferred_drag_coefficient(10.0, 2.0)` < `inferred_drag_coefficient(2.0, 2.0)` (fusiforme < esfera).
- Test: `inferred_drag_coefficient` ∈ [DRAG_COEFF_MIN, DRAG_COEFF_BASE] para todo input positivo.

### MG-1C
- Test: alto Q_metabolic + alto I_solar → α cerca de ALBEDO_MAX (criatura caliente en desierto → blanca).
- Test: bajo Q_metabolic + bajo I_solar → α cerca de ALBEDO_MIN (criatura fría en cueva → oscura).
- Test: I_solar = 0 → α = ALBEDO_FALLBACK.
- Test: α siempre ∈ [ALBEDO_MIN, ALBEDO_MAX].

### MG-1D
- Test: `inferred_surface_rugosity` con bajo Q → rugosity ≈ 1.0 (lisa).
- Test: `inferred_surface_rugosity` con alto Q, bajo V → rugosity > 2.0 (necesita aletas).
- Test: rugosity siempre ∈ [RUGOSITY_MIN, RUGOSITY_MAX].
- Test: ΔT → 0 → rugosity = RUGOSITY_MAX (no puede disipar → máxima superficie).

### General
- `cargo test --lib` pasa sin regresión en tests existentes.
- Todas las funciones tienen `///` doc-comments con la fórmula.

## Referencias

- `src/blueprint/equations.rs` — `equivalent_temperature()`, `thermal_transfer()`, `drag_force()` como patrones
- `src/blueprint/constants/` — estructura existente de constantes por dominio (`mod.rs` + shards)
- `docs/design/MORPHOGENESIS.md` §3.2, §3.3
- Bejan (1997) — Ley Constructal: shape_cost minimization
- Myring (1976) — Body of revolution: C_D vs fineness ratio
