# Track: AXIOMATIC_DIVISION — Cell Division from Internal Energy Fields

**Objetivo:** Reemplazar el sistema de reproducción hardcodeado (`reproduction_spawn_system`) con división emergente derivada del campo de energía interno. La división ocurre cuando un valley en el campo interno llega a qe ≤ 0. Cero thresholds, cero flags, cero cooldowns.

**Estado:** ✅ ARCHIVADO (2026-03-27) — 5/5 sprints completados. reproduction_spawn_system reemplazado.
**Bloqueado por:** Nada (track independiente). AD-2 batch implementation done (src/batch/systems/internal_field.rs).
**Desbloquea:** División celular emergente (fisión, budding, fragmentación, esporulación)

---

## Principio axiomático

```
Axiom 1: existencia = qe > 0
Axiom 2: Σ children ≤ parent (conservation)
Axiom 4: disipación en toda conexión
Axiom 8: frecuencia modula coherencia interna

→ Cuando un nodo interno tiene qe ≤ 0, la conexión no existe.
→ Si el nodo está entre dos picos, la entidad se parte.
→ No es una decisión. Es física.
```

## Derivación completa

```
Estado sano:     [3, 5, 8, 10, 10, 8, 5, 3]    → un pico, estable
Acumulación:     [5, 8, 15, 20, 20, 15, 8, 5]   → crece uniformemente
Bimodal:         [12, 15, 10, 6, 6, 10, 15, 12]  → dos picos emergen
Valley drains:   [14, 16, 8, 2, 2, 8, 16, 14]    → valley se profundiza
SPLIT:           [14, 16, 8, 0, 0, 8, 16, 14]    → valley = 0 → ruptura

Child A: [14, 16, 8]   Child B: [8, 16, 14]
Conservation: 38 + 38 = 76 ≤ 78 (original - 2 qe disipados en valley)
```

## Tipos de división emergentes (NO programados)

| Patrón del campo | Resultado | Análogo biológico |
|-------------------|-----------|-------------------|
| Simétrico bimodal `[15,5,0,5,15]` | 2 hijos iguales | Fisión binaria |
| Asimétrico `[20,8,0,3,5]` | 1 grande + 1 chico | Budding (gemación) |
| Multi-valley `[10,0,10,0,10]` | 3 fragmentos | Fragmentación |
| Pico concentrado, resto a 0 `[0,0,0,0,0,0,0,15]` | 1 espora, padre muere | Esporulación |

**Qué determina el patrón (sin hardcode):**
- Intake asimétrico (dónde come) → picos asimétricos → budding
- Alta frecuencia (Axiom 8) → coherencia interna baja → más valleys
- Mucha energía → picos altos → valleys más profundos → fisión
- Disipación alta (Liquid/Gas) → valleys se forman rápido → división rápida

## 5 Sprints

| Sprint | Descripción | Archivos | Esfuerzo | Bloqueado por |
|--------|-------------|----------|----------|---------------|
| [AD-1](SPRINT_AD1_INTERNAL_FIELD_COMPONENT.md) | Componente InternalEnergyField([f32; 8]) para Bevy ECS | layers/, awakening | Bajo | — |
| [AD-2](SPRINT_AD2_FIELD_DIFFUSION_SYSTEM.md) | Sistema de difusión interna (stateless, por tick) | simulation/lifecycle/ | Bajo | AD-1 |
| [AD-3](SPRINT_AD3_VALLEY_DETECTION_EQUATIONS.md) | Ecuaciones puras: peak/valley detection, split viability | blueprint/equations/ | Bajo | — |
| [AD-4](SPRINT_AD4_SPLIT_SYSTEM.md) | Sistema de división: valley ≤ 0 → spawn 2 children | simulation/lifecycle/ | Medio | AD-1, AD-2, AD-3 |
| [AD-5](SPRINT_AD5_DEPRECATE_REPRODUCTION.md) | Deprecar reproduction_spawn_system, migrar a split | simulation/reproduction/ | Medio | AD-4 |

---

## Sprint Details

### AD-1: InternalEnergyField Component

**Qué:** Componente ECS con campo de 8 nodos. Se inserta en entidades que materializan o despiertan.

```rust
/// Internal energy distribution across the entity body (8 axis-aligned nodes).
/// Diffusion between nodes creates emergent gradients. Valleys trigger division.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct InternalEnergyField {
    pub nodes: [f32; 8],
}
```

**Reglas:**
- Max 4 fields → este componente tiene 1 campo (array). ✅
- Se inserta en: `awakening_system` (cuando entidad despierta), `materialization` (al spawn)
- Inicialización: `nodes[i] = entity.qe / 8.0` (distribución uniforme al nacer)

**Tests:**
- `default_distributes_uniformly`
- `total_qe_matches_sum`

---

### AD-2: Internal Diffusion System

**Qué:** Sistema stateless que difunde energía entre nodos vecinos del campo interno. Usa `diffusion_delta()` que ya existe.

**Fase:** `Phase::MorphologicalLayer` (antes de split detection)

```rust
pub fn internal_field_diffusion_system(
    mut query: Query<(&mut InternalEnergyField, &MatterCoherence)>,
) {
    for (mut field, matter) in &mut query {
        let k = dissipation_from_state(matter.state()); // Axiom 4: conductivity from state
        for i in 0..7 {
            let delta = diffusion_delta(field.nodes[i], field.nodes[i + 1], k, 1.0);
            field.nodes[i] -= delta;
            field.nodes[i + 1] += delta;
        }
        // Dissipation per node (Axiom 4)
        for node in &mut field.nodes {
            *node = (*node - *node * dissipation_from_state(matter.state())).max(0.0);
        }
    }
}
```

**Cero constantes nuevas.** Usa `dissipation_from_state()` (ya derivada de fundamentales) y `diffusion_delta()` (ya existe en `signal_propagation.rs`).

**Tests:**
- `diffusion_equalizes_gradient`
- `dissipation_reduces_total`
- `uniform_field_stays_uniform`

---

### AD-3: Valley Detection Equations

**Qué:** Ecuaciones puras en `blueprint/equations/`. Detecta picos y valleys en un array de f32.

```rust
/// Finds indices of valleys (local minima) in the field.
/// A valley at index i: field[i] < field[i-1] AND field[i] < field[i+1].
pub fn find_valleys(field: &[f32; 8]) -> Vec<usize>

/// A valley is a split point when its qe ≤ 0 (Axiom 1: no energy = no existence).
/// No threshold. No constant. Pure physics.
pub fn is_split_viable(field: &[f32; 8], valley_idx: usize) -> bool {
    field[valley_idx] <= 0.0
}

/// Split the field at a valley. Returns (left_nodes, right_nodes).
/// Conservation (Axiom 2): sum(left) + sum(right) ≤ sum(original).
pub fn split_field_at(field: &[f32; 8], valley_idx: usize) -> ([f32; 8], [f32; 8])
```

**Tests:**
- `no_valleys_in_monotonic_field`
- `center_valley_detected`
- `split_conserves_energy`
- `split_at_edge_produces_small_child` (budding)
- `zero_valley_is_split_viable`
- `positive_valley_not_split_viable`

---

### AD-4: Split System

**Qué:** Sistema que detecta valleys ≤ 0 y ejecuta el split. Fase: `Phase::MorphologicalLayer`.

```rust
pub fn axiomatic_split_system(
    mut commands: Commands,
    query: Query<(Entity, &InternalEnergyField, &Transform, &OscillatorySignature, ...)>,
    clock: Res<SimulationClock>,
) {
    for (entity, field, transform, osc, ...) in &query {
        let valleys = find_valleys(&field.nodes);
        for &v in &valleys {
            if !is_split_viable(&field.nodes, v) { continue; }
            let (left, right) = split_field_at(&field.nodes, v);
            // Spawn child A with left nodes
            // Spawn child B with right nodes
            // Despawn parent
            // Conservation: sum(A) + sum(B) ≤ sum(parent)
            break; // max 1 split per tick per entity
        }
    }
}
```

**Herencia:**
- Frequency: ambos hijos heredan parent frequency ± small drift (Axiom 8)
- InferenceProfile: se deriva del campo interno (no se copia hardcodeado)
- CulturalMemory: se copia al hijo más grande (el que tiene más qe en sus nodos)
- SenescenceProfile: tick_birth = now (nuevo organismo)

**Cero constantes nuevas. Cero flags. La división es consecuencia del campo.**

---

### AD-5: Deprecar reproduction_spawn_system

**Qué:** Migrar de reproducción hardcodeada a split axiomático.

**Pasos:**
1. Remover `reproduction_spawn_system` del MorphologicalPlugin
2. Remover `ReproductionCooldown` component
3. Remover `REPRODUCTION_RADIUS_FACTOR`, `SEED_ENERGY_FRACTION`, etc.
4. Remover capability check `can_branch() + MOVE + REPRODUCE`
5. Registrar `axiomatic_split_system` en su lugar
6. La reproducción ahora es: come → campo crece → valley se forma → split

**Lo que se elimina:**
- `FAUNA_REPRODUCTION_QE_MIN = 200` (threshold hardcodeado)
- `SEED_ENERGY_FRACTION = 0.15` (fracción arbitraria)
- `REPRODUCTION_COOLDOWN_TICKS = 60` (cooldown hardcodeado)
- `FAUNA_OFFSPRING_INITIAL_RADIUS = 0.35` (radius hardcodeado)
- `MAX_REPRODUCTIONS_PER_FRAME = 2` (budget hardcodeado)

**Lo que emerge en su lugar:**
- La entidad come → intake alimenta nodos del campo
- El intake no es uniforme → picos en los nodos donde come
- Picos crecen → valley se profundiza → qe valley → 0 → SPLIT
- No hay "momento de reproducción" — hay momento de desconexión interna

---

## Criterio de cierre del track

1. ✅ `reproduction_spawn_system` eliminado del pipeline
2. ✅ `axiomatic_split_system` registrado y funcionando
3. ✅ Cero constantes de reproducción (SEED_FRACTION, COOLDOWN, etc.)
4. ✅ Split ocurre cuando `valley.qe ≤ 0` (Axiom 1 puro)
5. ✅ Conservation verificada: `sum(children) ≤ sum(parent)`
6. ✅ Demo headless muestra splits visibles en telemetry
7. ✅ Tests: valley detection, split conservation, field diffusion

## Qué NO cambia

- Los 8 axiomas + 4 constantes
- El campo de energía del worldgen (EnergyFieldGrid)
- El awakening system
- El basal drain / senescence
- El behavior pipeline
- La transmisión cultural (se copia al hijo más grande)

## Riesgos

- **Performance:** 8 floats extra per entity. Con 400 entities: 3.2 KB. Negligible.
- **Split cascading:** Un split podría crear un hijo que inmediatamente splitea. Mitigación: 1 split per entity per tick.
- **Empty children:** Si un valley está en el borde, un hijo podría tener 1-2 nodos con poca energía. Mitigación: si `sum(child_nodes) < self_sustaining_qe_min()` → child no spawns (energía insuficiente para existir, Axiom 1).
