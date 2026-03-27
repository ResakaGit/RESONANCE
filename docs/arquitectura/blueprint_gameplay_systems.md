# Blueprint: Gameplay Systems — Contrato Arquitectural

> Track GS | Validado contra código fuente 2026-03-25
> Prerrequisito de lectura: `00_contratos_glosario.md`, `blueprint_simulation.md`

---

## GS-1: Netcode Lockstep

### 1) Propósito y frontera

- Resuelve: sincronización determinista de estado de simulación entre N clientes sin transmitir world state.
- No resuelve: transporte de red (TCP/UDP), serialización de mundo completo, rollback (GS-2).

### 2) Superficie pública (contrato)

- **Tipos exportados:** `InputPacket`, `LockstepConfig`, `InputBuffer`, `ChecksumLog`, `DesyncResult`.
- **Sistemas:** `lockstep_input_gate_system` (Phase::Input), `lockstep_checksum_record_system` (PostUpdate), `lockstep_desync_check_system` (PostUpdate).
- **Resources:** `LockstepRunCondition { can_advance: bool }` — puerta que bloquea el avance del tick.
- **Ecuaciones puras:** `input_delay_ticks`, `correction_cost_ticks`, `is_delay_acceptable`, `tick_checksum`.

### 3) Invariantes y precondiciones

- El tick NO avanza si `LockstepRunCondition::can_advance == false`.
- `InputBuffer.entries` y `ChecksumLog.entries` son Vec ordenados — sin HashMap.
- `tick_checksum` delega en `determinism::hash_f32_slice` — determinista.
- `InputPacket` es SparseSet — transient por tick.

### 4) Comportamiento runtime

- `lockstep_input_gate_system` corre en Phase::Input, `.before(PlatformWill)`.
- `lockstep_checksum_record_system` corre en PostUpdate — después del tick completo.
- Inputs del tick T se colectan con delay `input_delay_ticks` desde tick T-delay.
- Log purgado a cada 120 ticks (ventana de 6 segundos a 20Hz).

### 5) Implementación y trade-offs

- **Vec sobre HashMap:** orden determinista garantizado. O(n) lookup con n < 8 jugadores — correcto.
- **Input delay configurable:** `LockstepConfig` es Resource RON-loadable. No constante hardcoded.
- **No implementa transporte:** la red es responsabilidad de resonance-app. GS-1 es la capa de protocolo.

### 6) Fallas y observabilidad

- `DesyncResult::Desynced { player_a, player_b, tick_id }` — evento emitido.
- Si `can_advance` permanece false por N ticks → timeout → solicitar resync.
- Telemetría: `ChecksumLog` inspecccionable como Resource en debug HUD.

### 7) Checklist de atomicidad

- ✅ Una responsabilidad: verificar que inputs están disponibles y hashear el estado.
- ✅ No acopla física, rendering ni gameplay logic.

---

## GS-2: Netcode Rollback

### 1) Propósito y frontera

- Resuelve: corrección retroactiva cuando el input predicho difiere del real. Invisible al jugador si latencia < input delay.
- No resuelve: interpolación de entidades remotas, rollback multi-jugador (> 2 clientes), serialización (delegado a SF-5).

### 2) Superficie pública (contrato)

- **Tipos:** `RollbackFrame`, `RollbackBuffer`, `PredictedInput`, `RollbackState`.
- **Sistemas:** `rollback_frame_save_system` (Phase::Input), `rollback_detect_system` (PostUpdate), `rollback_apply_system` (PostUpdate).
- **Recursos:** `RollbackState { divergence_tick, correction_in_progress, blend_alpha }`.
- **Ecuaciones puras:** `resimulation_ticks`, `prefer_resync`, `correction_blend`, `blend_alpha_per_tick`.

### 3) Invariantes y precondiciones

- `RollbackBuffer` es Vec circular pre-allocado — cero allocations por tick en steady state.
- Si `resim_ticks > MAX_ROLLBACK_WINDOW` → solicitar resync completo, no rollback.
- La re-simulación usa `SimWorld::tick()` puro — determinista por INV-4.

### 4) Comportamiento runtime

- Frame guardado sólo cuando `LockstepRunCondition::can_advance == true`.
- Detección en PostUpdate — después de que todos los checksums están disponibles.
- Corrección = restore(snapshot) + replay(ticks) — misma ruta que `SimWorld::tick()`.

### 5) Implementación y trade-offs

- **Input prediction = repeat-last:** modelo simple, correcto ~90% del tiempo para movimiento.
- **Blend visual:** corrección suave sobre N ticks para evitar "pop". Afecta sólo transform visual, no estado físico.
- **Depende fuertemente de SF-5:** sin checkpoint, no hay rollback. Prerrequisito duro.

### 6) Fallas y observabilidad

- `RollbackState::correction_in_progress == true` → debug HUD puede mostrar indicador.
- Si resync solicitado → `RollbackState::divergence_tick = None` + evento a resonance-app.

### 7) Checklist de atomicidad

- ✅ Una responsabilidad: restaurar coherencia cuando predicción falla.
- ⚠️ Acoplamiento necesario con SF-5 (checkpoint) — aceptable por dependencia explícita.

---

## GS-3: Nash AI Targeting

### 1) Propósito y frontera

- Resuelve: selección de objetivo óptima por resonancia y resistencia. Extiende D1 (BehaviorMode) sin romperlo.
- No resuelve: formación de manada (GS-4), coordinación de equipo (GS-4), movement AI.

### 2) Superficie pública (contrato)

- **Tipos:** `BehaviorMode::FocusFire { target, team_priority }`, `BehaviorMode::Regroup { rally_pos }` (stub), `NashTargetConfig`.
- **Sistema:** `nash_target_select_system` (Phase::Input, in_set(BehaviorSet::Decide)).
- **Ecuaciones puras:** `resonance_factor`, `effective_extraction`, `extraction_resistance`, `threat_magnitude`, `threat_gradient`.

### 3) Invariantes y precondiciones

- `nash_target_select_system` SÓLO actúa si modo es `Hunt` o `FocusFire` — no sobreescribe Flee, Eat, Forage.
- Targets con `qe <= 0` son filtrados — no se atacan entidades muertas.
- Aliados son filtrados — sólo `faction != self_faction`.
- Sin HashMap — `detected_entities` es Vec<Entity> con iteración lineal.

### 4) Comportamiento runtime

- Corre en Phase::Input, dentro de `BehaviorSet::Decide`, después de D1 existente.
- Reutiliza `SensoryAwareness.detected_entities` — no re-escanea el mundo.
- `argmin(extraction_resistance)` sobre los detectados — O(n) con n < 20 en rango sensorial.

### 5) Implementación y trade-offs

- **Ecuaciones puras sin ECS:** `resonance_factor`, `effective_extraction`, `extraction_resistance` son fn puras — testables independientemente.
- **Frecuencia como proxy de vulnerabilidad:** mayor resonancia = mayor extracción = menor resistencia.

### 6) Fallas y observabilidad

- Si `SensoryAwareness.detected_entities` está vacío → `find_nash_target` retorna `None` → modo no cambia.
- `NashTargetConfig` inspecccionable como Resource.

### 7) Checklist de atomicidad

- ✅ Una responsabilidad: selección de objetivo óptimo.
- ✅ No toca física, locomotion, ni rendering.

---

## GS-4: Pack Dynamics

### 1) Propósito y frontera

- Resuelve: cohesión de manada + respuesta coordinada a amenazas. Extiende social_communication existente.
- No resuelve: formaciones posicionales rígidas, comunicación de habilidades, asignación automática de roles.

### 2) Superficie pública (contrato)

- **Tipos:** `PackDynamicsConfig` (Resource).
- **Sistemas:** `pack_regroup_system` (Phase::Input, after nash_target_select_system), `pack_cohesion_force_system` (Phase::MetabolicLayer).
- **Ecuaciones puras:** `pack_center`, `cohesion_force`, `is_pack_dispersed`, `pack_intent_vector`.
- **Escritura:** `WillActuator::social_intent` (Vec2) — no toca FlowVector directamente.

### 3) Invariantes y precondiciones

- `Regroup` no sobreescribe `FocusFire`, `Flee`, `Eat` — sólo reemplaza modos no-combate.
- Presencia calculada como suma de `InferenceProfile::extraction_capacity()` — física, no contador.
- `pack_center` usa Vec ponderado — sin HashMap.

### 4) Comportamiento runtime

- Cohesión: Phase::MetabolicLayer, after social/pack_formation.
- Regroup check: Phase::Input, after BehaviorSet::Decide.
- Intent de pack compite con intent de behavior D1 — locomotion resuelve prioridad.

### 5) Implementación y trade-offs

- **Reutiliza GS-3 threat equations:** sin duplicación. `threat_gradient` es fn pura importada.
- **Dead zone para cohesión:** sin dead zone, la fuerza oscilaría. Zona de 3 unidades = pack "unido".

### 6) Fallas y observabilidad

- Pack con todos en diferentes facciones → sin cohesión (filtrado por `pack_id`).
- `PackDynamicsConfig` inspecccionable. `WillActuator::social_intent` visible en debug HUD.

### 7) Checklist de atomicidad

- ✅ Una responsabilidad: mover el pack como unidad.
- ✅ Escribe sólo `WillActuator` — no toca otras capas directamente.

---

## GS-5: Victory Nucleus

### 1) Propósito y frontera

- Resuelve: condición de victoria como estado físico. La partida termina cuando `qe(nucleus_B) < QE_MIN`.
- No resuelve: animación de victoria, nodos de mapa (GS-6), spawn de núcleos (worldgen existente).

### 2) Superficie pública (contrato)

- **Tipos:** `VictoryNucleus` (SparseSet, marker), `VictoryEvent`, `GameOutcome` (Resource), `PlayState::Victory`.
- **Sistemas:** `nucleus_intake_decay_system` (Phase::ThermodynamicLayer), `victory_check_system` (Phase::MetabolicLayer, after metabolic_stress_death).
- **Ecuaciones puras:** `nucleus_effective_intake`, `is_nucleus_viable`, `comeback_potential`, `energy_advantage`.

### 3) Invariantes y precondiciones

- `GameOutcome.winner` no se sobreescribe si ya está seteado — primera victoria es definitiva.
- `VictoryNucleus` es SparseSet — máximo 2 entidades en toda la partida.
- `victory_check_system` sólo verifica `is_final_target == true`.

### 4) Comportamiento runtime

- `nucleus_intake_decay` corre en Phase::ThermodynamicLayer — antes de los cálculos de energía.
- `victory_check` corre en Phase::MetabolicLayer — después de que las entidades pueden morir.
- `PlayState::Victory` congela la simulación — `run_if(not(in_state(PlayState::Victory)))`.

### 5) Implementación y trade-offs

- **EnergyNucleus ya existe:** GS-5 sólo agrega `VictoryNucleus` marker vía `commands.entity(e).insert(...)` en startup. Sin tocar worldgen.
- **Snowball via intake decay:** el núcleo dañado genera menos energía → feedback loop negativo → derrota acelerada pero no instantánea.

### 6) Fallas y observabilidad

- `VictoryEvent` emitido en el tick de colapso — loggeable, replayable (SF-7).
- `GameOutcome` inspecccionable como Resource en debug HUD.

### 7) Checklist de atomicidad

- ✅ Una responsabilidad: declarar fin de partida cuando física lo justifica.
- ✅ No define quién ataca el núcleo — eso es la física existente.

---

## GS-6: Map Energy

### 1) Propósito y frontera

- Resuelve: nodos de control como mecanismo de snowball energético.
- No resuelve: posiciones de nodos (RON de mapa), visión de nodos (GS-7), pathfinding hacia nodos (GS-3).

### 2) Superficie pública (contrato)

- **Tipos:** `ControlNode` (SparseSet), `NodeControlState`, `NodeCapturedEvent`, `NodeControlConfig`.
- **Sistemas:** `node_control_update_system` (Phase::MetabolicLayer, after faction_identity, before victory_check), `nucleus_node_bonus_system` (Phase::MetabolicLayer, after node_control).
- **Ecuaciones puras:** `control_factor`, `controlling_faction`, `node_drain_rate`, `snowball_intake_factor`.

### 3) Invariantes y precondiciones

- Presencia = suma de `InferenceProfile::extraction_capacity()` — no counter de unidades.
- `ControlNode` SparseSet — pocos nodos (3-5) en toda la partida.
- Drain limitado por `field_qe` disponible — no puede drenar más de lo que hay.

### 4) Comportamiento runtime

- Control actualizado cada tick — sin throttling.
- Bonus al núcleo aplicado después del control check.
- `NodeCapturedEvent` emitido en cambio de control — chain de eventos correcto.

### 5) Implementación y trade-offs

- **Nodos como entidades existentes:** `ControlNode` se agrega a entidades worldgen — sin nuevas entidades. Patrón VictoryNucleus.
- **Snowball moderado:** 0.15 scaling → 3 nodos = +45% intake. Recuperable con contraataque.

### 6) Fallas y observabilidad

- `NodeControlState.control_factor` visible en debug HUD.
- `NodeCapturedEvent` loggeable.

### 7) Checklist de atomicidad

- ✅ Una responsabilidad: resolver control de nodo y aplicar su efecto al núcleo.

---

## GS-7: Visual Contract

### 1) Propósito y frontera

- Resuelve: mapping inyectivo estado físico → señal visual. Formaliza el contrato del renderer.
- No resuelve: rendering (sprites/meshes), UI/HUD, animaciones especiales, minimapa.

### 2) Superficie pública (contrato)

- **Tipos:** `EntityVisualHint`, `VisualHints` (Resource), `VisualContractConfig`.
- **Sistema:** `visual_contract_sync_system` (Phase::MorphologicalLayer) — read-only.
- **Ecuaciones puras:** `qe_to_luminance`, `damage_to_saturation`, `speed_to_trail_alpha`, `nucleus_pulse_factor`.
- **INV-5:** `visual_contract_sync_system` tiene cero queries mutables sobre ECS.

### 3) Invariantes y precondiciones

- `VisualHints.entities` siempre ordenado por `entity_id` — canónico.
- Cuatro dimensiones ortogonales: hue (freq), luminance (qe), saturation (dmg), trail (speed).
- Renderer consume SÓLO `VisualHints` — no lee ECS directamente.
- `nucleus_pulse_factor` usa `tick_id` como reloj — INV-8 compliant.

### 4) Comportamiento runtime

- Sync en Phase::MorphologicalLayer — último sistema de simulación.
- Update layer consume `VisualHints` — desacoplado del tick rate.
- Clear + fill cada tick — snapshot limpio.

### 5) Implementación y trade-offs

- **sqrt en luminance:** perceptual linearity — 250 qe / 1000 qe_max → `luminance = 0.5` (no 0.25).
- **Cuadrático en trail:** trail visible sólo en alta velocidad — no ruido visual en movimiento lento.

### 6) Fallas y observabilidad

- Entidad sin `MatterCoherence` → `saturation = 1.0` (default seguro).
- Test de injectividad: `is_injective_sample` en `#[cfg(test)]`.

### 7) Checklist de atomicidad

- ✅ Una responsabilidad: derivar hints visuales sin tocar estado físico.
- ✅ INV-5 verificable en compilación (cero &mut en query).

---

## GS-8: Archetype Config

### 1) Propósito y frontera

- Resuelve: personajes como configuraciones de física RON-loadables. Balance sin recompilar.
- No resuelve: selección de personaje en UI, hot-reload en runtime, versioning de configs.

### 2) Superficie pública (contrato)

- **Tipos:** `ArchetypeConfig` (8 sub-structs), `ArchetypeRegistry` (Resource), `ArchetypeIdentity`, etc.
- **Función:** `spawn_from_config(&mut Commands, &ArchetypeConfig, Vec2, &SimulationClock) -> Entity`.
- **Sistema startup:** `load_archetype_configs_system`.

### 3) Invariantes y precondiciones

- Sin `String` en `ArchetypeConfig` — sólo `f32`, `u32`, `u8`, `bool`, enums.
- `ArchetypeRegistry` es Vec ordenado por `archetype_id` — sin HashMap.
- Max 4 campos por sub-struct.

### 4) Comportamiento runtime

- Carga en Startup — una vez, no hot-reload.
- `spawn_from_config` es función libre, no sistema — usable en setup de demos/tutorial.
- Fallback a default constants si config no existe — sin pánico.

### 5) Implementación y trade-offs

- **8 sub-structs:** cubre las 14 capas distribuidas en bloques lógicos (identity, energy, oscillatory, matter, engine, locomotion, combat, social).
- **Serde derive bajo feature flag:** no contamina builds headless que no necesitan deserialización.

### 6) Fallas y observabilidad

- RON malformado → error at startup con path del archivo.
- `ArchetypeRegistry` inspecccionable como Resource.

### 7) Checklist de atomicidad

- ✅ Una responsabilidad: datos de configuración de arquetipo.
- ✅ `spawn_from_config` delega construcción a `EntityBuilder` — sin duplicar lógica.

---

## GS-9: Onboarding

### 1) Propósito y frontera

- Resuelve: secuencia de 5 experiencias emergentes que construyen intuición física.
- No resuelve: cinemáticas, selección de personaje, perfil de progreso persistente.

### 2) Superficie pública (contrato)

- **Estados:** `GameState::Tutorial`, `TutorialState` (5 escenas + Complete).
- **Plugin:** `OnboardingPlugin` — registra transiciones y sistemas de setup.
- **Sistemas:** `tutorial_advance_check_system` (Phase::MetabolicLayer, run_if Tutorial).
- **Cleanup:** `StateScoped(GameState::Tutorial)` en todas las entidades de tutorial.

### 3) Invariantes y precondiciones

- Condiciones de avance son estados físicos emergentes — no inputs de usuario forzados.
- `VictoryEvent` de Scene4 usa el mismo sistema de GS-5 — no simulación especial.
- Sin strings hardcodeados en código — textos en assets.

### 4) Comportamiento runtime

- `OnEnter(TutorialState::SceneN)` → setup de la escena (cleanup automático por StateScoped).
- `tutorial_advance_check_system` verifica condición cada tick.
- `TutorialState::Complete` → UI puede ofrecer ir a `GameState::Playing`.

### 5) Implementación y trade-offs

- **5 escenas, no más:** complejidad creciente graduada. Escena 4 = partida real completa.
- **StateScoped para cleanup:** evita sistemas de limpieza manuales — patrón Bevy idiomático.

### 6) Fallas y observabilidad

- Si `ArchetypeRegistry` no tiene archetypes de tutorial → error en `setup_scene*` con mensaje.
- `TutorialState` inspecccionable como estado de Bevy.

### 7) Checklist de atomicidad

- ✅ Una responsabilidad: secuenciar experiencias y detectar cuando avanzar.
- ✅ No define la física de las escenas — reutiliza sistemas existentes.

---

## Grafo de dependencias del track GS

```
[SF-5 Checkpoint]   [SF-7 Replay]   [sim_world.rs INV-4]   [D1 Behavior]
      │                   │                 │                     │
      ▼                   ▼                 ▼                     ▼
 GS-1 Lockstep     GS-5 Victoria     GS-2 Rollback          GS-3 Nash AI
      │                   │                 │                     │
      └─────────┬──────── ┤                 │                GS-4 Pack
                          │                 │
                   GS-6 Map Energy     GS-2 (via GS-1)
                          │
               ┌──────────┴──────────┐
               ▼                     ▼
        GS-7 Visual            GS-8 Arquetipos
               │                     │
               └──────────┬──────────┘
                           ▼
                     GS-9 Onboarding
```

## Invariantes del track

1. **Determinismo absoluto.** Toda AI, victoria, netcode: sólo physics observables — cero RNG excepto gated por `tick_id XOR entity_id`.
2. **AI no lee conceptos de juego.** `BehaviorMode` se deriva de `qe`, `velocity`, `frequency`, `structural_damage`. Sin kill-counters ni health bars.
3. **Victoria es estado físico.** `qe(nucleus) < QE_MIN` — sin timer ni reglas externas.
4. **Visual nunca escribe física.** `visual_contract_sync_system` — cero queries mutables.
5. **Arquetipos = constantes RON.** Balance sin recompilar.
6. **Netcode inputs-only.** Solo inputs cruzan la red en estado estacionario.
7. **Zero crates nuevos.** Reutiliza `serde` + `ron` + `bevy` existentes.
8. **Phase assignment explícito.** Cada sistema en `Phase::X` o `SystemSet` nombrado.
9. **Max 4 campos por componente/sub-struct.** Regla aplicada al track completo.
