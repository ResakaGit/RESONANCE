# Sprint GS-3 — Nash AI Targeting: Selección Óptima de Objetivo

**Modulo:** `src/blueprint/equations/tactical_ai.rs` (nuevo), `src/simulation/behavior.rs` (extensión), `src/blueprint/constants/tactical_ai.rs` (nuevo)
**Tipo:** Ecuaciones puras + extensión de BehaviorMode.
**Onda:** 0 — Bloquea GS-4 (pack dynamics).
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe

**Lo que SÍ existe:**

- `simulation/behavior.rs` — D1 Behavioral Intelligence completo:
  - `BehaviorMode` enum: `Idle | Hunt { prey: Entity } | Flee { threat: Entity } | Eat { target: Entity } | Forage`
  - `BehaviorSet::Assess` → `BehaviorSet::Decide` — pipeline de dos fases
  - `EnergyAssessment`, `SensoryAwareness` — observables del tick actual
  - `BehaviorCooldown` — anti-oscilación
- `layers/identity.rs::Faction` — equipo de la entidad.
- `world/SpatialIndex` — query_radius disponible.
- `blueprint/equations/` — `resonance_factor`, `extraction_rate_at_distance` (en energy_competition).
- `layers/energy_pool.rs::PoolParentLink` — tipo de extracción ya clasificado.

**Lo que NO existe:**

1. **Nash focus target.** D1 elige presa más cercana, no presa óptima (min qe / extraction_resistance).
2. **Team-scope awareness.** Las decisiones son puramente individuales. No hay razonamiento sobre el equipo.
3. **Extraction resistance.** No hay función que calcule cuánto cuesta extraerle qe a un objetivo dado.
4. **Resonance-weighted targeting.** El bonus de `cos²(Δfreq/2)` no se usa en selección de objetivo.
5. `BehaviorMode::FocusFire` — variante que coordina concentrar fuego.

---

## Objetivo

Extender D1 con selección de objetivo Nash-optimal: el objetivo es el que maximiza la extracción efectiva del equipo, ponderando por frecuencia (resonance_factor) y resistencia (qe / damage_rate). El BehaviorMode refleja la decisión; la física la ejecuta.

```
focus_target = argmin_j(qe(b_j) / effective_extraction(a_i → b_j))
effective_extraction(a→b) = extraction_capacity(a) × resonance_factor(freq_a, freq_b)
```

---

## Responsabilidades

### GS-3A: Ecuaciones de targeting táctico

```rust
// src/blueprint/equations/tactical_ai.rs

/// Factor de resonancia para extracción: máximo cuando frecuencias coinciden.
/// Reutiliza la ecuación del competition blueprint.
/// res ∈ [0, 1]. 1.0 = frecuencias idénticas (máxima eficiencia).
pub fn resonance_factor(freq_a_hz: f32, freq_b_hz: f32) -> f32 {
    let delta = (freq_a_hz - freq_b_hz).abs();
    // cos²(Δf / 2) normalizado por la banda máxima de diferencia
    let normalized = (delta / FREQ_BAND_MAX_HZ).min(1.0);
    (normalized * std::f32::consts::FRAC_PI_2).cos().powi(2)
}

/// Extracción efectiva de a sobre b: capacidad ponderada por resonancia.
pub fn effective_extraction(
    extraction_capacity: f32,
    freq_a_hz: f32,
    freq_b_hz: f32,
) -> f32 {
    extraction_capacity * resonance_factor(freq_a_hz, freq_b_hz)
}

/// Resistencia a la extracción: qe / tasa de daño esperada.
/// Menor = más fácil de matar. Usado para focus fire.
pub fn extraction_resistance(qe: f32, effective_ext: f32) -> f32 {
    if effective_ext <= 0.0 { return f32::MAX; }
    qe / effective_ext
}

/// Amenaza recibida: suma vectorial de fuerzas de extracción entrantes.
/// Retorna magnitud escalar. Inputs: vec de (pos_enemy, pos_self, ext_capacity).
pub fn threat_magnitude(
    self_pos: [f32; 2],
    enemies: &[([f32; 2], f32)],  // (pos, extraction_capacity) por enemigo
    sensory_radius: f32,
) -> f32 {
    enemies.iter()
        .filter(|(pos, _)| {
            let dx = pos[0] - self_pos[0];
            let dy = pos[1] - self_pos[1];
            (dx * dx + dy * dy).sqrt() <= sensory_radius
        })
        .map(|(_, cap)| cap)
        .sum()
}

/// Gradiente de amenaza: dirección desde la que viene el mayor peligro.
/// Retorna vector normalizado [x, z] o [0,0] si no hay amenazas.
pub fn threat_gradient(
    self_pos: [f32; 2],
    enemies: &[([f32; 2], f32)],  // (pos, extraction_capacity)
) -> [f32; 2] {
    let mut gx = 0.0f32;
    let mut gz = 0.0f32;
    for (pos, cap) in enemies {
        let dx = pos[0] - self_pos[0];
        let dz = pos[1] - self_pos[1];
        let dist = (dx * dx + dz * dz).sqrt().max(0.001);
        gx += (dx / dist) * cap;
        gz += (dz / dist) * cap;
    }
    let mag = (gx * gx + gz * gz).sqrt();
    if mag < 0.001 { [0.0, 0.0] } else { [gx / mag, gz / mag] }
}
```

### GS-3B: Extensión de BehaviorMode

```rust
// src/simulation/behavior.rs — agregar variante a BehaviorMode:

pub enum BehaviorMode {
    Idle,
    Hunt      { prey: Entity },
    Flee      { threat: Entity },
    Eat       { target: Entity },
    Forage,
    FocusFire { target: Entity, team_priority: u8 },  // ← nuevo GS-3
    Regroup   { rally_pos: Vec2 },                      // ← nuevo GS-4
}
```

### GS-3C: Sistema Nash target select (BehaviorSet::Decide)

```rust
/// Selecciona el objetivo Nash-optimal para entidades en modo Hunt/FocusFire.
/// Extiende la lógica de `behavior_decide_system` existente.
/// Phase::Input, in_set(BehaviorSet::Decide), after D1 existing systems.
pub fn nash_target_select_system(
    mut agents: Query<(
        Entity,
        &Faction,
        &OscillatorySignature,
        &InferenceProfile,
        &SensoryAwareness,
        &mut BehaviorMode,
    ), With<BehavioralAgent>>,
    targets: Query<(Entity, &BaseEnergy, &OscillatorySignature, &Faction)>,
    spatial: Res<SpatialIndex>,
    config: Res<NashTargetConfig>,
) {
    for (entity, faction, osc, profile, awareness, mut mode) in &mut agents {
        // Solo reasigna si está en modo de combate
        let in_combat = matches!(*mode, BehaviorMode::Hunt { .. } | BehaviorMode::FocusFire { .. });
        if !in_combat { continue; }

        let Some(best_target) = find_nash_target(
            entity, faction, osc, awareness,
            &targets, &config,
        ) else { continue; };

        let new_mode = BehaviorMode::FocusFire { target: best_target, team_priority: 0 };
        if *mode != new_mode {
            *mode = new_mode;
        }
    }
}

fn find_nash_target(
    self_entity: Entity,
    self_faction: &Faction,
    self_osc: &OscillatorySignature,
    awareness: &SensoryAwareness,
    targets: &Query<(Entity, &BaseEnergy, &OscillatorySignature, &Faction)>,
    config: &NashTargetConfig,
) -> Option<Entity> {
    // Filtrar: enemigos en rango, vivos
    // Calcular extraction_resistance para cada uno
    // Retornar argmin(resistance)
    let mut best = None::<(Entity, f32)>;

    for &candidate in awareness.detected_entities.iter() {
        let Ok((entity, energy, osc, faction)) = targets.get(candidate) else { continue; };
        if faction == self_faction { continue; }  // aliado
        if energy.qe() <= 0.0 { continue; }       // muerto

        let eff_ext = tactical_ai_eq::effective_extraction(
            config.base_extraction_capacity,
            self_osc.frequency_hz(),
            osc.frequency_hz(),
        );
        let resistance = tactical_ai_eq::extraction_resistance(energy.qe(), eff_ext);

        match best {
            None => { best = Some((entity, resistance)); }
            Some((_, best_res)) if resistance < best_res => { best = Some((entity, resistance)); }
            _ => {}
        }
    }
    best.map(|(e, _)| e)
}
```

### GS-3D: Config y constantes

```rust
// src/blueprint/constants/tactical_ai.rs

/// Banda máxima de frecuencia para normalización de resonance_factor.
pub const FREQ_BAND_MAX_HZ: f32 = 1100.0;   // Lux band upper bound
/// Capacidad de extracción base (override por InferenceProfile).
pub const BASE_EXTRACTION_CAPACITY: f32 = 10.0;
/// Threshold de amenaza para transición a Flee.
pub const FLEE_THREAT_THRESHOLD: f32 = 30.0;
/// Threshold mínimo de qe para iniciar combate (no atacar si propio qe muy bajo).
pub const HUNT_MINIMUM_OWN_QE: f32 = 50.0;

// src/simulation/behavior.rs — nuevo Resource
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct NashTargetConfig {
    pub base_extraction_capacity: f32,
    pub flee_threshold: f32,
    pub hunt_min_own_qe: f32,
}
```

---

## Tacticas

- **D1 existente no se rompe.** `nash_target_select_system` sólo se ejecuta si el modo ya es `Hunt` o `FocusFire` — no sobreescribe `Flee`, `Eat`, `Forage`.
- **Ecuaciones puras, sin ECS.** `resonance_factor`, `effective_extraction`, `extraction_resistance` no tocan `World`.
- **`SensoryAwareness.detected_entities`** — ya existe, ya tiene los candidatos. No se re-scannea.
- **Sin HashMap.** `detected_entities` es `Vec<Entity>` con iteración lineal. OK para N < 20 en rango sensorial.

---

## NO hace

- No implementa formación de manada — eso es GS-4.
- No implementa AI de equipo coordinada — GS-4 agrega cohesión, GS-3 sólo el targeting individual.
- No cambia el sistema de movement — el BehaviorMode es leído por `locomotion_system`.
- No agrega comunicación entre agentes — las decisiones son locales con información pública.

---

## Dependencias

- `simulation/behavior.rs` — D1 BehaviorMode, BehavioralAgent, BehaviorSet (modificados/extendidos).
- `layers/oscillatory.rs` — `OscillatorySignature::frequency_hz()`.
- `layers/energy.rs` — `BaseEnergy::qe()`.
- `layers/identity.rs` — `Faction`.
- `layers/inference.rs` — `SensoryAwareness`, `InferenceProfile`.
- `world/SpatialIndex` — query_radius para detección de targets.

---

## Criterios de aceptacion

### GS-3A (Ecuaciones)
- `resonance_factor(440.0, 440.0)` → `1.0` (misma frecuencia).
- `resonance_factor(440.0, 880.0)` → menor que `resonance_factor(440.0, 450.0)`.
- `resonance_factor(0.0, 1100.0)` → `0.0` (bandas opuestas).
- `effective_extraction(10.0, 440.0, 440.0)` → `10.0`.
- `extraction_resistance(100.0, 10.0)` → `10.0`.
- `extraction_resistance(100.0, 0.0)` → `f32::MAX`.
- `threat_gradient` con un solo enemigo → vector apunta hacia el enemigo.
- `threat_gradient` sin enemigos → `[0.0, 0.0]`.
- Determinismo: mismas entradas → mismas salidas.

### GS-3C (Sistema)
- Test (MinimalPlugins + D1): agente en `Hunt` con dos targets de distinta frecuencia → elige el de mayor resonancia.
- Test: agente en `Flee` → no cambia de modo (GS-3 no sobreescribe).
- Test: target con qe=0 → no es seleccionado.

### General
- `cargo test --lib` sin regresión.
- No `HashMap` en hot path.

---

## Referencias

- `src/simulation/behavior.rs` — BehaviorMode, BehaviorSet, BehavioralAgent
- `src/layers/inference.rs` — SensoryAwareness
- Blueprint §5: "Nash Equilibrium Problem", "Resonance Factor"
- `src/blueprint/equations/energy_competition/` — `resonance_factor` base (posible reutilización)
