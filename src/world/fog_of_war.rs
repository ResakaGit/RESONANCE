//! Grid de niebla (FoW) MOBA: cache por equipo con refcount.
//!
//! Codificación por celda: `< 0` no explorado (`-1`), `0` explorado sin visión activa, `> 0` refcount.
//! La geometría se alinea con [`crate::worldgen::EnergyFieldGrid`] (misma `cell_size` y `origin`).
//!
//! **G12 MVP:** disco radial por `VisionProvider.max_radius`. La señal L0+L2 vive en
//! [`crate::blueprint::equations::perception_signal`] para filtros finos futuros, no en el stamp de celdas.

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::layers::Faction;
use crate::worldgen::{EnergyFieldGrid, MapConfig};

/// Equipos con rejilla de niebla independiente (solo facciones PvP del MOBA).
pub const NUM_FOG_TEAMS: usize = 2;

#[derive(Resource, Debug, Clone)]
pub struct FogOfWarGrid {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub origin: Vec2,
    /// Se incrementa cuando cambia el stamp (overlay / UI pueden saltar frames si no cambia).
    pub fog_stamp_generation: u32,
    /// `cells[team][y * width + x]`
    cells: Vec<Vec<i32>>,
}

impl FogOfWarGrid {
    /// Crea un grid alineado al campo de energía (misma resolución; configurable vía mapa).
    pub fn aligned_with_energy_field(g: &EnergyFieldGrid) -> Self {
        let n = (g.width * g.height) as usize;
        Self {
            width: g.width,
            height: g.height,
            cell_size: g.cell_size,
            origin: g.origin,
            fog_stamp_generation: 0,
            cells: vec![vec![-1i32; n]; NUM_FOG_TEAMS],
        }
    }

    #[cfg(test)]
    fn new_test(width: u32, height: u32, cell_size: f32, origin: Vec2) -> Self {
        let n = (width * height) as usize;
        Self {
            width,
            height,
            cell_size,
            origin,
            fog_stamp_generation: 0,
            cells: vec![vec![-1i32; n]; NUM_FOG_TEAMS],
        }
    }

    /// Llamar tras mutar celdas por proveedores (invalida cache visual).
    #[inline]
    pub fn bump_stamp_generation(&mut self) {
        self.fog_stamp_generation = self.fog_stamp_generation.wrapping_add(1);
    }

    #[inline]
    fn idx(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(y as usize * self.width as usize + x as usize)
    }

    #[inline]
    pub fn cell_state(&self, team: usize, x: u32, y: u32) -> Option<i32> {
        let i = self.idx(x, y)?;
        self.cells.get(team)?.get(i).copied()
    }

    /// Convierte posición en el plano de sim (XZ o XY según layout) a índice de celda.
    #[inline]
    pub fn world_to_cell(&self, plane: Vec2) -> Option<(u32, u32)> {
        if !plane.is_finite() {
            return None;
        }
        let rel = plane - self.origin;
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

    #[inline]
    pub fn is_visible(&self, team: usize, x: u32, y: u32) -> bool {
        self.cell_state(team, x, y).is_some_and(|v| v > 0)
    }

    #[inline]
    pub fn is_explored(&self, team: usize, x: u32, y: u32) -> bool {
        self.cell_state(team, x, y).is_some_and(|v| v >= 0)
    }

    /// Fila mayor por equipo (`y * width + x`), solo lectura para GPU/overlay.
    #[inline]
    pub fn team_cells_row_major(&self, team: usize) -> Option<&[i32]> {
        self.cells.get(team).map(|v| v.as_slice())
    }

    fn bump_cell(&mut self, team: usize, x: u32, y: u32, delta: i32) {
        let Some(i) = self.idx(x, y) else {
            return;
        };
        let Some(row) = self.cells.get_mut(team) else {
            return;
        };
        let Some(v) = row.get_mut(i) else {
            return;
        };
        if delta > 0 {
            if *v < 0 {
                *v = 1;
            } else {
                *v = (*v).saturating_add(delta);
            }
        } else if delta < 0 {
            if *v > 1 {
                *v -= 1;
            } else if *v == 1 {
                *v = 0;
            }
        }
    }

    /// Incrementa refcount en disco world-space centrado en `center_plane`.
    pub fn stamp_disk(&mut self, team: usize, center_plane: Vec2, radius: f32) {
        if team >= NUM_FOG_TEAMS || !radius.is_finite() || radius <= 0.0 {
            return;
        }
        if !center_plane.is_finite() {
            return;
        }
        let r = radius.max(0.0);
        let r2 = r * r;
        let r_cells = (r / self.cell_size).ceil() as i32 + 1;
        let Some((cx, cy)) = self.world_to_cell(center_plane) else {
            return;
        };
        for dy in -r_cells..=r_cells {
            for dx in -r_cells..=r_cells {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let nx = nx as u32;
                let ny = ny as u32;
                if nx >= self.width || ny >= self.height {
                    continue;
                }
                let wx = self.origin.x + (nx as f32 + 0.5) * self.cell_size;
                let wy = self.origin.y + (ny as f32 + 0.5) * self.cell_size;
                let cell_c = Vec2::new(wx, wy);
                if (cell_c - center_plane).length_squared() <= r2 {
                    self.bump_cell(team, nx, ny, 1);
                }
            }
        }
    }

    /// Decrementa refcount (simétrico a [`Self::stamp_disk`]).
    pub fn unstamp_disk(&mut self, team: usize, center_plane: Vec2, radius: f32) {
        if team >= NUM_FOG_TEAMS || !radius.is_finite() || radius <= 0.0 {
            return;
        }
        if !center_plane.is_finite() {
            return;
        }
        let r = radius.max(0.0);
        let r2 = r * r;
        let r_cells = (r / self.cell_size).ceil() as i32 + 1;
        let Some((cx, cy)) = self.world_to_cell(center_plane) else {
            return;
        };
        for dy in -r_cells..=r_cells {
            for dx in -r_cells..=r_cells {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let nx = nx as u32;
                let ny = ny as u32;
                if nx >= self.width || ny >= self.height {
                    continue;
                }
                let wx = self.origin.x + (nx as f32 + 0.5) * self.cell_size;
                let wy = self.origin.y + (ny as f32 + 0.5) * self.cell_size;
                let cell_c = Vec2::new(wx, wy);
                if (cell_c - center_plane).length_squared() <= r2 {
                    self.bump_cell(team, nx, ny, -1);
                }
            }
        }
    }
}

/// Índice 0/1 para Red/Blue; facciones no PvP no participan en el grid.
#[inline]
pub fn fog_team_index(faction: Faction) -> Option<u8> {
    match faction {
        Faction::Red => Some(0),
        Faction::Blue => Some(1),
        _ => None,
    }
}

#[inline]
pub fn faction_for_fog_team(team: u8) -> Option<Faction> {
    match team {
        0 => Some(Faction::Red),
        1 => Some(Faction::Blue),
        _ => None,
    }
}

/// Inserta [`FogOfWarGrid`] alineado al campo de energía tras cargar el mapa, o lo quita si el RON
/// desactiva FoW (`fog_of_war: false`) para no dejar el overlay oscuro encima del sandbox.
pub fn init_fog_of_war_from_energy_field_system(
    mut commands: Commands,
    field: Res<EnergyFieldGrid>,
    map: Res<MapConfig>,
) {
    if !map.fog_of_war {
        commands.remove_resource::<FogOfWarGrid>();
        return;
    }
    commands.insert_resource(FogOfWarGrid::aligned_with_energy_field(&field));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_cell_inside() {
        let g = FogOfWarGrid::new_test(4, 4, 2.0, Vec2::new(-4.0, -4.0));
        assert_eq!(g.world_to_cell(Vec2::new(-3.0, -3.0)), Some((0, 0)));
        assert_eq!(g.world_to_cell(Vec2::new(3.0, 3.0)), Some((3, 3)));
    }

    #[test]
    fn world_to_cell_outside_or_non_finite() {
        let g = FogOfWarGrid::new_test(4, 4, 2.0, Vec2::new(-4.0, -4.0));
        assert!(g.world_to_cell(Vec2::new(-5.0, 0.0)).is_none());
        assert!(g.world_to_cell(Vec2::new(0.0, -5.0)).is_none());
        assert!(g.world_to_cell(Vec2::new(20.0, 0.0)).is_none());
        assert!(g.world_to_cell(Vec2::NAN).is_none());
    }

    #[test]
    fn fog_team_index_only_pvp() {
        use crate::layers::Faction;
        assert_eq!(super::fog_team_index(Faction::Red), Some(0));
        assert_eq!(super::fog_team_index(Faction::Blue), Some(1));
        assert_eq!(super::fog_team_index(Faction::Neutral), None);
        assert_eq!(super::fog_team_index(Faction::Wild), None);
    }

    #[test]
    fn stamp_team_isolation() {
        let mut g = FogOfWarGrid::new_test(16, 16, 2.0, Vec2::new(-16.0, -16.0));
        let c = Vec2::new(0.0, 0.0);
        g.stamp_disk(1, c, 4.0);
        let (cx, cy) = g.world_to_cell(c).unwrap();
        assert!(g.is_visible(1, cx, cy));
        assert!(!g.is_visible(0, cx, cy));
        assert_eq!(g.cell_state(0, cx, cy).unwrap(), -1);
    }

    #[test]
    fn refcount_stamp_unstamp_returns_to_explored() {
        let mut g = FogOfWarGrid::new_test(16, 16, 2.0, Vec2::new(-16.0, -16.0));
        let c = Vec2::new(0.0, 0.0);
        g.stamp_disk(0, c, 3.0);
        let (cx, cy) = g.world_to_cell(c).unwrap();
        assert!(g.is_visible(0, cx, cy));
        assert_eq!(g.cell_state(0, cx, cy).unwrap(), 1);
        g.unstamp_disk(0, c, 3.0);
        assert!(!g.is_visible(0, cx, cy));
        assert!(g.is_explored(0, cx, cy));
        assert_eq!(g.cell_state(0, cx, cy).unwrap(), 0);
    }

    #[test]
    fn two_stamps_same_cell_refcount_two() {
        let mut g = FogOfWarGrid::new_test(16, 16, 2.0, Vec2::new(-16.0, -16.0));
        let c = Vec2::new(1.0, 1.0);
        g.stamp_disk(0, c, 4.0);
        g.stamp_disk(0, c, 4.0);
        let (cx, cy) = g.world_to_cell(c).unwrap();
        assert!(g.cell_state(0, cx, cy).unwrap() >= 2);
        g.unstamp_disk(0, c, 4.0);
        assert!(g.is_visible(0, cx, cy));
        g.unstamp_disk(0, c, 4.0);
        assert!(!g.is_visible(0, cx, cy));
    }
}
