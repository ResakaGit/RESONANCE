//! ScaleManager — gestor de instancias por escala, con CosmicWorld f64/3D dedicado.

use bevy::prelude::*;
use crate::blueprint::equations::scale_temporal::ScaleTelescope;
use super::ScaleLevel;

// ─── CosmicEntity ─────────────────────────────────────────────────────────

/// Entidad flat para simulación cósmica (f64/3D, repr(C), Copy).
/// Max 4 campos por "cluster lógico" — respeta regla de diseño.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CosmicEntity {
    // L0 qe + L1 radius
    pub qe: f64,
    pub radius: f64,

    // L2 oscilación
    pub frequency_hz: f64,
    pub phase: f64,

    // L3 flow (3D)
    pub position: [f64; 3],
    pub velocity: [f64; 3],

    // L4 matter
    pub dissipation: f64,
    pub age_ticks: u64,

    // Meta
    pub entity_id: u32,
    pub alive: bool,
}

impl Default for CosmicEntity {
    fn default() -> Self {
        Self {
            qe: 0.0,
            radius: 1.0,
            frequency_hz: 0.0,
            phase: 0.0,
            position: [0.0; 3],
            velocity: [0.0; 3],
            dissipation: 0.0,
            age_ticks: 0,
            entity_id: 0,
            alive: false,
        }
    }
}

// ─── CosmicWorld ──────────────────────────────────────────────────────────

/// Mundo flat para una escala cósmica. Heap-allocated (Vec) porque el tamaño
/// varía entre escalas (pocos clusters vs muchos planetas).
#[derive(Clone, Debug)]
pub struct CosmicWorld {
    pub entities: Vec<CosmicEntity>,
    pub total_qe_initial: f64,
    pub tick_id: u64,
}

impl CosmicWorld {
    pub fn new(capacity: usize) -> Self {
        Self {
            entities: Vec::with_capacity(capacity),
            total_qe_initial: 0.0,
            tick_id: 0,
        }
    }

    /// qe total actual (Σ entidades vivas).
    pub fn total_qe(&self) -> f64 {
        self.entities.iter().filter(|e| e.alive).map(|e| e.qe).sum()
    }

    /// Cantidad de entidades vivas.
    pub fn n_alive(&self) -> usize {
        self.entities.iter().filter(|e| e.alive).count()
    }

    /// Push con alive=true y entity_id asignado secuencial.
    pub fn spawn(&mut self, mut e: CosmicEntity) -> u32 {
        e.alive = true;
        e.entity_id = self.entities.len() as u32;
        self.entities.push(e);
        e.entity_id
    }
}

// ─── ScaleInstance ────────────────────────────────────────────────────────

/// Instancia de una escala activa.
#[derive(Debug)]
pub struct ScaleInstance {
    pub level: ScaleLevel,
    pub world: CosmicWorld,
    /// ID de la entidad del nivel padre que se expandió para crear este.
    pub parent_entity_id: Option<u32>,
    pub zoom_seed: u64,
    pub frozen: bool,
    /// Scheduler temporal (K adaptativo). `None` cuando la instancia queda frozen
    /// y se libera la proyección — ahorra recursos (CT-7 §E1/criterio 4).
    /// Adaptive temporal scheduler; `None` frees projection when instance freezes.
    pub telescope: Option<ScaleTelescope>,
}

impl ScaleInstance {
    pub fn new(level: ScaleLevel, capacity: usize, seed: u64) -> Self {
        Self {
            level,
            world: CosmicWorld::new(capacity),
            parent_entity_id: None,
            zoom_seed: seed,
            frozen: false,
            telescope: Some(ScaleTelescope::for_depth(level.depth())),
        }
    }

    pub fn with_parent(mut self, parent_id: u32) -> Self {
        self.parent_entity_id = Some(parent_id);
        self
    }

    /// Marca la instancia como frozen y libera el telescope (ahorro de memoria).
    /// Freezes the instance and drops its telescope (memory saving).
    pub fn freeze(&mut self) {
        if !self.frozen { self.frozen = true; }
        if self.telescope.is_some() { self.telescope = None; }
    }

    /// Descongela y reinicializa el telescope.
    /// Unfreezes and reinstates the telescope.
    pub fn unfreeze(&mut self) {
        if self.frozen { self.frozen = false; }
        if self.telescope.is_none() {
            self.telescope = Some(ScaleTelescope::for_depth(self.level.depth()));
        }
    }
}

// ─── ScaleManager ─────────────────────────────────────────────────────────

/// Resource que gestiona todas las escalas activas y cuál es la observada.
#[derive(Resource, Debug)]
pub struct ScaleManager {
    pub observed: ScaleLevel,
    pub instances: Vec<ScaleInstance>,
    /// Seed raíz del universo, nunca cambia tras inicialización.
    pub universe_seed: u64,
}

impl Default for ScaleManager {
    fn default() -> Self {
        Self {
            observed: ScaleLevel::Ecological,
            instances: Vec::new(),
            universe_seed: 42,
        }
    }
}

impl ScaleManager {
    pub fn new(universe_seed: u64) -> Self {
        Self {
            observed: ScaleLevel::Ecological,
            instances: Vec::new(),
            universe_seed,
        }
    }

    /// Inserta una instancia. Si ya existe una con el mismo level, la reemplaza.
    pub fn insert(&mut self, instance: ScaleInstance) {
        if let Some(pos) = self.instances.iter().position(|i| i.level == instance.level) {
            self.instances[pos] = instance;
        } else {
            self.instances.push(instance);
        }
    }

    pub fn get(&self, level: ScaleLevel) -> Option<&ScaleInstance> {
        self.instances.iter().find(|i| i.level == level)
    }

    pub fn get_mut(&mut self, level: ScaleLevel) -> Option<&mut ScaleInstance> {
        self.instances.iter_mut().find(|i| i.level == level)
    }

    pub fn remove(&mut self, level: ScaleLevel) {
        self.instances.retain(|i| i.level != level);
    }

    pub fn has(&self, level: ScaleLevel) -> bool {
        self.get(level).is_some()
    }

    /// qe total sumando todas las instancias activas.
    pub fn total_qe_across_scales(&self) -> f64 {
        self.instances.iter().map(|i| i.world.total_qe()).sum()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosmic_world_tracks_total_qe() {
        let mut w = CosmicWorld::new(4);
        let mut e = CosmicEntity::default();
        e.qe = 100.0;
        w.spawn(e);
        e.qe = 50.0;
        w.spawn(e);
        assert!((w.total_qe() - 150.0).abs() < 1e-9);
        assert_eq!(w.n_alive(), 2);
    }

    #[test]
    fn cosmic_world_dead_entity_excluded() {
        let mut w = CosmicWorld::new(4);
        let mut e = CosmicEntity::default();
        e.qe = 100.0;
        w.spawn(e);
        w.entities[0].alive = false;
        assert_eq!(w.total_qe(), 0.0);
        assert_eq!(w.n_alive(), 0);
    }

    #[test]
    fn scale_manager_insert_and_get() {
        let mut mgr = ScaleManager::new(123);
        let inst = ScaleInstance::new(ScaleLevel::Cosmological, 16, 42);
        mgr.insert(inst);
        assert!(mgr.has(ScaleLevel::Cosmological));
        assert!(!mgr.has(ScaleLevel::Stellar));
    }

    #[test]
    fn scale_manager_insert_replaces() {
        let mut mgr = ScaleManager::default();
        mgr.insert(ScaleInstance::new(ScaleLevel::Cosmological, 16, 1));
        mgr.insert(ScaleInstance::new(ScaleLevel::Cosmological, 32, 2));
        assert_eq!(mgr.instances.len(), 1);
        assert_eq!(mgr.get(ScaleLevel::Cosmological).unwrap().zoom_seed, 2);
    }

    #[test]
    fn scale_manager_remove() {
        let mut mgr = ScaleManager::default();
        mgr.insert(ScaleInstance::new(ScaleLevel::Stellar, 4, 0));
        mgr.remove(ScaleLevel::Stellar);
        assert!(!mgr.has(ScaleLevel::Stellar));
    }

    #[test]
    fn scale_manager_default_observed_ecological() {
        let mgr = ScaleManager::default();
        assert_eq!(mgr.observed, ScaleLevel::Ecological);
    }

    // ─── CT-7 telescope-per-scale ─────────────────────────────────────────

    #[test]
    fn telescope_attached_per_scale_independently() {
        let a = ScaleInstance::new(ScaleLevel::Cosmological, 4, 1);
        let b = ScaleInstance::new(ScaleLevel::Molecular, 4, 2);
        let ta = a.telescope.expect("cosmological telescope");
        let tb = b.telescope.expect("molecular telescope");
        // Bounds differ per scale.
        assert_ne!(ta.bounds, tb.bounds);
        assert!(ta.bounds.k_max > tb.bounds.k_max, "cosmological k_max must exceed molecular");
    }

    #[test]
    fn telescope_released_on_freeze() {
        let mut inst = ScaleInstance::new(ScaleLevel::Stellar, 4, 1);
        assert!(inst.telescope.is_some());
        inst.freeze();
        assert!(inst.telescope.is_none());
        assert!(inst.frozen);
    }

    #[test]
    fn telescope_restored_on_unfreeze() {
        let mut inst = ScaleInstance::new(ScaleLevel::Stellar, 4, 1);
        inst.freeze();
        inst.unfreeze();
        assert!(inst.telescope.is_some());
        assert!(!inst.frozen);
    }
}
