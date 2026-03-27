# Sprint ET-8 — Dynamic Coalitions: Alianzas como Equilibrio de Nash

**Módulo:** `src/simulation/emergence/coalitions.rs` (nuevo), `src/blueprint/equations/emergence/coalitions.rs` (nuevo)
**Tipo:** Ecuaciones puras + sistema de formación/ruptura + evento.
**Tier:** T2-4. **Onda:** B.
**BridgeKind:** `CoalitionBridge` — cache **Large(512)** FxHashMap, clave `hash(sorted_member_ids)`. **CRÍTICO.**
**Estado:** ✅ Implementado (2026-03-25)

---

## Contexto: qué ya existe

- ET-5 `SymbiosisLink` — par a par. Las coaliciones generalizan esto a N > 2.
- ET-3 `CulturalMemory` — los grupos culturales son el canal de formación preferente.
- `layers/social_communication.rs::PackMembership` — grupos de hasta 8. Las coaliciones pueden cruzar packs.
- `blueprint/equations/emergence/symbiosis.rs::is_symbiosis_stable` — Nash para par. Coalitions extiende a N-tuplas.

**Lo que NO existe:**
1. Coalición N > 2 con estabilidad Nash colectiva.
2. `CoalitionRegistry` Resource — índice de coaliciones activas.
3. Distribución de beneficio colectivo entre miembros.
4. Evento de ruptura/formación (`CoalitionChangedEvent`).

---

## Objetivo

Una coalición es estable si ningún miembro gana más saliendo. La condición de Nash N-way:
`∀ i: intake(i | coalition) ≥ intake(i | coalition \ {i})`

El check es O(n²) sobre pares — la ecuación más costosa del track ET. El `CoalitionBridge` **Large(512)** es la respuesta de rendimiento.

```
stability(C) = min over i of [intake(i|C) - intake(i alone)]
defection(i) = max over C' ⊂ C of [intake(i|C') - intake(i|C)]
```

---

## Responsabilidades

### ET-8A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/coalitions.rs

/// Estabilidad global de la coalición: mínima ganancia individual de pertenecer.
/// intake_with: qe/tick de cada miembro en la coalición.
/// intake_without: qe/tick de cada miembro si saliera.
pub fn coalition_stability(
    intake_with: &[f32],
    intake_without: &[f32],
) -> f32 {
    intake_with.iter().zip(intake_without.iter())
        .map(|(w, wo)| w - wo)
        .fold(f32::MAX, f32::min)
}

/// Incentivo de deserción: ganancia neta de un miembro al irse a otra coalición.
pub fn defection_incentive(
    intake_current: f32,
    intake_alternative: f32,
    switching_cost: f32,
) -> f32 {
    (intake_alternative - intake_current - switching_cost).max(0.0)
}

/// Bonus de intake por tamaño de coalición (economía de escala).
/// Satura logarítmicamente: más miembros, menos incremento marginal.
pub fn coalition_intake_bonus(base_intake: f32, member_count: u8, scale_factor: f32) -> f32 {
    let scale = (1.0 + (member_count as f32).ln() * scale_factor).min(MAX_COALITION_BONUS);
    base_intake * scale
}

/// Tamaño óptimo de coalición dado el costo de coordinación por miembro.
pub fn optimal_coalition_size(
    marginal_benefit_per_member: f32,
    coordination_cost_per_member: f32,
) -> u8 {
    if coordination_cost_per_member <= 0.0 { return MAX_COALITION_MEMBERS; }
    let optimal = (marginal_benefit_per_member / coordination_cost_per_member).ceil() as u8;
    optimal.clamp(2, MAX_COALITION_MEMBERS)
}

/// Distribución equitativa de beneficio colectivo (Shapley simplificado: 1/n).
pub fn shapley_share(total_benefit: f32, member_count: u8) -> f32 {
    if member_count == 0 { return 0.0; }
    total_benefit / member_count as f32
}
```

### ET-8B: Tipos

```rust
// src/simulation/emergence/coalitions.rs

/// Miembro de una coalición activa.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]  // coaliciones son transientes
pub struct CoalitionMember {
    pub coalition_id:   u32,   // hash estable de la coalición
    pub role:           u8,    // 0=member, 1=leader (elected by max qe)
    pub join_tick:      u64,   // cuándo se unió
    pub coordination_cost: f32,// qe/tick que cuesta estar en coalición
}

/// Resource: índice de todas las coaliciones activas.
#[derive(Resource, Default, Debug)]
pub struct CoalitionRegistry {
    pub entries: Vec<CoalitionEntry>,  // sorted by coalition_id
}

#[derive(Debug, Clone)]
pub struct CoalitionEntry {
    pub coalition_id:  u32,
    pub member_ids:    [u32; MAX_COALITION_MEMBERS as usize],
    pub member_count:  u8,
    pub stability:     f32,   // último valor calculado
    pub formed_tick:   u64,
}

pub const MAX_COALITION_MEMBERS: u8 = 8;

/// Evento de cambio en composición de coalición.
#[derive(Event, Debug, Clone)]
pub struct CoalitionChangedEvent {
    pub coalition_id: u32,
    pub change_type:  CoalitionChange,
    pub entity:       Entity,
    pub tick_id:      u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum CoalitionChange { Formed, Joined, Left, Dissolved }
```

### ET-8C: Sistemas

```rust
/// Evalúa estabilidad Nash de coaliciones activas y expulsa desertores.
/// Phase::MetabolicLayer — after symbiosis_effect_system.
/// ⚠ CACHE CRÍTICO: usa CoalitionBridge Large(512) para evitar O(n²) cada tick.
pub fn coalition_stability_system(
    mut members: Query<(Entity, &WorldEntityId, &mut CoalitionMember, &mut BaseEnergy, &AlchemicalEngine)>,
    mut registry: ResMut<CoalitionRegistry>,
    mut cache: ResMut<BridgeCache<CoalitionBridge>>,
    mut events: EventWriter<CoalitionChangedEvent>,
    clock: Res<SimulationClock>,
    config: Res<CoalitionConfig>,
) {
    // Sólo re-evaluar coaliciones cada COALITION_EVAL_INTERVAL ticks
    if clock.tick_id % config.eval_interval as u64 != 0 { return; }

    for entry in registry.entries.iter_mut() {
        let cache_key = entry.coalition_id;

        // Intentar hit de caché primero
        if let Some(cached) = cache.get(cache_key) {
            entry.stability = cached;
            continue;
        }

        // Cache miss: calcular estabilidad
        let mut intake_with   = [0.0f32; MAX_COALITION_MEMBERS as usize];
        let mut intake_without = [0.0f32; MAX_COALITION_MEMBERS as usize];

        for (i, mid) in entry.member_ids[..entry.member_count as usize].iter().enumerate() {
            if let Some((_, _, _, energy, engine)) = members.iter()
                .find(|(_, id, _, _, _)| id.0 == *mid)
            {
                let bonus = coalition_eq::coalition_intake_bonus(
                    engine.base_intake(), entry.member_count, config.scale_factor,
                );
                intake_with[i]   = bonus;
                intake_without[i] = engine.base_intake();  // sin coalición
                // Restar costo de coordinación
                intake_with[i] -= energy.qe() * config.coordination_cost_rate;
            }
        }

        let stability = coalition_eq::coalition_stability(
            &intake_with[..entry.member_count as usize],
            &intake_without[..entry.member_count as usize],
        );
        entry.stability = stability;
        cache.insert(cache_key, stability);

        // Disolver si estabilidad negativa
        if stability < 0.0 {
            events.send(CoalitionChangedEvent {
                coalition_id: entry.coalition_id,
                change_type: CoalitionChange::Dissolved,
                entity: Entity::PLACEHOLDER,
                tick_id: clock.tick_id,
            });
        }
    }
}

/// Aplica bonus de intake colectivo a miembros de coaliciones estables.
/// Phase::MetabolicLayer — after coalition_stability_system.
pub fn coalition_intake_bonus_system(
    mut members: Query<(&CoalitionMember, &mut AlchemicalEngine)>,
    registry: Res<CoalitionRegistry>,
    config: Res<CoalitionConfig>,
) {
    for (member, mut engine) in &mut members {
        let Some(entry) = registry.entries.iter()
            .find(|e| e.coalition_id == member.coalition_id) else { continue };
        if entry.stability < 0.0 { continue; }

        let boosted = coalition_eq::coalition_intake_bonus(
            engine.base_intake(), entry.member_count, config.scale_factor,
        );
        let net = (boosted - member.coordination_cost).max(engine.base_intake() * 0.5);
        if engine.intake() != net { engine.set_intake(net); }
    }
}
```

### ET-8D: Constantes y BridgeKind

```rust
pub struct CoalitionBridge;
impl BridgeKind for CoalitionBridge {}
// Large(512): O(n²) Nash check cacheado por coalition_id. Invalida en CoalitionChangedEvent.
// Hit rate esperado: ~85% (coaliciones estables por EVAL_INTERVAL ticks)

pub const COALITION_EVAL_INTERVAL: u8 = 10;    // re-evalúa cada 10 ticks
pub const COALITION_DEFAULT_SCALE_FACTOR: f32 = 0.15;  // +15% por cada ln(n) miembros
pub const COALITION_COORDINATION_COST_RATE: f32 = 0.02; // 2% qe/tick por estar en coalición
pub const MAX_COALITION_BONUS: f32 = 2.5;       // tope 2.5× intake base
```

---

## Tacticas

- **`Large(512)` FxHashMap es mandatorio.** Nash pairwise es O(n²). Con 50 coaliciones de 8 miembros = 1600 pares × frecuencia de tick = CPU death. Cache con key `coalition_id` (hash de miembros ordenados) + invalidación en `CoalitionChangedEvent` → hit rate ~85%.
- **Evaluación lazy.** `COALITION_EVAL_INTERVAL = 10` ticks. Las coaliciones son estructuras lentas — no necesitan reevaluación cada tick. 10× speedup inmediato.
- **`CoalitionMember` como SparseSet.** La mayoría de entidades NO están en coalición. SparseSet → iteración sobre subset sin pagar el costo del componente vacío.
- **Invalidación por evento, no por tick.** Cuando `CoalitionChangedEvent` se emite, el sistema de invalidación borra el cache entry. Sin staleness.
- **Shapley como default de distribución.** División equitativa 1/n es el baseline — coaliciones pueden implementar distribuciones asimétricas vía `role`.

---

## NO hace

- No forma coaliciones automáticamente — eso es lógica de `BehaviorMode::FormCoalition` (GS-3).
- No implementa votación o liderazgo complejo — `role: u8` es suficiente para líder/miembro.
- No persiste coaliciones entre sesiones — `CoalitionRegistry` se vacía en cada nueva partida.

---

## Dependencias

- ET-5 `SymbiosisLink` — las relaciones parasíticas pueden existir dentro de coaliciones (tensión interna).
- ET-3 `CulturalMemory` — grupos culturales tienen `imitation_radius` que facilita formación de coalición.
- `layers/social_communication.rs::PackMembership` — canal de invitación preferente.
- `bridge/` — `BridgeCache<CoalitionBridge>` registrado en `EmergenceTier2Plugin`.

---

## Criterios de Aceptación

### ET-8A
- `coalition_stability(&[10.0, 10.0], &[5.0, 5.0])` → `5.0`.
- `coalition_stability(&[10.0, 3.0], &[5.0, 5.0])` → `-2.0` (un miembro pierde).
- `defection_incentive(10.0, 15.0, 2.0)` → `3.0`.
- `defection_incentive(10.0, 8.0, 1.0)` → `0.0` (no hay incentivo).
- `coalition_intake_bonus(100.0, 4, 0.15)` → `~120.8` (+20.8% por ln(4)×0.15).
- `optimal_coalition_size(5.0, 1.0)` → `5`.

### ET-8C
- Test: coalición con stability > 0 → todos los miembros reciben intake bonus.
- Test: coalición con stability < 0 → `CoalitionChangedEvent::Dissolved` emitido.
- Test: `CoalitionChangedEvent` invalida cache → próximo tick recalcula.
- Test: entidad sin `CoalitionMember` → sin modificación de intake.

### General
- `cargo test --lib` sin regresión. `CoalitionBridge` usa `CacheBackend::Large(512)` explícitamente.

---

## Referencias

- ET-5 Symbiosis — Nash para pares (fundación)
- ET-3 Cultural Transmission — canal de formación
- `src/bridge/` — `BridgeCache<B>` + `CacheBackend`
- Blueprint §T2-4: "Dynamic Coalitions", Nash equilibrium N-way
