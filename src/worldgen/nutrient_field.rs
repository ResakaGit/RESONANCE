use crate::math_types::Vec2;
use bevy::prelude::{Reflect, Resource};

use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::worldgen::{EnergyFieldGrid, EnergyNucleus};

/// Máxima diferencia por canal para evitar escrituras ruidosas.
pub const NUTRIENT_WRITE_EPS: f32 = 1e-4;

/// EA7: presión por celda materializada (misma rejilla que nutrientes); qe por tick y por competidor extra.
pub const COMPETITION_BASE_DRAIN_PER_EXTRA_COMPETITOR_QE: f32 = 0.5;

#[derive(Clone, Copy, Debug, Reflect, PartialEq)]
pub struct NutrientCell {
    pub carbon_norm: f32,
    pub nitrogen_norm: f32,
    pub phosphorus_norm: f32,
    pub water_norm: f32,
}

impl Default for NutrientCell {
    fn default() -> Self {
        Self {
            carbon_norm: 0.0,
            nitrogen_norm: 0.0,
            phosphorus_norm: 0.0,
            water_norm: 0.0,
        }
    }
}

impl NutrientCell {
    pub fn new(
        carbon_norm: f32,
        nitrogen_norm: f32,
        phosphorus_norm: f32,
        water_norm: f32,
    ) -> Self {
        Self {
            carbon_norm: sanitize_norm(carbon_norm),
            nitrogen_norm: sanitize_norm(nitrogen_norm),
            phosphorus_norm: sanitize_norm(phosphorus_norm),
            water_norm: sanitize_norm(water_norm),
        }
    }

    pub fn add_scaled(&mut self, other: NutrientCell, scale: f32) {
        if !scale.is_finite() || scale <= 0.0 {
            return;
        }
        self.carbon_norm = sanitize_norm(self.carbon_norm + other.carbon_norm * scale);
        self.nitrogen_norm = sanitize_norm(self.nitrogen_norm + other.nitrogen_norm * scale);
        self.phosphorus_norm = sanitize_norm(self.phosphorus_norm + other.phosphorus_norm * scale);
        self.water_norm = sanitize_norm(self.water_norm + other.water_norm * scale);
    }

    pub fn regenerate(&mut self, delta: f32) {
        self.carbon_norm = sanitize_norm(self.carbon_norm + delta);
        self.nitrogen_norm = sanitize_norm(self.nitrogen_norm + delta);
        self.phosphorus_norm = sanitize_norm(self.phosphorus_norm + delta);
        self.water_norm = sanitize_norm(self.water_norm + delta);
    }
}

#[derive(Clone, Debug, Resource, Reflect)]
pub struct NutrientFieldGrid {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub origin: Vec2,
    cells: Vec<NutrientCell>,
}

impl NutrientFieldGrid {
    pub fn new(width: u32, height: u32, cell_size: f32, origin: Vec2) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let cell_size = if cell_size.is_finite() {
            cell_size.max(0.001)
        } else {
            1.0
        };
        let len = width as usize * height as usize;
        Self {
            width,
            height,
            cell_size,
            origin,
            cells: vec![NutrientCell::default(); len],
        }
    }

    pub fn align_with_energy_grid(energy: &EnergyFieldGrid) -> Self {
        Self::new(energy.width, energy.height, energy.cell_size, energy.origin)
    }

    pub fn sync_dimensions_with_energy_grid(&mut self, energy: &EnergyFieldGrid) {
        if self.width == energy.width
            && self.height == energy.height
            && (self.cell_size - energy.cell_size).abs() <= f32::EPSILON
            && self.origin == energy.origin
        {
            return;
        }
        *self = Self::align_with_energy_grid(energy);
    }

    pub fn cell_coords(&self, world_pos: Vec2) -> Option<(u32, u32)> {
        if !world_pos.is_finite() {
            return None;
        }
        let rel = world_pos - self.origin;
        if rel.x < 0.0 || rel.y < 0.0 {
            return None;
        }
        let x = (rel.x / self.cell_size).floor() as i32;
        let y = (rel.y / self.cell_size).floor() as i32;
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        Some((x as u32, y as u32))
    }

    pub fn cell_xy(&self, x: u32, y: u32) -> Option<&NutrientCell> {
        let idx = self.index_of(x, y)?;
        self.cells.get(idx)
    }

    pub fn cell_xy_mut(&mut self, x: u32, y: u32) -> Option<&mut NutrientCell> {
        let idx = self.index_of(x, y)?;
        self.cells.get_mut(idx)
    }

    /// Reset all cells to default (zero nutrients). Preserves grid dimensions.
    pub fn reset_cells(&mut self) {
        self.cells.iter_mut().for_each(|c| *c = NutrientCell::default());
    }

    /// Seeds all cells with uniform nutrient values.
    /// Used for Big Bang: initial nutrient substrate enables faster recycling.
    pub fn seed_uniform(&mut self, carbon: f32, nitrogen: f32, phosphorus: f32, water: f32) {
        for cell in &mut self.cells {
            cell.carbon_norm = carbon.clamp(0.0, 1.0);
            cell.nitrogen_norm = nitrogen.clamp(0.0, 1.0);
            cell.phosphorus_norm = phosphorus.clamp(0.0, 1.0);
            cell.water_norm = water.clamp(0.0, 1.0);
        }
    }

    pub fn iter_cells_mut(&mut self) -> impl Iterator<Item = &mut NutrientCell> {
        self.cells.iter_mut()
    }

    fn index_of(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(y as usize * self.width as usize + x as usize)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NutrientBias {
    pub profile: NutrientCell,
    pub radius: f32,
}

pub fn nutrient_bias_from_frequency(frequency_hz: f32, radius: f32) -> NutrientBias {
    // Bias explícitos del sprint TL2.
    if (frequency_hz - 75.0).abs() <= 70.0 {
        return NutrientBias {
            profile: NutrientCell::new(1.0, 0.45, 0.95, 0.40),
            radius,
        };
    }
    if (frequency_hz - 250.0).abs() <= 90.0 {
        return NutrientBias {
            profile: NutrientCell::new(0.40, 0.55, 0.35, 1.0),
            radius,
        };
    }
    if (frequency_hz - 450.0).abs() <= 90.0 {
        return NutrientBias {
            profile: NutrientCell::new(0.30, 0.10, 0.30, 0.25),
            radius,
        };
    }
    NutrientBias {
        profile: NutrientCell::new(0.40, 0.40, 0.40, 0.40),
        radius,
    }
}

pub fn apply_nucleus_bias(grid: &mut NutrientFieldGrid, nucleus_pos: Vec2, bias: NutrientBias) {
    if !bias.radius.is_finite() || bias.radius <= 0.0 {
        return;
    }
    let radius_sq = bias.radius * bias.radius;
    for y in 0..grid.height {
        for x in 0..grid.width {
            let world_x = grid.origin.x + (x as f32 + 0.5) * grid.cell_size;
            let world_y = grid.origin.y + (y as f32 + 0.5) * grid.cell_size;
            let d2 = Vec2::new(world_x, world_y).distance_squared(nucleus_pos);
            if d2 > radius_sq {
                continue;
            }
            let falloff = 1.0 - (d2.sqrt() / bias.radius);
            if let Some(cell) = grid.cell_xy_mut(x, y) {
                cell.add_scaled(bias.profile, falloff.max(0.0));
            }
        }
    }
}

pub fn seed_nutrient_field_from_nuclei_system(
    mut nutrient: bevy::prelude::ResMut<NutrientFieldGrid>,
    nuclei: bevy::prelude::Query<(&EnergyNucleus, &bevy::prelude::Transform)>,
    layout: bevy::prelude::Res<crate::runtime_platform::compat_2d3d::SimWorldTransformParams>,
) {
    for cell in nutrient.iter_cells_mut() {
        *cell = NutrientCell::default();
    }
    for (nucleus, transform) in &nuclei {
        let pos = sim_plane_pos(transform.translation, layout.use_xz_ground);
        let bias =
            nutrient_bias_from_frequency(nucleus.frequency_hz(), nucleus.propagation_radius());
        apply_nucleus_bias(&mut nutrient, pos, bias);
    }
}

pub fn sync_nutrient_field_len_system(
    nutrient: Option<bevy::prelude::ResMut<NutrientFieldGrid>>,
    energy: bevy::prelude::Res<EnergyFieldGrid>,
) {
    let Some(mut nutrient) = nutrient else {
        return;
    };
    nutrient.sync_dimensions_with_energy_grid(&energy);
}

#[inline]
fn sanitize_norm(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NutrientCell, NutrientFieldGrid, apply_nucleus_bias, nutrient_bias_from_frequency,
        sync_nutrient_field_len_system,
    };
    use crate::worldgen::EnergyFieldGrid;
    use bevy::math::Vec2;
    use bevy::prelude::*;

    #[test]
    fn nutrient_grid_sync_aligns_with_energy_grid() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(EnergyFieldGrid::new(7, 9, 2.0, Vec2::new(-3.0, 1.0)));
        app.insert_resource(NutrientFieldGrid::new(1, 1, 1.0, Vec2::ZERO));
        app.add_systems(Update, sync_nutrient_field_len_system);
        app.update();
        let nutrient = app.world().resource::<NutrientFieldGrid>();
        assert_eq!(nutrient.width, 7);
        assert_eq!(nutrient.height, 9);
        assert!((nutrient.cell_size - 2.0).abs() < f32::EPSILON);
        assert_eq!(nutrient.origin, Vec2::new(-3.0, 1.0));
    }

    #[test]
    fn terra_bias_near_center_is_high_carbon_and_phosphorus() {
        let mut grid = NutrientFieldGrid::new(11, 11, 1.0, Vec2::ZERO);
        let bias = nutrient_bias_from_frequency(75.0, 5.0);
        apply_nucleus_bias(&mut grid, Vec2::new(5.5, 5.5), bias);
        let center = grid.cell_xy(5, 5).copied().unwrap_or_default();
        let edge = grid.cell_xy(0, 0).copied().unwrap_or_default();
        assert!(
            center.carbon_norm > 0.8,
            "C too low: {}",
            center.carbon_norm
        );
        assert!(
            center.phosphorus_norm > 0.75,
            "P too low: {}",
            center.phosphorus_norm
        );
        assert!(center.carbon_norm > center.nitrogen_norm);
        assert!(center.phosphorus_norm > center.water_norm);
        assert!(center.carbon_norm > edge.carbon_norm);
    }

    #[test]
    fn aqua_bias_near_center_is_high_water() {
        let mut grid = NutrientFieldGrid::new(11, 11, 1.0, Vec2::ZERO);
        let bias = nutrient_bias_from_frequency(250.0, 5.0);
        apply_nucleus_bias(&mut grid, Vec2::new(5.5, 5.5), bias);
        let center = grid.cell_xy(5, 5).copied().unwrap_or_default();
        let edge = grid.cell_xy(0, 0).copied().unwrap_or_default();
        assert!(
            center.water_norm > 0.8,
            "water too low: {}",
            center.water_norm
        );
        assert!(center.water_norm > center.carbon_norm);
        assert!(center.water_norm > center.nitrogen_norm);
        assert!(center.water_norm > center.phosphorus_norm);
        assert!(center.water_norm > edge.water_norm);
    }

    #[test]
    fn nutrient_cell_regenerate_clamps_to_one() {
        let mut cell = NutrientCell::new(0.95, 0.95, 0.95, 0.95);
        cell.regenerate(0.2);
        assert_eq!(cell.carbon_norm, 1.0);
        assert_eq!(cell.nitrogen_norm, 1.0);
        assert_eq!(cell.phosphorus_norm, 1.0);
        assert_eq!(cell.water_norm, 1.0);
    }
}
