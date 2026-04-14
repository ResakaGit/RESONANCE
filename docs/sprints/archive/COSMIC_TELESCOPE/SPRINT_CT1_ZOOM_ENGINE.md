# CT-1: Zoom Collapse Engine — Inferencia axiomática + seed branching

**Esfuerzo:** L (3–5 sesiones)
**Bloqueado por:** CT-0
**ADR:** ADR-036 §D2, §D5

## Objetivo

El motor central del Cosmic Telescope: al hacer zoom en una entidad, colapsar
su estado en N entidades hijas respetando todos los axiomas. Al hacer zoom-out,
agregar hijas de vuelta al padre.

Este sprint es la pieza más crítica. Si la inferencia viola axiomas, todo el
sistema falla.

## Precondiciones

- CT-0 completado (`ScaleLevel`, `ScaleManager`)
- `SimWorldFlat` con `EntitySlot` disponible
- `derived_thresholds.rs` con las 4 constantes fundamentales

## Entregables

### E1: `scale_inference.rs` — pure math (blueprint/equations/)

Funciones puras sin ECS, sin side effects:

```rust
/// Cuántas entidades caben en un padre de energía qe (Kleiber scaling).
pub fn kleiber_child_count(parent_qe: f64, scale: ScaleLevel) -> usize;

/// Distribuir qe del padre entre N hijos respetando Pool Invariant.
/// Retorna Vec<f64> donde sum <= parent_qe × (1 - dissipation).
pub fn distribute_energy(
    parent_qe: f64, n_children: usize, state: MatterState, seed: u64,
) -> Vec<f64>;

/// Frecuencias de hijos derivadas del padre (Axiom 8).
pub fn distribute_frequencies(
    parent_freq: f64, n_children: usize, bandwidth: f64, seed: u64,
) -> Vec<f64>;

/// Posiciones iniciales dentro del radio del padre.
pub fn distribute_positions_3d(
    parent_pos: [f64; 3], parent_radius: f64, n_children: usize, seed: u64,
) -> Vec<[f64; 3]>;

/// Agregar N hijos en un padre: qe, freq dominante, posición centroide.
pub fn aggregate_to_parent(children: &[ChildState]) -> ParentState;

/// Verificar Pool Invariant post-zoom.
pub fn verify_pool_invariant(parent_qe: f64, children_qe: &[f64]) -> bool;
```

### E2: `zoom.rs` — event handlers

```rust
pub struct ZoomInEvent {
    pub target_entity_idx: u32,  // índice en SimWorldFlat del padre
    pub seed: u64,                // seed del observador (multiverso)
}

pub struct ZoomOutEvent;  // siempre regresa al nivel superior

/// Sistema que procesa ZoomInEvent.
pub fn zoom_in_system(
    mut events: EventReader<ZoomInEvent>,
    mut scale_mgr: ResMut<ScaleManager>,
) {
    // 1. Leer estado del padre
    // 2. Llamar scale_inference::distribute_*
    // 3. Crear ScaleInstance con nuevo SimWorldFlat
    // 4. Poblar con entidades inferidas
    // 5. Relajar N steps (fuerzas según escala)
    // 6. Marcar nivel anterior como frozen/coarsened
}

/// Sistema que procesa ZoomOutEvent.
pub fn zoom_out_system(
    mut events: EventReader<ZoomOutEvent>,
    mut scale_mgr: ResMut<ScaleManager>,
) {
    // 1. Agregar hijos via scale_inference::aggregate_to_parent
    // 2. Actualizar padre con estado agregado
    // 3. Destruir ScaleInstance del nivel inferior
    // 4. Descongelar nivel superior
}
```

### E3: Round-trip conservation tests

```rust
#[test]
fn zoom_in_out_preserves_pool_invariant() {
    // parent.qe = 1000
    // zoom_in → N children
    // assert: sum(children.qe) <= 1000
    // zoom_out → parent'
    // assert: parent'.qe <= 1000
    // assert: parent'.qe >= sum(children.qe) × (1 - DISSIPATION)
}

#[test]
fn zoom_deterministic_with_same_seed() {
    // Same parent + same seed → identical children
}

#[test]
fn zoom_different_seed_different_children() {
    // Same parent + different seed → different but valid children
}

#[test]
fn zoom_frequencies_within_bandwidth() {
    // All child freqs within parent.freq ± 3*COHERENCE_BANDWIDTH
}

#[test]
fn zoom_dissipation_applied() {
    // sum(children.qe) < parent.qe (strictly less, Axiom 4)
}
```

## Tasks

- [ ] Crear `src/blueprint/equations/scale_inference.rs` con funciones puras
- [ ] Tests unitarios de cada función de inferencia (≥8 tests)
- [ ] Crear `src/cosmic/zoom.rs` con event handlers
- [ ] Test de integración: zoom-in/out round-trip con SimWorldFlat minimal
- [ ] Test de determinismo: mismo seed = mismo resultado
- [ ] Test de multiverso: distinto seed = distinto resultado, ambos válidos
- [ ] Verificar 0 warnings, 0 clippy

## Criterios de aceptación

1. `zoom_in` produce entidades cuya `sum(qe) < parent.qe` (Pool + Dissipation)
2. `zoom_out` agrega correctamente (qe, freq, pos)
3. Round-trip: `parent → zoom_in → zoom_out → parent'` con `parent'.qe <= parent.qe`
4. Determinismo: mismo seed = mismos hijos (bit-exact)
5. Frecuencias de hijos dentro de `parent.freq ± 3*COHERENCE_BANDWIDTH`
6. No toca archivos de otros módulos excepto `lib.rs` y `blueprint/equations/mod.rs`
