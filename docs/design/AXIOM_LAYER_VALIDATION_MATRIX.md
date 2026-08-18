---
title: Axiom × Layer Validation Matrix
status: canonical
audience: cualquier agente que valide conformidad axiomática de una capa
ram_policy: static-first — nunca `cargo test` completo, nunca features GPU/bridge
---

# Matriz de Validación Axioma × Capa

Referencia canónica de **cómo se valida cada capa contra el core** (8 axiomas +
4 constantes fundamentales), y con qué método más barato se cierra cada par
**(Capa, Axioma)** sin reventar la RAM.

**Por qué existe:** la conformidad axiomática está dispersa (doc-comments que
citan "Ax N" en ~129 archivos, suite `Rn`, property tests, matriz regulatoria)
pero no había un mapa único capa→axioma→método→locus. Correr `cargo test`
completo linkea decenas de test-binaries contra Bevy y arrastra `wgpu`/`naga` →
pico de RAM inviable con múltiples agentes en paralelo. Esta matriz permite
validar una capa en minutos, mayormente sin compilar.

---

## 1. El core

**4 constantes fundamentales** (únicas entradas no derivables) —
`src/blueprint/equations/derived_thresholds.rs:14-34`:

| Constante | Valor | Axioma raíz |
|---|---|---|
| `KLEIBER_EXPONENT` | 0.75 | Ax4 (escalado alométrico) |
| `DISSIPATION_{SOLID,LIQUID,GAS,PLASMA}` | 0.005, 0.02, 0.08, 0.25 | Ax4 (2ª Ley) |
| `COHERENCE_BANDWIDTH` | 50.0 Hz | Ax8 (ventana de interferencia) |
| `DENSITY_SCALE` | 20.0 | normalización de grid |

Todo threshold del motor es una `pub fn` pura que combina algebraicamente estas
4 (ver `derived_thresholds.rs`, 40 derivaciones + 17 tests de invariante en
:369-476). **Calibrables:** sólo `COHERENCE_BANDWIDTH` y `DENSITY_SCALE`.

**8 axiomas** (`CLAUDE.md:26-42`, `docs/ARCHITECTURE.md:85-107`):

| # | Axioma | Tipo | Fórmula/locus canónico |
|---|--------|------|------------------------|
| 1 | Everything is Energy | primitivo | `core_physics::density_from_qe_radius` |
| 2 | Pool Invariant `Σ(children) ≤ parent` | primitivo | `conservation.rs`, `energy_competition/pool_equations.rs` |
| 3 | Competition | derivado ←8 | `energy_competition/*` |
| 4 | Dissipation `> 0` | primitivo | `DISSIPATION_*`, `core_physics` (drag) |
| 5 | Conservation | derivado ←2+4 | `conservation.rs`, `property_conservation.rs` |
| 6 | Emergence at Scale | derivado | `emergence/*` (guard-rail) + **SYNC-analytic** (referencia, ADR-046) + **SYNC-ECS = primer par medido sobre el motor real** (L2×N, ADR-047) (§5) |
| 7 | Distance Attenuation (monótona ↓) | primitivo | `signal_propagation.rs`, `radial_field.rs` |
| 8 | Oscillatory `cos(Δf·t+Δφ)` | primitivo | `core_physics::interference`, `exp(-Δf²/2B²)` |

---

## 2. Taxonomía de validación (el *tipo*)

Cada par (Capa, Axioma) es de uno de 5 tipos. El tipo dicta el **método más
barato que lo cierra**:

| Tipo | Significa | Método (más barato) | Costo RAM |
|------|-----------|---------------------|-----------|
| **N/A** | El axioma no gobierna esta capa | marcar N/A + porqué | 0 |
| **Estructural** (E) | Invariante codificada en el tipo/constructor (clamp, `pub(crate)`, enum exhaustivo) | leer componente + confirmar clamp/getter | 0 (estático) |
| **Derivacional** (D) | Consume constantes que deben derivar de las 4 fundamentales, sin números mágicos | grep de literales + trazar símbolo a `derived_thresholds.rs`/`constants/*` | 0 (estático) |
| **Comportamental** (C) | Invariante sólo se sostiene en la transformación (system o fn pura) | `cargo test --lib <módulo de equations>` | Bajo (lib, sin runtime) |
| **Compositional** (K) | Requiere componer 2+ axiomas (cross-layer, AXIOMATIC_CLOSURE) | `cargo test --test <suite> <mod>::` puntual (último recurso) | Medio |

**Regla de oro:** subir un escalón sólo cuando el anterior no puede cerrar el
par. La mayoría se cierran en E/D = costo cero.

---

## 3. Escalera de ejecución (barato → caro, techo de RAM controlado)

| Paso | Acción | Cierra | RAM |
|------|--------|--------|-----|
| 0 | **Lectura estática** — doc-comment + definición + clamps/getters | pares E | 0 |
| 1 | **Auditoría por grep** — invariantes negativas (ver abajo) | pares D + Hard Blocks | 0 |
| 2 | **`cargo check --lib`** — type-check, un build compartido, sin test-binaries ni wgpu | "compila con la forma esperada" | Bajo |
| 3 | **`cargo test --lib <ruta::módulo>`** — unit tests de math pura de un módulo | pares C | Bajo |
| 4 | **`cargo test --test <suite> <mod>::`** (excepcional, un módulo a la vez, `PROPTEST_CASES=8`) | pares K | Medio |

**Paso 1 — grep de invariantes negativas** (sin compilar):
- `NO números mágicos`: literal float en `layers/` que no cite su derivación.
- `NO unwrap()/expect()/panic!()` en systems (Hard Block 11).
- `NO HashMap` en hot path, `NO String` en components (Hard Blocks 6-7).
- `clamp presente` donde el doc declara invariante (`qe>=0`, `∈[0,1]`).

**Prohibido siempre:** `cargo test` sin filtro · `--features gpu_cell_field_snapshot`
· `--features bridge_optimizer` · `--features experimental_bins` (arrastran wgpu /
bins pesados / decenas de test-binaries linkeados = pico de RAM).

---

## 4. Matriz maestra

**Leyenda de celda:** `<tipo> <estado>` — tipo ∈ {E,D,C,K,N/A}; estado ∈
{`PASS`, `WARN`, `GAP`, `·`=N/A}. Loci `file:line` en las fichas §5.

| Capa \ Axioma | 1 Energy | 2 Pool | 3 Comp | 4 Dissip | 5 Consv | 6 Emerg | 7 Dist | 8 Oscil |
|---|---|---|---|---|---|---|---|---|
| **L0** BaseEnergy | E PASS | C PASS | · | C PASS | C PASS | · | · | · |
| **L1** SpatialVolume | E PASS | · | · | · | · | · | · | · |
| **L2** OscillatorySignature | · | · | C PASS | · | · | · | · | E PASS |
| **L3** FlowVector | · | · | · | E PASS · D PASS | · | · | · | · |
| **L4** MatterCoherence | E PASS | · | · | D PASS | · | · | · | K — |
| **L5** AlchemicalEngine | E PASS | E PASS | C PASS | C PASS | · | · | · | C PASS |
| **L6** AmbientPressure | C PASS | · | · | D PASS | · | · | · | · |
| **L7** WillActuator | · | · | · | · | · | E PASS | · | · |
| **L8** AlchemicalInjector | E PASS | · | · | · | · | · | E PASS · C — | E PASS |
| **L9** MobaIdentity | · | · | E WARN | · | · | E PASS | · | · |
| **L10** ResonanceLink | · | E PASS | · | · | E PASS | · | · | · |
| **L11** TensionField | · | · | · | · | · | · | E PASS · C PASS | · |
| **L12** Homeostasis | · | C PASS | · | E PASS | E PASS | · | · | C PASS |
| **L13** StructuralLink | · | · | · | · | · | · | E PASS | · |
| **AP** SpeciesGrid+ReactionNetwork | E PASS | C PASS | · | D PASS | C PASS | · | · | D PASS |
| **AI** species_to_qe + fission bridge | C PASS | · | · | · | C PASS | · | D PASS | D PASS |
| **SYNC-analytic** Kuramoto mean-field (referencia, sin ECS) | · | · | · | · | · | C PASS | · | C PASS |
| **SYNC-ECS** `entrainment_system` (L2×N, motor real) | · | · | · | · | · | K PASS | K — | K PASS |

**Resumen (tras correcciones + prueba Ax6 analítica y sobre el motor):** 46 pares
gobernados (celdas no-N/A) — 43 `PASS`, 1 `WARN` (L9·Ax3, calibración de gameplay,
aceptada por diseño), 0 `GAP`, 2 compositional diferidos (L4·Ax8 y SYNC-ECS·Ax7,
`K —`). Los 3 GAP y el WARN de L11 fueron resueltos (§6).

**Por qué la fila SYNC se partió en dos (ADR-047):** la fila única
`SYNC Kuramoto order param (L2×N) · Ax6 C PASS` era un **sobreclaim**: el
experimento de ADR-046 es un mean-field de fase analítico que no instancia
ninguna entidad — L2 no participa. Ahora `SYNC-analytic` declara lo que
realmente prueba (que el modelo de Kuramoto sincroniza, referencia de 1975) y
`SYNC-ECS` carga el par (L2, Ax6) **medido sobre el `entrainment_system` real**
con dos ablaciones causales. Nota fuera de celda: campos `pub` en L8 (§6.3).

---

## 5. Fichas por capa

Formato: **invariante declarada · locus del clamp/derivación · comando que la
valida · hallazgo**. El estado `PENDING` se resuelve con Fase B (pasos 2-3).

### L0 · BaseEnergy — `src/layers/energy.rs`
- **Invariante:** `qe >= 0.0 (clamped in every system)` (doc `energy.rs:17`).
- **Ax1 (E PASS):** `new` → `qe.max(0.0)` (`:40`); `drain` acota a `self.qe` (`:44-48`); `is_dead` en `qe<=0` (`:55-57`).
- **Ax2/5 (C PASS):** conservación de pool en `conservation.rs`; validada por `property_conservation.rs`, `r1_conservation.rs`.
- **Ax4 (C PASS):** el drenaje basal (`basal_drain_rate()=1.0`, `derived_thresholds.rs:44-48`) actúa sobre L0; la disipación vive en la fase metabólica.
- **Comando:** `cargo test --lib blueprint::equations::conservation`.

### L1 · SpatialVolume — `src/layers/volume.rs`
- **Invariante:** `densidad = qe/((4/3)πr³)` computada, no almacenada (doc `:13`); `radius >= VOLUME_MIN_RADIUS`.
- **Ax1 (E PASS):** clamp en `new` (`:32`) y `set_radius` (`:51`); `density()` delega a `equations::density` (`:45`) — sin literales.
- **Comando:** lectura estática (paso 0). Sin math pura propia que testear aquí.

### L2 · OscillatorySignature — `src/layers/oscillatory.rs`
- **Invariante:** portadora de Ax8; doc cita `I(a,b)=cos(2π|f_a−f_b|t+(φ_a−φ_b)) → [-1,1]`.
- **Ax8 (E PASS):** frecuencia+fase son el sustrato; la interferencia se computa en `core_physics::interference`.
- **Ax3 (C PASS):** alineación de frecuencia modula transferencia (Ax3←Ax8) en `energy_competition/*`.
- **Ax6 (celda en la fila SYNC-ECS):** Ax6 es una propiedad **de escala**, no de la
  capa aislada — un solo `OscillatorySignature` no tiene "sincronización". El par
  (L2, Ax6) se mide con N=64 instancias en `tests/emergence_ecs.rs`; ver ficha
  SYNC-ECS. Por eso no hay celda Ax6 en esta fila (evita doble conteo).
- **Comando:** `cargo test --lib blueprint::equations::core_physics`.

### L3 · FlowVector — `src/layers/flow.rs`
- **Invariante:** 2ª Ley nombrada (doc `:6-12`): `qe -= (dissipation + friction·|v|²)·dt`.
- **Ax4 (E PASS):** `dissipation_rate.max(0.0)` en `new` (`:36`) y setter (`:70`); guarda velocidad no-finita → `Vec2::ZERO` (`:48`).
- **Ax4 (D GAP):** `DEFAULT_DISSIPATION_RATE = 5.0` (`layer03_flow_dissipation.rs:3`) es un literal **calibrado, NO derivado** de la familia `DISSIPATION_*` (0.005/0.02/0.08/0.25) — confirmado por grep (Fase B). Rompe el principio "todo deriva de las 4 fundamentales". Ver §6.
- **Comando:** confirmado estáticamente; no requiere compilar.

### L4 · MatterCoherence — `src/layers/coherence.rs`
- **Invariante:** "si energía > enlace, muta de fase"; `bond>=0`, `conductivity∈[0,1]`.
- **Ax1 (E PASS):** clamps en constructor; `state` reusa enum canónico `MatterState` (`domain_enums.rs`).
- **Ax4 (D PASS):** el estado mapea a tasa de disipación vía umbrales derivados (`derived_thresholds.rs:52-70`, `liquid/gas/plasma_density_threshold`).
- **Ax8 (K —):** acoplamiento coherencia↔frecuencia es compositional (AXIOMATIC_CLOSURE AC-4). Fuera del alcance de la capa; validar sólo si se toca `culture.rs`/`entrainment.rs`.
- **Comando:** `cargo test --lib blueprint::equations::derived_thresholds`.

### L5 · AlchemicalEngine — `src/layers/engine.rs`
- **Invariante:** capacitor L0→L8; `buffer∈[0,max]`; eficiencia `<1`.
- **Ax1/2 (E PASS):** `current_buffer: initial.clamp(0.0, max_buffer)` (`:47`); `free_space().max(0.0)` (`:90`).
- **Ax4 (C PASS):** `creation_efficiency` → `.clamp(0.0,1.0)` (`:227`) — sin 100% de eficiencia (Ax4); usa `ENGINE_EFFICIENCY_FALLOFF`.
- **Ax8 (C PASS):** eficiencia modulada por frecuencia (`ENGINE_EFFICIENCY_FREQ_DIVISOR`).
- **Comando:** `cargo test --lib blueprint::equations::engine` (o el módulo que aloje `engine_intake_allometric`).

### L6 · AmbientPressure — `src/layers/pressure.rs`
- **Invariante:** inyecta/drena qe por bioma; disipación `>0` incluso en vacío.
- **Ax4 (D PASS):** **único archivo de `layers/` que cita "Axiom 4" por número** (`:72`, `:78`); `vacuum()` usa `DISSIPATION_SOLID` (import de `derived_thresholds`, `:75`) con comentario de derivación.
- **Ax1 (C PASS):** `delta_qe_constant` **deliberadamente sin clamp** (`:38-41`) — biomas drenan (negativo permitido). `viscosity.max(0.0)` (`:39`).
- **Comando:** lectura estática (paso 0); confirmación `DISSIPATION_SOLID` en `derived_thresholds.rs:24`.

### L7 · WillActuator — `src/layers/will.rs`
- **Invariante:** traduce input → drenaje de L5 → vectores en L3. Capa de gameplay/input.
- **Ax6 (E PASS):** no viola emergencia bottom-up: sólo actúa sobre L5/L3, no programa comportamiento top-down. Guards de finitud (`:51`, `:79`), umbral por constante nombrada.
- **Axiomas físicos: N/A** — no produce/consume qe directamente ni tiene distancia/frecuencia propias.
- **Comando:** lectura estática (paso 0).

### L8 · AlchemicalInjector — `src/layers/injector.rs`
- **Invariante:** spell payload; `projected_qe>=0`, `radius>=min`.
- **Ax1/8 (E PASS):** clamps en `new` — `projected_qe.max(0.0)` (`:40`), `forced_frequency.max(0.0)` (`:41`), `influence_radius.max(INJECTOR_MIN_INFLUENCE_RADIUS)` (`:42`).
- **Ax7 (E PASS · C —):** `influence_radius` acota el efecto; la **atenuación monótona** con distancia se computa en el system (`radial_field.rs`), no aquí → validar Ax7 comportamental si se toca la inyección.
- **Hallazgo (WARN):** campos `pub`, sin setters — la invariante sólo se garantiza en construcción; mutación directa post-`new` puede violar `>=0`. Ver §6.
- **Comando:** `cargo test --lib blueprint::equations::radial_field`.

### L9 · MobaIdentity — `src/layers/identity.rs`
- **Invariante:** capa de gameplay sobre la física (doc `:50-54`); `critical_multiplier>=0`.
- **Ax3 (E WARN):** `set_critical_multiplier` clampa `>=0` (`:107`); pero `FACTION_ALLY_BONUS=0.2`/`FACTION_ENEMY_MALUS=-0.2` (`layer01_faction.rs:3,6`) son **calibrados, no derivados** de las 4 fundamentales. Aceptable por diseño (capa de gameplay), pero fuera del cierre axiomático. Ver §6.
- **Axiomas físicos: N/A.**
- **Comando:** grep `FACTION_*` en `blueprint/constants/`.

### L10 · ResonanceLink — `src/layers/link.rs`
- **Invariante:** buff/debuff vida-ligado; modifica un campo del target mientras la fuente viva.
- **Ax2/5 (C GAP):** **no hay constructor ni setter con clamp** (`link.rs:28-38`) — `magnitude` no se valida; un buff podría crear qe si el system consumidor no acota. La conservación depende del consumidor, no del tipo. Ver §6.
- **Comando:** auditar el system que aplica `ResonanceLink` (¿acota el efecto a la conservación?); si hay fn pura, `cargo test --lib` de ella.

### L11 · TensionField — `src/layers/tension_field.rs`
- **Invariante:** fuerza a distancia (gravedad/magnetismo); falloff `InverseSquare`/`InverseLinear`.
- **Ax7 (E PASS):** `enum FieldFalloffMode` (`:6-9`) exhaustivo; ambos modos teóricamente decrecientes; `radius.max(0.0)` en `new` (`:30`).
- **Ax7 (C WARN):** **este archivo sólo declara el enum, no computa el falloff** — la monotonicidad decreciente no es verificable aquí; vive en `simulation/thermodynamic/structural_runtime.rs` (locus confirmado, Fase B). `gravity_gain`/`magnetic_gain` sin clamp. Doc **no cita Ax7**. Ver §6.
- **Comando:** `cargo test --lib thermodynamic::structural_runtime` (si aloja la fn de decaimiento).

### L12 · Homeostasis — `src/layers/homeostasis.rs`
- **Invariante:** adaptación frecuencial con costo energético (no hay adaptación gratis).
- **Ax4/5 (E PASS):** los tres `f32` clampados `>=0` en `new` (`:23-25`) — `adapt_rate`, `qe_cost_per_hz`, `stability_band`.
- **Ax8 (C PASS):** adapta frecuencia hacia target dentro de la banda; validar en el system de homeostasis.
- **Comando:** `cargo test --lib blueprint::equations::homeostasis`.

### L13 · StructuralLink — `src/layers/structural_link.rs`
- **Invariante:** resorte entre nodos; `rest_length/stiffness/break_stress >= 0`.
- **Ax7 (E PASS):** los tres clampados `>=0` en `new` (`:24-26`); fuerza de Hooke ∝ desplazamiento (monótona).
- **Comando:** lectura estática (paso 0); si hay fn pura de fuerza, `cargo test --lib`.

### AP · SpeciesGrid + ReactionNetwork — `src/layers/{species_grid,reaction_network}.rs`
- **Invariante:** química explícita mass-action (Resources, opt-in); estequiometría conserva masa.
- **Ax8 (E/D PASS):** `species_grid.rs:7` cita Ax8; `REACTION_FREQ_BANDWIDTH_DEFAULT = COHERENCE_BANDWIDTH` (`chemistry.rs:44`).
- **Ax4 (D PASS):** `REACTION_EFFICIENCY = 1 - DISSIPATION_LIQUID` (`chemistry.rs:34`); `SPECIES_DIFFUSION_RATE = DISSIPATION_LIQUID` (`:37`, Ax7).
- **Ax5 (C PASS):** conservación global de qe validada por `tests/chemistry_equivalence.rs`; `seed` clampa `max(0.0)` (`species_grid.rs:118`).
- **Ax2 (C PASS):** `from_spec`/`from_reactions` validan `MAX_*` y `is_well_formed`.
- **Comando:** `cargo test --test axioms chemistry_equivalence::` (paso 4, sólo si es necesario).

### AI · species_to_qe + autopoiesis_bridge — `src/simulation/{species_to_qe,autopoiesis_bridge}.rs`
- **Invariante:** puente AP↔ECS; conserva energía al inyectar qe y al spawnear hijos.
- **Ax8/7 (D PASS):** `SPECIES_TO_QE_COUPLING = DISSIPATION_LIQUID` (`chemistry.rs:110`, test `:149-153`); alignment Ax8 citado (`species_to_qe.rs:6`).
- **Ax5 (C PASS):** cada hijo recibe `BaseEnergy::new(ev.qe_per_child)` (`autopoiesis_bridge.rs:140`); guard `qe_per_child<=0 → skip` (`:136`); cap `MAX_FISSION_EVENTS_PER_TICK=4` (`:33`, aplicado `:76`); test verifica `Σqe = 2×qe_per_child` (`:272`).
- **Ax1 (C PASS):** inyección `dqe = cell_qe·alignment·SPECIES_TO_QE_COUPLING·dt` con guards de finitud (`species_to_qe.rs:47,58`).
- **Comando:** `cargo test --lib simulation::species_to_qe` + `cargo test --lib simulation::autopoiesis_bridge`.

### SYNC-analytic · Kuramoto mean-field de fase — `equations/emergence/synchronization.rs` + `use_cases/experiments/emergence_sync.rs`
- **Alcance honesto:** mean-field **analítico**, all-to-all, sin ECS. No instancia
  entidades: **L2 no participa**. Es la *referencia* contra la cual se compara el
  motor, no una propiedad medida de RESONANCE. Ver ADR-046 §10.
- **Invariante:** el orden global (sincronía de fase) es **consecuencia de la regla
  local** Kuramoto sobre N−1 vecinos; sin la regla, no hay orden (Ax6 bottom-up, no
  top-down).
- **Ax6 (C PASS):** order parameter `R = |(1/N)·Σ e^{iθ}| ∈ [0,1]` sube de ~0 a
  `> SYNC_R_PASS_MIN` (0.5) con acoplamiento; la **ablación causal** (`coupling = 0`
  sobre el mismo estado inicial) lo colapsa a `< SYNC_ABLATED_R_MAX` (0.1); gap
  `> SYNC_GAP_MIN` (0.4). Medido: `R = 0.9619` vs `0.0560`, gap `0.9059` (media de
  6 seeds; el ablado se sienta sobre el piso de N finito `1/√256 = 0.0625`).
  `R` es **observación pasiva**: no altera estado, sólo mide.
- **Ax8 (C PASS):** `R = |Σ e^{iθ}|` usa la misma álgebra `cos/sin` del sustrato
  oscilatorio; el acoplamiento es consecuencia de Ax8 (entrainment AC-2).
- **Comando:** `cargo test --lib blueprint::equations::emergence::synchronization` · `cargo test --test axioms emergence_sync::` · `cargo test --test axioms r10_emergence_gates::`.

### SYNC-ECS · `entrainment_system` sobre L2×N — `simulation/emergence/entrainment.rs` + `tests/emergence_ecs.rs`
- **Alcance:** corre **el system real** (la misma fn que registra `AtomicPlugin`)
  sobre 64 entidades con `OscillatorySignature` y `SpatialIndex` reales, schedule
  `Update` mínimo, headless. Éste es el par (L2, Ax6) medido. Ver ADR-047 §10.
- **Invariante:** el motor acopla **frecuencia**, no fase → la métrica es el colapso
  de la dispersión `S = 1 − σ_ω(T)/σ_ω(0) ∈ [0,1]`, no el order parameter de fase.
- **Ax6 (K PASS):** `S = 0.847` medio (mín 0.788) `≥ SYNC_ECS_S_PASS_MIN` (0.5) en
  5/5 seeds. **Dos** ablaciones causales dan `S == 0` **exacto** (identidad bit a
  bit de ω(0) vs ω(T) vía `hash_f32_slice`): A1 saca el system del schedule
  (contrafáctico N−1→N puro), A2 lo deja ACTIVO con la población a
  `3 × ENTRAINMENT_SCAN_RADIUS` (vecindad vacía → el orden exige interacción
  efectiva, no la mera presencia del código).
- **Ax8 (K PASS):** el sustrato es L2 real y la regla bajo prueba **es** la
  consecuencia AC-2 de Ax8; el estado final llega a `lock fraction = 1.0000`
  medido con la propia `entrainment_lock_achieved` del motor.
- **Ax7 (K —, parcial por diseño):** A2 ejercita el **alcance acotado** real (cutoff
  duro del scan radius). La **atenuación continua** (`entrainment_coupling_at_distance`)
  NO se mide aquí — con el decay solo el sistema aún convergería (`K_eff(36) ≈ 0.0075`).
  Un barrido S(spacing) intra-rango queda como extensión (ADR-047 §9). Ax7 ya está
  cerrado en L8/L11/L13.
- **Comando:** `cargo test --test axioms emergence_ecs::` (1.2 s, sin GPU) · `cargo test --test axioms r10_emergence_gates::`.

---

## 6. Hallazgos (auditoría estática) y correcciones

Los 3 GAP y el WARN de L11 fueron **corregidos** de forma que respeta arquitectura
(clamp en constructor patrón L0/L3, derivación desde fundamentales, math pura en
`equations/`) y sin cambiar la dinámica de simulación. `[RESUELTO]` marca lo hecho.

1. **L10 ResonanceLink — conservación no forzada en el tipo (C GAP → E PASS). `[RESUELTO]`**
   `magnitude` pasó a `pub(crate)` con `new()`/`set_magnitude()` que clampan a `>= 0`
   (`link.rs:37-70`) — un multiplicador negativo sobre disipación/energía crearía qe
   (Ax4/Ax5). Sitios de construcción ruteados por `new()`: `composition.rs:122`,
   `observers.rs:157`; lecturas por getter `magnitude()`: `pre_physics.rs:239-274`.
   Tests nuevos: `new_clamps_negative_magnitude_to_zero`, `set_magnitude_clamps_and_is_idempotent`.

2. **L11 TensionField — Ax7 no verificable en la capa (C WARN → C PASS). `[RESUELTO]`**
   El falloff ya era fn pura (`equations/field_body/mod.rs:9`, respeta arquitectura);
   se le añadió doc citando Ax7 y el test `falloff_is_monotonically_decreasing` (barrido
   0.5→50, ambos modos) + `falloff_softens_singularity_at_zero`. Doc de `TensionField`
   ahora cita Ax7 y documenta que los `*_gain` van sin clamp **a propósito** (negativo =
   repulsión); el radio sí se clampa (`>= 0`).

3. **L8 AlchemicalInjector — invariante sólo en construcción (WARN, PENDIENTE).**
   Campos `pub`, sin setters; mutación directa post-`new` puede romper `projected_qe>=0`.
   No es un GAP axiomático (el clamp existe en `new`), sólo un endurecimiento. **Acción
   sugerida:** `pub(crate)` + getters + setters con clamp (mismo patrón que L10, arriba).

4. **L3 FlowVector — `DEFAULT_DISSIPATION_RATE` no derivaba (D GAP → D PASS). `[RESUELTO]`**
   Reescrito como `DENSITY_SCALE * DISSIPATION_PLASMA` (`layer03_flow_dissipation.rs:2-11`)
   = `20.0 × 0.25 = 5.0` **exacto** (cero cambio de comportamiento). Historia Ax4: el qe en
   vacío fluye desligado (régimen no-ligado tipo plasma), su pérdida entrópica usa el
   coeficiente plasma a escala de grid. Ya no es un literal suelto.

5. **L9 MobaIdentity — bonuses de facción calibrados, no derivados (E WARN, aceptado).**
   `FACTION_ALLY_BONUS=0.2`, `FACTION_ENEMY_MALUS=-0.2` (`layer01_faction.rs:3,6`) fuera del
   cierre axiomático. **Aceptable por diseño** (capa de gameplay sobre la física, `CLAUDE.md`
   L9). Se deja como excepción consciente, no como GAP.

6. **Citación de axiomas en doc-comments (parcial).** Se añadió cita de Ax7 al doc de
   `TensionField` y de `safe_falloff`, y de Ax4 al de `DEFAULT_DISSIPATION_RATE`. Mejora
   barata restante (opcional): `// Ax N` en L0 (Ax1), L2 (Ax8), L4 (Ax4).

---

## 7. Apéndice: comandos por capa (copy-paste, RAM-safe)

```bash
# Paso 2 — un solo build compartido (hazlo una vez por sesión)
cargo check --lib

# Paso 3 — math pura por capa (sin Bevy runtime, sin GPU)
cargo test --lib blueprint::equations::conservation        # L0 Ax2/5, AP Ax5
cargo test --lib blueprint::equations::core_physics        # L2 Ax8, L0 Ax1
cargo test --lib blueprint::equations::derived_thresholds  # L4 Ax4, thresholds
cargo test --lib blueprint::equations::radial_field        # L8 Ax7
cargo test --lib blueprint::equations::homeostasis         # L12 Ax8
cargo test --lib simulation::species_to_qe                 # AI Ax7/8
cargo test --lib simulation::autopoiesis_bridge            # AI Ax5
cargo test --lib blueprint::equations::emergence::synchronization  # SYNC Ax6 (métrica)

# Paso 4 — SÓLO si un par Compositional lo exige (un módulo a la vez)
# Las 4 suites (ADR-048): axioms · probes · pipeline · platform
PROPTEST_CASES=8 cargo test --test axioms property_conservation::
PROPTEST_CASES=8 cargo test --test axioms chemistry_equivalence::
cargo test --test axioms r1_conservation::
cargo test --test axioms emergence_ecs::        # SYNC-ECS: L2×Ax6 sobre el motor (1.2 s)
cargo test --test axioms emergence_sync::       # SYNC-analytic: referencia Kuramoto
cargo test --test axioms r10_emergence_gates::  # gates SYNC_* + SYNC_ECS_*

# Bajar pico de RAM si hace falta
cargo test --lib <módulo> --jobs 1 -- --test-threads=1

# PROHIBIDO: cargo test (sin filtro) · --features gpu_cell_field_snapshot
#            · --features bridge_optimizer · --features experimental_bins
```

**Contrato de costo:** la validación completa de cualquier capa por esta escalera
**nunca pasa del paso 3** (lib), evitando el linkeo de test-binaries de
integración contra Bevy. El paso 4 es excepcional y de un solo módulo; desde
ADR-048 cuesta 1-de-4 binarios posibles (antes 1-de-37).
