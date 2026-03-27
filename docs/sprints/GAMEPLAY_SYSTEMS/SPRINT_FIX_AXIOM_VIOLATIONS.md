# SPRINT: Fix — Violaciones de Axiomas y Comportamiento

**Track:** Gameplay Systems (GS)
**Fecha:** 2026-03-26
**Origen:** Audit completo del codebase contra CLAUDE.md (Hard Blocks + Coding Rules) + revisiones de composicion de entidades + cobertura de tests de capas.

---

## Objetivo

Corregir todas las violaciones detectadas en audit, clasificadas por severidad.
Cada ticket es autocontenido: archivo, linea, violacion exacta, fix propuesto.

---

## Clasificacion

### BLOCK (must fix before next merge)

| ID | Tipo | Archivo | Descripcion |
|----|------|---------|-------------|
| AX-FIELDS-1 | Axioma | `src/layers/metabolic_graph.rs:61-67` | `MetabolicGraph` tiene 5 campos (limite: 4) |

**AX-FIELDS-1** — MetabolicGraph viola Coding Rule 2 ("Max 4 fields per component").

Campos actuales:
```
nodes:              [ExergyNode; MAX_NODES]
nodes_len:          u8
edges:              [ExergyEdge; MAX_EDGES]
edges_len:          u8
total_entropy_rate: f32          // <-- campo 5
```

Fix propuesto: Extraer `total_entropy_rate` a un componente auxiliar `EntropyLedger` (SparseSet, co-localizado con MetabolicGraph). Alternativa: wrappear `nodes`+`nodes_len` en un sub-struct `FixedVec<ExergyNode, N>` que cuente como un solo campo semantico — pero esto requiere validar que Reflect + Copy siguen funcionando.

---

### WARN (fix in current sprint)

| ID | Tipo | Archivo | Descripcion |
|----|------|---------|-------------|
| AX-UNSAFE-1 | Axioma | `src/worldgen/cell_field_snapshot/gpu_layout.rs:140-143` | `unsafe impl bytemuck::Pod/Zeroable` para structs GPU |
| AX-UNSAFE-2 | Axioma | `src/worldgen/map_config.rs:474-485` | `unsafe { std::env::set_var() }` en tests |
| IMPL-LAYER-1 | Test gap | `src/layers/link.rs` | L10 ResonanceLink sin test de conservacion de qe |
| IMPL-LAYER-2 | Test gap | `src/layers/tension_field.rs` | L11 TensionField sin test de simetria Newton-3 |
| IMPL-LAYER-3 | Test gap | `src/layers/structural_link.rs` | L13 StructuralLink stress sin limite documentado / test de overflow |
| IMPL-SYSTEM-1 | Inline math | `src/simulation/thermodynamic/osmosis.rs` | Auditar delegacion completa a `blueprint/equations/` |
| IMPL-SYSTEM-2 | Phase scope | `src/plugins/worldgen_plugin.rs:67-69` | WorldgenPhase embebida dentro de Phase::ThermodynamicLayer |
| ENT-2 | Composicion | `src/entities/archetypes/catalog.rs` (spawn_animal) | Animal no tiene L6 AmbientPressure — terreno no afecta movimiento |
| ENT-3 | Composicion | `src/entities/archetypes/heroes.rs` (spawn_hero) | Hero no tiene L6 AmbientPressure por defecto |
| ENT-4 | Composicion | `src/entities/archetypes/flora.rs` | Flora (EA2: Rosa, Roble, Musgo) no tiene L12 Homeostasis — frecuencia fija |
| BEH-1 | Comportamiento | `src/simulation/behavior.rs:300-338` | `behavior_will_bridge_system` retorna `Vec2::ZERO` si target invalido — sin fallback activo |
| BEH-2 | Comportamiento | `src/simulation/metabolic/trophic.rs` | Trophic consumers sin deteccion de starvation prolongada |
| BEH-3 | Performance | `src/simulation/metabolic/social_communication.rs` | Pack formation sin `PACK_MAX_SIZE` — riesgo O(n^2) |

---

### INFO (document decision or defer)

| ID | Tipo | Archivo | Descripcion |
|----|------|---------|-------------|
| ENT-1 | Composicion | `src/entities/archetypes/catalog.rs` (spawn_virus) | Virus no tiene L12 Homeostasis — decision intencional (capside rigida) |
| DOC-1 | Docs | `docs/arquitectura/blueprint_v6.md` | Dice "17 sub-modulos" en runtime_platform pero hay 18 |
| DOC-2 | Docs | `docs/design/BLUEPRINT.md` | Usa `capas/` (legacy) en module map; debe ser `layers/` |
| DOC-3 | Docs | `docs/arquitectura/` | Docs redundantes: `blueprint_visual_quantization.md`, `blueprint_quantized_color.md`, `blueprint_sim_layers.md` |

---

## Detalle por ticket

### AX-UNSAFE-1: unsafe en gpu_layout.rs

**Violacion:** Hard Block 1 — "NO unsafe blocks — zero tolerance."
**Archivo:** `src/worldgen/cell_field_snapshot/gpu_layout.rs:140-143`

```rust
unsafe impl bytemuck::Pod for GpuCellFieldSnapshotHeader {}
unsafe impl bytemuck::Zeroable for GpuCellFieldSnapshotHeader {}
unsafe impl bytemuck::Pod for GpuCellFieldPacked {}
unsafe impl bytemuck::Zeroable for GpuCellFieldPacked {}
```

**Contexto:** Necesario para layout GPU (bytemuck exige unsafe impl si no se usa derive). El codigo es correcto — los structs son `#[repr(C)]` con campos Pod y padding explicito `_pad = 0`.

**Fix propuesto:**
1. Preferido: Reemplazar con `#[derive(bytemuck::Pod, bytemuck::Zeroable)]` + `#[repr(C)]`. Requiere verificar que bytemuck en Cargo.toml soporta derives (feature `derive`).
2. Alternativa: Si el derive no es posible sin agregar dependencia, documentar la excepcion con `// SAFETY:` comment y registrar en un `AXIOM_EXCEPTIONS.md`.

---

### AX-UNSAFE-2: unsafe en tests de map_config.rs

**Violacion:** Hard Block 1 — "NO unsafe blocks"
**Archivo:** `src/worldgen/map_config.rs:474-485`

```rust
unsafe { std::env::set_var("RESONANCE_MAP", "__nonexistent_map__"); }
```

**Contexto:** Rust 2024 edition marca `std::env::set_var` como unsafe (race condition con threads). Solo aparece en `#[cfg(test)]`.

**Fix propuesto:** Refactorizar `load_map_config_from_env()` para aceptar un parametro `map_name: Option<&str>` que override la lectura de env var. El test pasa el nombre directamente sin tocar el entorno. El entry point real sigue leyendo env var.

---

### IMPL-LAYER-1: L10 ResonanceLink sin test de conservacion

**Capa:** L10 ResonanceLink (buff/debuff — effect -> target)
**Archivo:** `src/layers/link.rs`

**Problema:** ResonanceLink transfiere energia entre entidades (buff que drena qe del caster, debuff que inyecta qe negativo). No hay test que verifique `sum(qe_before) == sum(qe_after) +/- epsilon`.

**Impacto:** Posible leak de qe durante transferencias L10. En partida larga, esto rompe el axioma de conservacion.

**Fix:**
```rust
#[test]
fn resonance_link_conserves_total_qe() {
    // Spawn caster + target con L0 + L10
    // Run transfer system 1 tick
    // Assert: qe_caster + qe_target == initial_total +/- EPSILON
}
```

---

### IMPL-LAYER-2: L11 TensionField sin test de simetria

**Capa:** L11 TensionField (gravity/magnetic force at distance)
**Archivo:** `src/layers/tension_field.rs`

**Problema:** No hay test que verifique la tercera ley de Newton: `F(A->B) == -F(B->A)`. Si la fuerza no es simetrica, se puede crear energia del vacio.

**Fix:**
```rust
#[test]
fn tension_field_force_is_antisymmetric() {
    let f_ab = equations::tension_force(pos_a, mass_a, pos_b, mass_b);
    let f_ba = equations::tension_force(pos_b, mass_b, pos_a, mass_a);
    assert!((f_ab + f_ba).length() < EPSILON);
}
```

---

### IMPL-LAYER-3: L13 StructuralLink stress sin limite

**Capa:** L13 StructuralLink (spring joint entre entidades)
**Archivo:** `src/layers/structural_link.rs`

**Problema:** `break_stress` esta definido en el componente, pero no hay test que verifique que `StructuralLinkBreakEvent` (registrado en `src/events.rs` y manejado en `src/simulation/thermodynamic/structural_runtime.rs`) se emite antes de que `stress` alcance valores que causen overflow numerico (f32::MAX ~ 3.4e38).

**Fix:** Test de integracion que verifica que con stiffness alta y distancia creciente, el link se rompe antes de `stress > break_stress * SAFETY_MARGIN`.

---

### IMPL-SYSTEM-1: Inline math en osmosis.rs

**Archivo:** `src/simulation/thermodynamic/osmosis.rs`

**Problema:** El sistema `osmotic_diffusion_system` delega correctamente a `equations::osmotic_frequency_mix` para la mezcla de frecuencia, pero hay operaciones aritmeticas directas:
- `qe_after[src_idx] -= moved` / `qe_after[dst_idx] += moved` (lineas 32-33)
- `cell_volume = grid.cell_size * grid.cell_size * grid.cell_size` (linea 57)

**Evaluacion:** La transferencia directa `qe -= moved` / `qe += moved` es trivial y conservativa por definicion. El calculo de volumen cubico es geometria directa. Ambos son candidatos a permanecer inline bajo el principio de que no son "formulas" sino operaciones de contabilidad.

**Fix:** Auditar el resto del archivo para confirmar que ninguna formula no-trivial queda fuera de `blueprint/equations/`. Si todo es transferencia/contabilidad, marcar como PASS con comentario inline.

---

### IMPL-SYSTEM-2: WorldgenPhase dentro de ThermodynamicLayer

**Archivo:** `src/plugins/worldgen_plugin.rs:67-69`

```rust
WorldgenPhase
    .in_set(Phase::ThermodynamicLayer)
    .after(update_spatial_index_system)
```

**Problema:** Los 30+ sistemas de worldgen V7 (campo, propagacion, materializacion, terreno, visual, performance, LOD) corren todos dentro de `Phase::ThermodynamicLayer`. Esto hace que:
1. ThermodynamicLayer sea desproporcionadamente pesada
2. Se mezclen preocupaciones (worldgen no es termodinamica)
3. Sea dificil perfilar/depurar latencia por fase

**Fix propuesto:** Crear `Phase::WorldgenLayer` o mantener `WorldgenPhase` como SystemSet de primer nivel en FixedUpdate, fuera de Phase, con ordering explicito `.after(Phase::ThermodynamicLayer)` o `.before(Phase::Input)` segun dependencias. Requiere analisis de que datos de worldgen consumen los sistemas termodinamicos y viceversa.

**Riesgo:** Cambiar el ordering puede romper invariantes de propagacion que dependen de que las celdas del grid esten actualizadas antes de que los sistemas de entidades corran.

---

### ENT-1: Virus sin L12 Homeostasis (INFO)

**Archivo:** `src/entities/archetypes/catalog.rs`
**Test existente:** `assert!(!ent.contains::<Homeostasis>(), "virus no se adapta");`

**Decision de diseno:** El virus es una capside rigida que inyecta frecuencia forzada (L8 AlchemicalInjector). No se adapta al ambiente — impone su frecuencia. Esto es correcto para el arquetipo actual.

**Accion:** Documentar la decision en el archetype file con comentario `// Design: virus is a rigid capsid — no frequency adaptation (L12). See SPRINT_FIX_AXIOM_VIOLATIONS.md`.

---

### ENT-2: Animal sin L6 AmbientPressure

**Archivo:** `src/entities/archetypes/catalog.rs` (funcion `spawn_animal` / builder chain)

**Problema:** El animal (movil, trophic consumer) no tiene `.ambient(delta_qe, viscosity)` en su builder chain. Sin L6, el terreno no afecta su movimiento: un animal corre igual en pantano (viscosity alta) que en pradera (viscosity baja).

**Fix:** Agregar `.ambient(0.0, 0.1)` al builder chain del animal:
- `delta_qe = 0.0` (el animal no inyecta/roba qe ambiental)
- `viscosity = 0.1` (baja — animal es movil, pero terreno lo afecta)

---

### ENT-3: Hero sin L6 AmbientPressure

**Archivo:** `src/entities/archetypes/heroes.rs`

**Mismo problema que ENT-2.** El heroe (jugador controlado) no recibe presion ambiental.

**Fix:** Agregar `.ambient(0.0, 0.15)` o parametrizar por bioma de spawn. Viscosidad ligeramente mayor que animal (heroe carga equipo).

---

### ENT-4: Flora (EA2) sin L12 Homeostasis

**Archivo:** `src/entities/archetypes/flora.rs`

**Problema:** Los presets florales del sistema EA2 (emergent flora) no tienen L12 Homeostasis. Su frecuencia oscilatoria es fija desde spawn. Si el ambiente cambia de frecuencia (por ejemplo, por osmosis o por un spell), la flora no puede adaptarse y muere por desajuste frecuencial.

**Contraste:** `spawn_planta` en `catalog.rs` SI tiene Homeostasis (linea 129). `spawn_animal` tambien (linea 245). Solo flora EA2 carece.

**Fix:** Agregar L12 con `adaptation_rate` segun especie:
- Rosa: alta (0.4) — se adapta rapido, fragil
- Roble: baja (0.05) — rigido, resiste por inercia
- Musgo: media (0.2) — generalista

---

### BEH-1: behavior_will_bridge_system sin fallback

**Archivo:** `src/simulation/behavior.rs:300-338`

**Problema:** Cuando `BehaviorIntent.target_entity` es `None` o el target no existe en el World (despawned), `direction_to_target` retorna `None`, y el sistema asigna `Vec2::ZERO` al `WillActuator`. Esto deja al animal inmovil indefinidamente — no hay timeout ni fallback a wander.

**Fix propuesto:** Si `direction_to_target` retorna `None` para modos que requieren target (`Forage`, `Hunt`, `Reproduce`), inyectar fallback:
1. Resetear `BehaviorMode` a `Idle` (fuerza re-evaluacion en siguiente tick de decision)
2. O generar wander direction aleatorio basado en entity hash + tick (determinista)

---

### BEH-2: Trophic consumers sin deteccion de starvation

**Archivo:** `src/simulation/metabolic/trophic.rs`
**Relacionado:** `src/simulation/metabolic/metabolic_stress.rs` (sistema `metabolic_stress_death_system`)

**Problema:** Si un herbivoro no encuentra plantas, su qe baja a 0. Verificar que `metabolic_stress_death_system` cubre el caso de `qe <= 0` y emite `DeathEvent`. Si no, la entidad queda como zombie (qe=0, viva, sin poder hacer nada).

**Fix:** Verificar cobertura en `metabolic_stress_death_system`. Si no cubre qe=0, agregar condicion. Agregar test:
```rust
#[test]
fn entity_with_zero_qe_triggers_death() {
    // spawn con qe=0, run metabolic_stress_death_system
    // assert DeathEvent emitted
}
```

---

### BEH-3: Pack formation sin limite de tamano

**Archivo:** `src/simulation/metabolic/social_communication.rs` (funcion `social_pack_formation_system`)
**Ecuacion:** `src/blueprint/equations/social_communication.rs:33` — `pack_hunt_bonus(pack_size, _prey_qe)`

**Problema:** No existe constante `PACK_MAX_SIZE`. La ecuacion de bonus usa `sqrt(pack_size)` que crece sin bound. Con N animales gregarios en rango, todos se agrupan en un solo pack. El sistema de cohesion es O(n) por miembro pero el calculo de vecinos puede ser O(n^2) si no hay corte.

**Fix:**
1. Agregar `PACK_MAX_SIZE: u32 = 8` en `blueprint/constants/` (o dominio social)
2. En `social_pack_formation_system`, rechazar nuevos miembros si pack alcanza limite
3. `pack_hunt_bonus` ya escala con sqrt — el limite previene el caso patologico

---

### DOC-1: Conteo incorrecto de sub-modulos

**Archivo:** `docs/arquitectura/blueprint_v6.md`
**Problema:** Dice "17 sub-modulos" para runtime_platform pero hay 18.
**Fix:** Actualizar conteo a 18.

---

### DOC-2: BLUEPRINT.md usa paths legacy

**Archivo:** `docs/design/BLUEPRINT.md`
**Problema:** El module map referencia `src/capas/capa0_energia_base.rs` etc. Estos paths ya no existen — el modulo actual es `src/layers/`.
**Fix:** Buscar/reemplazar `capas/` -> `layers/` y actualizar nombres de archivo (de espanol a ingles).

---

### DOC-3: Docs redundantes en arquitectura/

**Archivos:**
- `docs/arquitectura/blueprint_visual_quantization.md`
- `docs/arquitectura/blueprint_quantized_color.md`
- `docs/arquitectura/blueprint_sim_layers.md`

**Problema:** Contenido duplicado o superseded por `docs/design/` equivalentes.
**Fix:** Verificar que `docs/design/` tiene la version canonica. Si si, eliminar los de `arquitectura/` y dejar redirect comment en `arquitectura/README.md`.

---

## Tabla resumen

| ID | Tipo | Severidad | Archivo | Status |
|----|------|-----------|---------|--------|
| AX-FIELDS-1 | Axioma (4 fields) | BLOCK | `src/layers/metabolic_graph.rs` | OPEN |
| AX-UNSAFE-1 | Axioma (unsafe) | WARN | `src/worldgen/cell_field_snapshot/gpu_layout.rs` | OPEN |
| AX-UNSAFE-2 | Axioma (unsafe) | WARN | `src/worldgen/map_config.rs` | OPEN |
| IMPL-LAYER-1 | Test gap (conservacion) | WARN | `src/layers/link.rs` | OPEN |
| IMPL-LAYER-2 | Test gap (simetria) | WARN | `src/layers/tension_field.rs` | OPEN |
| IMPL-LAYER-3 | Test gap (overflow) | WARN | `src/layers/structural_link.rs` | OPEN |
| IMPL-SYSTEM-1 | Inline math audit | WARN | `src/simulation/thermodynamic/osmosis.rs` | OPEN |
| IMPL-SYSTEM-2 | Phase scope | WARN | `src/plugins/worldgen_plugin.rs` | OPEN |
| ENT-1 | Composicion (virus) | INFO | `src/entities/archetypes/catalog.rs` | OPEN — documentar decision |
| ENT-2 | Composicion (animal) | WARN | `src/entities/archetypes/catalog.rs` | OPEN |
| ENT-3 | Composicion (hero) | WARN | `src/entities/archetypes/heroes.rs` | OPEN |
| ENT-4 | Composicion (flora EA2) | WARN | `src/entities/archetypes/flora.rs` | OPEN |
| BEH-1 | Comportamiento (will bridge) | WARN | `src/simulation/behavior.rs` | OPEN |
| BEH-2 | Comportamiento (starvation) | WARN | `src/simulation/metabolic/trophic.rs` | OPEN |
| BEH-3 | Performance (pack size) | WARN | `src/simulation/metabolic/social_communication.rs` | OPEN |
| DOC-1 | Docs (conteo) | INFO | `docs/arquitectura/blueprint_v6.md` | OPEN |
| DOC-2 | Docs (paths legacy) | INFO | `docs/design/BLUEPRINT.md` | OPEN |
| DOC-3 | Docs (redundancia) | INFO | `docs/arquitectura/*.md` | OPEN |

---

## Orden de ejecucion sugerido

1. **AX-FIELDS-1** (BLOCK) — Desbloquea merge. Extraer campo 5 de MetabolicGraph.
2. **AX-UNSAFE-1 + AX-UNSAFE-2** — Eliminar unsafe donde sea posible.
3. **ENT-2 + ENT-3 + ENT-4** — Composicion de entidades. Rapido, alto impacto en gameplay.
4. **IMPL-LAYER-1/2/3** — Tests de conservacion/simetria. Fundamentales para simulacion correcta.
5. **BEH-1 + BEH-2 + BEH-3** — Robustez de comportamiento. Previene zombies y O(n^2).
6. **IMPL-SYSTEM-1 + IMPL-SYSTEM-2** — Audit/refactor de fases. Mas invasivo.
7. **DOC-1/2/3** — Cleanup de docs. Sin riesgo, baja prioridad.

---

## Criterio de cierre

- Todos los BLOCK resueltos
- Todos los WARN resueltos o con excepcion documentada
- Todos los INFO documentados (decision explicita en codigo o en este archivo)
- `cargo test` pasa sin regresiones
- Nuevos tests cubren IMPL-LAYER-1/2/3 y BEH-2
