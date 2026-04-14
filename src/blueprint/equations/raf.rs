//! AP-1/2: RAF detection (Hordijk-Steel) + kinetic stability (Pross).
//! AP-1/2: RAF detection (Hordijk-Steel) + kinetic stability (Pross).
//!
//! # RAF (Reflexively Autocatalytic and Food-generated)
//!
//! Una subred `R' ⊆ R` es RAF sobre un food set `F` si todo reactivo de toda
//! reacción `r ∈ R'` es producible a partir de `F` mediante reacciones en `R'`.
//!
//! En Resonance, la catálisis de Hordijk-Steel se **absorbe** en la alineación
//! de frecuencia (Axiom 8): toda reacción es auto-catalítica por el ambiente en
//! el que corre.  Por eso la condición reflexive se reduce al food-closure.
//!
//! El algoritmo es un punto fijo: partimos de `R' = R` y eliminamos iterativa-
//! mente reacciones cuyos reactivos no sean producibles.  `O(R² · S)` peor caso
//! — para `R ≤ 256`, `S ≤ 32` corre en microsegundos.
//!
//! # Closures
//!
//! `raf_closures` descompone la RAF maximal en componentes conexas por especie
//! compartida (Union-Find).  Cada componente es un ciclo químico independiente.
//!
//! # Kinetic stability (Pross)
//!
//! `K = reconstruction_rate / decay_rate`.  `K ≥ 1` ⇒ la closure se reconstruye
//! más rápido de lo que se diluye — es dinámicamente estable.  Derivado del
//! cap. 5 de `docs/sintesis_patron_vida_universo.md`.

use std::collections::BTreeMap;

use crate::blueprint::constants::chemistry::{
    FOOD_PRESENCE_THRESHOLD, KINETIC_STABILITY_EPSILON, MAX_SPECIES, RAF_MIN_CLOSURE_REACTIONS,
    SPECIES_DIFFUSION_RATE,
};
use crate::blueprint::equations::reaction_kinetics::mass_action_rate;
use crate::layers::reaction::{Reaction, SpeciesId};
use crate::layers::reaction_network::{ReactionId, ReactionNetwork};

/// Bitmask denso de `MAX_SPECIES` bits empaquetado como `[bool; MAX_SPECIES]`.
/// Compacto en stack (32 B) — más rápido que `HashSet<SpeciesId>` para los
/// tamaños del dominio (`MAX_SPECIES ≤ 32`).  Lo exponemos como type-alias
/// local para claridad de firmas.
type SpeciesMask = [bool; MAX_SPECIES];

#[inline]
fn empty_mask() -> SpeciesMask { [false; MAX_SPECIES] }

/// Máscara de todas las especies (reactivos ∪ productos) de una reacción.
#[inline]
pub fn reaction_species_mask(r: &Reaction) -> SpeciesMask {
    let mut m = empty_mask();
    for e in r.reactants_active().chain(r.products_active()) {
        m[e.species.index()] = true;
    }
    m
}

#[inline]
fn masks_overlap(a: &SpeciesMask, b: &SpeciesMask) -> bool {
    a.iter().zip(b.iter()).any(|(x, y)| *x && *y)
}

#[inline]
fn mask_to_species(m: &SpeciesMask) -> Vec<SpeciesId> {
    m.iter().enumerate()
        .filter_map(|(i, b)| b.then(|| SpeciesId(i as u8)))
        .collect()
}

// ── Food set inference ──────────────────────────────────────────────────────

/// Dado un vector de totales por especie, devuelve las que superan
/// `FOOD_PRESENCE_THRESHOLD` — son el "food set" del grid.
pub fn food_set_from_totals(totals: &[f32; MAX_SPECIES]) -> Vec<SpeciesId> {
    totals.iter().enumerate()
        .filter(|(_, v)| **v >= FOOD_PRESENCE_THRESHOLD)
        .filter_map(|(i, _)| SpeciesId::new(i as u8))
        .collect()
}

/// Variante que acepta una lista arbitraria de índices — útil en tests.
pub fn food_mask(food: &[SpeciesId]) -> [bool; MAX_SPECIES] {
    let mut mask = [false; MAX_SPECIES];
    for s in food {
        if s.is_some() { mask[s.index()] = true; }
    }
    mask
}

// ── RAF detection (Hordijk-Steel fixed point) ───────────────────────────────

/// Subconjunto maximal de reacciones food-generable.  Puede ser vacío.
///
/// Algoritmo forward de Hornijk-Steel: partiendo de `available = food`,
/// activa una reacción si todos sus reactivos están en `available`, y agrega
/// sus productos.  Itera hasta punto fijo.  Descarta ciclos huérfanos
/// (sin origen en food) — el filtrado reverso los aceptaría por error.
///
/// Devuelve `ReactionId`s en orden creciente (estable para hashing).
pub fn find_raf(network: &ReactionNetwork, food: &[SpeciesId]) -> Vec<ReactionId> {
    let n = network.len();
    if n == 0 { return Vec::new(); }

    let mut available = food_mask(food);
    let mut active: Vec<bool> = vec![false; n];

    loop {
        let mut changed = false;
        for (i, r) in network.reactions().iter().enumerate() {
            if active[i] { continue; }
            let ready = r.reactants_active().all(|e| available[e.species.index()]);
            if !ready { continue; }
            active[i] = true;
            for e in r.products_active() {
                if !available[e.species.index()] {
                    available[e.species.index()] = true;
                }
            }
            changed = true;
        }
        if !changed { break; }
    }

    active.iter().enumerate()
        .filter_map(|(i, &b)| b.then_some(ReactionId(i as u32)))
        .collect()
}

// ── Closure ─────────────────────────────────────────────────────────────────

/// Componente conexa de una RAF — "bucle cerrado" químico independiente.
#[derive(Clone, Debug)]
pub struct Closure {
    pub reactions: Vec<ReactionId>,
    pub species: Vec<SpeciesId>,
    pub hash: u64,
}

impl Closure {
    #[inline] pub fn len(&self) -> usize { self.reactions.len() }
    #[inline] pub fn is_empty(&self) -> bool { self.reactions.is_empty() }
}

/// Descompone la RAF maximal en closures disjuntas (Union-Find por especies).
/// Closures con `< RAF_MIN_CLOSURE_REACTIONS` reacciones se descartan.
///
/// Precomputa una `SpeciesMask` por reacción una sola vez — elimina el O(R²·S)
/// recomputo que tendría un naïve share-species check.  Total: O(R² + R·S).
pub fn raf_closures(network: &ReactionNetwork, food: &[SpeciesId]) -> Vec<Closure> {
    let raf = find_raf(network, food);
    if raf.is_empty() { return Vec::new(); }

    // Una máscara por reacción RAF — enumerada por posición en `raf`.
    let masks: Vec<SpeciesMask> = raf.iter()
        .map(|rid| reaction_species_mask(&network.reactions()[rid.index()]))
        .collect();

    let mut uf = UnionFind::new(raf.len());
    for i in 0..raf.len() {
        for j in (i + 1)..raf.len() {
            if masks_overlap(&masks[i], &masks[j]) { uf.union(i, j); }
        }
    }

    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..raf.len() {
        groups.entry(uf.find(i)).or_default().push(i);
    }

    groups.into_values()
        .filter(|members| members.len() >= RAF_MIN_CLOSURE_REACTIONS)
        .map(|members| build_closure_from_members(&raf, &masks, &members))
        .collect()
}

fn build_closure_from_members(
    raf: &[ReactionId],
    masks: &[SpeciesMask],
    members: &[usize],
) -> Closure {
    let mut reactions: Vec<ReactionId> = members.iter().map(|&i| raf[i]).collect();
    reactions.sort_unstable();

    let mut union_mask = empty_mask();
    for &i in members {
        for (dst, src) in union_mask.iter_mut().zip(masks[i].iter()) {
            *dst |= *src;
        }
    }
    let species = mask_to_species(&union_mask);
    let hash = closure_hash(&reactions, &species);
    Closure { reactions, species, hash }
}

/// Hash FNV-1a 64 bits sobre `(reactions ordenadas, species ordenadas)`.
/// Invariante a permutaciones por construcción.
pub fn closure_hash(reactions: &[ReactionId], species: &[SpeciesId]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut h: u64 = FNV_OFFSET;
    for r in reactions {
        h ^= r.0 as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    for s in species {
        h ^= s.0 as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

// ── Kinetic stability (Pross) ───────────────────────────────────────────────

/// Razón reconstrucción / decaimiento para una closure en una celda.
///
/// * `reconstruction = Σ_r∈closure rate(r) × Σ stoich_out`
/// * `decay          = Σ_s∈closure [s] × SPECIES_DIFFUSION_RATE`
/// * `K = reconstruction / max(decay, ε)`
///
/// `K ≥ 1` ⇒ persistente.  Pura, sin side effects.
pub fn kinetic_stability(
    closure: &Closure,
    species: &[f32; MAX_SPECIES],
    network: &ReactionNetwork,
    cell_freq: f32,
    bandwidth: f32,
) -> f32 {
    let mut reconstruction = 0.0_f32;
    for rid in &closure.reactions {
        let r = &network.reactions()[rid.index()];
        let rate = mass_action_rate(species, r, cell_freq, bandwidth);
        let out_stoich: f32 = r.products_active().map(|e| e.count as f32).sum();
        reconstruction += rate * out_stoich;
    }
    let decay: f32 = closure.species.iter()
        .map(|s| species[s.index()] * SPECIES_DIFFUSION_RATE)
        .sum();
    reconstruction / decay.max(KINETIC_STABILITY_EPSILON)
}

// ── Union-Find (local, minimal) ─────────────────────────────────────────────

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self { parent: (0..n).collect(), rank: vec![0; n] }
    }
    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]]; // path compression (halving)
            x = self.parent[x];
        }
        x
    }
    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb { return; }
        let (small, big) = if self.rank[ra] < self.rank[rb] { (ra, rb) } else { (rb, ra) };
        self.parent[small] = big;
        if self.rank[small] == self.rank[big] { self.rank[big] += 1; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::chemistry::REACTION_FREQ_BANDWIDTH_DEFAULT as BW;

    /// Red mínima de 3 reacciones en ciclo cerrado:
    ///   r0:  A + B → C
    ///   r1:  C     → A + D
    ///   r2:  D + B → B + C    (B cataliza + se regenera)
    /// Food set: {A, B}.  RAF = {r0, r1, r2}.
    const MINIMAL_RAF: &str = r#"(
        reactions: [
            (reactants: [(0,1),(1,1)], products: [(2,1)],      k: 1.0, freq: 50.0),
            (reactants: [(2,1)],       products: [(0,1),(3,1)], k: 0.5, freq: 50.0),
            (reactants: [(3,1),(1,1)], products: [(1,1),(2,1)], k: 0.8, freq: 50.0),
        ]
    )"#;

    /// Red sin cierre: todos los reactivos necesitan especies fuera del food.
    const NO_CLOSURE: &str = r#"(
        reactions: [
            (reactants: [(10,1)], products: [(11,1)], k: 1.0, freq: 50.0),
            (reactants: [(11,1)], products: [(12,1)], k: 1.0, freq: 50.0),
        ]
    )"#;

    fn food_ab() -> Vec<SpeciesId> {
        vec![SpeciesId::new(0).unwrap(), SpeciesId::new(1).unwrap()]
    }

    // ── find_raf ───────────────────────────────────────────────────────────

    #[test]
    fn raf_minimal_contains_all_three() {
        let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
        let raf = find_raf(&net, &food_ab());
        assert_eq!(raf.len(), 3);
        assert_eq!(raf, vec![ReactionId(0), ReactionId(1), ReactionId(2)]);
    }

    #[test]
    fn raf_empty_when_food_missing() {
        let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
        let raf = find_raf(&net, &[]);
        assert!(raf.is_empty(), "sin food, nada se sostiene");
    }

    #[test]
    fn raf_filters_unreachable_reactions() {
        let net = ReactionNetwork::from_ron_str(NO_CLOSURE).unwrap();
        let raf = find_raf(&net, &food_ab()); // A, B present — pero r0 necesita especie 10.
        assert!(raf.is_empty());
    }

    #[test]
    fn raf_empty_network_returns_empty() {
        let net = ReactionNetwork::empty();
        assert!(find_raf(&net, &food_ab()).is_empty());
    }

    // ── raf_closures ───────────────────────────────────────────────────────

    #[test]
    fn closures_group_connected_reactions() {
        let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
        let closures = raf_closures(&net, &food_ab());
        assert_eq!(closures.len(), 1, "3 reacciones conectadas → 1 closure");
        assert_eq!(closures[0].reactions.len(), 3);
    }

    #[test]
    fn closures_separate_disjoint_cycles() {
        // Dos ciclos independientes (no comparten ninguna especie).
        let spec = r#"(
            reactions: [
                (reactants: [(0,1),(1,1)], products: [(2,1)],       k: 1.0, freq: 50.0),
                (reactants: [(2,1)],       products: [(0,1),(1,1)], k: 1.0, freq: 50.0),
                (reactants: [(2,1),(0,1)], products: [(1,1)],       k: 1.0, freq: 50.0),
                (reactants: [(10,1),(11,1)], products: [(12,1)],     k: 1.0, freq: 50.0),
                (reactants: [(12,1)],        products: [(10,1),(11,1)], k: 1.0, freq: 50.0),
                (reactants: [(10,1),(12,1)], products: [(11,1)],     k: 1.0, freq: 50.0),
            ]
        )"#;
        let net = ReactionNetwork::from_ron_str(spec).unwrap();
        let food = vec![
            SpeciesId::new(0).unwrap(),  SpeciesId::new(1).unwrap(),
            SpeciesId::new(10).unwrap(), SpeciesId::new(11).unwrap(),
        ];
        let closures = raf_closures(&net, &food);
        assert_eq!(closures.len(), 2, "got {} closures", closures.len());
    }

    #[test]
    fn closure_discards_short_cycles() {
        // Dos reacciones — menor que RAF_MIN_CLOSURE_REACTIONS=3 — se descarta.
        let spec = r#"(
            reactions: [
                (reactants: [(0,1)], products: [(1,1)], k: 1.0, freq: 50.0),
                (reactants: [(1,1)], products: [(0,1)], k: 1.0, freq: 50.0),
            ]
        )"#;
        let net = ReactionNetwork::from_ron_str(spec).unwrap();
        let closures = raf_closures(&net, &[SpeciesId::new(0).unwrap()]);
        assert!(closures.is_empty());
    }

    // ── hash stability ─────────────────────────────────────────────────────

    #[test]
    fn hash_invariant_to_input_order() {
        let a_rxns = vec![ReactionId(2), ReactionId(0), ReactionId(1)];
        let a_sp   = vec![SpeciesId(3), SpeciesId(0), SpeciesId(2)];
        let b_rxns = vec![ReactionId(0), ReactionId(1), ReactionId(2)];
        let b_sp   = vec![SpeciesId(0), SpeciesId(2), SpeciesId(3)];
        // `build_closure` sorts rxns; species vienen ordenados por construcción.
        let mut a_sorted = a_rxns.clone(); a_sorted.sort_unstable();
        let mut a_sp_sorted = a_sp.clone(); a_sp_sorted.sort_unstable();
        assert_eq!(
            closure_hash(&a_sorted, &a_sp_sorted),
            closure_hash(&b_rxns,   &b_sp),
        );
    }

    #[test]
    fn hash_distinguishes_different_closures() {
        let h1 = closure_hash(&[ReactionId(0), ReactionId(1)], &[SpeciesId(0)]);
        let h2 = closure_hash(&[ReactionId(0), ReactionId(2)], &[SpeciesId(0)]);
        assert_ne!(h1, h2);
    }

    // ── food_set_from_totals ───────────────────────────────────────────────

    #[test]
    fn food_set_filters_by_threshold() {
        let mut totals = [0.0; MAX_SPECIES];
        totals[0] = FOOD_PRESENCE_THRESHOLD * 2.0;
        totals[5] = FOOD_PRESENCE_THRESHOLD / 2.0; // below threshold
        totals[7] = FOOD_PRESENCE_THRESHOLD;        // exactly at threshold
        let food = food_set_from_totals(&totals);
        assert!(food.contains(&SpeciesId(0)));
        assert!(food.contains(&SpeciesId(7)));
        assert!(!food.contains(&SpeciesId(5)));
    }

    // ── kinetic_stability (Pross) ──────────────────────────────────────────

    #[test]
    fn k_stability_rises_with_food_abundance() {
        let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
        let closures = raf_closures(&net, &food_ab());
        let c = &closures[0];
        let mut scarce = [0.0; MAX_SPECIES];
        scarce[0] = 0.1; scarce[1] = 0.1; scarce[2] = 0.1; scarce[3] = 0.1;
        let mut plenty = [0.0; MAX_SPECIES];
        plenty[0] = 10.0; plenty[1] = 10.0; plenty[2] = 1.0; plenty[3] = 1.0;
        let k_scarce = kinetic_stability(c, &scarce, &net, 50.0, BW);
        let k_plenty = kinetic_stability(c, &plenty, &net, 50.0, BW);
        assert!(k_plenty > k_scarce, "k_scarce={k_scarce}  k_plenty={k_plenty}");
    }

    #[test]
    fn k_stability_favors_frequency_alignment() {
        let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
        let c = &raf_closures(&net, &food_ab())[0];
        let mut s = [0.0; MAX_SPECIES];
        s[0] = 1.0; s[1] = 1.0; s[2] = 1.0; s[3] = 1.0;
        let k_aligned = kinetic_stability(c, &s, &net, 50.0, BW);
        let k_misaligned = kinetic_stability(c, &s, &net, 500.0, BW);
        assert!(k_aligned > k_misaligned);
    }

    #[test]
    fn k_stability_zero_on_empty_species() {
        let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
        let c = &raf_closures(&net, &food_ab())[0];
        let s = [0.0; MAX_SPECIES];
        let k = kinetic_stability(c, &s, &net, 50.0, BW);
        assert_eq!(k, 0.0);
    }

    #[test]
    fn k_stability_finite_for_trace_concentrations() {
        let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
        let c = &raf_closures(&net, &food_ab())[0];
        let mut s = [0.0; MAX_SPECIES];
        s[0] = 1e-12; s[1] = 1e-12;
        let k = kinetic_stability(c, &s, &net, 50.0, BW);
        assert!(k.is_finite());
    }

    // ── End-to-end substrate: react + diffuse multi-tick ───────────────────
    // Compone kinetics + difusión + detección + métrica sobre un grid real.
    // Estos tests son el "smoke test" de AP-0/1/2 — si pasan, la onda A cierra.

    mod substrate {
        use super::*;
        use crate::blueprint::constants::chemistry::SPECIES_DIFFUSION_RATE;
        use crate::blueprint::equations::reaction_kinetics::{
            diffuse_species, step_grid_reactions,
        };
        use crate::layers::species_grid::{SpeciesCell, SpeciesGrid};

        const GRID_W: usize = 6;
        const GRID_H: usize = 6;
        const CELL_FREQ: f32 = 50.0;
        const DT: f32 = 0.01;
        const WARMUP_TICKS: usize = 5;
        const LONG_TICKS: usize = 100;
        /// Concentración de food baja → rate × dt « availability → el sistema
        /// permanece en régimen quasi-estático durante el warmup.
        const FOOD_SEED: f32 = 1.0;
        const TRACE_SEED: f32 = 0.01;

        fn seeded_grid() -> SpeciesGrid {
            let a = SpeciesId::new(0).unwrap();
            let b = SpeciesId::new(1).unwrap();
            let c = SpeciesId::new(2).unwrap();
            let d = SpeciesId::new(3).unwrap();
            let mut g = SpeciesGrid::new(GRID_W, GRID_H, CELL_FREQ);
            for y in 0..GRID_H { for x in 0..GRID_W {
                g.seed(x, y, a, FOOD_SEED);
                g.seed(x, y, b, FOOD_SEED);
                g.seed(x, y, c, TRACE_SEED);
                g.seed(x, y, d, TRACE_SEED);
            }}
            g
        }

        fn run_ticks(g: &mut SpeciesGrid, net: &ReactionNetwork, ticks: usize) -> f32 {
            let mut scratch: Vec<SpeciesCell> = Vec::with_capacity(g.len());
            let mut total = 0.0_f32;
            for _ in 0..ticks {
                total += step_grid_reactions(g, net, BW, DT);
                diffuse_species(g, &mut scratch, SPECIES_DIFFUSION_RATE, DT);
            }
            total
        }

        #[test]
        fn conservation_global_react_plus_diffuse() {
            let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
            let mut g = seeded_grid();
            let pre = g.total_qe();
            let diss = run_ticks(&mut g, &net, LONG_TICKS);
            let post = g.total_qe();
            let balance = (pre - post - diss).abs();
            assert!(
                balance < 1e-2,
                "conservation: pre={pre:.4} post={post:.4} diss={diss:.4} Δ={balance:.6}",
            );
        }

        #[test]
        fn dissipation_strictly_positive_under_reaction() {
            let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
            let mut g = seeded_grid();
            let diss = run_ticks(&mut g, &net, LONG_TICKS);
            assert!(diss > 0.0, "Axiom 4 violated: diss = {diss}");
        }

        #[test]
        fn food_set_detected_from_grid_state() {
            let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
            let mut g = seeded_grid();
            let _ = run_ticks(&mut g, &net, WARMUP_TICKS);
            let totals = g.totals_per_species();
            let food = food_set_from_totals(&totals);
            assert!(food.contains(&SpeciesId(0)));
            assert!(food.contains(&SpeciesId(1)));
        }

        #[test]
        fn kinetic_stability_persistent_at_steady_state() {
            let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
            let mut g = seeded_grid();
            let _ = run_ticks(&mut g, &net, WARMUP_TICKS);
            let totals = g.totals_per_species();
            let food = food_set_from_totals(&totals);
            let closures = raf_closures(&net, &food);
            assert_eq!(closures.len(), 1);
            let c = &closures[0];
            let cell = g.cell(GRID_W / 2, GRID_H / 2);
            let k = kinetic_stability(c, &cell.species, &net, cell.freq, BW);
            assert!(k.is_finite(), "k must be finite: {k}");
            assert!(
                k >= 1.0,
                "closure must be kinetically persistent at steady state, got k = {k:.4}",
            );
        }

        #[test]
        fn closure_hash_deterministic_across_redetections() {
            let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
            let food = food_ab();
            let h1 = raf_closures(&net, &food)[0].hash;
            let h2 = raf_closures(&net, &food)[0].hash;
            assert_eq!(h1, h2);
        }
    }
}
