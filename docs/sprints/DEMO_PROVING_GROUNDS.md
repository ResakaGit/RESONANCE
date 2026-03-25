# Demo — Proving Grounds (3D)

**Mapa:** `RESONANCE_MAP=proving_grounds cargo run`
**Modulo:** `src/world/proving_grounds.rs`
**Objetivo:** Escenario 3D interactivo que ejerce las **14 capas ECS**, los 5 subsistemas (worldgen, eco, topology, bridge, simulation) y toda la pipeline de gameplay en un solo mapa jugable.

---

## 1. Motivacion

La demo_arena existente prueba 6 heroes + 5 biomas + cristales/piedras/particulas, pero:
- Solo usa capas 0-9 + 12 (coherencia). **No usa L10 (ResonanceLink), L11 (TensionField), L12 (Homeostasis), L13 (StructuralLink).**
- El Grimoire se inserta vacio — ningun heroe tiene habilidades.
- No hay proyectiles en escena (catalisis solo ocurre con el LavaKnight).
- No hay entidades con adaptacion frecuencial (Homeostasis).
- No hay campos de tension (gravedad/magnetismo).
- No hay vinculos estructurales visibles (resortes).

Proving Grounds es un mapa disenado para que **cada capa y cada sistema tenga al menos una interaccion observable**.

---

## 2. Layout del mapa

```
                    N
                    │
        ╔═══════════════════════╗
        ║   ETHER SANCTUM (NW)  ║ ← Ley Line + Homeostasis entities
        ║   Lux/Eter nuclei     ║
        ╠═══════════════════════╣
        ║                       ║
        ║   TENSION ARENA (N)   ║ ← Gravity wells + magnetic traps
        ║   TensionField L11    ║
        ╠═══════╦═══════╦═══════╣
        ║ SWAMP ║ NEXUS ║ FORGE ║ ← Center nexus = spawn + linked crystal pair
        ║  (W)  ║ (C)   ║  (E)  ║
        ╠═══════╩═══════╩═══════╣
        ║                       ║
        ║   PROVING LANE (S)    ║ ← Projectile corridor + target dummies
        ║   ResonanceLink L10   ║
        ╠═══════════════════════╣
        ║   VOLCANO PIT (SE)    ║ ← Lava Knights + phase transitions
        ║   Structural pairs    ║
        ╚═══════════════════════╝
                    │
                    S
```

**Grid:** 64×64 celdas, `cell_size: 2.0`, origen `(-64, -64)`.
**Nuclei:** 7 (ver seccion 4).

---

## 3. Que prueba cada zona

| Zona | Capas ejercidas | Sistemas ejercidos | Que observar |
|------|----------------|--------------------|--------------|
| **Nexus (centro)** | L0-L9, L13 | Spawn, engine drain, will input | Hero se mueve con WASD/click; engine drena buffer; par de cristales unidos por StructuralLink se atraen como resorte |
| **Forge (este)** | L0-L8, L5, L6 | Catalisis, presion ambiental | Entrar al bioma Volcano sube temperatura → transicion de fase Solid→Liquid→Gas; cristales Ignis transfieren energia |
| **Swamp (oeste)** | L0-L6 | Viscosidad, disipacion | Heroe se mueve mas lento (viscosity 3.0); disipacion alta roba qe; particulas Umbra pierden energia |
| **Tension Arena (norte)** | L11 | TensionField evaluation | Pozos de gravedad atraen entidades cercanas; trampas magneticas repelen; heroe siente pull/push |
| **Ether Sanctum (noroeste)** | L12 | Homeostasis adapt | Entidades con Homeostasis adaptan frecuencia hacia la dominante del nucleo Lux/Eter; costo qe visible |
| **Proving Lane (sur)** | L8, L10 | Catalisis, projectile spawn, ResonanceLink | Dummies reciben proyectiles; entidades-efecto aplican slow/haste via ResonanceLink; interferencia constructiva/destructiva |
| **Volcano Pit (sureste)** | L4, L13 | Phase transitions, StructuralLink break | Par de caballeros de lava unidos por StructuralLink; si uno muere el stress rompe el vinculo; transiciones Plasma visibles |

---

## 4. Configuracion del mapa RON

**Archivo:** `assets/maps/proving_grounds.ron`

```ron
(
    width_cells: 64,
    height_cells: 64,
    cell_size: 2.0,
    origin: (-64.0, -64.0),
    warmup_ticks: Some(80),
    seed: Some(314159),
    nuclei: [
        // Centro — Terra estable (base del mapa)
        (name: "terra_nexus", position: (0.0, 0.0), frequency_hz: 75.0,
         radius: 20.0, qe: 400.0, decay_rate: 0.3),

        // Este — Ignis forja
        (name: "ignis_forge", position: (40.0, 0.0), frequency_hz: 450.0,
         radius: 16.0, qe: 600.0, decay_rate: 0.8),

        // Oeste — Umbra pantano
        (name: "umbra_swamp", position: (-40.0, 0.0), frequency_hz: 20.0,
         radius: 14.0, qe: 150.0, decay_rate: 0.4),

        // Norte — Ventus tension
        (name: "ventus_tension", position: (0.0, 40.0), frequency_hz: 700.0,
         radius: 12.0, qe: 300.0, decay_rate: 0.5),

        // Noroeste — Lux sanctum
        (name: "lux_sanctum", position: (-35.0, 35.0), frequency_hz: 1000.0,
         radius: 10.0, qe: 500.0, decay_rate: 0.2),

        // Sur — Aqua proving lane
        (name: "aqua_lane", position: (0.0, -35.0), frequency_hz: 250.0,
         radius: 14.0, qe: 200.0, decay_rate: 0.6),

        // Sureste — Ignis pit (volcánico secundario)
        (name: "ignis_pit", position: (35.0, -35.0), frequency_hz: 450.0,
         radius: 12.0, qe: 800.0, decay_rate: 1.0),
    ],
    seasons: [],
)
```

---

## 5. Entidades del escenario

### 5.1 — Heroes (jugador + aliados + enemigos)

| # | Clase | Faccion | Posicion | Capas extra | Proposito |
|---|-------|---------|----------|-------------|-----------|
| 1 | **FireMage** | Red | (0, -5) | PlayerControlled, Grimoire con 3 abilities | **Jugador principal** — testea input, engine, will, catalisis |
| 2 | EarthWarrior | Red | (-3, -5) | Homeostasis (adapt_rate=5.0, cost=0.1, band=50) | Aliado que adapta frecuencia cuando entra a zona ajena |
| 3 | WindShooter | Red | (3, -5) | TensionField (radius=8, gravity=2.0, magnetic=0) | Aliado que atrae particulas cercanas (campo de gravedad) |
| 4 | WaterTank | Blue | (0, 50) | StructuralLink → LightHealer | Tanque vinculado al healer; si se alejan, resorte los jala |
| 5 | LightHealer | Blue | (3, 50) | StructuralLink → WaterTank | Healer vinculado al tanque; comparten stress |
| 6 | PlantAssassin | Blue | (-30, -30) | Homeostasis (adapt_rate=10.0, cost=0.05, band=30) | Asesino emboscado en Umbra; adapta frecuencia al bioma |

### 5.2 — Grimoire del jugador (FireMage)

3 habilidades que ejercen distintos paths de la pipeline:

| Slot | Nombre | Tipo | Costo qe | Descripcion |
|------|--------|------|----------|-------------|
| 0 | **Fireball** | Projectile | 50 | Proyectil Ignis, freq forzada 450Hz, radio 0.5, vel (0, 15), despawn on contact, interference constructiva con Ignis |
| 1 | **Ember Shield** | SelfBuff | 30 | Aplica ResonanceLink: ConductivityMultiplier × 0.3 (reduce conductividad = menos transferencia termica) |
| 2 | **Lava Surge** | Projectile | 80 | Proyectil grande, freq 450Hz, radio 1.0, vel (0, 10), OnContactEffect: DissipationMultiplier × 3.0 al target |

### 5.3 — Biomas (presion ambiental L6)

| # | Tipo | Posicion | Radio | Capas extras | Proposito |
|---|------|----------|-------|-------------|-----------|
| 1 | Volcano | (40, 0) | 12.0 | — | Forge: alta energia, viscosidad, drena qe |
| 2 | Swamp | (-40, 0) | 10.0 | — | Oeste: alta viscosidad, movimiento lento |
| 3 | LeyLine | (-35, 35) | 8.0 | — | Sanctum: boost de energia, baja viscosidad |
| 4 | Tundra | (0, 40) | 10.0 | — | Norte: frio, alta coherencia, zona de tension |
| 5 | Desert | (20, -20) | 7.0 | — | Sur: baja energia, transicion suave |

### 5.4 — Cristales con StructuralLink (L13)

Par de cristales Terra en el nexo central, unidos por StructuralLink:

| # | Elemento | Posicion | qe | StructuralLink |
|---|----------|----------|-----|----------------|
| 1 | Terra | (-2, 0) | 300 | → Cristal 2, rest_length=4.0, stiffness=50.0, break_stress=500.0 |
| 2 | Terra | (2, 0) | 300 | → Cristal 1, rest_length=4.0, stiffness=50.0, break_stress=500.0 |

**Que observar:** Los cristales se mantienen a ~4 unidades de distancia. Si el jugador empuja uno, el otro se resiste (resorte). Si la energia de uno baja por catalisis destructiva, el stress puede romper el vinculo → evento `StructuralLinkBreakEvent`.

### 5.5 — Campos de tension (L11)

Entidades estáticas con TensionField en la Tension Arena:

| # | Nombre | Posicion | TensionField params | Proposito |
|---|--------|----------|---------------------|-----------|
| 1 | Gravity Well | (0, 45) | radius=12, gravity=5.0, magnetic=0, InverseSquare | Atrae entidades que entren al radio |
| 2 | Magnetic Trap | (-10, 40) | radius=8, gravity=0, magnetic=-3.0, InverseLinear | Repele entidades (trampa) |
| 3 | Vortex | (10, 40) | radius=10, gravity=3.0, magnetic=2.0, InverseSquare | Atrae + rota (gravedad + magnetismo combinados) |

**Que observar:** El heroe siente fuerzas al acercarse. Particulas arrastradas hacia los pozos. Velocidad del heroe cambia sin input.

### 5.6 — Entidades con Homeostasis (L12)

| # | Nombre | Posicion | Homeostasis params | Proposito |
|---|--------|----------|--------------------|-----------|
| 1 | Adaptive Sentinel | (-30, 30) | adapt_rate=8.0, cost=0.2, band=100, enabled=true | Adapta freq al nucleo Lux; pierde qe al hacerlo |
| 2 | Resistant Crystal | (-35, 40) | adapt_rate=2.0, cost=0.5, band=20, enabled=true | Adapta lento, costo alto, banda angosta |

**Que observar:** Frecuencia del sentinel sube gradualmente hacia 1000Hz (Lux). Su qe baja con cada adaptacion. El cristal resistente adapta mas lento pero con mayor costo.

### 5.7 — Dummies de prueba (Proving Lane — L8, L10)

Entidades estáticas (sin Will) que reciben proyectiles y efectos:

| # | Nombre | Posicion | qe | Elemento | Proposito |
|---|--------|----------|-----|----------|-----------|
| 1 | Target Dummy A | (0, -30) | 500 | Terra | Target de Fireball — interferencia destructiva (Ignis vs Terra) |
| 2 | Target Dummy B | (5, -30) | 500 | Ignis | Target de Fireball — interferencia constructiva (Ignis = Ignis) |
| 3 | Target Dummy C | (-5, -30) | 500 | Aqua | Target de Lava Surge — OnContactEffect aplica dissipation mult |

**Que observar:** Dummy A pierde qe rapido (destructiva). Dummy B gana qe (constructiva, resonancia). Dummy C pierde qe por disipacion amplificada.

### 5.8 — Entidades-efecto con ResonanceLink (L10)

Efectos pre-spawneados que modifican targets:

| # | Efecto | Target | ModifiedField | Magnitude | qe inicial |
|---|--------|--------|---------------|-----------|------------|
| 1 | Slow Field | Target Dummy A | VelocityMultiplier | 0.3 | 200 |
| 2 | Haste Field | Target Dummy B | VelocityMultiplier | 2.0 | 200 |
| 3 | Armor Buff | Cristal Terra 1 | BondEnergyMultiplier | 1.5 | 300 |

**Que observar:** Dummy A tiene velocity × 0.3 (slow visual en gizmo si se mueve). Dummy B tiene velocity × 2.0. Cristal Terra 1 resiste mas danio. Cuando la entidad-efecto se queda sin qe, el efecto desaparece.

### 5.9 — Lava Knights con StructuralLink (Volcano Pit — L13)

| # | Posicion | StructuralLink | Proposito |
|---|----------|----------------|-----------|
| 1 | (30, -30) | → Knight 2, rest=6.0, stiffness=80.0, break=400.0 | Par de guardias vinculados |
| 2 | (40, -30) | → Knight 1, rest=6.0, stiffness=80.0, break=400.0 | Si uno muere, stress → break event |

**Que observar:** Los knights mantienen distancia de ~6 unidades. Si uno pierde qe y muere (DeathEvent), el StructuralLink rompe y dispara StructuralLinkBreakEvent. El otro queda libre.

### 5.10 — Particulas ambiente

| Zona | Elemento | Cantidad | Proposito |
|------|----------|----------|-----------|
| Swamp (W) | Umbra | 5 | Particulas que pierden qe por disipacion del bioma |
| Forge (E) | Ignis | 4 | Particulas en estado Plasma |
| Tension Arena (N) | Ventus | 4 | Particulas que son atraidas por los pozos de gravedad |
| Proving Lane (S) | Aqua | 3 | Particulas que reaccionan con proyectiles Ignis |

---

## 6. Capas verificadas por entidad

| Entidad | L0 | L1 | L2 | L3 | L4 | L5 | L6 | L7 | L8 | L9 | L10 | L11 | L12 | L13 |
|---------|----|----|----|----|----|----|----|----|----|----|-----|-----|-----|-----|
| FireMage (player) | x | x | x | x | x | x | · | x | · | x | · | · | · | · |
| EarthWarrior | x | x | x | x | x | x | · | x | · | x | · | · | x | · |
| WindShooter | x | x | x | x | x | x | · | x | · | x | · | x | · | · |
| WaterTank | x | x | x | x | x | x | · | x | · | x | · | · | · | x |
| LightHealer | x | x | x | x | x | x | · | x | · | x | · | · | · | x |
| PlantAssassin | x | x | x | x | x | x | · | x | · | x | · | · | x | · |
| Biomas | x | x | x | x | x | · | x | · | · | · | · | · | · | · |
| Cristales (nexo) | x | x | x | x | x | x | · | · | · | · | · | · | · | x |
| Gravity Well | x | x | x | x | · | · | · | · | · | · | · | x | · | · |
| Adaptive Sentinel | x | x | x | x | x | · | · | · | · | · | · | · | x | · |
| Dummies | x | x | x | x | x | · | · | · | · | · | · | · | · | · |
| Efecto ResonanceLink | x | x | · | x | · | · | · | · | · | · | x | · | · | · |
| Lava Knights | x | x | x | x | x | x | x | x | x | · | · | · | · | x |
| Proyectiles (runtime) | x | x | x | x | · | · | · | · | x | · | · | · | · | · |
| Particulas | x | x | x | x | · | · | · | · | · | · | · | · | · | · |

**Cobertura: 14/14 capas activas en al menos 1 entidad.**

---

## 7. Sistemas verificados

| Sistema | Como se ejerce | Zona |
|---------|---------------|------|
| `will_input_system` | Player mueve con WASD/click | Todo el mapa |
| `engine_processing_system` | Buffer drena al moverse | Todo el mapa |
| `flow_integration_system` | Velocidad → posicion | Todo el mapa |
| `ambient_pressure_system` | Biomas inyectan/roban qe | Forge, Swamp, Sanctum |
| `catalysis_resolution_system` | Proyectiles impactan dummies | Proving Lane |
| `phase_transition_system` | Temperatura sube en Forge | Forge, Volcano Pit |
| `containment_system` | Entidades dentro de biomas | Swamp, Forge |
| `resonance_link_system` | Efectos modifican targets | Proving Lane, Nexus |
| `tension_field_system` | Pozos atraen/repelen | Tension Arena |
| `homeostasis_adapt_system` | Frecuencia se adapta | Ether Sanctum |
| `structural_runtime_system` | Resortes entre pares | Nexus, Volcano Pit |
| `eco_boundaries_system` | Zonas clasificadas | Todo (transiciones entre nuclei) |
| `worldgen_propagation_system` | Campo de energia se propaga | Warmup + runtime |
| `worldgen_materialization_system` | Entidades V7 aparecen | Todo el mapa |
| `worldgen_visual_system` | Colores/escala por freq | Todo el mapa |
| `collision_contact_system` | Esfera-esfera 3D | Todo (heroes vs biomas, vs cristales) |
| `spatial_index_update_system` | Indice espacial rebuilt | Todo el mapa |
| `dissipation_system` | qe baja por frame | Swamp, particulas Umbra |
| `death_system` | Entidades con qe=0 mueren | Particulas, dummies si les baja mucho |

---

## 8. Flujo de prueba (walkthrough)

### Paso 1 — Spawn y orientacion (~10 seg)
1. El jugador (FireMage Red) aparece en el Nexus (0, -5).
2. Camara orbital sigue al jugador.
3. Se ven los cristales Terra linkados (par central).
4. El suelo muestra el campo de energia V7: verde (Terra centro), naranja (Ignis este), oscuro (Umbra oeste).

### Paso 2 — Movimiento y biomas (~30 seg)
1. Moverse al **oeste (Swamp)**: velocidad baja, qe drena. Las particulas Umbra son pequenas y pierden energia.
2. Moverse al **este (Forge)**: velocidad normal pero el bioma inyecta calor. Si el heroe se queda, su temperatura sube → puede transicionar Solid→Liquid. Cristales Ignis brillan.
3. Volver al centro. El heroe recupera estado Solid.

### Paso 3 — Cristales vinculados (~15 seg)
1. Empujar un cristal Terra (colision) hacia un lado.
2. Observar que el otro cristal resiste (StructuralLink actua como resorte).
3. Observar el efecto Armor Buff (ResonanceLink L10) en el cristal 1: tiene bond_energy × 1.5.

### Paso 4 — Tension Arena (~20 seg)
1. Caminar al norte hacia la Tension Arena.
2. Acercarse al Gravity Well: el heroe es atraido hacia el centro (velocidad adicional en direccion al pozo).
3. Acercarse al Magnetic Trap: el heroe es repelido.
4. Observar particulas Ventus orbitando el Vortex.

### Paso 5 — Ether Sanctum (~15 seg)
1. Caminar al noroeste.
2. Observar el Adaptive Sentinel: su frecuencia sube gradualmente (label muestra Hz cambiando).
3. Observar que su qe baja (costo de adaptacion).
4. El EarthWarrior aliado (con Homeostasis) tambien adapta cuando entra a la zona.

### Paso 6 — Proving Lane (~30 seg)
1. Caminar al sur hacia los dummies.
2. **[Ability 0 — Fireball]:** Disparar hacia Dummy A (Terra) → interferencia destructiva, pierde qe.
3. **[Ability 0 — Fireball]:** Disparar hacia Dummy B (Ignis) → interferencia constructiva, gana qe (resonancia).
4. **[Ability 2 — Lava Surge]:** Disparar hacia Dummy C (Aqua) → OnContactEffect aplica dissipation × 3.0, drena rapido.
5. Observar ResonanceLink: Dummy A se mueve mas lento (slow × 0.3), Dummy B mas rapido (haste × 2.0).

### Paso 7 — Volcano Pit (~20 seg)
1. Caminar al sureste.
2. Observar los 2 Lava Knights vinculados por StructuralLink.
3. Si uno pierde suficiente qe (por catalisis o disipacion), muere → StructuralLinkBreakEvent → el otro queda libre.
4. Observar estado Plasma (mesh grande, color naranja).

---

## 9. Implementacion — Archivos a crear/modificar

| Archivo | Accion | Contenido |
|---------|--------|-----------|
| `assets/maps/proving_grounds.ron` | Crear | MapConfig con 7 nuclei (seccion 4) |
| `src/world/proving_grounds.rs` | Crear | `spawn_proving_grounds()`: ~60 entidades (seccion 5) |
| `src/world/mod.rs` | Modificar | Agregar `pub mod proving_grounds;` + re-exports |
| `src/plugins/debug_plugin.rs` | Modificar | Agregar branch `"proving_grounds"` en env var check |

### Funciones nuevas necesarias en `archetypes.rs` o `proving_grounds.rs`:

1. **`spawn_hero_with_layers`** — extension de `spawn_hero` que acepta opciones para L11 (TensionField), L12 (Homeostasis), L13 (StructuralLink). El builder YA soporta estas capas; solo hay que usarlas.

2. **`spawn_tension_entity`** — entidad estatica con TensionField (particula + L11). No existe actualmente.

3. **`spawn_adaptive_entity`** — entidad estatica con Homeostasis (particula + L12). No existe actualmente.

4. **`spawn_linked_pair`** — spawna 2 entidades con StructuralLink bidireccional. Retorna `(Entity, Entity)`.

5. **`spawn_effect_link`** — spawna entidad-efecto con ResonanceLink hacia un target. Usa `EffectConfig::spawn_components()` existente.

6. **`spawn_dummy`** — entidad con MatterCoherence pero sin Will ni Engine (target pasivo).

7. **`equip_grimoire`** — funcion que arma el Grimoire del FireMage con 3 AbilitySlots (Fireball, Ember Shield, Lava Surge). Usa `AbilitySlot` existente.

### Wiring de habilidades:

El Grimoire existe pero el input routing para **disparar abilities** necesita verificacion:
- `WillActuator` tiene `channeling_ability: Option<usize>` (slot index).
- `input_capture` setea `channeling_ability` cuando Space esta presionado.
- `pre_physics.rs` procesa el channeling y dispara `GrimoireProjectileCastPending` event.
- `reactions.rs` procesa el evento y llama `spawn_projectile`.

**Verificar que este path funciona end-to-end.** Si no, wiring minimo necesario (el slot ya existe).

---

## 10. Criterios de aceptacion

### Funcionales
- [ ] `RESONANCE_MAP=proving_grounds cargo run` arranca sin panic.
- [ ] El jugador se mueve con WASD y click-to-move.
- [ ] La camara sigue al FireMage.
- [ ] Los 7 nuclei generan campo de energia con colores distintos.
- [ ] Los biomas aplican presion ambiental (velocidad cambia en Swamp).
- [ ] Los cristales vinculados resisten separacion (StructuralLink).
- [ ] Los pozos de gravedad atraen al heroe.
- [ ] Las entidades con Homeostasis adaptan frecuencia (Hz visible en label).
- [ ] Los proyectiles (Fireball) impactan dummies y transfieren energia.
- [ ] Los efectos ResonanceLink modifican velocity de los dummies.
- [ ] Los Lava Knights mantienen distancia de ~6 unidades (StructuralLink).
- [ ] Las transiciones de fase son visibles en Forge (cambio de color del mesh).

### Tecnicos
- [ ] `cargo test` pasa (demo no rompe tests existentes).
- [ ] 14/14 capas ECS presentes en al menos 1 entidad spawneada.
- [ ] ~60 entidades en escena (no causa frame drops <30fps en release).
- [ ] Determinismo: misma seed → misma escena siempre.

### Observabilidad
- [ ] Debug gizmos muestran anillos por entidad con color elemental.
- [ ] Labels muestran symbol + Hz de cada entidad.
- [ ] Compound ring en compuestos (si los hay).
- [ ] Click-to-move marker verde en el suelo.
- [ ] Nucleus orbs visibles en las 7 posiciones de nuclei.

---

## 11. Entidades totales

| Tipo | Cantidad | Capas usadas |
|------|----------|-------------|
| Heroes | 6 | L0-L9, L11, L12, L13 (variado) |
| Biomas | 5 | L0-L6 |
| Cristales (nexo, linkados) | 2 | L0-L5, L13 |
| Cristales (forja, sanctum) | 4 | L0-L5 |
| Tension entities | 3 | L0-L3, L11 |
| Homeostasis entities | 2 | L0-L4, L12 |
| Dummies | 3 | L0-L4 |
| Efecto ResonanceLink | 3 | L0, L3, L10 |
| Lava Knights (linkados) | 2 | L0-L8, L13 |
| Particulas | 16 | L0-L3 |
| **Total** | **~46 spawned + V7 materialized** | **14/14 capas** |

---

## 12. Dependencias

- M1-M5 completados (estructura de carpetas actual).
- Toda la base code existente (14 capas, pipeline, worldgen, eco, topology, bridge).
- Verificar wiring de Grimoire → projectile spawn (puede necesitar fix minimo).

---

## 13. NO hace

- No implementa AI para heroes NPC (se quedan estaticos o con movimiento fijo).
- No implementa HUD de abilities/cooldowns (debug labels son suficientes para validacion).
- No implementa netcode.
- No agrega capas nuevas ni sistemas nuevos — solo usa lo que existe.
- No implementa animaciones ni efectos de particulas visuales.
- No modifica la pipeline de simulacion.
