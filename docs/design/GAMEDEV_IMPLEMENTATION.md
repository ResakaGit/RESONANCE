# Blueprint — Implementacion de Patrones Gamedev (MOBA + Bevy 0.15)

**Scope:** Como los patrones MOBA estandar se implementan en Resonance sin violar el axioma energetico.
**Audiencia:** Humanos + modelos de IA (Claude, Cursor) que implementen los sprints G1-G12.
**Prerequisito:** Leer `DESIGNING.md` (filosofia de capas) y `GAMEDEV_PATTERNS.md` (catalogo de patrones).

---

## 1. El Problema Central

Los patrones MOBA estandar (cooldowns, buffs, targeting, fog of war) estan disenados para juegos con stats hardcodeados (HP, ATK, DEF, durations). Resonance no tiene stats — todo es energia (`qe`) fluyendo a traves de 14 capas ortogonales.

**El desafio:** implementar las INTERFACES estandar de MOBA (QWER, cooldown bars, HUD, minimap) sin introducir mecanicas que rompan el modelo energetico.

### Regla de Oro

> **Si un patron MOBA introduce un timer, un modifier, o un stat que no emerge de las 14 capas, debe ser reinterpretado como energia antes de implementarlo.**

---

## 2. Mapeo: Mecanica MOBA → Modelo Energetico

| Mecanica MOBA | Implementacion Tradicional | Implementacion Resonance | Por que |
|---------------|---------------------------|-------------------------|---------|
| **Cooldown** | `remaining_secs: f32` timer | Motor (L5): `buffer < cost → no puede castear` | El cooldown ES el tiempo que tarda L5 en rellenar el buffer. `cd = cost / intake_rate`. |
| **Buff duration** | `duration_secs: f32` timer | ResonanceLink (L10) entidad Tipo B con `qe` y `dissipation` | La duracion ES la energia del efecto decayendo por disipacion. `dur = effect_qe / dissipation`. |
| **Damage** | `hp -= dmg` | Interferencia destructiva: `target.qe -= projected_qe × |I|` | El dano emerge de la interaccion de ondas (L2 × L8). |
| **Heal** | `hp += heal` | Interferencia constructiva: `target.qe += projected_qe × I` | La curacion emerge de la misma ecuacion con signo positivo. |
| **Slow** | `speed *= 0.5` modifier | ResonanceLink entidad: `{ target, VelocityMultiplier, 0.5 }` | El slow es una entidad energetica que decae naturalmente. |
| **Armor** | `def: f32` stat | `MatterCoherence.bond_energy` (L4) | La "defensa" es la cohesion del material — resiste cambio de estado. |
| **Mana** | `mana: f32` resource | `AlchemicalEngine.current_buffer` (L5) | El mana es energia refinada en el motor del heroe. |
| **Vision** | `vision_range: f32` stat | `signal = qe × visibility(freq) / dist²` | La vision emerge de las capas 0 + 2. Umbra (20 Hz) es invisible, Lux (1000 Hz) brilla. |
| **Critical hit** | `crit_chance: f32` % | `|interference| > CRITICAL_THRESHOLD` × `crit_multiplier` (L9) | El critico emerge de la resonancia perfecta de ondas. |
| **Stun** | `can_act: false` flag | ResonanceLink entidad: `{ target, MotorOutputMultiplier, 0.0 }` | El stun bloquea la salida del motor → la voluntad (L7) no tiene potencia. |

---

## 3. Patron Cero: Cooldown Energetico (NO timer)

Este es el patron mas critico porque afecta TODA la implementacion de abilities.

### Cooldown Tradicional (INCORRECTO para Resonance)

```rust
// NUNCA — viola el axioma
struct AbilitySlot {
    cooldown_remaining: f32,  // timer hardcodeado
    cooldown_total: f32,      // duracion hardcodeada
    state: AbilityState,      // Ready/OnCooldown — estado artificial
}

fn cooldown_tick(slot: &mut AbilitySlot, dt: f32) {
    if let AbilityState::OnCooldown { remaining } = &mut slot.state {
        *remaining -= dt;  // timer decrementando — no es energia
    }
}
```

### Cooldown Energetico (CORRECTO para Resonance)

```rust
// SIEMPRE — el cooldown emerge del motor
struct AbilitySlot {
    cost_qe: f32,              // cuanto buffer drena el hechizo
    forced_frequency: f32,     // elemento del hechizo
    influence_radius: f32,     // rango
    targeting: TargetingMode,  // como se apunta
}

// El "cooldown" es: ¿el motor tiene suficiente buffer para costear?
fn can_cast(engine: &AlchemicalEngine, slot: &AbilitySlot) -> bool {
    engine.current_buffer >= slot.cost_qe
}

// El "cooldown bar" del HUD muestra: buffer / cost
fn cooldown_fraction(engine: &AlchemicalEngine, slot: &AbilitySlot) -> f32 {
    (engine.current_buffer / slot.cost_qe).min(1.0)
}

// El "cooldown time" es derivado, NUNCA almacenado:
fn estimated_cooldown_secs(engine: &AlchemicalEngine, slot: &AbilitySlot) -> f32 {
    let deficit = slot.cost_qe - engine.current_buffer;
    if deficit <= 0.0 { return 0.0; }
    deficit / (engine.input_valve) // intake_rate determina el tiempo
}
```

### Por que importa

Con cooldowns energeticos:
- Un bioma Ley Line (L6, +qe/s) REDUCE el cooldown naturalmente
- Un debuff que baje `input_valve` (via ResonanceLink) AUMENTA el cooldown
- Un buff de "mana regen" no es un stat — es una entidad-efecto que multiplica `input_valve`
- Gastar habilidades seguidas REALMENTE agota el buffer — no es un timer independiente por ability
- **Todas las abilities comparten el mismo motor.** Lanzar Fireball deja menos buffer para Ember Shield. Esto crea decision-making emergente sin ninguna regla extra.

### Excepcion: Minimo entre casts (GCD)

Un Global Cooldown minimo (0.5s entre casts) se justifica como **tiempo de channeling** (L7: `channeling_ability`), no como timer. Mientras el motor esta "expulsando" energia, el actuador no puede iniciar otro cast. Esto emerge de la fisica del inyector.

---

## 4. Patron Uno: Status Effects como Entidades Energeticas

### Dos Modelos — Cuando Usar Cada Uno

| Modelo | Mecanismo | Duracion | Purga | Uso |
|--------|-----------|----------|-------|-----|
| **ResonanceLink (L10)** | Entidad Tipo B con qe propia | `qe / dissipation` | Destruir la entidad-efecto | Efectos con duracion variable, interactuables |
| **SparseSet Marker** | Componente transitorio | Sistema tick-down o motor-driven | `commands.remove::<T>()` | Marcadores de estado simples (immune, grounded) |

**Regla:** Si el efecto tiene DURACION que deberia ser modificable por el entorno → ResonanceLink entidad.
Si el efecto es binario (on/off) sin duracion energetica → SparseSet marker.

### Ejemplos

```rust
// Slow: ResonanceLink entidad (interactuable, duracion energetica)
// Se spawea como entidad independiente
EntityBuilder::new()
    .energy(30.0)                    // 30 qe de combustible
    .flow(Vec2::ZERO, 10.0)         // disipacion alta → dura 3s
    .resonance_link(target, ModifiedField::VelocityMultiplier, 0.5)
    .spawn(commands);

// Immune: SparseSet marker (binario, no interactuable)
#[derive(Component)]
#[component(storage = "SparseSet")]
struct Immune;  // puesto/quitado por sistema, no tiene energia propia
```

### Integracion con SparseSet (G1)

Los componentes SparseSet en Resonance son para:
1. `DespawnOnContact` — flag de consumo unico
2. `SpellMarker` — tag transitorio de tipo
3. `PlayerControlled` — flag de control
4. `Immune`, `Grounded`, `Channeling` — estados binarios sin duracion energetica

Los componentes SparseSet NO son para buffs/debuffs con duracion. Esos son ResonanceLink (L10).

---

## 5. Patron Dos: Targeting como Lectura de Capas

El targeting en un MOBA es: "selecciona algo para que una habilidad actue sobre ello." En Resonance, targeting es leer capas para determinar compatibilidad.

### Flujo de Targeting

```
1. Jugador presiona Q           → TargetingState.active = Some(slot_0)
2. Sistema lee slot.targeting   → PointTarget { range: 30.0 }
3. Jugador clickea el suelo     → world_pos = ray-plane intersection
4. Validacion:
   a. ¿engine.buffer >= slot.cost_qe?        (L5 — motor)
   b. ¿distancia <= range?                    (L1 — volumen)
   c. ¿no esta channeling?                    (L7 — voluntad)
5. Si valida → motor drena buffer, se spawea entidad-hechizo (L8)
6. El hechizo viaja, interactua, y muere por sus propias reglas de energia
```

### UnitTarget — Seleccion por Capas

Para habilidades de target unitario (Hex, Lina stun):

```rust
fn is_valid_target(
    caster: (&MobaIdentity, &Transform),
    target: (&MobaIdentity, &Transform, &BaseEnergy),
    slot: &AbilitySlot,
) -> bool {
    // Rango (L1)
    let dist = caster.1.translation.distance(target.1.translation);
    if dist > slot.range() { return false; }

    // Vivo (L0)
    if target.2.qe() < QE_MIN_EXISTENCE { return false; }

    // Faccion (L9) — no target aliado con hechizo ofensivo
    if caster.0.faction() == target.0.faction() && slot.is_offensive() {
        return false;
    }

    true
}
```

Toda la logica de targeting usa capas existentes. No se crean stats nuevos.

---

## 6. Patron Tres: Camara y Percepcion

### Camara MOBA — No Toca Energia

La camara es puro runtime_platform. Free pan + lock toggle no interactua con las 14 capas. Implementacion directa sin conflicto filosofico.

### Minimap — Refleja Percepcion Energetica

El minimap NO debe mostrar entidades como "dots en un mapa." Debe reflejar la percepcion emergente:

```
signal = source.qe × visibility(source.frequency) / distance²
```

Un heroe Umbra (20 Hz) es MAS TENUE en el minimap que un heroe Lux (1000 Hz). Esto emerge naturalmente — no es un stat "stealth".

### Fog of War — Emerge de la Onda

DESIGNING.md establece que la percepcion emerge de L0 + L2:
- `qe` = brillo de la fuente
- `frequency_visibility()` = cuanto emite cada elemento
- `distance²` = caida natural

El FoW grid es una optimizacion (cache de que celdas estan iluminadas), pero la logica subyacente usa ecuaciones de percepcion energetica, no un `vision_range: f32` stat.

---

## 7. Patron Cuatro: Pathfinding — Integra con Flujo (L3)

El pathfinding calcula una ruta. Pero en Resonance, el movimiento es FlowVector (L3) + WillActuator (L7). El path solo alimenta `movement_intent`:

```
NavMesh → waypoints → direction → WillActuator.movement_intent
WillActuator → will_force (ecuacion pura) → FlowVector.velocity
FlowVector → drag_force (ecuacion pura) → Transform.translation
```

Cada paso usa ecuaciones de `blueprint/equations.rs`. El pathfinding NO calcula velocidad — solo direccion. La velocidad emerge de la fisica (L3, L1, L6).

**Consecuencia emergente:** Un heroe en zona de alta viscosidad (L6) se mueve MAS LENTO por la misma ruta. No hace falta codigo extra — la ecuacion de arrastre ya lo hace.

---

## 8. Patron Cinco: GameState — Coordina sin Romper

`GameState` (Loading/Playing/Paused) es infraestructura pura. No introduce stats ni modifica energia. Solo controla QUE SISTEMAS corren.

**Mapeo critico:**
- `WorldgenState::Warming` → `PlayState::Warmup` — los sistemas de worldgen propagan energia
- `WorldgenState::Ready` → `PlayState::Active` — los sistemas de gameplay empiezan
- `Paused` — TODOS los sistemas de FixedUpdate se detienen → la energia se congela

`StateScoped(GameState::Playing)` en entidades de gameplay habilita auto-cleanup. La energia de esas entidades se "pierde" al salir del estado — consistente con la disipacion natural.

---

## 9. Invariantes de Implementacion

Estas reglas aplican a TODOS los sprints G1-G12:

### 9.1 — No Timers

Ningun componente nuevo debe tener un campo `remaining_secs`, `duration`, `timer`, o `elapsed`. Si algo tiene duracion, esa duracion emerge de `qe / dissipation_rate` o de `buffer / intake_rate`.

**Excepcion:** `Casting { timer: f32 }` para channeling time (0.3-1.0s) se justifica como constraint fisico del inyector, no como cooldown.

### 9.2 — No Stats Derivados Almacenados

No crear componentes con `damage: f32`, `heal_power: f32`, `cooldown_reduction: f32`. Estos valores se computan en punto de uso desde las capas existentes.

### 9.3 — No Modifiers como Campos

Un "buff de velocidad" no es `speed_bonus: f32` en el heroe. Es una entidad ResonanceLink (L10) con `VelocityMultiplier`. Cuando la entidad-efecto muere, el bonus desaparece automaticamente.

### 9.4 — Ecuaciones en equations.rs

Toda formula nueva (targeting range check, pathfinding cost, fog signal strength) va en `blueprint/equations.rs`. Los sistemas llaman funciones puras.

### 9.5 — HUD Muestra Capas, No Stats

El HUD no muestra "HP", "Mana", "Cooldown". Muestra:
- Barra de energia: `BaseEnergy.qe` (L0)
- Barra de buffer: `AlchemicalEngine.current_buffer / max_buffer` (L5)
- Cooldown visual: `buffer / slot.cost_qe` (L5 / slot config)
- Estado de materia: `MatterCoherence.state` (L4) — color/icono

La ETIQUETA puede decir "HP" o "Mana" (interfaz estandar), pero el DATO viene de las capas.

---

## 10. Grafo de Dependencia: Sprints vs Capas

```
Sprint    Capas que toca    Capas que lee    Nuevo estado ECS
────────────────────────────────────────────────────────────────
G1        -                 -                SparseSet annotations
G2        -                 -                GameState, PlayState
G3        L5,L7,L8          L0,L2,L9         TargetingMode, TargetingState, AbilityCastEvent
G4        -                 -                MobaCameraConfig, CameraMode
G5        L3,L7             L1,L6            NavAgent, NavPath
G6        L0-L9             -                AlchemicalBase, WaveEntity, Champion markers
G7        L0,L10            -                Observers (OnAdd, OnRemove)
G8        L0-L13            -                Guards en setters
G9        -                 -                .chain()/.after() constraints
G10       L0,L2,L9          L0,L2            MinimapConfig, MinimapIcon
G11       -                 -                ChampionId, WorldEntityId, EffectId
G12       L0,L2             L0,L2,L9         FogOfWarGrid, VisionProvider
```

Ningun sprint introduce una CAPA nueva. Todos operan sobre la infraestructura existente de 14 capas o sobre la plataforma runtime.

---

## 11. Checklist de Filosofia

Antes de implementar CUALQUIER sprint G*, verificar:

- [ ] **Test de energia:** ¿El nuevo dato emerge de las 14 capas o es un stat inventado?
- [ ] **Test de disipacion:** ¿Si algo tiene duracion, su duracion viene de qe/dissipation?
- [ ] **Test de interferencia:** ¿El nuevo sistema puede ser afectado por la interaccion de ondas?
- [ ] **Test de derivacion:** ¿Se puede computar en punto de uso en vez de almacenar?
- [ ] **Test de motor:** ¿El cooldown emerge del buffer/intake_rate del motor?
- [ ] **Test de HUD:** ¿El display muestra datos de capas, no stats artificiales?
- [ ] **Test de ecuacion:** ¿La formula esta en equations.rs, no inline en el sistema?

Si algun test falla, el sprint necesita reinterpretacion antes de implementar.

---

## 12. Referencias

- `DESIGNING.md` — Filosofia de capas, axioma energetico, 5 tests
- `docs/design/GAMEDEV_PATTERNS.md` — Catalogo de patrones y anti-patrones
- `docs/sprints/GAMEDEV_PATTERNS/` — Sprints de implementacion G1-G12
- `.cursor/skills/bevy-ecs-resonance/SKILL.md` — Skill de ECS para IA
- `src/blueprint/equations.rs` — Ecuaciones puras
- `src/layers/` — Las 14 capas
