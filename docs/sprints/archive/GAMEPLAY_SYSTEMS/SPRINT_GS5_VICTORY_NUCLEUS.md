# Sprint GS-5 — Victory Nucleus: Condición de Victoria Física

**Modulo:** `src/simulation/game_loop.rs` (nuevo), `src/blueprint/equations/game_loop.rs` (nuevo), `src/blueprint/constants/game_loop.rs` (nuevo)
**Tipo:** Ecuaciones puras + componentes marcadores + sistema de victoria.
**Onda:** 0 — Bloquea GS-6 (mapa), GS-7 (visual), GS-8 (arquetipos).
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe

**Lo que SÍ existe:**

- `worldgen/EnergyNucleus` — componente que genera el campo energético. Ya existe, ya tiene `qe` (via BaseEnergy).
- `simulation/states.rs` — `GameState::Playing`, `PlayState::Active`. El estado finito ya está.
- `simulation/post.rs::faction_identity_system` — rastrea qué facción es cada entidad. Corre en `Phase::MetabolicLayer`.
- `layers/identity.rs` — `Faction` componente. Ya existe.
- `world/Scoreboard` — Resource de puntuación. Ya existe.
- `layers/coherence.rs` — `MatterCoherence` con `structural_damage` field. La degradación estructural ya existe.

**Lo que NO existe:**

1. **Marca de objetivo de victoria.** `EnergyNucleus` no tiene flag "es el núcleo de victoria del equipo X".
2. **Sistema de check de victoria.** Nada verifica si `qe(nucleus_B) < QE_MIN`.
3. **`VictoryEvent`**. No hay evento de fin de partida.
4. **Degradación de intake por daño.** El núcleo no reduce su intake cuando tiene daño estructural.
5. **`PlayState::Victory`**. El estado de fin de juego no existe en el enum.

---

## Objetivo

Hacer que la partida termine cuando el núcleo enemigo es colapsado — no como regla, sino como verificación de un estado físico. El núcleo ya existe en la simulación; GS-5 lo convierte en el objetivo final conectando física → condición de victoria.

```
victory(team_A) ↔ qe(nucleus_B) < QE_MIN_EXISTENCE
```

---

## Responsabilidades

### GS-5A: Ecuaciones de game loop

```rust
// src/blueprint/equations/game_loop.rs

/// Intake reducido por daño estructural. Simula que un núcleo dañado no puede absorber energía.
/// damage ∈ [0,1]. intake_base en qe/tick.
pub fn nucleus_effective_intake(intake_base: f32, structural_damage: f32) -> f32 {
    intake_base * (1.0 - structural_damage.clamp(0.0, 1.0)).max(0.0)
}

/// ¿Es el núcleo viable? Retorna false si debe activarse el check de victoria.
pub fn is_nucleus_viable(qe: f32, qe_min: f32) -> bool {
    qe > qe_min
}

/// Ventana de comeback: potencial de reversión basado en masa inercial baja.
/// masa_promedio ∈ (0, ∞). Resultado normalizado: más alto = más reversión potencial.
pub fn comeback_potential(mean_inertial_mass: f32) -> f32 {
    if mean_inertial_mass <= 0.0 { return 0.0; }
    1.0 / mean_inertial_mass
}

/// Ventaja de energy: positivo = team_A va ganando.
pub fn energy_advantage(total_qe_a: f32, total_qe_b: f32) -> f32 {
    total_qe_a - total_qe_b
}
```

### GS-5B: Tipos y marcadores

```rust
// src/simulation/game_loop.rs

/// Marca un EnergyNucleus como objetivo de victoria de su facción.
/// SparseSet: sólo 1-2 entidades en toda la partida lo tienen.
/// Comparte Faction con el núcleo (ya existente en worldgen).
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct VictoryNucleus {
    /// Si es true: la destrucción de este núcleo termina el juego.
    pub is_final_target: bool,
    /// Intake base en qe/tick cuando el núcleo está intacto.
    pub base_intake_qe: f32,
}

/// Evento de victoria. Emitido una vez cuando se detecta la condición.
/// INV-6: vive 1 tick, drenado por GameStatePlugin.
#[derive(Event, Debug, Clone)]
pub struct VictoryEvent {
    pub winner_faction: Faction,
    pub loser_nucleus: Entity,
    pub tick_id: u64,
}

/// Resource de estado de victoria. Singleton.
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct GameOutcome {
    pub winner: Option<Faction>,
    pub winning_tick: Option<u64>,
}
```

### GS-5C: Sistemas

```rust
/// Reduce el intake del núcleo basado en su daño estructural.
/// Phase::ThermodynamicLayer — antes de los cálculos de energía.
pub fn nucleus_intake_decay_system(
    mut query: Query<(&mut AlchemicalEngine, &MatterCoherence), With<VictoryNucleus>>,
) {
    for (mut engine, coherence) in &mut query {
        let effective = game_loop_eq::nucleus_effective_intake(
            engine.base_intake(),
            coherence.structural_damage(),
        );
        if engine.intake() != effective {
            engine.set_intake(effective);
        }
    }
}

/// Verifica condición de victoria: núcleo enemigo bajo umbral.
/// Phase::MetabolicLayer — después de metabolic_stress_death_system.
pub fn victory_check_system(
    nuclei: Query<(Entity, &BaseEnergy, &Faction, &VictoryNucleus)>,
    clock: Res<SimulationClock>,
    mut outcome: ResMut<GameOutcome>,
    mut events: EventWriter<VictoryEvent>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    if outcome.winner.is_some() { return; }  // ya hay ganador

    for (entity, energy, faction, nucleus) in &nuclei {
        if !nucleus.is_final_target { continue; }
        if !game_loop_eq::is_nucleus_viable(energy.qe(), QE_NUCLEUS_VIABILITY_THRESHOLD) {
            let winner = faction.opponent();  // helper en Faction
            outcome.winner = Some(winner);
            outcome.winning_tick = Some(clock.tick_id);
            events.send(VictoryEvent { winner_faction: winner, loser_nucleus: entity, tick_id: clock.tick_id });
            next_state.set(PlayState::Victory);
        }
    }
}
```

### GS-5D: Constantes

```rust
// src/blueprint/constants/game_loop.rs

/// qe mínimo para que un VictoryNucleus se considere viable.
/// Diferente de QE_MIN_EXISTENCE (más alto — el núcleo muere antes que una entidad normal).
pub const QE_NUCLEUS_VIABILITY_THRESHOLD: f32 = 100.0;

/// Intake base del núcleo cuando no hay daño. En qe/tick.
pub const NUCLEUS_BASE_INTAKE_QE_PER_TICK: f32 = 50.0;

/// Factor de snowball: qué tan rápido se amplifica la ventaja de energía en intake.
pub const SNOWBALL_INTAKE_SCALING: f32 = 0.01;
```

### GS-5E: PlayState extension

```rust
// src/simulation/states.rs — agregar variante:
pub enum PlayState {
    Active,
    Paused,
    Victory,   // ← nuevo: juego terminado, simulación congelada
}
```

---

## Tacticas

- **EnergyNucleus ya existe** — GS-5 sólo le agrega `VictoryNucleus` como componente marcador (SparseSet). Sin tocar el worldgen.
- **Faction::opponent()** — helper puro, sin estado. Facción B es la oponente de A en un 1v1.
- **Un sistema, una transformación.** `nucleus_intake_decay` ≠ `victory_check`. Separados.
- **GameOutcome como fuente de verdad.** No verificar en múltiples lugares; sólo `victory_check_system` escribe `GameOutcome`.

---

## NO hace

- No define qué entidades atacan el núcleo — eso es la física existente (catalysis, predation, collision).
- No implementa animación de victoria — eso es resonance-app (renderer).
- No agrega nodos de mapa — eso es GS-6.
- No cambia el sistema de spawn de EnergyNucleus en worldgen.

---

## Dependencias

- `worldgen/EnergyNucleus` — entidad existente a la que se le agrega `VictoryNucleus`.
- `layers/identity.rs::Faction` — facción del núcleo (ya existente).
- `layers/coherence.rs::MatterCoherence` — `structural_damage` (ya existente).
- `layers/engine.rs::AlchemicalEngine` — intake del núcleo (ya existente).
- `simulation/states.rs` — `PlayState` (modificado para agregar `Victory`).
- `blueprint/constants/QE_MIN_EXISTENCE` — umbral mínimo de existencia.

---

## Criterios de aceptacion

### GS-5A (Ecuaciones)
- `nucleus_effective_intake(100.0, 0.0)` → `100.0`.
- `nucleus_effective_intake(100.0, 0.5)` → `50.0`.
- `nucleus_effective_intake(100.0, 1.0)` → `0.0`.
- `is_nucleus_viable(101.0, 100.0)` → `true`.
- `is_nucleus_viable(50.0, 100.0)` → `false`.
- `comeback_potential(0.1)` → mayor que `comeback_potential(10.0)`.
- Test determinismo: mismas entradas → mismo resultado.

### GS-5B/C (Tipos + Sistemas)
- Test (MinimalPlugins): spawnar VictoryNucleus con qe=0 → `VictoryEvent` emitido en el siguiente tick.
- Test: spawnar VictoryNucleus con qe=1000 → no event.
- Test: `PlayState` transiciona a `Victory` cuando nucleus colapsa.
- Test: `GameOutcome.winner` no se sobreescribe si ya está seteado.
- Test: `nucleus_intake_decay_system` con `structural_damage=0.5` → intake reducido a 50%.

### General
- `cargo test --lib` sin regresión.
- `VictoryNucleus` SparseSet.

---

## Referencias

- `src/worldgen/` — `EnergyNucleus` (entidad existente)
- `src/simulation/states.rs` — `GameState`, `PlayState`
- `src/layers/identity.rs` — `Faction`
- `src/layers/coherence.rs` — `MatterCoherence`
- Blueprint §6: "Energy Nucleus as Victory Target"
- `docs/design/BLUEPRINT.md` — QE_MIN_EXISTENCE
