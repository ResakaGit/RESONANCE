//! `EcoBoundaryField`: cache alineado al `EnergyFieldGrid` (marcadores + contextos por `zone_id`).

use std::collections::HashMap;

use bevy::math::Vec2;
use bevy::prelude::Resource;

use crate::eco::boundary_detector::{NEIGHBOR_OFFSETS, detect_boundary};
use crate::eco::constants::BOUNDARY_RECOMPUTE_COOLDOWN;
use crate::eco::contracts::{BoundaryMarker, ZoneClass, ZoneContext};
use crate::eco::zone_classifier::classify_cell;
use crate::layers::MatterState;
use crate::worldgen::propagation::cell_density;
use crate::worldgen::{EnergyCell, EnergyFieldGrid};

/// Union-find 8-vecinos para agrupar celdas con la misma `ZoneClass` (alineado a `detect_boundary`).
struct ZoneUnionFind {
    parent: Vec<usize>,
}

impl ZoneUnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }

    fn find(&mut self, i: usize) -> usize {
        if self.parent[i] != i {
            self.parent[i] = self.find(self.parent[i]);
        }
        self.parent[i]
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parent[rb] = ra;
        }
    }
}

fn neighbor_cells8(grid: &EnergyFieldGrid, x: u32, y: u32) -> [EnergyCell; 8] {
    let center = grid.cell_xy(x, y).cloned().unwrap_or_default();
    let cx = x as i32;
    let cy = y as i32;
    let mut out = [
        center.clone(),
        center.clone(),
        center.clone(),
        center.clone(),
        center.clone(),
        center.clone(),
        center.clone(),
        center.clone(),
    ];
    for (k, &(dx, dy)) in NEIGHBOR_OFFSETS.iter().enumerate() {
        let nx = cx + dx;
        let ny = cy + dy;
        if nx >= 0
            && ny >= 0
            && nx < grid.width as i32
            && ny < grid.height as i32
            && let Some(c) = grid.cell_xy(nx as u32, ny as u32)
        {
            out[k] = c.clone();
        }
    }
    out
}

fn assign_compact_zone_ids(
    zones: &[ZoneClass],
    width: u32,
    height: u32,
    uf: &mut ZoneUnionFind,
) -> Vec<u16> {
    let w = width as usize;
    let h = height as usize;
    const D8: [(i32, i32); 8] = [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let z = zones[i];
            for &(dx, dy) in &D8 {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
                    let j = ny as usize * w + nx as usize;
                    if zones[j] == z {
                        uf.union(i, j);
                    }
                }
            }
        }
    }

    let mut map: HashMap<usize, u16> = HashMap::new();
    let mut next: u16 = 0;
    let len = w * h;
    let mut ids = vec![0u16; len];
    for i in 0..len {
        let r = uf.find(i);
        let id = *map.entry(r).or_insert_with(|| {
            let v = next;
            next = next.saturating_add(1);
            v
        });
        ids[i] = id;
    }
    ids
}

/// Promedia contexto por `zone_id` usando solo celdas marcadas como `Interior`.
pub fn aggregate_zone_contexts(
    markers: &[BoundaryMarker],
    grid: &EnergyFieldGrid,
) -> HashMap<u16, ZoneContext> {
    let cell_size = grid.cell_size;
    let w = grid.width as usize;
    #[derive(Default)]
    struct Acc {
        count: u32,
        sum_temp: f32,
        sum_density: f32,
        solid: u32,
        liquid: u32,
        gas: u32,
        plasma: u32,
        any_void: bool,
    }
    let mut buckets: HashMap<u16, Acc> = HashMap::new();

    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y as usize * w + x as usize;
            let Some(m) = markers.get(idx) else {
                continue;
            };
            let BoundaryMarker::Interior { zone_id } = m else {
                continue;
            };
            let Some(cell) = grid.cell_linear(idx) else {
                continue;
            };
            let zclass = classify_cell(cell, cell_size);
            if zclass == ZoneClass::Void {
                buckets.entry(*zone_id).or_default().any_void = true;
            }
            let rho = cell_density(cell.accumulated_qe, cell_size);
            let acc = buckets.entry(*zone_id).or_default();
            acc.count += 1;
            acc.sum_temp += cell.temperature;
            acc.sum_density += rho;
            match cell.matter_state {
                MatterState::Solid => acc.solid += 1,
                MatterState::Liquid => acc.liquid += 1,
                MatterState::Gas => acc.gas += 1,
                MatterState::Plasma => acc.plasma += 1,
            }
        }
    }

    let mut out = HashMap::new();
    for (id, acc) in buckets {
        let n = acc.count.max(1) as f32;
        let ctx = if acc.any_void {
            // Blueprint §5.1 Void: reglas desactivadas, valores base nulos.
            ZoneContext {
                pressure: 0.0,
                viscosity: 0.0,
                temperature_base: 0.0,
                dissipation_mod: 0.0,
                reactivity_mod: 0.0,
            }
        } else {
            let mean_temp = acc.sum_temp / n;
            let mean_rho = acc.sum_density / n;
            let pressure = (1.0 + mean_rho * 0.1).max(0.0);
            let (liquid, gas, solid, plasma) = (acc.liquid, acc.gas, acc.solid, acc.plasma);
            let viscosity = if liquid >= gas && liquid >= solid && liquid >= plasma {
                crate::blueprint::constants::BIOME_SWAMP_VISCOSITY
            } else if gas >= solid && gas >= liquid && gas >= plasma {
                crate::blueprint::constants::BIOME_LEY_LINE_VISCOSITY
            } else if solid >= plasma {
                crate::blueprint::constants::BIOME_TUNDRA_VISCOSITY * 1.8
            } else {
                crate::blueprint::constants::BIOME_VOLCANO_VISCOSITY
            };
            let dissipation_mod = if mean_rho <= crate::eco::constants::THIN_ATMOSPHERE_DENSITY_MAX
            {
                1.15
            } else {
                1.0
            };
            ZoneContext {
                pressure,
                viscosity,
                temperature_base: mean_temp,
                dissipation_mod,
                reactivity_mod: 1.0,
            }
        };
        out.insert(id, ctx);
    }
    out
}

/// Promedia `ZoneContext` por `ZoneClass` usando solo celdas `Interior` (para lerp en fronteras).
pub fn aggregate_zone_class_contexts(
    markers: &[BoundaryMarker],
    grid: &EnergyFieldGrid,
    zone_contexts: &HashMap<u16, ZoneContext>,
    cell_size_m: f32,
) -> HashMap<ZoneClass, ZoneContext> {
    #[derive(Default)]
    struct Acc {
        n: u32,
        pressure: f32,
        viscosity: f32,
        temperature_base: f32,
        dissipation_mod: f32,
        reactivity_mod: f32,
    }
    let mut buckets: HashMap<ZoneClass, Acc> = HashMap::new();
    let w = grid.width as usize;

    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y as usize * w + x as usize;
            let Some(BoundaryMarker::Interior { zone_id }) = markers.get(idx) else {
                continue;
            };
            let Some(cell) = grid.cell_linear(idx) else {
                continue;
            };
            let Some(ctx) = zone_contexts.get(zone_id) else {
                continue;
            };
            let zc = classify_cell(cell, cell_size_m);
            let acc = buckets.entry(zc).or_default();
            acc.n += 1;
            acc.pressure += ctx.pressure;
            acc.viscosity += ctx.viscosity;
            acc.temperature_base += ctx.temperature_base;
            acc.dissipation_mod += ctx.dissipation_mod;
            acc.reactivity_mod += ctx.reactivity_mod;
        }
    }

    let mut out = HashMap::new();
    for (zc, acc) in buckets {
        let n = acc.n.max(1) as f32;
        out.insert(
            zc,
            ZoneContext {
                pressure: acc.pressure / n,
                viscosity: acc.viscosity / n,
                temperature_base: acc.temperature_base / n,
                dissipation_mod: acc.dissipation_mod / n,
                reactivity_mod: acc.reactivity_mod / n,
            },
        );
    }
    out
}

/// Campo derivado alineado dimensionalmente al grid V7.
#[derive(Resource, Debug, Clone)]
pub struct EcoBoundaryField {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub origin: Vec2,
    pub markers: Vec<BoundaryMarker>,
    /// `zone_id` por celda (mismo orden que `markers`); útil en fronteras para `ctx_a` O(1).
    pub cell_zone_ids: Vec<u16>,
    /// Promedio de contexto por clase zonal (vecinos en `Boundary`); clave para `ctx_b` sin `zone_id` ajeno.
    pub zone_class_context: HashMap<ZoneClass, ZoneContext>,
    pub zone_contexts: HashMap<u16, ZoneContext>,
    pub last_seen_grid_generation: u32,
    pub last_recompute_sim_tick: u64,
}

impl Default for EcoBoundaryField {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            cell_size: 1.0,
            origin: Vec2::ZERO,
            markers: Vec::new(),
            cell_zone_ids: Vec::new(),
            zone_class_context: HashMap::new(),
            zone_contexts: HashMap::new(),
            last_seen_grid_generation: 0,
            last_recompute_sim_tick: 0,
        }
    }
}

impl EcoBoundaryField {
    /// Sincroniza metadatos con el grid (sin reclasificar).
    pub fn align_to_grid(&mut self, grid: &EnergyFieldGrid) {
        self.width = grid.width;
        self.height = grid.height;
        self.cell_size = grid.cell_size;
        self.origin = grid.origin;
    }

    /// Recomputa marcadores y contextos. Respeta cooldown salvo layout incompleto.
    ///
    /// **Staleness:** si `grid.generation` sube cada tick y el cooldown bloquea, el campo puede
    /// quedar 1–N ticks desfasado respecto al grid (trade-off blueprint §11 / sprint E2).
    ///
    /// Retorna `true` si hubo recomputo real.
    pub fn recompute_if_needed(&mut self, grid: &EnergyFieldGrid, sim_tick: u64) -> bool {
        let len = grid.width as usize * grid.height as usize;
        let need_layout =
            self.markers.len() != len || self.width != grid.width || self.height != grid.height;
        let gen_changed = grid.generation != self.last_seen_grid_generation;

        if !need_layout && !gen_changed {
            return false;
        }

        let cooldown_ok = need_layout
            || sim_tick.saturating_sub(self.last_recompute_sim_tick)
                >= BOUNDARY_RECOMPUTE_COOLDOWN as u64;
        if gen_changed && !cooldown_ok {
            return false;
        }

        self.align_to_grid(grid);
        let w = grid.width as usize;

        let mut zones: Vec<ZoneClass> = Vec::with_capacity(len);
        for y in 0..grid.height {
            for x in 0..grid.width {
                let cell = grid.cell_xy(x, y).cloned().unwrap_or_default();
                zones.push(classify_cell(&cell, grid.cell_size));
            }
        }

        let mut uf = ZoneUnionFind::new(len);
        let zone_ids = assign_compact_zone_ids(&zones, grid.width, grid.height, &mut uf);

        let mut markers = Vec::with_capacity(len);
        let mut cell_zone_ids = Vec::with_capacity(len);
        for y in 0..grid.height {
            for x in 0..grid.width {
                let idx = y as usize * w + x as usize;
                let cell = grid.cell_xy(x, y).cloned().unwrap_or_default();
                let z = zones[idx];
                let zid = zone_ids[idx];
                cell_zone_ids.push(zid);
                let neigh = neighbor_cells8(grid, x, y);
                markers.push(detect_boundary(&cell, &neigh, z, zid, grid.cell_size));
            }
        }

        let zone_contexts = aggregate_zone_contexts(&markers, grid);
        let zone_class_context =
            aggregate_zone_class_contexts(&markers, grid, &zone_contexts, grid.cell_size);
        self.markers = markers;
        self.cell_zone_ids = cell_zone_ids;
        self.zone_class_context = zone_class_context;
        self.zone_contexts = zone_contexts;
        self.last_seen_grid_generation = grid.generation;
        self.last_recompute_sim_tick = sim_tick;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eco::contracts::BoundaryMarker;

    #[test]
    fn aggregate_dos_zone_id_promedia_por_grupo() {
        let mut grid = EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO);
        for y in 0..2 {
            for x in 0..2 {
                let c = grid.cell_xy_mut(x, y).unwrap();
                c.accumulated_qe = 8.0;
                c.temperature = if x == 0 { 10.0 } else { 30.0 };
                c.matter_state = MatterState::Liquid;
                c.dominant_frequency_hz = 250.0;
            }
        }
        // Índice fila mayor: (0,0)=0, (1,0)=1, (0,1)=2, (1,1)=3
        let markers = vec![
            BoundaryMarker::Interior { zone_id: 0 },
            BoundaryMarker::Interior { zone_id: 1 },
            BoundaryMarker::Interior { zone_id: 0 },
            BoundaryMarker::Interior { zone_id: 1 },
        ];
        let map = aggregate_zone_contexts(&markers, &grid);
        let c0 = map.get(&0).expect("zona 0");
        let c1 = map.get(&1).expect("zona 1");
        assert!((c0.temperature_base - 10.0).abs() < 0.01);
        assert!((c1.temperature_base - 30.0).abs() < 0.01);
    }

    #[test]
    fn recompute_skipea_si_generacion_igual() {
        let mut grid = EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO);
        for y in 0..2 {
            for x in 0..2 {
                let c = grid.cell_xy_mut(x, y).unwrap();
                c.accumulated_qe = 4.0;
                c.temperature = 1.5;
                c.matter_state = MatterState::Liquid;
                c.dominant_frequency_hz = 250.0;
            }
        }
        grid.generation = 7;
        let mut field = EcoBoundaryField::default();
        assert!(field.recompute_if_needed(&grid, 10));
        let snap = field.markers.clone();
        assert!(!field.recompute_if_needed(&grid, 11));
        assert_eq!(field.markers, snap);
    }

    #[test]
    fn recompute_corre_si_generacion_cambia_y_cooldown() {
        let mut grid = EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO);
        for y in 0..2 {
            for x in 0..2 {
                let c = grid.cell_xy_mut(x, y).unwrap();
                c.accumulated_qe = 4.0;
                c.temperature = 1.5;
                c.matter_state = MatterState::Liquid;
                c.dominant_frequency_hz = 250.0;
            }
        }
        grid.generation = 1;
        let mut field = EcoBoundaryField::default();
        assert!(field.recompute_if_needed(&grid, 0));
        grid.generation = 2;
        assert!(!field.recompute_if_needed(&grid, 0), "cooldown same tick");
        assert!(
            field.recompute_if_needed(&grid, 2),
            "tras >= COOLDOWN ticks"
        );
    }

    #[test]
    fn borde_grid_es_interior() {
        let mut grid = EnergyFieldGrid::new(3, 3, 1.0, Vec2::ZERO);
        for y in 0..3 {
            for x in 0..3 {
                let c = grid.cell_xy_mut(x, y).unwrap();
                c.accumulated_qe = 4.0;
                c.temperature = 1.5;
                c.matter_state = MatterState::Liquid;
                c.dominant_frequency_hz = 250.0;
            }
        }
        let mut field = EcoBoundaryField::default();
        field.recompute_if_needed(&grid, 0);
        assert!(
            matches!(field.markers[0], BoundaryMarker::Interior { .. }),
            "corner (0,0) must be interior: out-of-bounds neighbors duplicate as same zone"
        );
    }
}
