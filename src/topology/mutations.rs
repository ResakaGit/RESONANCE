//! Mutaciones runtime del terreno (Sprint T10): cráter, uplift, erosión puntual y flatten.

use crate::math_types::Vec2;
use bevy::prelude::Event;

use crate::topology::TerrainField;
use crate::topology::constants::{ALTITUDE_MAX_DEFAULT, ALTITUDE_MIN_DEFAULT};
use crate::topology::generators::classifier::{ClassificationThresholds, classify_terrain};
use crate::topology::generators::drainage::{compute_flow_accumulation, compute_flow_direction};
use crate::topology::generators::slope::derive_slope_aspect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TerrainMutation {
    Crater {
        center: Vec2,
        radius: f32,
        depth: f32,
    },
    Uplift {
        center: Vec2,
        radius: f32,
        height: f32,
    },
    Erosion {
        cell: (u32, u32),
        amount: f32,
    },
    Flatten {
        center: Vec2,
        radius: f32,
    },
}

#[derive(Event, Clone, Copy, Debug, PartialEq)]
pub struct TerrainMutationEvent(pub TerrainMutation);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtyRegion {
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
}

impl DirtyRegion {
    #[inline]
    pub fn single(x: u32, y: u32) -> Self {
        Self {
            min_x: x,
            min_y: y,
            max_x: x,
            max_y: y,
        }
    }

    #[inline]
    pub fn all(width: u32, height: u32) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: width.saturating_sub(1),
            max_y: height.saturating_sub(1),
        }
    }

    #[inline]
    pub fn expanded(self, margin: u32, width: u32, height: u32) -> Self {
        Self {
            min_x: self.min_x.saturating_sub(margin),
            min_y: self.min_y.saturating_sub(margin),
            max_x: self
                .max_x
                .saturating_add(margin)
                .min(width.saturating_sub(1)),
            max_y: self
                .max_y
                .saturating_add(margin)
                .min(height.saturating_sub(1)),
        }
    }

    #[inline]
    pub fn union(self, other: Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }
}

#[inline]
fn smoothstep_falloff(distance: f32, radius: f32) -> f32 {
    if !distance.is_finite() || !radius.is_finite() || radius <= 0.0 {
        return 0.0;
    }
    let d = (distance / radius).clamp(0.0, 1.0);
    1.0 - d * d * (3.0 - 2.0 * d)
}

#[inline]
fn safe_scalar(v: f32) -> f32 {
    if v.is_finite() { v } else { 0.0 }
}

#[inline]
fn safe_altitude(v: f32) -> f32 {
    safe_scalar(v).clamp(ALTITUDE_MIN_DEFAULT, ALTITUDE_MAX_DEFAULT)
}

fn circle_bounds(field: &TerrainField, center: Vec2, radius: f32) -> Option<DirtyRegion> {
    if !center.is_finite() || !radius.is_finite() || radius <= 0.0 {
        return None;
    }
    let min_world = center - Vec2::splat(radius);
    let max_world = center + Vec2::splat(radius);
    let min_xf = ((min_world.x - field.origin.x) / field.cell_size).floor();
    let min_yf = ((min_world.y - field.origin.y) / field.cell_size).floor();
    let max_xf = ((max_world.x - field.origin.x) / field.cell_size).floor();
    let max_yf = ((max_world.y - field.origin.y) / field.cell_size).floor();
    let w = field.width as i32;
    let h = field.height as i32;
    if max_xf < 0.0 || max_yf < 0.0 || min_xf >= w as f32 || min_yf >= h as f32 {
        return None;
    }
    let min_x = (min_xf as i32).clamp(0, w - 1) as u32;
    let min_y = (min_yf as i32).clamp(0, h - 1) as u32;
    let max_x = (max_xf as i32).clamp(0, w - 1) as u32;
    let max_y = (max_yf as i32).clamp(0, h - 1) as u32;
    Some(DirtyRegion {
        min_x,
        min_y,
        max_x,
        max_y,
    })
}

pub fn apply_mutation(field: &mut TerrainField, mutation: &TerrainMutation) -> Option<DirtyRegion> {
    match *mutation {
        TerrainMutation::Crater {
            center,
            radius,
            depth,
        } => {
            let Some(region) = circle_bounds(field, center, radius) else {
                return None;
            };
            let depth = safe_scalar(depth).max(0.0);
            for y in region.min_y..=region.max_y {
                for x in region.min_x..=region.max_x {
                    let p = field.cell_to_world(x, y);
                    let dist = p.distance(center);
                    if dist > radius {
                        continue;
                    }
                    let i = field.cell_index(x, y);
                    let falloff = smoothstep_falloff(dist, radius);
                    let z = safe_scalar(field.altitude[i]) - depth * falloff;
                    field.altitude[i] = safe_altitude(z);
                }
            }
            Some(region)
        }
        TerrainMutation::Uplift {
            center,
            radius,
            height,
        } => {
            let Some(region) = circle_bounds(field, center, radius) else {
                return None;
            };
            let height = safe_scalar(height).max(0.0);
            for y in region.min_y..=region.max_y {
                for x in region.min_x..=region.max_x {
                    let p = field.cell_to_world(x, y);
                    let dist = p.distance(center);
                    if dist > radius {
                        continue;
                    }
                    let i = field.cell_index(x, y);
                    let falloff = smoothstep_falloff(dist, radius);
                    let z = safe_scalar(field.altitude[i]) + height * falloff;
                    field.altitude[i] = safe_altitude(z);
                }
            }
            Some(region)
        }
        TerrainMutation::Erosion { cell, amount } => {
            let (x, y) = cell;
            if !field.is_valid(x, y) {
                return None;
            }
            let i = field.cell_index(x, y);
            let z = safe_scalar(field.altitude[i]) - safe_scalar(amount).max(0.0);
            field.altitude[i] = safe_altitude(z);
            Some(DirtyRegion::single(x, y))
        }
        TerrainMutation::Flatten { center, radius } => {
            let Some(region) = circle_bounds(field, center, radius) else {
                return None;
            };
            let mut sum = 0.0_f32;
            let mut count = 0_u32;
            for y in region.min_y..=region.max_y {
                for x in region.min_x..=region.max_x {
                    let p = field.cell_to_world(x, y);
                    if p.distance(center) > radius {
                        continue;
                    }
                    let i = field.cell_index(x, y);
                    sum += safe_scalar(field.altitude[i]);
                    count += 1;
                }
            }
            if count == 0 {
                return None;
            }
            let avg = safe_altitude(sum / count as f32);
            for y in region.min_y..=region.max_y {
                for x in region.min_x..=region.max_x {
                    let p = field.cell_to_world(x, y);
                    if p.distance(center) > radius {
                        continue;
                    }
                    let i = field.cell_index(x, y);
                    field.altitude[i] = avg;
                }
            }
            Some(region)
        }
    }
}

pub fn rederive_region(
    field: &mut TerrainField,
    region: &DirtyRegion,
    thresholds: &ClassificationThresholds,
) {
    if field.width == 0 || field.height == 0 || field.altitude.is_empty() {
        return;
    }
    let _expanded = region.expanded(1, field.width, field.height);
    let (new_slope, new_aspect) =
        derive_slope_aspect(&field.altitude, field.width, field.height, field.cell_size);
    let new_drainage = compute_flow_direction(&field.altitude, field.width, field.height);
    let new_acc =
        compute_flow_accumulation(&field.altitude, &new_drainage, field.width, field.height);
    // Correctitud primero: drenaje/acc es global; escribir parcial deja estado inconsistente downstream.
    for y in 0..field.height {
        for x in 0..field.width {
            let i = field.cell_index(x, y);
            field.slope[i] = safe_scalar(new_slope[i]);
            field.aspect[i] = safe_scalar(new_aspect[i]);
            field.drainage[i] = if new_drainage[i].is_finite() {
                new_drainage[i]
            } else {
                Vec2::ZERO
            };
            field.drainage_accumulation[i] = safe_scalar(new_acc[i]).max(0.0);
            field.terrain_type[i] = classify_terrain(
                field.altitude[i],
                field.slope[i],
                field.drainage_accumulation[i],
                thresholds,
            );
        }
    }
    field.generation = field.generation.wrapping_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::TerrainType;

    fn mk_field() -> TerrainField {
        TerrainField::new(11, 11, 1.0, Vec2::ZERO, 77)
    }

    #[test]
    fn crater_center_lowers_near_depth() {
        let mut f = mk_field();
        let c = f.cell_to_world(5, 5);
        let i = f.cell_index(5, 5);
        f.altitude[i] = 100.0;
        let _ = apply_mutation(
            &mut f,
            &TerrainMutation::Crater {
                center: c,
                radius: 3.0,
                depth: 10.0,
            },
        );
        assert!(
            (f.altitude[i] - 90.0).abs() < 1.2,
            "center={}",
            f.altitude[i]
        );
    }

    #[test]
    fn crater_falloff_border_lowers_less_than_center() {
        let mut f = mk_field();
        let c = f.cell_to_world(5, 5);
        let i_center = f.cell_index(5, 5);
        let i_border = f.cell_index(8, 5);
        f.altitude[i_center] = 100.0;
        f.altitude[i_border] = 100.0;
        let _ = apply_mutation(
            &mut f,
            &TerrainMutation::Crater {
                center: c,
                radius: 3.0,
                depth: 10.0,
            },
        );
        let delta_center = 100.0 - f.altitude[i_center];
        let delta_border = 100.0 - f.altitude[i_border];
        assert!(delta_center > delta_border);
    }

    #[test]
    fn uplift_center_raises_near_height() {
        let mut f = mk_field();
        let c = f.cell_to_world(5, 5);
        let i = f.cell_index(5, 5);
        let _ = apply_mutation(
            &mut f,
            &TerrainMutation::Uplift {
                center: c,
                radius: 3.0,
                height: 10.0,
            },
        );
        assert!(
            (f.altitude[i] - 10.0).abs() < 1.2,
            "center={}",
            f.altitude[i]
        );
    }

    #[test]
    fn flatten_converges_region_to_average() {
        let mut f = mk_field();
        let c = f.cell_to_world(5, 5);
        let radius = 1.6;
        for y in 0..f.height {
            for x in 0..f.width {
                let i = f.cell_index(x, y);
                f.altitude[i] = (x as f32 * 2.0) + (y as f32 * 0.5);
            }
        }
        let before = f.altitude.clone();
        let mut sum = 0.0_f32;
        let mut count = 0.0_f32;
        for y in 0..f.height {
            for x in 0..f.width {
                if f.cell_to_world(x, y).distance(c) <= radius {
                    sum += before[f.cell_index(x, y)];
                    count += 1.0;
                }
            }
        }
        let expected_avg = sum / count;
        let _ = apply_mutation(&mut f, &TerrainMutation::Flatten { center: c, radius });
        for y in 0..f.height {
            for x in 0..f.width {
                let i = f.cell_index(x, y);
                let inside = f.cell_to_world(x, y).distance(c) <= radius;
                if inside {
                    assert!((f.altitude[i] - expected_avg).abs() < 1e-4);
                } else {
                    assert!((f.altitude[i] - before[i]).abs() < 1e-4);
                }
            }
        }
    }

    #[test]
    fn erosion_lowers_single_cell() {
        let mut f = mk_field();
        let i = f.cell_index(3, 4);
        f.altitude[i] = 12.0;
        let _ = apply_mutation(
            &mut f,
            &TerrainMutation::Erosion {
                cell: (3, 4),
                amount: 2.5,
            },
        );
        assert!((f.altitude[i] - 9.5).abs() < 1e-4);
    }

    #[test]
    fn dirty_region_covers_cells_plus_margin_on_rederive() {
        let mut f = mk_field();
        let c = f.cell_to_world(5, 5);
        let before = f.altitude.clone();
        let region = apply_mutation(
            &mut f,
            &TerrainMutation::Crater {
                center: c,
                radius: 2.0,
                depth: 3.0,
            },
        )
        .expect("region");
        let expanded = region.expanded(1, f.width, f.height);
        assert!(expanded.min_x <= region.min_x);
        assert!(expanded.min_y <= region.min_y);
        assert!(expanded.max_x >= region.max_x);
        assert!(expanded.max_y >= region.max_y);
        for y in 0..f.height {
            for x in 0..f.width {
                let i = f.cell_index(x, y);
                let inside_region = x >= region.min_x
                    && x <= region.max_x
                    && y >= region.min_y
                    && y <= region.max_y;
                if !inside_region {
                    assert_eq!(f.altitude[i], before[i]);
                }
            }
        }
    }

    #[test]
    fn rederive_region_recomputes_slope_and_terrain_type_and_generation() {
        let mut f = mk_field();
        let c = f.cell_to_world(5, 5);
        let region = apply_mutation(
            &mut f,
            &TerrainMutation::Crater {
                center: c,
                radius: 2.5,
                depth: 40.0,
            },
        )
        .expect("region");
        let before_gen = f.generation;
        let center_idx = f.cell_index(5, 5);
        f.terrain_type[center_idx] = TerrainType::Peak;
        rederive_region(&mut f, &region, &ClassificationThresholds::default());
        let i = f.cell_index(5, 5);
        assert!(f.slope[i].is_finite());
        assert!(f.slope[i] >= 0.0);
        assert_ne!(f.terrain_type[i], TerrainType::Peak);
        assert_eq!(f.generation, before_gen + 1);
    }

    #[test]
    fn union_of_overlapping_regions_is_bounding_box() {
        let a = DirtyRegion {
            min_x: 1,
            min_y: 1,
            max_x: 4,
            max_y: 4,
        };
        let b = DirtyRegion {
            min_x: 3,
            min_y: 2,
            max_x: 6,
            max_y: 8,
        };
        let u = a.union(b);
        assert_eq!(u.min_x, 1);
        assert_eq!(u.min_y, 1);
        assert_eq!(u.max_x, 6);
        assert_eq!(u.max_y, 8);
    }

    #[test]
    fn out_of_grid_mutation_is_noop() {
        let mut f = mk_field();
        let prev = f.altitude.clone();
        let r = apply_mutation(
            &mut f,
            &TerrainMutation::Crater {
                center: Vec2::new(-999.0, -999.0),
                radius: 4.0,
                depth: 10.0,
            },
        );
        assert!(r.is_none());
        assert_eq!(f.altitude, prev);
    }

    #[test]
    fn mutation_does_not_produce_nan_inf() {
        let mut f = mk_field();
        let c = f.cell_to_world(5, 5);
        let _ = apply_mutation(
            &mut f,
            &TerrainMutation::Uplift {
                center: c,
                radius: 4.0,
                height: 10.0,
            },
        );
        assert!(f.altitude.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn mutation_clamps_altitude_to_contract_range() {
        let mut f = mk_field();
        let c = f.cell_to_world(5, 5);
        let _ = apply_mutation(
            &mut f,
            &TerrainMutation::Uplift {
                center: c,
                radius: 2.5,
                height: 99_999.0,
            },
        );
        assert!(
            f.altitude
                .iter()
                .all(|&z| (ALTITUDE_MIN_DEFAULT..=ALTITUDE_MAX_DEFAULT).contains(&z))
        );
    }

    #[test]
    fn custom_thresholds_are_used_in_rederive() {
        let mut f = mk_field();
        let i = f.cell_index(5, 5);
        f.altitude[i] = 120.0;
        f.slope[i] = 10.0;
        f.drainage_accumulation[i] = 0.0;
        let strict = ClassificationThresholds {
            peak_altitude: 1000.0,
            ridge_altitude: 900.0,
            plateau_altitude: 800.0,
            cliff_slope: 80.0,
            slope_threshold: 40.0,
            river_accumulation: 500.0,
            basin_max_slope: 0.1,
            basin_max_altitude: 0.1,
            valley_max_altitude: 0.1,
        };
        let region = DirtyRegion::single(5, 5);
        rederive_region(&mut f, &region, &strict);
        assert_eq!(f.terrain_type[i], TerrainType::Plain);
    }

    #[test]
    fn batch_overlapping_regions_rederive_once_increments_generation_once() {
        let mut f = mk_field();
        let c1 = f.cell_to_world(4, 5);
        let c2 = f.cell_to_world(6, 5);
        let r1 = apply_mutation(
            &mut f,
            &TerrainMutation::Crater {
                center: c1,
                radius: 2.0,
                depth: 5.0,
            },
        )
        .expect("r1");
        let r2 = apply_mutation(
            &mut f,
            &TerrainMutation::Crater {
                center: c2,
                radius: 2.0,
                depth: 5.0,
            },
        )
        .expect("r2");
        let merged = r1.union(r2);
        let g0 = f.generation;
        rederive_region(&mut f, &merged, &ClassificationThresholds::default());
        assert_eq!(f.generation, g0.wrapping_add(1));
    }
}
