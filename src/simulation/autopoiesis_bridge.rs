//! AI-2 (ADR-044): bridge AP-* `SoupSim` ↔ ECS world.
//! AI-2 (ADR-044): bridge AP-* `SoupSim` ↔ ECS world.
//!
//! Tres responsabilidades, una phase (`Phase::ChemicalLayer`):
//!
//! 1. **`step_soup_sim_system`** — avanza `SoupSim` un tick por `FixedUpdate`
//!    cuando el resource está presente (opt-in).
//! 2. **`emit_fission_events_system`** — convierte `FissionEventRecord`s
//!    nuevos del cursor en `FissionEvent`s ECS.
//! 3. **`on_fission_spawn_entity`** — consume `FissionEvent` y spawn una
//!    entity por hijo con `BaseEnergy + OscillatorySignature + LineageTag +
//!    Transform + StateScoped`.
//!
//! `SoupSim` permanece Bevy-free (ADR-040 §2).  El bridge es opt-in via
//! presencia de `SoupSimResource` — sin él, el system es no-op y los
//! tracks que no usan química explícita quedan intactos.

use bevy::prelude::*;

use crate::events::FissionEvent;
use crate::layers::energy::BaseEnergy;
use crate::layers::lineage_tag::LineageTag;
use crate::layers::oscillatory::OscillatorySignature;
use crate::layers::reaction_network::ReactionNetwork;
use crate::layers::species_grid::SpeciesGrid;
use crate::math_types::Vec2;
use crate::simulation::states::GameState;
use crate::use_cases::experiments::autopoiesis::SoupSim;

/// Cota dura por tick para evitar explosión combinatoria en cascadas
/// de fisión patológicas (ADR-044 §9).  Cada evento spawnea 2 entities
/// ⇒ tope = 8 entities/tick.
const MAX_FISSION_EVENTS_PER_TICK: usize = 4;

// ── Resources ──────────────────────────────────────────────────────────────

/// Wrapper de `SoupSim` para uso como `Resource` Bevy.  Mantiene el stepper
/// Bevy-free de ADR-040 — sólo este wrapper lo expone al world ECS.
#[derive(Resource)]
pub struct SoupSimResource(pub SoupSim);

/// Cursor que recuerda cuántos `FissionEventRecord`s ya se emitieron como
/// `FissionEvent`s.  Resetear al recargar la sopa.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct FissionEventCursor {
    pub last_processed: usize,
}

// ── Systems ────────────────────────────────────────────────────────────────

/// Avanza `SoupSim` un tick por `FixedUpdate` cuando el resource está presente.
/// No-op silencioso sin sopa cargada.
pub fn step_soup_sim_system(sim: Option<ResMut<SoupSimResource>>) {
    let Some(mut sim) = sim else { return; };
    if !sim.0.is_done() { sim.0.step(); }
}

/// Lee `SoupSim::fission_events()` desde `cursor.last_processed..` y emite
/// los nuevos como `FissionEvent`s.  Avanza el cursor al final.
///
/// Cap `MAX_FISSION_EVENTS_PER_TICK`: si hay más eventos pendientes, sólo
/// procesa los primeros y log warning.  El resto queda para el próximo tick
/// (cursor avanza sólo lo procesado).
pub fn emit_fission_events_system(
    sim: Option<Res<SoupSimResource>>,
    network: Option<Res<ReactionNetwork>>,
    species: Option<Res<SpeciesGrid>>,
    mut cursor: ResMut<FissionEventCursor>,
    mut events: EventWriter<FissionEvent>,
) {
    let (Some(sim), Some(network), Some(species)) = (sim, network, species)
        else { return; };
    let records = sim.0.fission_events();
    let pending = records.len().saturating_sub(cursor.last_processed);
    if pending == 0 { return; }
    let take = pending.min(MAX_FISSION_EVENTS_PER_TICK);
    if pending > MAX_FISSION_EVENTS_PER_TICK {
        warn!(
            "AI-2: {pending} fission events pending, capping at {take} per tick \
             (cursor advances only by {take})",
        );
    }
    let end = cursor.last_processed + take;
    for record in &records[cursor.last_processed..end] {
        let event = build_fission_event(record, &network, &species);
        events.send(event);
    }
    cursor.last_processed = end;
}

/// Convierte un `FissionEventRecord` (raw del stepper AP-*) a `FissionEvent`
/// (Bevy event con datos derivados para spawn).
///
/// Pure fn — testeable sin App.  Computa `mean_freq` desde la red usando la
/// composición de species en la celda del centroide.
fn build_fission_event(
    record: &crate::use_cases::experiments::autopoiesis::FissionEventRecord,
    network: &ReactionNetwork,
    species: &SpeciesGrid,
) -> FissionEvent {
    let cx = record.centroid.0;
    let cy = record.centroid.1;
    // mean_freq: muestra la celda más cercana al centroide (round half-down).
    // Si fuera del grid, fallback a freq promedio de toda la red.
    let mean_freq = species
        .cell_xy_clamped(cx, cy)
        .map(|cell| network.mean_product_frequency(&cell.species))
        .unwrap_or_else(|| network_mean_freq_fallback(network));
    FissionEvent {
        tick: record.tick,
        parent_lineage: record.parent,
        children_lineages: record.children,
        centroid: Vec2::new(cx, cy),
        mean_freq,
        qe_per_child: record.qe_per_child,
    }
}

/// Promedio simple de `freq` de todas las reacciones de la red.  Fallback
/// usado por `build_fission_event` cuando el centroide cae fuera del grid.
fn network_mean_freq_fallback(network: &ReactionNetwork) -> f32 {
    let n = network.reactions().len();
    if n == 0 { return 0.0; }
    let sum: f32 = network.reactions().iter().map(|r| r.freq).sum();
    sum / n as f32
}

/// Spawn una entity por hijo (2 por evento) con los atributos derivados
/// del blob padre.  `qe_per_child <= 0` ⇒ skip (blob colapsado, no genera vida).
/// `lineage == 0` ⇒ skip (sopa primordial, no entity concreta).
pub fn on_fission_spawn_entity(
    mut commands: Commands,
    mut events: EventReader<FissionEvent>,
) {
    for ev in events.read() {
        if ev.qe_per_child <= 0.0 { continue; }
        for &lineage in &ev.children_lineages {
            if lineage == 0 { continue; }
            commands.spawn((
                BaseEnergy::new(ev.qe_per_child),
                OscillatorySignature::new(ev.mean_freq, 0.0),
                LineageTag::new(lineage),
                Transform::from_translation(Vec3::new(ev.centroid.x, ev.centroid.y, 0.0)),
                StateScoped(GameState::Playing),
                Name::new(format!("protocell_lin{:08x}", (lineage & 0xFFFF_FFFF) as u32)),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::experiments::autopoiesis::FissionEventRecord;

    fn formose_net() -> ReactionNetwork {
        let text = std::fs::read_to_string("assets/reactions/formose.ron").unwrap();
        ReactionNetwork::from_ron_str(&text).unwrap()
    }

    fn record(tick: u64, children: [u64; 2], qe_per_child: f32, centroid: (f32, f32))
        -> FissionEventRecord
    {
        FissionEventRecord {
            tick, parent: 0,
            children, dissipated_qe: 1.0,
            centroid, qe_per_child,
        }
    }

    // ── build_fission_event ────────────────────────────────────────────────

    #[test]
    fn build_event_uses_centroid_cell_for_freq() {
        let net = formose_net();
        let mut species = SpeciesGrid::new(8, 8, 50.0);
        // sembrar producto C2 (id=1) en la celda (4,4) ⇒ mean_product_frequency
        // ponderará con freq de reacciones que producen C2 (r0, r3 → freq=50)
        species.seed(4, 4, crate::layers::reaction::SpeciesId::new(1).unwrap(), 10.0);
        let r = record(7, [0xAA, 0xBB], 1.5, (4.0, 4.0));
        let ev = build_fission_event(&r, &net, &species);
        assert_eq!(ev.tick, 7);
        assert_eq!(ev.children_lineages, [0xAA, 0xBB]);
        assert_eq!(ev.centroid, Vec2::new(4.0, 4.0));
        assert!(ev.mean_freq > 0.0, "got freq={}", ev.mean_freq);
        assert!((ev.qe_per_child - 1.5).abs() < 1e-6);
    }

    #[test]
    fn build_event_falls_back_when_centroid_out_of_grid() {
        let net = formose_net();
        let species = SpeciesGrid::new(4, 4, 50.0);
        let r = record(0, [1, 2], 0.5, (100.0, 100.0)); // fuera de grid
        let ev = build_fission_event(&r, &net, &species);
        // Fallback: promedio de freqs en formose = 50 Hz (todas son 50).
        assert!((ev.mean_freq - 50.0).abs() < 1e-3, "fallback freq={}", ev.mean_freq);
    }

    #[test]
    fn build_event_zero_freq_on_empty_network() {
        let net = ReactionNetwork::default();
        let species = SpeciesGrid::new(4, 4, 50.0);
        let r = record(0, [1, 2], 1.0, (1.0, 1.0));
        let ev = build_fission_event(&r, &net, &species);
        assert_eq!(ev.mean_freq, 0.0);
    }

    // ── on_fission_spawn_entity (system) ──────────────────────────────────

    fn run_app_with_event(ev: FissionEvent) -> App {
        use bevy::state::app::StatesPlugin;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_event::<FissionEvent>();
        app.init_state::<GameState>();
        app.add_systems(Update, on_fission_spawn_entity);
        app.world_mut().send_event(ev);
        app.update();
        app
    }

    fn make_event(qe: f32, lin_a: u64, lin_b: u64) -> FissionEvent {
        FissionEvent {
            tick: 1, parent_lineage: 0,
            children_lineages: [lin_a, lin_b],
            centroid: Vec2::new(2.0, 3.0),
            mean_freq: 50.0, qe_per_child: qe,
        }
    }

    #[test]
    fn observer_spawns_two_entities_with_distinct_lineages() {
        let mut app = run_app_with_event(make_event(2.5, 0xAAAA, 0xBBBB));
        let count = app.world_mut().query::<&LineageTag>().iter(app.world()).count();
        assert_eq!(count, 2, "expected 2 entities");
        let lineages: std::collections::HashSet<u64> = app
            .world_mut().query::<&LineageTag>()
            .iter(app.world()).map(|t| t.0).collect();
        assert!(lineages.contains(&0xAAAA));
        assert!(lineages.contains(&0xBBBB));
    }

    #[test]
    fn observer_skips_when_qe_zero() {
        let mut app = run_app_with_event(make_event(0.0, 1, 2));
        let count = app.world_mut().query::<&LineageTag>().iter(app.world()).count();
        assert_eq!(count, 0, "qe=0 ⇒ no spawn");
    }

    #[test]
    fn observer_skips_primordial_lineage() {
        // Si parent es sopa primordial y children incluye 0 (degenerate),
        // el spawner descarta sólo el 0 — el otro hijo sí spawna.
        let mut app = run_app_with_event(make_event(1.0, 0, 0xDEAD));
        let count = app.world_mut().query::<&LineageTag>().iter(app.world()).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn entities_carry_correct_qe_and_freq() {
        let ev = make_event(7.5, 0x123, 0x456);
        let mut app = run_app_with_event(ev);
        let mut total_qe = 0.0_f32;
        let mut freq_match = 0;
        for (energy, osc) in app.world_mut()
            .query::<(&BaseEnergy, &OscillatorySignature)>()
            .iter(app.world())
        {
            total_qe += energy.qe();
            if (osc.frequency_hz() - 50.0).abs() < 1e-3 { freq_match += 1; }
        }
        assert!((total_qe - 15.0).abs() < 1e-3, "Σqe = 2 × qe_per_child");
        assert_eq!(freq_match, 2);
    }

    // ── cursor / emit_fission_events_system ────────────────────────────────
    //
    // No probamos `step_soup_sim_system` ni `emit_fission_events_system` con App
    // completo aquí — requerirían construir un SoupSim real, que es trabajo de
    // integración cubierto por el escenario E2E del binario `autopoietic_lab`.
    // Los unit tests de las pure fns + del observer cubren la lógica interna.

    #[test]
    fn cursor_default_is_zero() {
        let c = FissionEventCursor::default();
        assert_eq!(c.last_processed, 0);
    }

    #[test]
    fn max_fission_events_per_tick_constant_is_sane() {
        assert!(MAX_FISSION_EVENTS_PER_TICK > 0);
        assert!(MAX_FISSION_EVENTS_PER_TICK <= 16);
    }
}
