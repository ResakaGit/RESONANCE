# Sprint GS-9 — Onboarding: Secuencia de Experiencia Física

**Modulo:** `src/plugins/onboarding_plugin.rs` (nuevo), `src/world/demos/onboarding/` (nuevo), `src/blueprint/constants/onboarding.rs` (nuevo)
**Tipo:** Plugin de secuenciación + demos pre-configurados + UI mínima de guía.
**Onda:** C — Requiere GS-5 (victoria), GS-7 (visual contract), GS-8 (arquetipos).
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe

**Lo que SÍ existe:**

- `world/demos/` — demos proceduales: `demo_level.rs`, `competition_arena.rs`, etc. Patrón existente.
- `simulation/states.rs` — `GameState { MainMenu, Playing, Credits }`, `PlayState { Active, Paused, Victory }`. Estados del juego.
- `runtime_platform/hud.rs` — HUD overlay. Puede mostrar texto de guía.
- `rendering/quantized_color/` + GS-7 `VisualHints` — visual ya legible.
- GS-8 `ArchetypeRegistry` — personajes cargados desde RON.
- GS-5 `VictoryEvent` — evento de fin de experiencia.
- `world/SpatialIndex` + `EnergyFieldGrid` — el mundo ya responde.

**Lo que NO existe:**

1. **Secuencia de onboarding.** No hay flujo guiado de experiencias ordenadas.
2. **Demos educativos.** Los demos existentes son de stress-test, no de aprendizaje.
3. **Checkpoints de comprensión.** No hay detección de "el jugador entendió X".
4. **UI de guía mínima.** No hay texto contextual vinculado al estado de la simulación.
5. **`GameState::Tutorial`.** El estado de tutorial no existe en el enum.

---

## Objetivo

Construir una secuencia de 5 escenas que construyen intuición física progresivamente. Cada escena es una situación emergente preconfigurada — no explicaciones, sino experiencias que el jugador vive. La secuencia termina cuando el jugador elige "jugar" desde el menú.

```
Escena 0: Un núcleo, sin enemigos. "¿Qué es la energía?"
Escena 1: Dos entidades. Una absorbe a la otra. "¿Qué es la resonancia?"
Escena 2: Pack de 3 vs 1 lento. "¿Qué es la cohesión?"
Escena 3: Dos núcleos, nodos de control. "¿Qué es el territorio?"
Escena 4: Partida corta completa. "¿Qué es la victoria?"
```

---

## Responsabilidades

### GS-9A: Estado Tutorial

```rust
// src/simulation/states.rs — agregar estado:
pub enum GameState {
    MainMenu,
    Tutorial,   // ← nuevo GS-9: onboarding sequence
    Playing,
    Credits,
}

// src/simulation/states.rs
pub enum TutorialState {
    Scene0_Nucleus,
    Scene1_Resonance,
    Scene2_Pack,
    Scene3_Territory,
    Scene4_Match,
    Complete,
}
```

### GS-9B: Escenas de onboarding

```rust
// src/world/demos/onboarding/scene0_nucleus.rs

/// Escena 0: Un núcleo solo, campo estable. Jugador observa cómo la energía pulsa.
/// Objetivo implícito: que el jugador note el pulso del núcleo (GS-7 visual).
pub fn setup_scene0_nucleus(
    mut commands: Commands,
    mut field: ResMut<EnergyFieldGrid>,
    registry: Res<ArchetypeRegistry>,
    clock: Res<SimulationClock>,
) {
    // Generar campo energético base — mediana densidad
    field.seed_uniform(SCENE0_FIELD_QE);

    // Spawnar un VictoryNucleus de facción A sin enemigos
    let nucleus_config = registry.get(NUCLEUS_ARCHETYPE_ID)
        .expect("nucleus archetype must be loaded");
    let nucleus = spawn_from_config(&mut commands, nucleus_config, Vec2::ZERO, &clock);
    commands.entity(nucleus).insert((
        VictoryNucleus { is_final_target: false, base_intake_qe: 30.0 },
        StateScoped(GameState::Tutorial),
    ));
}

// src/world/demos/onboarding/scene1_resonance.rs

/// Escena 1: Dos entidades con frecuencias distintas. Una con alta resonancia al núcleo, otra no.
/// Jugador observa: la que resuena absorbe más rápido.
pub fn setup_scene1_resonance(
    mut commands: Commands,
    registry: Res<ArchetypeRegistry>,
    clock: Res<SimulationClock>,
) {
    let resonant = registry.get(RESONANT_ARCHETYPE_ID).expect("resonant archetype");
    let dissonant = registry.get(DISSONANT_ARCHETYPE_ID).expect("dissonant archetype");

    for (config, pos) in [
        (resonant, Vec2::new(-5.0, 0.0)),
        (dissonant, Vec2::new(5.0, 0.0)),
    ] {
        let e = spawn_from_config(&mut commands, config, pos, &clock);
        commands.entity(e).insert(StateScoped(GameState::Tutorial));
    }
}
```

### GS-9C: Sistema de avance de secuencia

```rust
/// Verifica si la escena actual cumplió su condición de avance.
/// Phase::MetabolicLayer, in_set(TutorialSet::CheckAdvance).
pub fn tutorial_advance_check_system(
    state: Res<State<TutorialState>>,
    mut next_state: ResMut<NextState<TutorialState>>,
    victory_events: EventReader<VictoryEvent>,
    entities: Query<&BaseEnergy>,
    clock: Res<SimulationClock>,
    config: Res<OnboardingConfig>,
) {
    let advance = match state.get() {
        TutorialState::Scene0_Nucleus => {
            // Avanzar si jugador estuvo N ticks en la escena (observación pasiva)
            clock.tick_id >= config.scene0_duration_ticks
        }
        TutorialState::Scene1_Resonance => {
            // Avanzar cuando la entidad resonante tiene >2x qe que la disonante
            let qe_values: Vec<f32> = entities.iter().map(|e| e.qe()).collect();
            if qe_values.len() >= 2 {
                let max_qe = qe_values.iter().cloned().fold(f32::MIN, f32::max);
                let min_qe = qe_values.iter().cloned().fold(f32::MAX, f32::min);
                max_qe > min_qe * 2.0
            } else { false }
        }
        TutorialState::Scene2_Pack => {
            // Avanzar cuando el pack elimina al objetivo solitario
            entities.iter().filter(|e| e.qe() <= 0.0).count() >= 1
        }
        TutorialState::Scene3_Territory => {
            // Avanzar cuando el jugador controla al menos 1 nodo (leer NodeControlState)
            clock.tick_id >= config.scene3_min_ticks
        }
        TutorialState::Scene4_Match => {
            // Avanzar cuando VictoryEvent llega
            !victory_events.is_empty()
        }
        TutorialState::Complete => false,
    };

    if advance {
        let next = match state.get() {
            TutorialState::Scene0_Nucleus    => TutorialState::Scene1_Resonance,
            TutorialState::Scene1_Resonance  => TutorialState::Scene2_Pack,
            TutorialState::Scene2_Pack       => TutorialState::Scene3_Territory,
            TutorialState::Scene3_Territory  => TutorialState::Scene4_Match,
            TutorialState::Scene4_Match      => TutorialState::Complete,
            TutorialState::Complete          => TutorialState::Complete,
        };
        next_state.set(next);
    }
}

/// Limpia escena anterior al transicionar. StateScoped handle cleanup automático.
/// OnEnter(TutorialState::*) — re-setup de la nueva escena.
```

### GS-9D: Plugin de onboarding

```rust
// src/plugins/onboarding_plugin.rs

pub struct OnboardingPlugin;

impl Plugin for OnboardingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<TutorialState>()
            .add_systems(OnEnter(TutorialState::Scene0_Nucleus), setup_scene0_nucleus)
            .add_systems(OnEnter(TutorialState::Scene1_Resonance), setup_scene1_resonance)
            .add_systems(OnEnter(TutorialState::Scene2_Pack), setup_scene2_pack)
            .add_systems(OnEnter(TutorialState::Scene3_Territory), setup_scene3_territory)
            .add_systems(OnEnter(TutorialState::Scene4_Match), setup_scene4_match)
            .add_systems(
                FixedUpdate,
                tutorial_advance_check_system
                    .in_set(Phase::MetabolicLayer)
                    .run_if(in_state(GameState::Tutorial)),
            )
            .insert_resource(OnboardingConfig::default());
    }
}
```

### GS-9E: Constantes

```rust
// src/blueprint/constants/onboarding.rs

pub const SCENE0_FIELD_QE: f32 = 200.0;
pub const SCENE0_DURATION_TICKS: u64 = 200;   // 10 segundos a 20Hz
pub const SCENE3_MIN_TICKS: u64 = 300;         // mínimo tiempo en escena de territorio
pub const NUCLEUS_ARCHETYPE_ID: u32 = 100;
pub const RESONANT_ARCHETYPE_ID: u32 = 101;
pub const DISSONANT_ARCHETYPE_ID: u32 = 102;

#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct OnboardingConfig {
    pub scene0_duration_ticks: u64,
    pub scene3_min_ticks: u64,
    pub skip_enabled: bool,
}
```

---

## Tacticas

- **`StateScoped` para cleanup automático.** Todas las entidades de tutorial tienen `StateScoped(GameState::Tutorial)` — Bevy las despawnea automáticamente al salir del estado.
- **Condiciones emergentes, no scripted.** Las condiciones de avance verifican estados físicos reales (`qe`, `VictoryEvent`) — no "presioná X para continuar".
- **Sin strings de tooltip hardcoded.** Los textos de guía viven en RON/assets, no en código.
- **Scene4 ES la partida.** La última escena usa el sistema de victoria real de GS-5 — misma física, no simulación especial.

---

## NO hace

- No implementa cinemáticas o cutscenes — todo emergente.
- No implementa selector de personaje en UI — eso es resonance-app.
- No traduce la física a lenguaje de juego ("vida", "daño") — el juego no tiene esas palabras.
- No implementa perfil de progreso persistente — el onboarding es lineal y se puede repetir.

---

## Dependencias

- GS-5 — `VictoryEvent`, `PlayState::Victory` (condición de fin de Escena 4).
- GS-7 — `VisualHints` (legibilidad necesaria para que el onboarding sea comprensible).
- GS-8 — `ArchetypeRegistry`, `spawn_from_config` (personajes pre-configurados por escena).
- `world/demos/` — patrón existente de setup de demos.
- `simulation/states.rs` — `GameState` extendido con `Tutorial`.
- `runtime_platform/hud.rs` — overlay para texto de guía (consumidor de TutorialState).

---

## Criterios de aceptacion

### GS-9A (Estados)
- `TutorialState` es sub-estado de `GameState::Tutorial`.
- Transición `GameState::MainMenu → Tutorial` limpia recursos previos.
- `TutorialState::Complete` → transición a `GameState::Playing` disponible.

### GS-9B/C (Escenas + Avance)
- Test (MinimalPlugins): `setup_scene0_nucleus` → VictoryNucleus spawneado.
- Test: `tutorial_advance_check_system` en Scene0 con `clock.tick_id >= scene0_duration_ticks` → avanza a Scene1.
- Test: Scene1 con entidades de igual qe → no avanza.
- Test: `StateScoped` limpia entidades de Scene0 al transicionar a Scene1.
- Test: `VictoryEvent` en Scene4 → transición a `TutorialState::Complete`.

### General
- `cargo test --lib` sin regresión.
- Sin texto hardcodeado en código (strings en assets).

---

## Referencias

- `src/world/demos/` — demos existentes (patrón de setup)
- `src/simulation/states.rs` — `GameState`, `PlayState`
- `docs/sprints/GAMEPLAY_SYSTEMS/SPRINT_GS5_VICTORY_NUCLEUS.md` — VictoryEvent
- `docs/sprints/GAMEPLAY_SYSTEMS/SPRINT_GS7_VISUAL_CONTRACT.md` — VisualHints
- `docs/sprints/GAMEPLAY_SYSTEMS/SPRINT_GS8_ARCHETYPE_CONFIG.md` — spawn_from_config
- Blueprint §9: "Onboarding Through Emergent Experience"
