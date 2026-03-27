# Sprint: Evolution & Group Behavior

**Objetivo:** Cerrar los gaps para que la simulación produzca evolución real y comportamiento grupal emergente.

**Axiomas respetados:** 1 (energía), 3 (competencia), 4 (disipación), 6 (emergencia a escala), 8 (oscilación).
**Principios:** TDD, stateless equations, DoD (max 4 fields), no code duplication, HOFs where applicable.

---

## Estado actual

### Reproducción
- Flora: **funcional** — `reproduction_spawn_system` hereda InferenceProfile con mutación (`mutate_bias`).
- Fauna: **no se reproduce** — no hay spawn de offspring para MOVE entities.
- `mobility_bias` **no muta** — se copia directo del parent. Forma corporal no evoluciona.

### Emergence Tiers

| Tier | ID | Módulo | Component | Equations | System | Gap |
|------|----|--------|-----------|-----------|--------|-----|
| T1 | ET-2 | Theory of Mind | ✅ | ✅ | ❌ | System falta |
| T1 | ET-3 | Cultural Transmission | ✅ | ✅ | ✅ | Completo |
| T2 | ET-5 | Symbiosis | ✅ | ✅ | ❌ | System falta |
| T2 | ET-6 | Epigenetics | ✅ | ✅ | ❌ | System falta |
| T2 | ET-7 | Senescence | ✅ | ✅ | ⚠️ | Parcial |
| T2 | ET-8 | Coalitions | ✅ | ✅ | ⚠️ | Solo stability eval, no crea coaliciones |
| T2 | ET-9 | Niche | ✅ | ✅ | ❌ | System falta |

**Entrainment (AC-2)** y **Cooperation (AC-5)** están funcionales.

---

## Tareas

### EV-1: Fauna reproduction (evolución de animales)

**Problema:** Solo flora se reproduce. Fauna muere sin descendencia.

**Solución:** Generalizar `reproduction_spawn_system` para MOVE entities.

**Ecuación pura** (`blueprint/equations/`):
```
can_reproduce_fauna(qe, satiation, age) -> bool
  = qe > FAUNA_REPRODUCTION_QE_MIN
    && satiation > FAUNA_REPRODUCTION_SATIATION_MIN
    && age > FAUNA_REPRODUCTION_MIN_AGE
```

**Cambios:**
- `simulation/reproduction/mod.rs`: branch para fauna (MOVE + REPRODUCE caps)
- Offspring hereda InferenceProfile completo incluyendo `mobility_bias` con mutación
- Offspring hereda `CapabilitySet` del parent
- Offspring tiene velocidad inicial = 0 (nace estacionario)
- Fauna offspring recibe: L0-L5, L7 (will), L6 (ambient del parent), behavior stack

**Constantes** (`blueprint/constants/`):
```
FAUNA_REPRODUCTION_QE_MIN: f32 = 200.0
FAUNA_REPRODUCTION_SATIATION_MIN: f32 = 0.6
FAUNA_REPRODUCTION_MIN_AGE: u64 = 500  // ticks
FAUNA_SEED_ENERGY_FRACTION: f32 = 0.3
FAUNA_OFFSPRING_INITIAL_RADIUS: f32 = 0.2
```

**Tests:**
- `fauna_does_not_reproduce_below_threshold`
- `fauna_offspring_inherits_mutated_profile`
- `fauna_offspring_has_move_capability`
- `mobility_bias_mutates_across_generations`

---

### EV-2: Mutation de mobility_bias (forma evoluciona)

**Problema:** `mobility_bias` se copia sin mutar. La forma corporal (constructal) no evoluciona.

**Solución:** Mutar todos los campos de InferenceProfile, incluyendo mobility_bias.

**Cambio:** 1 línea en `reproduction_spawn_system`:
```rust
// Antes:
profile.mobility_bias,
// Después:
equations::mutate_bias(profile.mobility_bias, d_mobility, constants::MUTATION_MAX_DRIFT),
```

Donde `d_mobility` se calcula igual que los otros drifts (deterministic hash del entity).

**Tests:**
- `mobility_bias_drifts_across_generations`
- `mutation_preserves_valid_range` (0.0..1.0)

---

### EV-3: Theory of Mind system (ET-2)

**Problema:** `OtherModelSet` component + equations existen. No hay system que los actualice.

**Solución:** System stateless que lee vecinos y actualiza predicciones.

**Ecuación pura** (ya existe):
```
update_prediction(old_pred, observed, accuracy) -> f32
model_maintenance_cost(accuracy, distance) -> f32
is_model_worth_maintaining(benefit, cost) -> bool
```

**System** (`simulation/emergence/theory_of_mind.rs`):
```rust
pub fn theory_of_mind_update_system(
    mut query: Query<(&mut OtherModelSet, &Transform, &BaseEnergy), With<BehavioralAgent>>,
    targets: Query<(&Transform, &OscillatorySignature, &BaseEnergy)>,
    spatial: Res<SpatialIndex>,
) {
    // For each agent: query nearby entities via SpatialIndex
    // For each neighbor within model range:
    //   if slot available: create new model (predicted_freq, accuracy=0.1)
    //   if existing model: update_prediction(old, observed, accuracy)
    //   debit maintenance cost from qe
    //   evict worst model if is_model_worth_maintaining returns false
}
```

**Phase:** `Phase::Input` (before behavior decisions)

**Tests:**
- `theory_of_mind_creates_model_for_nearby_entity`
- `theory_of_mind_updates_prediction_on_observation`
- `theory_of_mind_evicts_low_value_model`
- `theory_of_mind_costs_qe`

---

### EV-4: Symbiosis effect system (ET-5)

**Problema:** `SymbiosisLink` component + equations existen. No hay system.

**Ecuación pura** (ya existe):
```
mutualism_benefit(qe_a, qe_b, bonus_factor) -> f32
parasitism_drain(qe_host, drain_rate) -> f32
is_symbiosis_stable(benefit_a, benefit_b, cost) -> bool
```

**System** (`simulation/emergence/symbiosis.rs` — new system, not component file):
```rust
pub fn symbiosis_effect_system(
    mut query: Query<(&SymbiosisLink, &mut BaseEnergy)>,
    partner_qe: Query<&BaseEnergy>,
) {
    // For each entity with SymbiosisLink:
    //   read partner's qe
    //   compute benefit/drain from equations
    //   apply to both entities (guard change detection)
    //   if !is_symbiosis_stable → remove SymbiosisLink via commands
}
```

**Phase:** `Phase::MetabolicLayer`

**Tests:**
- `mutualism_increases_both_entities_qe`
- `parasitism_drains_host`
- `unstable_symbiosis_link_removed`

---

### EV-5: Epigenetic adaptation system (ET-6)

**Problema:** `EpigeneticState` component + equations existen. No hay system.

**Ecuación pura** (ya existe):
```
should_express_gene(env_signal, threshold, mask) -> bool
effective_phenotype(base, expression_mask) -> f32
silencing_cost(mask_complexity) -> f32
```

**System:**
```rust
pub fn epigenetic_adaptation_system(
    mut query: Query<(&mut EpigeneticState, &InferenceProfile, &AmbientPressure, &mut BaseEnergy)>,
) {
    // Read environment (AmbientPressure as proxy)
    // Update expression_mask based on environmental signals
    // Debit silencing_cost from qe
    // effective_phenotype modifies how InferenceProfile is read downstream
}
```

**Phase:** `Phase::MorphologicalLayer` (after albedo, before constructal)

**Tests:**
- `cold_environment_suppresses_growth_gene`
- `expression_cost_drains_qe`
- `mask_reverts_when_environment_changes`

---

### EV-6: Niche adaptation system (ET-9)

**Problema:** `NicheProfile` component + equations existen. No hay system.

**Ecuación pura** (ya existe):
```
niche_overlap(a, b) -> f32
competitive_pressure(overlap, population) -> f32
character_displacement(overlap, pressure) -> Vec4
```

**System:**
```rust
pub fn niche_adaptation_system(
    mut query: Query<(&mut NicheProfile, &Transform, &OscillatorySignature)>,
    spatial: Res<SpatialIndex>,
) {
    // For each entity with NicheProfile:
    //   query nearby competitors
    //   compute niche_overlap with each
    //   if overlap > threshold: character_displacement shifts niche center
    //   update displacement_rate, specialization
}
```

**Phase:** `Phase::MorphologicalLayer` (after constructal, before abiogenesis)

**Tests:**
- `overlapping_niches_displace`
- `isolated_entity_niche_stable`
- `displacement_respects_niche_width_bounds`

---

## Orden de implementación

```
EV-2 (mobility mutates)      → 1 línea. Desbloquea evolución de forma.
EV-1 (fauna reproduction)    → ~80 líneas. Desbloquea evolución de fauna.
EV-3 (theory of mind)        → ~60 líneas. Desbloquea T1 completo.
EV-4 (symbiosis)             → ~40 líneas. Desbloquea interacciones obligadas.
EV-5 (epigenetics)           → ~50 líneas. Desbloquea plasticidad fenotípica.
EV-6 (niche adaptation)      → ~50 líneas. Desbloquea diversificación ecológica.
```

**Total: ~280 líneas de sistemas nuevos.** Las ecuaciones ya existen. Los componentes ya existen. Solo falta wiring.

---

## Invariantes del sprint

1. **Axioma 4:** Toda reproducción drena qe del parent. No se crea energía.
2. **Axioma 5:** `Σ qe(parent) + Σ qe(offspring) ≤ Σ qe(parent_before)`. Conservación.
3. **Axioma 6:** Comportamiento grupal emerge de interacciones individuales. Ningún system programa grupos top-down.
4. **Axioma 8:** Herencia usa frecuencia como identidad. Offspring hereda OscillatorySignature del parent (con posible drift).
5. **DoD:** Todos los systems ≤ 5 component types en query. Math en equations/. Constantes en constants/.
6. **TDD:** Cada system tiene ≥ 3 tests antes de implementar.
7. **Stateless:** Ecuaciones no acceden ECS. Systems no almacenan estado entre ticks (excepto `Local<>` documentado).
8. **HOF:** Donde aplique, usar closures/iterators sobre loops imperativos.
9. **No duplicación:** Reusar `mutate_bias`, `reproduction_spawn_system` generalizado, `SpatialIndex::query_radius`.

---

## Criterio de cierre

- [ ] `cargo test --lib` pasa sin regresiones
- [ ] Fauna se reproduce, offspring hereda profile mutado
- [ ] mobility_bias muta → forma corporal evoluciona entre generaciones
- [ ] Theory of Mind actualiza modelos de vecinos
- [ ] Symbiosis aplica drain/benefit entre pares
- [ ] Epigenetics modula expresión por ambiente
- [ ] Niche displacement separa competidores
- [ ] Demo: `RESONANCE_MAP=demo_animal cargo run` muestra reproducción + mutación visible en 200 ticks
