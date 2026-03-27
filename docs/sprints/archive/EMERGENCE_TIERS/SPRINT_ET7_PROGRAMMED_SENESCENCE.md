# Sprint ET-7 — Programmed Senescence: Mortalidad Intrínseca por Edad

**Módulo:** `src/layers/senescence.rs` (nuevo), `src/blueprint/equations/emergence/senescence.rs` (nuevo)
**Tipo:** Nueva capa + ecuaciones puras.
**Tier:** T2-3. **Onda:** 0.
**BridgeKind:** `SenescenceBridge` — cache Small(32), LUT por banda de edad + coeficiente.
**Estado:** ✅ Implementado (2026-03-25)

---

## Objetivo

La tasa de disipación no es constante — aumenta con la edad independientemente del daño externo. La mortalidad intrínseca crea recambio generacional que acelera la evolución. La estrategia de historia de vida (reprodución temprana vs. longevidad) emerge del trade-off energético.

```
dissipation_rate(t) = dissipation_base × (1 + senescence_coeff × tick_age)
P(survival, t) = exp(-∫₀ᵗ dissipation_rate(s) ds)
```

---

## Responsabilidades

### ET-7A: Ecuaciones

```rust
// src/blueprint/equations/emergence/senescence.rs

/// Tasa de disipación dependiente de la edad.
pub fn age_dependent_dissipation(
    base_dissipation: f32,
    tick_age: u64,
    senescence_coeff: f32,
) -> f32 {
    base_dissipation * (1.0 + senescence_coeff * tick_age as f32)
}

/// Probabilidad de sobrevivir hasta la edad t dada la tasa de senescencia.
/// Aproximación discreta de la integral continua.
pub fn survival_probability(tick_age: u64, base_dissipation: f32, senescence_coeff: f32) -> f32 {
    let integrated = base_dissipation * tick_age as f32
        + 0.5 * base_dissipation * senescence_coeff * (tick_age as f32).powi(2);
    (-integrated).exp().clamp(0.0, 1.0)
}

/// Estrategia óptima de reproducción: semelparidad vs. iteroparidad.
/// env_variance: varianza de qe en el entorno. Alta varianza → más hijos pequeños.
/// offspring_survival_rate: [0,1] probabilidad de que un hijo sobreviva.
pub fn optimal_reproduction_strategy(
    env_variance: f32,
    offspring_survival_rate: f32,
) -> ReproductionStrategy {
    // Alta varianza o baja supervivencia → iteroparidad (muchos hijos, poca inversión)
    if env_variance > 0.5 || offspring_survival_rate < 0.3 {
        ReproductionStrategy::Iteroparous
    } else {
        ReproductionStrategy::Semelparous
    }
}

/// Presión kin-selection: valor de un acto de ayuda a un pariente.
/// relatedness: [0,1] parentesco genético. benefit_to_kin: qe ganado por el pariente.
pub fn kin_selection_value(relatedness: f32, benefit_to_kin: f32, cost_to_self: f32) -> f32 {
    relatedness * benefit_to_kin - cost_to_self
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReproductionStrategy { Iteroparous, Semelparous }
```

### ET-7B: Componente

```rust
// src/layers/senescence.rs

/// Capa T2-3: SenescenceProfile — parámetros de mortalidad intrínseca.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct SenescenceProfile {
    pub tick_birth:       u64,    // tick_id en que nació (set en spawn)
    pub senescence_coeff: f32,    // velocidad de envejecimiento
    pub max_viable_age:   u64,    // ticks máximos antes de colapso garantizado
    pub strategy:         u8,     // 0=Iteroparous, 1=Semelparous (u8 no String)
}

impl SenescenceProfile {
    pub fn age(&self, current_tick: u64) -> u64 {
        current_tick.saturating_sub(self.tick_birth)
    }
}
```

### ET-7C: Sistema

```rust
/// Aplica senescencia: incrementa disipación con la edad.
/// Phase::MetabolicLayer — integrado en metabolic_stress_system.
pub fn senescence_dissipation_system(
    mut agents: Query<(&SenescenceProfile, &mut BaseEnergy)>,
    clock: Res<SimulationClock>,
    config: Res<SenescenceConfig>,
) {
    for (profile, mut energy) in &mut agents {
        let age = profile.age(clock.tick_id);
        let extra_diss = senescence_eq::age_dependent_dissipation(
            0.0,   // el base_dissipation lo maneja el sistema metabólico existente
            age,
            profile.senescence_coeff,
        ) - 0.0;  // sólo el incremento por edad

        // Drenamos el incremento sobre la disipación base
        let age_drain = config.base_drain_per_tick * age as f32 * profile.senescence_coeff;
        let new_qe = (energy.qe() - age_drain).max(0.0);
        if energy.qe() != new_qe { energy.set_qe(new_qe); }
    }
}
```

### ET-7D: Constantes y BridgeKind (LUT)

```rust
pub struct SenescenceBridge;
impl BridgeKind for SenescenceBridge {}

// LUT: bandas de edad precalculadas — evita exp() cada tick
pub const SENESCENCE_AGE_BANDS: [u64; 8] = [0, 100, 300, 600, 1000, 2000, 5000, u64::MAX];
pub const SENESCENCE_DEFAULT_COEFF: f32 = 0.0001;  // lento — 10k ticks de vida media
pub const SENESCENCE_BASE_DRAIN_PER_TICK: f32 = 0.0001;
```

---

## Tacticas

- **LUT de bandas de edad.** `SenescenceBridge` usa `band_index_of(age, SENESCENCE_AGE_BANDS)` como clave. La función de supervivencia es suave → misma banda → mismo resultado. Evita exp() cada tick.
- **Separado del sistema metabólico.** `senescence_dissipation_system` agrega el drain incremental por edad; el sistema metabólico existente maneja el base dissipation. Sin duplicación.
- **`strategy: u8` no enum en componente.** Hard Block 7 compliance. El enum existe en blueprint/equations para la lógica; el componente guarda el discriminante como u8.

---

## Criterios de Aceptación

- `age_dependent_dissipation(1.0, 0, 0.0001)` → `1.0`.
- `age_dependent_dissipation(1.0, 10000, 0.0001)` → `2.0`.
- `survival_probability(0, 0.01, 0.0001)` → `1.0`.
- `survival_probability(10000, 0.01, 0.0001)` → `< 0.5`.
- `kin_selection_value(0.5, 10.0, 3.0)` → `2.0`.
- Test: entidad joven → drain pequeño. Entidad vieja → drain mayor.
- Test: entidad con `tick_age > max_viable_age` → qe → 0 en pocos ticks.
- `cargo test --lib` sin regresión.

---

## Referencias

- `src/simulation/metabolic/mod.rs` — sistema metabólico base (senescence lo extiende)
- `src/blueprint/equations/calibration.rs` — patrón LUT existente
- Blueprint §T2-3: "Programmed Senescence", life history equations
