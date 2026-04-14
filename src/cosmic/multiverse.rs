//! Multiverse — registro de branches por zoom y observables emergentes.
//! Multiverse — per-zoom branch log and emergent observables.
//!
//! Cada `rebranch_observed` (Tab) genera un branch determinista del mismo
//! padre con `seed` distinta. `MultiverseLog` acumula snapshots para que
//! observables como `life_probability` emerjan del sistema, no de parámetros.
//!
//! CT-9 / ADR-036 §D7. Funciones estadísticas son puras: `BranchSnapshot`
//! y `MultiverseSummary` se computan del estado, no se hardcodean.

use bevy::prelude::Resource;

use super::scale_manager::ScaleInstance;
use super::ScaleLevel;

/// Límite superior del log — evita crecimiento sin techo en sesiones largas.
/// Upper bound on log size — prevents unbounded growth.
pub const MAX_LOG_BRANCHES: usize = 256;

/// Sentinela `parent_entity_id` para branches sin padre (raíz del universo S0).
/// Sentinel parent_id for root branches (S0 has no parent).
pub const ROOT_PARENT_ID: u32 = u32::MAX;

// ─── Snapshot ──────────────────────────────────────────────────────────────

/// Métricas observables de un branch en el momento del registro. Computable
/// en tiempo O(n_entities); seguro de invocar en cada Tab.
#[derive(Clone, Debug, PartialEq)]
pub struct BranchSnapshot {
    pub total_qe: f64,
    pub n_entities: usize,
    pub has_life: bool,
    pub max_q_folding: f64,
    pub species_count: usize,
}

impl BranchSnapshot {
    /// Extrae el snapshot de una `ScaleInstance`. `freq_bandwidth` viene de
    /// `COHERENCE_BANDWIDTH` — el mismo que usan los bridges para heredar freq.
    ///
    /// `has_life` = hay entidades vivas en S3/S4 (la vida requiere escala
    ///   ecológica o molecular; en escalas superiores el término no aplica).
    /// `max_q_folding` = fracción de qe preservada tras disipación molecular
    ///   (proxy barato de estabilidad del proteome, en [0,1]).
    /// `species_count` = bins discretos de frecuencia (proxy de diversidad).
    pub fn from_instance(inst: &ScaleInstance, freq_bandwidth: f64) -> Self {
        let alive = inst.world.entities.iter().filter(|e| e.alive);
        let (mut total_qe, mut n_entities, mut bins) = (0.0_f64, 0_usize, std::collections::BTreeSet::new());
        let bw = freq_bandwidth.max(1e-9);
        for e in alive {
            total_qe += e.qe;
            n_entities += 1;
            bins.insert((e.frequency_hz / bw).floor() as i64);
        }

        let has_life = matches!(inst.level, ScaleLevel::Ecological | ScaleLevel::Molecular)
            && n_entities > 0;

        let max_q_folding = if matches!(inst.level, ScaleLevel::Molecular) {
            let initial = inst.world.total_qe_initial.max(1e-9);
            (total_qe / initial).clamp(0.0, 1.0)
        } else {
            0.0
        };

        Self {
            total_qe,
            n_entities,
            has_life,
            max_q_folding,
            species_count: bins.len(),
        }
    }
}

// ─── Branch ────────────────────────────────────────────────────────────────

/// Un branch del multiverso: un zoom-in con seed específico sobre un padre
/// concreto, junto con sus observables al momento del registro.
#[derive(Clone, Debug)]
pub struct MultiverseBranch {
    pub parent_entity_id: u32,
    pub scale: ScaleLevel,
    pub seed: u64,
    /// Tick del mundo de la escala al momento del registro.
    pub timestamp: u64,
    pub snapshot: BranchSnapshot,
}

impl MultiverseBranch {
    /// Construye un branch a partir de la instancia que se está abandonando.
    /// `parent_entity_id` = `ROOT_PARENT_ID` si la escala es S0.
    pub fn from_instance(inst: &ScaleInstance, freq_bandwidth: f64) -> Self {
        Self {
            parent_entity_id: inst.parent_entity_id.unwrap_or(ROOT_PARENT_ID),
            scale: inst.level,
            seed: inst.zoom_seed,
            timestamp: inst.world.tick_id,
            snapshot: BranchSnapshot::from_instance(inst, freq_bandwidth),
        }
    }
}

// ─── Log ───────────────────────────────────────────────────────────────────

/// Registro de todos los branches visitados. FIFO acotado en `MAX_LOG_BRANCHES`.
#[derive(Resource, Default, Debug)]
pub struct MultiverseLog {
    pub branches: Vec<MultiverseBranch>,
}

impl MultiverseLog {
    pub fn record(&mut self, branch: MultiverseBranch) {
        self.branches.push(branch);
        let overflow = self.branches.len().saturating_sub(MAX_LOG_BRANCHES);
        if overflow > 0 { self.branches.drain(..overflow); }
    }

    pub fn len(&self) -> usize { self.branches.len() }
    pub fn is_empty(&self) -> bool { self.branches.is_empty() }

    /// Itera branches que comparten `(parent_entity_id, scale)`.
    pub fn branches_for(
        &self,
        parent_entity_id: u32,
        scale: ScaleLevel,
    ) -> impl Iterator<Item = &MultiverseBranch> {
        self.branches
            .iter()
            .filter(move |b| b.parent_entity_id == parent_entity_id && b.scale == scale)
    }

    /// Branch más reciente para el mismo `(padre, escala)`. `None` si nunca se registró uno.
    pub fn most_recent_for(
        &self,
        parent_entity_id: u32,
        scale: ScaleLevel,
    ) -> Option<&MultiverseBranch> {
        self.branches
            .iter()
            .rev()
            .find(|b| b.parent_entity_id == parent_entity_id && b.scale == scale)
    }

    /// Fracción de branches de `(padre, escala)` con `has_life = true`. 0 si vacío.
    /// Probabilidad emergente — no hardcoded.
    pub fn life_probability(&self, parent_entity_id: u32, scale: ScaleLevel) -> f64 {
        let (mut total, mut alive) = (0_usize, 0_usize);
        for b in self.branches_for(parent_entity_id, scale) {
            total += 1;
            if b.snapshot.has_life { alive += 1; }
        }
        if total == 0 { 0.0 } else { alive as f64 / total as f64 }
    }

    /// Resumen estadístico global del log.
    pub fn summary(&self) -> MultiverseSummary {
        let n = self.branches.len();
        if n == 0 { return MultiverseSummary::default(); }
        let inv_n = 1.0 / n as f64;
        let mean_qe = self.branches.iter().map(|b| b.snapshot.total_qe).sum::<f64>() * inv_n;
        let mean_species = self
            .branches
            .iter()
            .map(|b| b.snapshot.species_count as f64)
            .sum::<f64>()
            * inv_n;
        let mean_q_folding = self
            .branches
            .iter()
            .map(|b| b.snapshot.max_q_folding)
            .sum::<f64>()
            * inv_n;
        let alive = self.branches.iter().filter(|b| b.snapshot.has_life).count();
        MultiverseSummary {
            n_branches: n,
            life_ratio: alive as f64 / n as f64,
            mean_qe,
            mean_species_count: mean_species,
            mean_q_folding,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MultiverseSummary {
    pub n_branches: usize,
    pub life_ratio: f64,
    pub mean_qe: f64,
    pub mean_species_count: f64,
    pub mean_q_folding: f64,
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cosmic::observer::{
        rebranch_observed, seed_universe, zoom_via_bridge, BigBangParams,
    };
    use crate::cosmic::scale_manager::ScaleManager;
    use crate::cosmic::{largest_entity_in, ScaleInstance};
    use crate::blueprint::equations::derived_thresholds::COHERENCE_BANDWIDTH;

    fn bw() -> f64 { COHERENCE_BANDWIDTH as f64 }

    fn sample_branch(parent: u32, scale: ScaleLevel, seed: u64, has_life: bool) -> MultiverseBranch {
        MultiverseBranch {
            parent_entity_id: parent,
            scale,
            seed,
            timestamp: 0,
            snapshot: BranchSnapshot {
                total_qe: 100.0,
                n_entities: 5,
                has_life,
                max_q_folding: 0.3,
                species_count: 2,
            },
        }
    }

    // ─── log semantics ────────────────────────────────────────────────────

    #[test]
    fn log_accumulates_branches() {
        let mut log = MultiverseLog::default();
        log.record(sample_branch(1, ScaleLevel::Stellar, 10, true));
        log.record(sample_branch(1, ScaleLevel::Stellar, 11, false));
        log.record(sample_branch(2, ScaleLevel::Planetary, 20, true));
        assert_eq!(log.len(), 3);
    }

    #[test]
    fn log_respects_upper_bound() {
        let mut log = MultiverseLog::default();
        for i in 0..(MAX_LOG_BRANCHES + 10) {
            log.record(sample_branch(0, ScaleLevel::Stellar, i as u64, false));
        }
        assert_eq!(log.len(), MAX_LOG_BRANCHES);
        // FIFO: los primeros 10 fueron desalojados.
        assert_eq!(log.branches[0].seed, 10);
    }

    #[test]
    fn life_probability_between_0_and_1() {
        let mut log = MultiverseLog::default();
        for (i, life) in [(0, false), (1, true), (2, true), (3, false), (4, true)] {
            log.record(sample_branch(99, ScaleLevel::Ecological, i, life));
        }
        let p = log.life_probability(99, ScaleLevel::Ecological);
        assert!((0.0..=1.0).contains(&p));
        assert!((p - 0.6).abs() < 1e-9);
    }

    #[test]
    fn life_probability_empty_bucket_is_zero() {
        let log = MultiverseLog::default();
        assert_eq!(log.life_probability(42, ScaleLevel::Molecular), 0.0);
    }

    #[test]
    fn most_recent_picks_last_matching() {
        let mut log = MultiverseLog::default();
        log.record(sample_branch(7, ScaleLevel::Stellar, 1, false));
        log.record(sample_branch(7, ScaleLevel::Stellar, 2, true));
        log.record(sample_branch(8, ScaleLevel::Stellar, 3, false));
        let last = log.most_recent_for(7, ScaleLevel::Stellar).unwrap();
        assert_eq!(last.seed, 2);
    }

    #[test]
    fn summary_aggregates_means_and_ratios() {
        let mut log = MultiverseLog::default();
        log.record(sample_branch(0, ScaleLevel::Molecular, 1, true));
        log.record(sample_branch(0, ScaleLevel::Molecular, 2, false));
        let s = log.summary();
        assert_eq!(s.n_branches, 2);
        assert!((s.life_ratio - 0.5).abs() < 1e-9);
        assert!((s.mean_qe - 100.0).abs() < 1e-9);
    }

    // ─── snapshot semantics ───────────────────────────────────────────────

    #[test]
    fn snapshot_has_life_only_at_ecological_or_molecular() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(13));
        let s0 = BranchSnapshot::from_instance(mgr.get(ScaleLevel::Cosmological).unwrap(), bw());
        assert!(!s0.has_life);
    }

    #[test]
    fn snapshot_q_folding_bounded_and_zero_off_molecular() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(3));
        let s = BranchSnapshot::from_instance(mgr.get(ScaleLevel::Cosmological).unwrap(), bw());
        assert_eq!(s.max_q_folding, 0.0);
        assert!((0.0..=1.0).contains(&s.max_q_folding));
    }

    // ─── determinismo y divergencia con observer ─────────────────────────

    #[test]
    fn same_seed_same_branch() {
        let mut a = ScaleManager::default();
        let mut b = ScaleManager::default();
        seed_universe(&mut a, &BigBangParams::interactive(21));
        seed_universe(&mut b, &BigBangParams::interactive(21));
        let pa = largest_entity_in(&a, ScaleLevel::Cosmological).unwrap();
        let pb = largest_entity_in(&b, ScaleLevel::Cosmological).unwrap();
        zoom_via_bridge(&mut a, pa, ScaleLevel::Cosmological).unwrap();
        zoom_via_bridge(&mut b, pb, ScaleLevel::Cosmological).unwrap();

        let sa = BranchSnapshot::from_instance(a.get(ScaleLevel::Stellar).unwrap(), bw());
        let sb = BranchSnapshot::from_instance(b.get(ScaleLevel::Stellar).unwrap(), bw());
        assert_eq!(sa, sb);
    }

    #[test]
    fn different_seeds_different_branches() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(21));
        let pid = largest_entity_in(&mgr, ScaleLevel::Cosmological).unwrap();
        zoom_via_bridge(&mut mgr, pid, ScaleLevel::Cosmological).unwrap();
        let s_before = BranchSnapshot::from_instance(mgr.get(ScaleLevel::Stellar).unwrap(), bw());

        rebranch_observed(&mut mgr, 99_999).expect("rebranch");
        let s_after = BranchSnapshot::from_instance(mgr.get(ScaleLevel::Stellar).unwrap(), bw());
        assert_ne!(s_before, s_after, "distinct seeds must diverge");
    }

    #[test]
    fn record_from_instance_populates_seed_and_parent() {
        let mut mgr = ScaleManager::default();
        seed_universe(&mut mgr, &BigBangParams::interactive(7));
        let pid = largest_entity_in(&mgr, ScaleLevel::Cosmological).unwrap();
        zoom_via_bridge(&mut mgr, pid, ScaleLevel::Cosmological).unwrap();

        let inst: &ScaleInstance = mgr.get(ScaleLevel::Stellar).unwrap();
        let branch = MultiverseBranch::from_instance(inst, bw());
        assert_eq!(branch.scale, ScaleLevel::Stellar);
        assert_eq!(branch.parent_entity_id, pid);
        assert_eq!(branch.seed, inst.zoom_seed);
    }
}
