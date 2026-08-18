//! Prueba comportamental del Axioma 6 sobre el **motor real** (ADR-047).
//!
//! ADR-046 probó que el modelo de Kuramoto sincroniza (mean-field analítico de
//! fase, all-to-all, sin ECS). Eso es un resultado de 1975, no una propiedad de
//! RESONANCE. Este probe corre el `entrainment_system` real — la misma fn que
//! registra `AtomicPlugin` — sobre una población L2 real con `SpatialIndex` real,
//! y mide si el orden global emerge de la regla local y desaparece al ablarla.
//!
//! El system acopla **frecuencia**, no fase: la métrica es el colapso de la
//! dispersión `S = 1 − σ_ω(T)/σ_ω(0) ∈ [0,1]`, no el order parameter de fase.
//!
//! Dos ablaciones causales, ambas sobre la misma población y las mismas seeds:
//! - **A1 (regla):** el system no se agrega al schedule → contrafáctico N−1→N puro.
//! - **A2 (alcance):** system ACTIVO, layout a `3 × ENTRAINMENT_SCAN_RADIUS` → el
//!   broadphase devuelve vecindad vacía y la regla corre en el vacío. Prueba que
//!   el orden requiere *interacción efectiva*, no la mera presencia del código.
//!   El supresor es el cutoff duro del radio de scan (Ax7 como alcance acotado),
//!   NO la atenuación continua: con el decay solo, `K_eff(36) ≈ 0.0075` y el
//!   sistema aún convergería.
//!
//! cargo test --test emergence_ecs

use bevy::prelude::*;

use resonance::blueprint::constants::{
    ENTRAINMENT_SCAN_RADIUS, KURAMOTO_LOCK_THRESHOLD_HZ, SYNC_ECS_CENTER_HZ, SYNC_ECS_S_PASS_MIN,
    SYNC_ECS_SPREAD_HZ, SYNC_ECS_TICKS,
};
use resonance::blueprint::equations::determinism::{gaussian_f32, hash_f32_slice, next_u64};
use resonance::blueprint::equations::emergence::entrainment::{
    ENTRAINMENT_MAX_NEIGHBOURS, entrainment_lock_achieved,
};
use resonance::blueprint::equations::emergence::synchronization::{
    frequency_collapse_s, frequency_std, sync_ecs_verdict,
};
use resonance::layers::{OscillatorySignature, SpatialVolume};
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::simulation::emergence::entrainment::entrainment_system;
use resonance::world::space::{SpatialEntry, SpatialIndex};

// ─── Escenario ───────────────────────────────────────────────────────────────

/// Lado de la grilla. 8×8 = 64 entidades.
const GRID_SIDE: usize = 8;
const POPULATION: usize = GRID_SIDE * GRID_SIDE;

/// Layout denso: ⅔ del scan radius. A este spacing un nodo interior tiene
/// exactamente 8 candidatos en rango (4 ortogonales a d=8, 4 diagonales a
/// d=11.31; d=16 queda fuera del filtro `d ≤ 12 + 0.5`) → el cap
/// `ENTRAINMENT_MAX_NEIGHBOURS` **no trunca** y el grafo es el king-graph
/// simétrico. A spacing 6 habría 12 candidatos y la truncación retendría los 8
/// de menor `Entity::to_bits` → grafo dirigido sesgado por orden de spawn.
const DENSE_SPACING: f32 = 8.0;

/// Layout A2: fuera del alcance del broadphase (36 > 12.5).
const SPARSE_SPACING: f32 = 3.0 * ENTRAINMENT_SCAN_RADIUS;

/// Radio de `SpatialEntry` — el que gobierna el filtro del índice. El system NO
/// lee `SpatialVolume`; los tests del propio módulo insertan 1.0 (trampa de copia).
const ENTITY_RADIUS: f32 = 0.5;

/// Paridad con `tests/emergence_sync.rs::emergence_holds_across_seeds`.
const SEEDS: [u64; 5] = [1, 2, 3, 100, 777];

/// Seed de referencia para los tests de una sola corrida (precedente `emergence_sync.rs`).
const REFERENCE_SEED: u64 = 11;

// ─── Builder ─────────────────────────────────────────────────────────────────

struct Probe {
    app: App,
    entities: Vec<Entity>,
}

/// Población inicial de ω: `SYNC_ECS_CENTER_HZ + gaussian(0, SYNC_ECS_SPREAD_HZ)`.
///
/// El offset es obligatorio: `gaussian_f32` centra en 0 y L2 clampea a `max(0.0)`
/// — sin él la gaussiana queda **rectificada** y el experimento se corrompe en
/// silencio. El stream avanza **dos `next_u64` por draw** porque `gaussian_f32`
/// consume dos estados internos (Box-Muller); avanzar de a uno reproduciría la
/// correlación intra-stream del bug histórico de `emergence_sync.rs::init_state`.
fn draw_frequencies(seed: u64) -> Vec<f32> {
    let mut state = next_u64(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    (0..POPULATION)
        .map(|_| {
            state = next_u64(state);
            let omega = SYNC_ECS_CENTER_HZ + gaussian_f32(state, SYNC_ECS_SPREAD_HZ);
            state = next_u64(state);
            omega
        })
        .collect()
}

/// Tres Apps del mismo builder: acoplado (`DENSE_SPACING`, system ON), A1
/// (`DENSE_SPACING`, system OFF) y A2 (`SPARSE_SPACING`, system ON). Los draws de
/// ω son idénticos por seed — el PCG es determinista y no depende del layout.
///
/// El `SpatialIndex` se construye una vez: las entidades no se mueven (sin L3
/// `FlowVector`). Replica el patrón del `#[cfg(test)] build_index` del propio
/// módulo vía API pública. `SimWorldTransformParams::default()` → plano XY.
///
/// Schedule: `Update` + `app.update()` explícito. **Nunca `FixedUpdate`**: con
/// `MinimalPlugins` corre 0..k veces por `update()` según wall-clock → ni conteo
/// de ticks garantizado ni determinismo entre máquinas.
fn build_probe(seed: u64, spacing: f32, with_system: bool) -> Probe {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SimWorldTransformParams::default());

    let frequencies = draw_frequencies(seed);
    let mut entities = Vec::with_capacity(POPULATION);
    let mut index = SpatialIndex::new(ENTRAINMENT_SCAN_RADIUS);

    for (i, &freq) in frequencies.iter().enumerate() {
        let position = Vec2::new(
            (i % GRID_SIDE) as f32 * spacing,
            (i / GRID_SIDE) as f32 * spacing,
        );
        // φ = 0: don't-care declarado. El system no lee fase (snapshot toma sólo
        // `frequency_hz`, muta sólo `set_frequency_hz`) y S es función sólo de ω.
        let entity = app
            .world_mut()
            .spawn((
                OscillatorySignature::new(freq, 0.0),
                SpatialVolume::new(ENTITY_RADIUS),
                Transform::from_xyz(position.x, position.y, 0.0),
            ))
            .id();
        index.insert(SpatialEntry {
            entity,
            position,
            radius: ENTITY_RADIUS,
        });
        entities.push(entity);
    }

    app.insert_resource(index);
    if with_system {
        app.add_systems(Update, entrainment_system);
    }
    Probe { app, entities }
}

// ─── Lectura canónica y métricas ─────────────────────────────────────────────

/// ω en orden canónico (`Entity::index` ascendente).
///
/// La suma f32 no es asociativa: el "S == 0 exacto" de las ablaciones exige que
/// ambas mediciones lean el mismo orden, y se aserta por identidad bit a bit del
/// vector (`hash_f32_slice`), no por el valor derivado.
fn canonical_frequencies(probe: &Probe) -> Vec<f32> {
    let mut ids = probe.entities.clone();
    ids.sort_unstable_by_key(|e| e.index());
    ids.iter()
        .map(|&e| {
            probe
                .app
                .world()
                .get::<OscillatorySignature>(e)
                .map(|osc| osc.frequency_hz())
                .unwrap_or(f32::NAN)
        })
        .collect()
}

fn advance(probe: &mut Probe, ticks: u32) {
    for _ in 0..ticks {
        probe.app.update();
    }
}

/// Vecinos efectivos de `entity` según el índice real, **excluyendo self**:
/// `query_radius` devuelve al propio nodo (d=0 pasa el filtro `d ≤ radius + r`).
fn neighbour_count(probe: &Probe, entity: Entity) -> usize {
    let Some(transform) = probe.app.world().get::<Transform>(entity) else {
        return 0;
    };
    let position = transform.translation.truncate();
    probe
        .app
        .world()
        .resource::<SpatialIndex>()
        .query_radius(position, ENTRAINMENT_SCAN_RADIUS)
        .iter()
        .filter(|entry| entry.entity != entity)
        .count()
}

/// Métrica secundaria: fracción de pares vecinos `(i<j)` con lock de frecuencia,
/// reusando la definición de lock del propio motor.
fn locked_pair_fraction(probe: &Probe) -> f32 {
    let index = probe.app.world().resource::<SpatialIndex>();
    let mut total = 0usize;
    let mut locked = 0usize;

    for &entity in &probe.entities {
        let Some(transform) = probe.app.world().get::<Transform>(entity) else {
            continue;
        };
        let Some(osc_i) = probe.app.world().get::<OscillatorySignature>(entity) else {
            continue;
        };
        let position = transform.translation.truncate();
        let freq_i = osc_i.frequency_hz();
        for entry in index.query_radius(position, ENTRAINMENT_SCAN_RADIUS) {
            if entry.entity.index() <= entity.index() {
                continue; // self + mitad simétrica del par
            }
            let Some(osc_j) = probe.app.world().get::<OscillatorySignature>(entry.entity) else {
                continue;
            };
            total += 1;
            if entrainment_lock_achieved(freq_i, osc_j.frequency_hz(), KURAMOTO_LOCK_THRESHOLD_HZ) {
                locked += 1;
            }
        }
    }

    if total == 0 {
        0.0
    } else {
        locked as f32 / total as f32
    }
}

struct Outcome {
    sigma_0: f32,
    sigma_t: f32,
    s: f32,
    locked_fraction: f32,
    hash_0: u64,
    hash_t: u64,
}

fn measure(seed: u64, spacing: f32, with_system: bool) -> Outcome {
    let mut probe = build_probe(seed, spacing, with_system);
    let omega_0 = canonical_frequencies(&probe);
    advance(&mut probe, SYNC_ECS_TICKS);
    let omega_t = canonical_frequencies(&probe);

    let sigma_0 = frequency_std(&omega_0);
    let sigma_t = frequency_std(&omega_t);
    Outcome {
        sigma_0,
        sigma_t,
        s: frequency_collapse_s(sigma_0, sigma_t),
        locked_fraction: locked_pair_fraction(&probe),
        hash_0: hash_f32_slice(&omega_0),
        hash_t: hash_f32_slice(&omega_t),
    }
}

// ─── Guards estructurales del escenario ──────────────────────────────────────

/// El layout denso NO ejercita el régimen de truncación de vecinos.
///
/// Si alguien cambia N o spacing y reaparecen >8 candidatos, la truncación
/// retiene los 8 de menor `Entity::to_bits` → grafo dirigido sesgado por orden de
/// spawn, y el probe deja de medir lo que dice medir.
#[test]
fn dense_layout_is_an_untruncated_king_graph() {
    let probe = build_probe(REFERENCE_SEED, DENSE_SPACING, true);
    for (i, &entity) in probe.entities.iter().enumerate() {
        let count = neighbour_count(&probe, entity);
        assert!(
            count <= ENTRAINMENT_MAX_NEIGHBOURS,
            "nodo {i}: {count} vecinos > cap {ENTRAINMENT_MAX_NEIGHBOURS} → truncación con sesgo por spawn index"
        );
        let (row, col) = (i / GRID_SIDE, i % GRID_SIDE);
        let interior = row > 0 && row + 1 < GRID_SIDE && col > 0 && col + 1 < GRID_SIDE;
        if interior {
            assert_eq!(
                count, ENTRAINMENT_MAX_NEIGHBOURS,
                "nodo interior ({row},{col}) debe tener exactamente 8 vecinos (king-graph)"
            );
        }
    }
}

/// A2: a `3 × ENTRAINMENT_SCAN_RADIUS` el broadphase no devuelve ningún vecino —
/// la regla corre cada tick sobre vecindad vacía.
#[test]
fn sparse_layout_has_no_neighbours_in_range() {
    let probe = build_probe(REFERENCE_SEED, SPARSE_SPACING, true);
    for (i, &entity) in probe.entities.iter().enumerate() {
        assert_eq!(
            neighbour_count(&probe, entity),
            0,
            "nodo {i}: A2 exige vecindad vacía (spacing {SPARSE_SPACING} > scan {ENTRAINMENT_SCAN_RADIUS})"
        );
    }
}

/// La población nace desordenada y lejos del clamp `max(0.0)` de L2.
#[test]
fn initial_population_is_disordered_and_never_rectified() {
    for seed in SEEDS {
        let probe = build_probe(seed, DENSE_SPACING, false);
        let omega_0 = canonical_frequencies(&probe);
        let sigma_0 = frequency_std(&omega_0);
        assert!(
            omega_0.iter().all(|&f| f > 0.0),
            "seed {seed}: alguna ω tocó el clamp de L2 → gaussiana rectificada"
        );
        assert!(
            (sigma_0 - SYNC_ECS_SPREAD_HZ).abs() < 0.35 * SYNC_ECS_SPREAD_HZ,
            "seed {seed}: σ(0)={sigma_0} lejos del nominal {SYNC_ECS_SPREAD_HZ}"
        );
    }
}

// ─── Ax6 sobre el motor: orden acoplado vs ablaciones ────────────────────────

/// Ax6-ECS-1: con la regla local activa y vecindad efectiva, la dispersión de
/// frecuencias de la población colapsa.
#[test]
fn coupled_population_collapses_frequency_spread() {
    for seed in SEEDS {
        let out = measure(seed, DENSE_SPACING, true);
        println!(
            "[coupled] seed={seed} sigma_0={:.4} sigma_T={:.4} S={:.4} locked={:.4}",
            out.sigma_0, out.sigma_t, out.s, out.locked_fraction
        );
        assert!(
            out.s >= SYNC_ECS_S_PASS_MIN,
            "seed {seed}: S={} < umbral {SYNC_ECS_S_PASS_MIN} (σ0={}, σT={})",
            out.s,
            out.sigma_0,
            out.sigma_t
        );
    }
}

/// Ax6-ECS-2 (A1, ablación de regla): sin el system en el schedule, ω(T) es
/// **bit-idéntico** a ω(0) → `S == 0`. Contrafáctico N−1→N puro.
#[test]
fn rule_ablation_leaves_frequencies_bit_identical() {
    for seed in SEEDS {
        let out = measure(seed, DENSE_SPACING, false);
        println!(
            "[A1 rule] seed={seed} sigma_0={:.4} sigma_T={:.4} S={:.4} locked={:.4}",
            out.sigma_0, out.sigma_t, out.s, out.locked_fraction
        );
        assert_eq!(
            out.hash_0, out.hash_t,
            "seed {seed}: sin la regla nada puede mutar ω (hash ω(0) != hash ω(T))"
        );
        assert_eq!(out.s, 0.0, "seed {seed}: S ablado debe ser 0 exacto");
    }
}

/// Ax6-ECS-3 (A2, ablación de alcance): con el system ACTIVO pero vecindad vacía,
/// ω(T) también es bit-idéntico a ω(0). El orden exige interacción efectiva, no
/// la mera presencia del código en el schedule.
#[test]
fn range_ablation_leaves_frequencies_bit_identical() {
    for seed in SEEDS {
        let out = measure(seed, SPARSE_SPACING, true);
        println!(
            "[A2 range] seed={seed} sigma_0={:.4} sigma_T={:.4} S={:.4} locked={:.4}",
            out.sigma_0, out.sigma_t, out.s, out.locked_fraction
        );
        assert_eq!(
            out.hash_0, out.hash_t,
            "seed {seed}: fuera del scan radius la regla no puede mutar ω"
        );
        assert_eq!(out.s, 0.0, "seed {seed}: S ablado debe ser 0 exacto");
    }
}

/// Ax6-ECS-4: veredicto formal contra **cada** ablación por separado.
#[test]
fn verdict_holds_against_both_ablations() {
    for seed in SEEDS {
        let coupled = measure(seed, DENSE_SPACING, true);
        let rule_ablated = measure(seed, DENSE_SPACING, false);
        let range_ablated = measure(seed, SPARSE_SPACING, true);
        println!(
            "[verdict] seed={seed} S={:.4} S_A1={:.4} S_A2={:.4} locked={:.4} sigma_0={:.4} sigma_T={:.4}",
            coupled.s,
            rule_ablated.s,
            range_ablated.s,
            coupled.locked_fraction,
            coupled.sigma_0,
            coupled.sigma_t
        );
        assert!(
            sync_ecs_verdict(coupled.s, rule_ablated.s),
            "seed {seed}: veredicto vs A1 (regla) FAIL — S={} S_A1={}",
            coupled.s,
            rule_ablated.s
        );
        assert!(
            sync_ecs_verdict(coupled.s, range_ablated.s),
            "seed {seed}: veredicto vs A2 (alcance) FAIL — S={} S_A2={}",
            coupled.s,
            range_ablated.s
        );
    }
}

/// Ax6-ECS-5: reproducibilidad bit a bit entre dos corridas de la misma seed.
///
/// Cubre las dos fuentes de orden determinista: orden de spawn → orden de
/// iteración de query, y `query_radius` ordenando por `Entity::to_bits`. Si Bevy
/// cambia el orden de iteración o el backend espacial su criterio de sort, esto
/// lo detecta.
#[test]
fn probe_is_bit_reproducible() {
    let run_a = measure(REFERENCE_SEED, DENSE_SPACING, true);
    let run_b = measure(REFERENCE_SEED, DENSE_SPACING, true);
    assert_eq!(
        run_a.hash_0, run_b.hash_0,
        "poblaciones iniciales divergen entre corridas de la misma seed"
    );
    assert_eq!(
        run_a.hash_t, run_b.hash_t,
        "estado final divergente: hash ω(T) {} != {}",
        run_a.hash_t, run_b.hash_t
    );
    assert_eq!(run_a.s.to_bits(), run_b.s.to_bits(), "S no reproducible");
}
