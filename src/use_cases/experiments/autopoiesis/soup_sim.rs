//! AP-6c (ADR-040): `SoupSim` — streaming incremental stepper.
//! AP-6c (ADR-040): `SoupSim` — streaming incremental stepper.
//!
//! Encapsula el estado mutable que vivía en variables locales de
//! `run_soup_with_network` (ver mod.rs `run_to_report_legacy`).  Expone:
//!
//!   - `new(cfg, net)` — prepara grid + food + estado vacío.
//!   - `step()`        — avanza 1 tick.  Idempotente sobre `self.tick >= ticks`.
//!   - `finish()`      — consume, agrega ventanas y produce `SoupReport`.
//!   - `run_to_end()`  — loop `while self.tick < ticks { step(); } finish()`.
//!
//! Invariante **byte-equivalence** (ADR-040 §5 + golden test):
//!   `SoupSim::new(cfg.clone(), net).run_to_end()` produce un `SoupReport`
//!   idéntico byte-a-byte al legacy `run_to_report_legacy(cfg, net)` cuando
//!   `fission_events` está vacío.  Cuando hay fisiones el reporte diverge
//!   **por diseño** (ADR-041) — el legacy no las genera.
//!
//! Bevy-free: ningún import `bevy::*`.  La viz Bevy de AP-6c.1+ envuelve esta
//! struct con `#[derive(Resource)]` en el bin.
//!
//! ADR-041: el stepper wirea `apply_fission` + `LineageGrid` + registra
//! `FissionEventRecord`s.  El legacy sólo contaba `pressure_events`.

use crate::blueprint::constants::chemistry::{
    BLOB_STRENGTH_FRACTION, FISSION_PRESSURE_RATIO, REACTION_FREQ_BANDWIDTH_DEFAULT,
    SPECIES_DIFFUSION_RATE,
};
use crate::blueprint::equations::fission::{apply_fission, hash_to_lineage, pinch_axis};
use crate::blueprint::equations::{
    compute_membrane_field, compute_strength_field, diffuse_species, find_blobs,
    kinetic_stability, pressure_ratio, raf_closures, step_grid_reactions,
};
use crate::layers::closure_membrane_mask::ClosureMembraneMask;
use crate::layers::lineage_grid::LineageGrid;
use crate::layers::reaction::SpeciesId;
use crate::layers::reaction_network::ReactionNetwork;
use crate::layers::species_grid::{SpeciesCell, SpeciesGrid};

use super::{
    ClosureFate, FateTrack, FissionEventRecord, SoupConfig, SoupReport, random_food_set,
};

/// Estado mutable completo de una corrida.  ADR-040 §2.
///
/// Campos en 3 grupos (agrupados por rol, no por orden de uso):
///   - configuración inmutable: `cfg`, `net`, `food`, `bandwidth`
///   - hot state (por tick): `grid`, `mask`, scratch vecs, `tick`, `lineage_grid`
///   - cold aggregates (sólo detection ticks + finish): `tracks`, dissipated,
///     `n_closures_initial`, `last_final_hashes`, `fission_events`
pub struct SoupSim {
    // Config (inmutables tras `new`).
    cfg: SoupConfig,
    net: ReactionNetwork,
    food: Vec<SpeciesId>,
    bandwidth: f32,

    // Hot state.
    grid: SpeciesGrid,
    mask: ClosureMembraneMask,
    lineage_grid: LineageGrid,
    scratch_cells: Vec<SpeciesCell>,
    damp_field: Vec<f32>,
    strength_field: Vec<f32>,
    tick: u64,

    // Aggregates.
    total_dissipated: f32,
    tracks: Vec<FateTrack>,
    n_closures_initial: u32,
    initial_snapshot_taken: bool,
    last_final_hashes: Vec<u64>,
    fission_events: Vec<FissionEventRecord>,
}

impl SoupSim {
    /// Inicializa grid + food + estado vacío.  No corre ningún tick.
    /// Equivale al pre-loop de `run_to_report_legacy`.
    pub fn new(cfg: SoupConfig, net: ReactionNetwork) -> Self {
        // Cuando la red viene cargada (RON / test), su `species_upper_bound` es
        // la fuente de verdad — evita incoherencias `--network X --species N`.
        let n_species = if net.is_empty() { cfg.n_species } else { net.species_upper_bound() };
        let food = random_food_set(cfg.seed, n_species, cfg.food_size);

        let (w, h) = cfg.grid;
        let mut grid = SpeciesGrid::new(w, h, 50.0);
        // Dos modos de siembra (ver `SoupConfig::food_spot_radius`):
        //   uniform → legacy, AP-5 proptest
        //   spot    → AP-6 items 2+3 (rompe simetría para permitir blobs/fisión)
        match cfg.food_spot_radius {
            None => {
                for &s in &food {
                    for y in 0..h {
                        for x in 0..w {
                            grid.seed(x, y, s, cfg.initial_food_qe);
                        }
                    }
                }
            }
            Some(radius) => {
                let cx = w / 2;
                let cy = h / 2;
                let r = radius as isize;
                for &s in &food {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            let x = cx as isize + dx;
                            let y = cy as isize + dy;
                            if x < 0 || y < 0 || x as usize >= w || y as usize >= h { continue; }
                            grid.seed(x as usize, y as usize, s, cfg.initial_food_qe);
                        }
                    }
                }
            }
        }

        Self {
            scratch_cells: Vec::with_capacity(grid.len()),
            damp_field: Vec::with_capacity(grid.len()),
            strength_field: Vec::with_capacity(grid.len()),
            mask: ClosureMembraneMask::new(),
            lineage_grid: LineageGrid::new(w, h),
            grid,
            bandwidth: REACTION_FREQ_BANDWIDTH_DEFAULT,
            tick: 0,
            total_dissipated: 0.0,
            tracks: Vec::new(),
            n_closures_initial: 0,
            initial_snapshot_taken: false,
            last_final_hashes: Vec::new(),
            fission_events: Vec::new(),
            cfg,
            net,
            food,
        }
    }

    // ── Getters (lectura sólo — la viz observa, no muta) ───────────────────

    #[inline] pub fn tick(&self) -> u64 { self.tick }
    #[inline] pub fn config(&self) -> &SoupConfig { &self.cfg }
    #[inline] pub fn network(&self) -> &ReactionNetwork { &self.net }
    #[inline] pub fn food(&self) -> &[SpeciesId] { &self.food }
    #[inline] pub fn grid(&self) -> &SpeciesGrid { &self.grid }
    #[inline] pub fn mask(&self) -> &ClosureMembraneMask { &self.mask }
    #[inline] pub fn lineage_grid(&self) -> &LineageGrid { &self.lineage_grid }
    #[inline] pub fn total_dissipated(&self) -> f32 { self.total_dissipated }
    #[inline] pub fn fission_events(&self) -> &[FissionEventRecord] { &self.fission_events }

    /// `true` si la simulación ya alcanzó `cfg.ticks`.
    #[inline] pub fn is_done(&self) -> bool { self.tick >= self.cfg.ticks }

    // ── step: avanza 1 tick ────────────────────────────────────────────────

    /// Avanza un tick.  Idempotente tras `is_done()` — llamadas extra no-op.
    pub fn step(&mut self) {
        if self.is_done() { return; }
        let (w, h) = self.cfg.grid;
        let dt = self.cfg.dt;
        let bw = self.bandwidth;

        self.total_dissipated += step_grid_reactions(&mut self.grid, &self.net, bw, dt);

        // Mask vacía ⇒ damping inactivo; saltamos el cómputo del campo (hot path).
        let damping_slice: Option<&[f32]> = if self.mask.is_empty() {
            None
        } else {
            compute_membrane_field(&self.grid, self.mask.as_array(), 1.0, &mut self.damp_field);
            Some(self.damp_field.as_slice())
        };
        diffuse_species(
            &mut self.grid, &mut self.scratch_cells,
            SPECIES_DIFFUSION_RATE, dt, damping_slice,
        );

        let tick_now = self.tick;
        self.tick += 1;

        if tick_now % self.cfg.detection_every != 0 { return; }

        // Detection window — re-evalúa RAF closures + mask + pressure.
        let closures = raf_closures(&self.net, &self.food);
        let alive_hashes: Vec<u64> = closures.iter().map(|c| c.hash).collect();
        self.mask.clear();
        for c in &closures { self.mask.mark_closure_products(c, &self.net); }

        // Snapshot inicial tras equilibración — tracks + stamping inicial
        // de lineage_grid (ADR-041 §5 paso 4).
        if !self.initial_snapshot_taken && tick_now >= self.cfg.equilibration_ticks {
            self.tracks = closures.iter().map(|c| FateTrack {
                fate: ClosureFate {
                    hash: c.hash,
                    survived: false,
                    pressure_events: 0,
                    k_stability_mean_last: 0.0,
                },
                k_history: Vec::new(),
            }).collect();
            self.n_closures_initial = self.tracks.len() as u32;
            self.initial_snapshot_taken = true;

            // Stamp inicial de linaje por closure — primera-que-marca gana.
            if !self.mask.is_empty() {
                compute_strength_field(
                    &self.grid, self.mask.as_array(), 1.0, &mut self.strength_field,
                );
                let max_s = self.strength_field.iter().copied().fold(0.0_f32, f32::max);
                let thr = max_s * BLOB_STRENGTH_FRACTION;
                if thr > 0.0 {
                    let blobs = find_blobs(&self.strength_field, w, h, thr);
                    // Cada blob recibe un linaje por hash de la closure con
                    // mayor intersección — proxy: la closure con menor hash
                    // entre las alive.  Empate imposible (hashes únicos).
                    if let Some(first_closure_hash) = alive_hashes.iter().copied().min() {
                        let lineage = hash_to_lineage(first_closure_hash);
                        for b in &blobs {
                            self.lineage_grid.stamp_if_unowned(&b.cells, lineage);
                        }
                    }
                }
            }
        }

        // Evaluar pressure + fisionar blobs que crucen (ADR-041 wiring real).
        // freq_ref = cell(0,0): bajo Axiom 7 la sopa equilibra tras
        // `equilibration_ticks`, así que cualquier celda da la misma freq.
        let mut pressure_crossed = false;
        if !self.mask.is_empty() {
            compute_strength_field(
                &self.grid, self.mask.as_array(), 1.0, &mut self.strength_field,
            );
            let max_s = self.strength_field.iter().copied().fold(0.0_f32, f32::max);
            let thr = max_s * BLOB_STRENGTH_FRACTION;
            if thr > 0.0 {
                let blobs = find_blobs(&self.strength_field, w, h, thr);
                for blob in &blobs {
                    let ratio = pressure_ratio(
                        blob, &self.grid, &self.net, self.mask.as_array(), bw,
                    );
                    if ratio > FISSION_PRESSURE_RATIO {
                        pressure_crossed = true;
                        // ADR-041 §5 paso 4: disparar fisión real, stampear
                        // los dos lados y registrar el evento.
                        let axis = pinch_axis(blob);
                        let parent = self.lineage_grid.dominant_lineage(&blob.cells);
                        let outcome = apply_fission(
                            &mut self.grid, blob, axis, parent, tick_now,
                        );
                        self.lineage_grid.stamp(&outcome.cells_a, outcome.lineage_a);
                        self.lineage_grid.stamp(&outcome.cells_b, outcome.lineage_b);
                        self.fission_events.push(FissionEventRecord {
                            tick: tick_now,
                            parent,
                            children: [outcome.lineage_a, outcome.lineage_b],
                            dissipated_qe: outcome.dissipated_qe,
                        });
                        // Tax de dissipation (ADR-039 Axiom 4) — `apply_fission`
                        // ya redujo cada celda por `1 - DISSIPATION_PLASMA`, la
                        // cantidad disipada queda reportada; sumamos al total.
                        self.total_dissipated += outcome.dissipated_qe;
                    }
                }
            }
        }

        // Actualizar fates con closures vivas que pertenezcan al snapshot inicial.
        if self.initial_snapshot_taken {
            let totals = self.grid.totals_per_species();
            let freq_ref = self.grid.cell(0, 0).freq;
            for track in self.tracks.iter_mut() {
                if let Some(pos) = alive_hashes.iter().position(|&h| h == track.fate.hash) {
                    let k = kinetic_stability(
                        &closures[pos], &totals, &self.net, freq_ref, bw,
                    );
                    track.k_history.push(k);
                    if pressure_crossed { track.fate.pressure_events += 1; }
                }
            }
        }

        self.last_final_hashes = alive_hashes;
    }

    // ── finish + run_to_end ────────────────────────────────────────────────

    /// Consume el sim y produce el reporte final.  Equivalente al post-loop
    /// de `run_to_report_legacy`.
    pub fn finish(self) -> SoupReport {
        let window_samples = (self.cfg.last_window_ticks
            / self.cfg.detection_every.max(1)).max(1) as usize;
        let last_final_hashes = self.last_final_hashes;
        let fates: Vec<ClosureFate> = self.tracks.into_iter().map(|mut t| {
            t.fate.survived = last_final_hashes.contains(&t.fate.hash);
            if !t.k_history.is_empty() {
                let n = window_samples.min(t.k_history.len());
                let slice = &t.k_history[t.k_history.len() - n..];
                t.fate.k_stability_mean_last = slice.iter().sum::<f32>() / slice.len() as f32;
            }
            t.fate
        }).collect();

        SoupReport {
            seed: self.cfg.seed,
            n_ticks: self.cfg.ticks,
            n_closures_initial: self.n_closures_initial,
            n_closures_final: last_final_hashes.len() as u32,
            total_dissipated: self.total_dissipated,
            fates,
            fission_events: self.fission_events,
        }
    }

    /// Avanza todos los ticks y produce el reporte.  Equivale al wrapper
    /// público `run_soup_with_network`.
    pub fn run_to_end(mut self) -> SoupReport {
        while !self.is_done() { self.step(); }
        self.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::experiments::autopoiesis::{
        random_reaction_network, run_soup, run_soup_with_network,
    };

    fn fast_cfg(seed: u64) -> SoupConfig {
        SoupConfig {
            seed,
            n_species: 6, n_reactions: 12, food_size: 2,
            grid: (6, 6),
            ticks: 400, equilibration_ticks: 50,
            detection_every: 50, last_window_ticks: 200,
            initial_food_qe: 2.0, dt: 0.1,
            food_spot_radius: None,
        }
    }

    // ── ADR-040: streaming equivalence + determinism ──────────────────────

    #[test]
    fn step_advances_tick_counter() {
        let cfg = fast_cfg(1);
        let net = random_reaction_network(cfg.seed, cfg.n_species, cfg.n_reactions, 2, 2);
        let mut sim = SoupSim::new(cfg, net);
        assert_eq!(sim.tick(), 0);
        sim.step();
        assert_eq!(sim.tick(), 1);
        sim.step();
        assert_eq!(sim.tick(), 2);
    }

    #[test]
    fn step_past_done_is_noop() {
        let mut cfg = fast_cfg(2);
        cfg.ticks = 3;
        let net = random_reaction_network(cfg.seed, cfg.n_species, cfg.n_reactions, 2, 2);
        let mut sim = SoupSim::new(cfg, net);
        for _ in 0..10 { sim.step(); }
        assert_eq!(sim.tick(), 3, "no overshoot");
        assert!(sim.is_done());
    }

    #[test]
    fn run_to_end_matches_run_soup_with_network_wrapper() {
        // El wrapper público delega en SoupSim::run_to_end — ambos paths
        // deben producir reportes byte-idénticos sobre el mismo input.
        let cfg = fast_cfg(123);
        let net_a = random_reaction_network(cfg.seed, cfg.n_species, cfg.n_reactions, 2, 2);
        let net_b = random_reaction_network(cfg.seed, cfg.n_species, cfg.n_reactions, 2, 2);
        let via_sim = SoupSim::new(cfg.clone(), net_a).run_to_end();
        let via_wrapper = run_soup_with_network(&cfg, net_b);
        let ja = serde_json::to_string(&via_sim).unwrap();
        let jb = serde_json::to_string(&via_wrapper).unwrap();
        assert_eq!(ja, jb, "wrapper must delegate byte-exact");
    }

    #[test]
    fn run_to_end_is_deterministic_across_two_runs() {
        // Determinismo es el invariante 7 del track (README.md:116).
        for seed in [11_u64, 42, 1337] {
            let a = run_soup(&fast_cfg(seed));
            let b = run_soup(&fast_cfg(seed));
            let ja = serde_json::to_string(&a).unwrap();
            let jb = serde_json::to_string(&b).unwrap();
            assert_eq!(ja, jb, "non-deterministic at seed {seed}");
        }
    }

    // ── ADR-041: fission wiring ────────────────────────────────────────────

    #[test]
    fn formose_runs_end_to_end_over_autopoietic_harness() {
        // Smoke check: formose.ron carga, corre a través de SoupSim, produce
        // un reporte válido.  NO afirmamos `fission_events > 0`: el criterio
        // del track (README.md:124) es un test de observación (`--ticks 100000`),
        // no unit test — la presión depende de tuning empírico.
        let text = std::fs::read_to_string("assets/reactions/formose.ron")
            .expect("asset must exist");
        let net = crate::layers::reaction_network::ReactionNetwork::from_ron_str(&text)
            .expect("valid RON");
        let cfg = SoupConfig {
            seed: 42, n_species: 4, n_reactions: 4, food_size: 2,
            ticks: 2000, grid: (10, 10),
            equilibration_ticks: 100, detection_every: 50,
            last_window_ticks: 500, initial_food_qe: 3.0, dt: 0.1,
            food_spot_radius: None,
        };
        let report = run_soup_with_network(&cfg, net);
        assert_eq!(report.seed, 42);
        assert_eq!(report.n_ticks, 2000);
        assert!(report.total_dissipated >= 0.0, "Axiom 4 conservation");
        // Si hay fissions, deben respetar su forma — cubierto en otro test.
        for ev in &report.fission_events {
            assert_ne!(ev.children[0], ev.children[1]);
        }
    }

    // ── AP-6 items 2 + 3: fission trigger unreachable finding ────────────
    //
    // Sweeps exploratorios (inicialmente propuestos para congelar un fixture
    // con ≥3 fissions) se removieron tras encontrar que `pressure_ratio`
    // empírico topa en ~0.23 sobre formose + hypercycle × 576 combos (seed,
    // food, grid, qe, spot), mientras que `FISSION_PRESSURE_RATIO=50`.  El
    // gap es 200× y no se cierra por tuning de parámetros — es un gap de
    // calibración entre `pressure_ratio` (equations/fission.rs:91) y el
    // umbral derivado en chemistry.rs:83.  Ver `SPRINT_AP6_AUTOPOIETIC_LAB.md`
    // "Findings" y follow-up ADR-039 revisit.
    //
    // Se conserva `spot_seeded_formose_produces_nonzero_gradient` como
    // regresión: confirma que `food_spot_radius` rompe la simetría del grid
    // (precondición necesaria, aunque no suficiente, para que emerja un blob).

    fn sweep_cfg(seed: u64, food: usize, grid_n: usize, qe: f32, ticks: u64) -> SoupConfig {
        SoupConfig {
            seed, n_species: 8, n_reactions: 16, food_size: food,
            grid: (grid_n, grid_n), ticks,
            equilibration_ticks: 100, detection_every: 50,
            last_window_ticks: 1000, initial_food_qe: qe, dt: 0.1,
            food_spot_radius: None,
        }
    }

    #[test]
    fn formose_spot_seeded_produces_at_least_one_fission() {
        // AP-6d regresión post-calibración: con `FISSION_PRESSURE_RATIO = 4`
        // (gas/liquid), formose bajo spot seeding entra en overdrive
        // transient al estabilizar el spot y dispara ≥1 fisión. Fixture
        // congelada para evitar regresiones futuras sobre el umbral.
        //
        // Combo ganador empírico del sweep exhaustivo (ver F-1a sprint doc):
        // seed=0, food_size=2, grid=16×16, qe=50, spot=2, ticks=5000.
        // Budget generoso (ticks=10_000) para tolerar reordering PRNG.
        let text = std::fs::read_to_string("assets/reactions/formose.ron").unwrap();
        let net = ReactionNetwork::from_ron_str(&text).unwrap();
        let cfg = SoupConfig {
            seed: 0, n_species: 4, n_reactions: 4, food_size: 2,
            grid: (16, 16), ticks: 10_000,
            equilibration_ticks: 100, detection_every: 50,
            last_window_ticks: 1000, initial_food_qe: 50.0, dt: 0.1,
            food_spot_radius: Some(2),
        };
        let r = run_soup_with_network(&cfg, net);
        assert!(
            !r.fission_events.is_empty(),
            "formose spot-seeded must trigger ≥1 fission post AP-6d; got {r:?}",
        );
        // Conservación: cada fisión reporta dissipated_qe ≥ 0 (Axiom 4).
        for ev in &r.fission_events {
            assert!(ev.dissipated_qe >= 0.0);
            assert_ne!(ev.children[0], ev.children[1]);
        }
    }

    #[test]
    fn spot_seeded_formose_produces_nonzero_membrane_gradient() {
        // AP-6 item 3 regresión: la siembra localizada es condición necesaria
        // (no suficiente — ver finding) para que un blob emerja.  Verificamos
        // que `max strength_field > 0`, contrastando con el modo uniforme.
        use crate::blueprint::equations::{compute_strength_field, raf_closures};
        use crate::layers::closure_membrane_mask::ClosureMembraneMask;
        let text = std::fs::read_to_string("assets/reactions/formose.ron").unwrap();
        let net = ReactionNetwork::from_ron_str(&text).unwrap();

        // Spot-seeded: un gradiente debe formarse.
        let mut cfg_spot = sweep_cfg(42, 4, 16, 50.0, 400);
        cfg_spot.detection_every = 50;
        cfg_spot.food_spot_radius = Some(3);
        let mut sim = SoupSim::new(cfg_spot.clone(), net.clone());
        let mut max_s_spot = 0.0_f32;
        while !sim.is_done() {
            sim.step();
            if sim.tick() % cfg_spot.detection_every != 1 { continue; }
            let closures = raf_closures(sim.network(), sim.food());
            let mut mask = ClosureMembraneMask::new();
            for c in &closures { mask.mark_closure_products(c, sim.network()); }
            if mask.is_empty() { continue; }
            let mut strength = Vec::with_capacity(sim.grid().len());
            compute_strength_field(sim.grid(), mask.as_array(), 1.0, &mut strength);
            let m = strength.iter().copied().fold(0.0_f32, f32::max);
            if m > max_s_spot { max_s_spot = m; }
        }
        assert!(max_s_spot > 0.0,
            "spot seeding must produce non-zero membrane gradient (got {max_s_spot})");

        // Uniform (control): gradiente cero por simetría traslacional.
        let cfg_uniform = SoupConfig { food_spot_radius: None, ..cfg_spot };
        let mut sim = SoupSim::new(cfg_uniform.clone(), net);
        let mut max_s_uniform = 0.0_f32;
        while !sim.is_done() {
            sim.step();
            if sim.tick() % cfg_uniform.detection_every != 1 { continue; }
            let closures = raf_closures(sim.network(), sim.food());
            let mut mask = ClosureMembraneMask::new();
            for c in &closures { mask.mark_closure_products(c, sim.network()); }
            if mask.is_empty() { continue; }
            let mut strength = Vec::with_capacity(sim.grid().len());
            compute_strength_field(sim.grid(), mask.as_array(), 1.0, &mut strength);
            let m = strength.iter().copied().fold(0.0_f32, f32::max);
            if m > max_s_uniform { max_s_uniform = m; }
        }
        assert_eq!(max_s_uniform, 0.0,
            "uniform seeding is translationally invariant ⇒ gradient ≡ 0 (got {max_s_uniform})");
    }

    #[test]
    #[ignore = "finding record — demonstrates ratio ceiling ≈ 0.23 << threshold 50"]
    fn sweep_canonical_finds_fission_fixture() {
        // Formose + hypercycle × seed × food × grid × qe × spot_radius.
        // spot_radius rompe simetría (gradiente ≠ 0 ⇒ blobs).
        let assets = [
            ("formose",    "assets/reactions/formose.ron"),
            ("hypercycle", "assets/reactions/hypercycle.ron"),
        ];
        let mut best: (usize, String) = (0, String::from("none"));
        for (label, path) in assets {
            let text = std::fs::read_to_string(path).unwrap();
            let net = ReactionNetwork::from_ron_str(&text).unwrap();
            for seed in 0_u64..8 {
                for &food in &[2_usize, 3] {
                    for &g in &[16_usize, 24] {
                        for &qe in &[50.0_f32, 200.0, 500.0] {
                            for &spot in &[2_usize, 4, 6] {
                                let mut cfg = sweep_cfg(seed, food, g, qe, 5000);
                                cfg.detection_every = 25;
                                cfg.food_spot_radius = Some(spot);
                                let r = run_soup_with_network(&cfg, net.clone());
                                let n = r.fission_events.len();
                                if n > best.0 {
                                    best = (n, format!(
                                        "{label} seed={seed} food={food} grid={g}x{g} qe={qe} \
                                         spot={spot} ticks=5000 \
                                         → closures_final={} fissions={} dissipated={:.1}",
                                        r.n_closures_final, n, r.total_dissipated,
                                    ));
                                    println!("NEW BEST: {}", best.1);
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("FINAL BEST: {} = {}", best.0, best.1);
        assert!(best.0 >= 3, "no combo reached ≥3 fissions (best={}); widen sweep", best.0);
    }

    #[test]
    #[ignore = "diagnostic — inspect max pressure_ratio reached"]
    fn diagnose_max_pressure_ratio_on_formose() {
        use crate::blueprint::equations::{
            compute_strength_field, find_blobs, pressure_ratio, raf_closures,
        };
        let text = std::fs::read_to_string("assets/reactions/formose.ron").unwrap();
        let net = ReactionNetwork::from_ron_str(&text).unwrap();
        for &qe in &[50.0_f32, 500.0] {
            for &g in &[8_usize, 16] {
                for &food in &[3_usize, 4] {
                    let mut cfg = sweep_cfg(42, food, g, qe, 2000);
                    cfg.detection_every = 10;
                    cfg.food_spot_radius = Some(3);
                    let mut sim = SoupSim::new(cfg.clone(), net.clone());
                    let mut max_ratio = 0.0_f32;
                    let mut max_blob_size = 0usize;
                    let mut n_closures_seen = 0usize;
                    let mut n_mask_nonempty = 0usize;
                    let mut max_max_s = 0.0_f32;
                    let mut n_detections = 0usize;
                    let food_ids: Vec<u8> = sim.food().iter().map(|s| s.raw()).collect();
                    while !sim.is_done() {
                        sim.step();
                        if sim.tick() % cfg.detection_every != 1 { continue; }
                        n_detections += 1;
                        let closures = raf_closures(sim.network(), sim.food());
                        if !closures.is_empty() { n_closures_seen += 1; }
                        let mut mask = crate::layers::closure_membrane_mask::ClosureMembraneMask::new();
                        for c in &closures { mask.mark_closure_products(c, sim.network()); }
                        if !mask.is_empty() { n_mask_nonempty += 1; }
                        if mask.is_empty() { continue; }
                        let mut strength = Vec::with_capacity(sim.grid().len());
                        compute_strength_field(sim.grid(), mask.as_array(), 1.0, &mut strength);
                        let max_s = strength.iter().copied().fold(0.0_f32, f32::max);
                        if max_s > max_max_s { max_max_s = max_s; }
                        if max_s <= 0.0 { continue; }
                        let thr = max_s * crate::blueprint::constants::chemistry::BLOB_STRENGTH_FRACTION;
                        let blobs = find_blobs(&strength, g, g, thr);
                        for b in &blobs {
                            max_blob_size = max_blob_size.max(b.cells.len());
                            let r = pressure_ratio(b, sim.grid(), sim.network(), mask.as_array(), 50.0);
                            if r > max_ratio { max_ratio = r; }
                        }
                    }
                    println!(
                        "qe={:>6} g={g} food={food} ids={:?} det={n_detections} clos+={n_closures_seen} \
                         mask+={n_mask_nonempty} max_s={:.3} max_blob={max_blob_size} max_ratio={:.3}",
                        qe, food_ids, max_max_s, max_ratio,
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "exploratory sweep — random soup viable defaults"]
    fn sweep_random_finds_persistent_closure_and_fission() {
        // Busca params de sopa aleatoria (red generada por seed) con
        // ≥1 closure persistente + ≥1 fission.  Requiere spot seeding.
        let mut best: (u32, usize, String) = (0, 0, String::from("none"));
        for seed in 0_u64..16 {
            for &food in &[2_usize, 3] {
                for &g in &[16_usize, 24] {
                    for &qe in &[50.0_f32, 200.0, 500.0] {
                        for &spot in &[2_usize, 4] {
                            let mut cfg = sweep_cfg(seed, food, g, qe, 10_000);
                            cfg.food_spot_radius = Some(spot);
                            let ticks = cfg.ticks;
                            let r = run_soup(&cfg);
                            let persistent = r.fates.iter().filter(|f| f.survived).count() as u32;
                            let fissions = r.fission_events.len();
                            // score: priorizar presencia de fisiones, después closures sobrevivientes.
                            let score = fissions * 100 + persistent as usize;
                            let best_score = best.1 * 100 + best.0 as usize;
                            if score > best_score {
                                best = (persistent, fissions, format!(
                                    "seed={seed} food={food} grid={g}x{g} qe={qe} spot={spot} ticks={ticks} \
                                     → closures_surv={persistent} fissions={fissions} \
                                     closures_final={} dissipated={:.2}",
                                    r.n_closures_final, r.total_dissipated,
                                ));
                                println!("NEW BEST: {}", best.2);
                            }
                        }
                    }
                }
            }
        }
        println!("FINAL BEST: persistent={} fissions={} | {}", best.0, best.1, best.2);
        assert!(best.0 >= 1 && best.1 >= 1,
            "no combo found ≥1 persistent + ≥1 fission (best={:?}); widen sweep", best);
    }

    #[test]
    fn fission_event_children_are_distinct_from_parent() {
        // Si hay eventos, cada uno debe cumplir: children[0] != children[1]
        // y ambos != parent (ADR-041 vía child_lineage).
        let cfg = SoupConfig {
            seed: 42, ticks: 2000, grid: (10, 10),
            food_size: 3, ..fast_cfg(42)
        };
        let report = run_soup(&cfg);
        for ev in &report.fission_events {
            assert_ne!(ev.children[0], ev.children[1], "sides differ");
            if ev.parent != 0 {
                assert_ne!(ev.children[0], ev.parent);
                assert_ne!(ev.children[1], ev.parent);
            }
            assert!(ev.dissipated_qe >= 0.0, "Axiom 4");
        }
    }
}

