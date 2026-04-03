//! Data bridge entre simulación y cualquier UI. Read-only para paneles.
//! Data bridge between simulation and any UI. Read-only for panels.
//!
//! Contrato: la simulación ESCRIBE estos Resources al final de cada tick.
//! El UI (egui, terminal, web, o cualquier otro) solo los LEE.
//! Zero acoplamiento: la simulación no sabe que el UI existe.
//!
//! Fase: `Phase::MorphologicalLayer` (último en FixedUpdate, datos asentados).

use bevy::prelude::*;

use crate::layers::BaseEnergy;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::simulation::Phase;

// ─── RingBuffer (stack-allocated, cache-friendly) ───────────────────────────

/// Capacidad del ring buffer. Potencia de 2 para módulo bitwise.
const RING_CAP: usize = 512;

/// Ring buffer de tamaño fijo para time series. Stack-allocated, Copy-friendly.
/// Fixed-size ring buffer for time series. Stack-allocated, Copy-friendly.
///
/// Push O(1), iterate O(n). Sin heap. Sin alloc. Cache-friendly (contiguous memory).
#[derive(Debug, Clone)]
pub struct RingBuffer {
    data: [f32; RING_CAP],
    head: usize,
    len: usize,
}

impl Default for RingBuffer {
    fn default() -> Self {
        Self {
            data: [0.0; RING_CAP],
            head: 0,
            len: 0,
        }
    }
}

impl RingBuffer {
    /// Añade un valor al buffer. Si está lleno, sobrescribe el más viejo.
    /// Pushes a value. If full, overwrites the oldest.
    #[inline]
    pub fn push(&mut self, value: f32) {
        self.data[self.head] = value;
        self.head = (self.head + 1) & (RING_CAP - 1); // bitwise mod (potencia de 2)
        if self.len < RING_CAP {
            self.len += 1;
        }
    }

    /// Número de muestras almacenadas.
    /// Number of stored samples.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// ¿Está vacío?
    /// Is it empty?
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterador en orden cronológico (oldest → newest).
    /// Iterator in chronological order (oldest → newest).
    pub fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        let start = if self.len < RING_CAP { 0 } else { self.head };
        (0..self.len).map(move |i| self.data[(start + i) & (RING_CAP - 1)])
    }

    /// Último valor añadido. `None` si vacío.
    /// Last pushed value. `None` if empty.
    #[inline]
    pub fn last(&self) -> Option<f32> {
        if self.len == 0 {
            return None;
        }
        let idx = if self.head == 0 {
            RING_CAP - 1
        } else {
            self.head - 1
        };
        Some(self.data[idx])
    }

    /// Convierte a slice contiguo (para plot APIs que esperan &[f32]).
    /// Copies to a contiguous Vec for plot APIs that expect &[f32].
    pub fn to_vec(&self) -> Vec<f32> {
        self.iter().collect()
    }
}

// ─── Resources ──────────────────────────────────────────────────────────────

/// Aggregados instantáneos del tick actual. Lectura barata para status bars.
/// Instant aggregates of the current tick. Cheap read for status bars.
#[derive(Resource, Debug, Clone, Default)]
pub struct SimTickSummary {
    pub tick: u64,
    pub total_qe: f32,
    pub alive_count: u32,
    pub species_count: u8,
}

/// Historial temporal para gráficos. Ring buffers stack-allocated.
/// Time history for charts. Stack-allocated ring buffers.
#[derive(Resource, Debug, Clone, Default)]
pub struct SimTimeSeries {
    pub qe_history: RingBuffer,
    pub pop_history: RingBuffer,
    pub species_history: RingBuffer,
}

/// Configuración de velocidad de simulación. Escrita por UI, leída por clock.
/// Simulation speed config. Written by UI, read by clock.
#[derive(Resource, Debug, Clone)]
pub struct SimSpeedConfig {
    pub time_scale: f32,
    pub paused: bool,
}

impl Default for SimSpeedConfig {
    fn default() -> Self {
        Self {
            time_scale: 1.0,
            paused: false,
        }
    }
}

/// Configuración visual. Escrita por UI, leída por render systems.
/// Visual config. Written by UI, read by render systems.
#[derive(Resource, Debug, Clone)]
pub struct ViewConfig {
    pub show_grid: bool,
    pub show_trajectories: bool,
    pub color_mode: ColorMode,
    pub camera_mode: CameraMode,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            show_grid: false,
            show_trajectories: false,
            color_mode: ColorMode::Frequency,
            camera_mode: CameraMode::Orbital,
        }
    }
}

/// Modo de coloreo de entidades.
/// Entity coloring mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    #[default]
    Frequency,
    Energy,
    Trophic,
    Age,
}

/// Modo de cámara.
/// Camera mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    #[default]
    Orbital,
    FollowPlayer,
    TopDown,
}

/// Entidad seleccionada para inspección. `None` = ninguna.
/// Selected entity for inspection. `None` = none.
#[derive(Resource, Debug, Clone, Default)]
pub struct SelectedEntity(pub Option<Entity>);

// ─── Update system (end of tick, read-only for UI) ──────────────────────────

/// Agrega datos de la simulación en Resources para el UI.
/// Aggregates simulation data into Resources for the UI.
///
/// Stateless: lee queries, escribe Resources. Zero side effects en la simulación.
pub fn update_dashboard_bridge(
    mut summary: ResMut<SimTickSummary>,
    mut series: ResMut<SimTimeSeries>,
    clock: Res<SimulationClock>,
    query: Query<&BaseEnergy>,
) {
    summary.tick = clock.tick_id;
    let (mut total_qe, mut alive) = (0.0_f32, 0_u32);
    for energy in &query {
        let qe = energy.qe();
        if qe > 0.0 {
            total_qe += qe;
            alive += 1;
        }
    }
    summary.total_qe = total_qe;
    summary.alive_count = alive;

    series.qe_history.push(total_qe);
    series.pop_history.push(alive as f32);
}

// ─── Plugin ─────────────────────────────────────────────────────────────────

/// Plugin que registra el data bridge. Opt-in: solo binarios que lo necesitan lo añaden.
/// Plugin that registers the data bridge. Opt-in: only binaries that need it add it.
pub struct DashboardBridgePlugin;

impl Plugin for DashboardBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimTickSummary>()
            .init_resource::<SimTimeSeries>()
            .init_resource::<SimSpeedConfig>()
            .init_resource::<ViewConfig>()
            .init_resource::<SelectedEntity>()
            .add_systems(
                FixedUpdate,
                update_dashboard_bridge.in_set(Phase::MorphologicalLayer),
            );
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── RingBuffer ──

    #[test]
    fn ring_empty_by_default() {
        let r = RingBuffer::default();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
        assert_eq!(r.last(), None);
    }

    #[test]
    fn ring_push_increments_len() {
        let mut r = RingBuffer::default();
        r.push(1.0);
        assert_eq!(r.len(), 1);
        r.push(2.0);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn ring_last_returns_newest() {
        let mut r = RingBuffer::default();
        r.push(1.0);
        r.push(2.0);
        r.push(3.0);
        assert_eq!(r.last(), Some(3.0));
    }

    #[test]
    fn ring_iter_chronological_order() {
        let mut r = RingBuffer::default();
        for i in 0..5 {
            r.push(i as f32);
        }
        let v: Vec<f32> = r.iter().collect();
        assert_eq!(v, vec![0.0, 1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn ring_wraps_at_capacity() {
        let mut r = RingBuffer::default();
        for i in 0..(RING_CAP + 10) {
            r.push(i as f32);
        }
        assert_eq!(r.len(), RING_CAP);
        // Oldest should be 10, newest should be RING_CAP + 9
        let first = r.iter().next().unwrap();
        assert_eq!(first, 10.0);
        assert_eq!(r.last(), Some((RING_CAP + 9) as f32));
    }

    #[test]
    fn ring_to_vec_matches_iter() {
        let mut r = RingBuffer::default();
        for i in 0..20 {
            r.push(i as f32);
        }
        let from_iter: Vec<f32> = r.iter().collect();
        assert_eq!(r.to_vec(), from_iter);
    }

    #[test]
    fn ring_push_after_wrap_maintains_chronological() {
        let mut r = RingBuffer::default();
        // Fill completely then push 3 more
        for i in 0..(RING_CAP + 3) {
            r.push(i as f32);
        }
        let v = r.to_vec();
        // Should be 3, 4, 5, ..., RING_CAP + 2
        for (idx, &val) in v.iter().enumerate() {
            assert_eq!(val, (idx + 3) as f32, "mismatch at index {idx}");
        }
    }

    // ── Resources ──

    #[test]
    fn sim_tick_summary_default_zeros() {
        let s = SimTickSummary::default();
        assert_eq!(s.tick, 0);
        assert_eq!(s.total_qe, 0.0);
        assert_eq!(s.alive_count, 0);
    }

    #[test]
    fn sim_speed_config_default_normal() {
        let c = SimSpeedConfig::default();
        assert_eq!(c.time_scale, 1.0);
        assert!(!c.paused);
    }

    #[test]
    fn view_config_default_sensible() {
        let v = ViewConfig::default();
        assert!(!v.show_grid);
        assert_eq!(v.color_mode, ColorMode::Frequency);
        assert_eq!(v.camera_mode, CameraMode::Orbital);
    }

    #[test]
    fn selected_entity_default_none() {
        let s = SelectedEntity::default();
        assert!(s.0.is_none());
    }
}
