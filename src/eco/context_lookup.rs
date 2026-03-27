//! `ContextLookup`: SystemParam stateless — posición → contexto (interior O(1), frontera lerp).
//! Ver `docs/sprints/ECO_BOUNDARIES/README.md` y `docs/design/ECO_BOUNDARIES.md` §6.

use bevy::ecs::system::SystemParam;
use crate::math_types::Vec2;
use bevy::prelude::{Res, Resource};

use crate::eco::boundary_field::EcoBoundaryField;
use crate::eco::climate::{ClimateState, SeasonProfile};
use crate::eco::contracts::{BoundaryMarker, ContextResponse, ZoneClass, ZoneContext};
use crate::eco::zone_classifier::classify_cell;
use crate::worldgen::EnergyFieldGrid;

/// Borde lógico: N celdas desde el perímetro del `EnergyFieldGrid` con contexto [`void_context_response`].
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct EcoPlayfieldMargin {
    pub cells: u32,
}

/// Respuesta de contexto para celdas fuera del grid, índice inválido o interior sin `zone_context` (sin panic).
/// Desalineación grid/campo: ver `context_at_inner` (degradación a baseline legado).
#[inline]
pub fn void_context_response() -> ContextResponse {
    ContextResponse {
        pressure: 0.0,
        viscosity: 0.0,
        temperature_base: 0.0,
        dissipation_mod: 0.0,
        reactivity_mod: 0.0,
        is_boundary: false,
        zone: ZoneClass::Void,
    }
}

/// Sin `EcoBoundaryField` la simulación debe comportarse como pre–E5 (sin multiplicadores de contexto).
#[inline]
pub fn context_response_legacy_baseline() -> ContextResponse {
    ContextResponse {
        pressure: 1.0,
        viscosity: 1.0,
        temperature_base: 0.0,
        dissipation_mod: 1.0,
        reactivity_mod: 1.0,
        is_boundary: false,
        zone: ZoneClass::Surface,
    }
}

#[inline]
pub fn cell_index_for_pos(grid: &EnergyFieldGrid, pos: Vec2) -> Option<usize> {
    let (x, y) = grid.cell_coords(pos)?;
    Some(y as usize * grid.width as usize + x as usize)
}

#[inline]
fn cell_xy_from_index(idx: usize, grid_width: u32) -> (u32, u32) {
    let x = (idx % grid_width as usize) as u32;
    let y = (idx / grid_width as usize) as u32;
    (x, y)
}

/// Celda en banda perimetral configurable → Void eco (no es frontera lerp entre zonas).
#[inline]
pub fn is_cell_in_logical_void_margin(
    cell_x: u32,
    cell_y: u32,
    grid_w: u32,
    grid_h: u32,
    margin_cells: u32,
) -> bool {
    if margin_cells == 0 {
        return false;
    }
    if grid_w <= 2 * margin_cells || grid_h <= 2 * margin_cells {
        return false;
    }
    cell_x < margin_cells
        || cell_y < margin_cells
        || cell_x >= grid_w - margin_cells
        || cell_y >= grid_h - margin_cells
}

/// Aplica offsets de estación sobre un `ZoneContext` (multiplicadores solo si ≠ 0 para respetar `SeasonProfile::ZERO`).
#[inline]
pub fn apply_season_to_zone_context(
    mut ctx: ZoneContext,
    zone: ZoneClass,
    profile: &SeasonProfile,
) -> ZoneContext {
    ctx.temperature_base += profile.temperature_offset;
    if profile.precipitation_factor != 0.0 {
        ctx.viscosity *= profile.precipitation_factor;
    }
    if zone == ZoneClass::HighAtmosphere && profile.wind_intensity != 0.0 {
        ctx.dissipation_mod *= profile.wind_intensity;
    }
    ctx
}

#[inline]
fn lerp_zone_context(a: ZoneContext, b: ZoneContext, t: f32) -> ZoneContext {
    let t = if t.is_finite() {
        t.clamp(0.0, 1.0)
    } else {
        0.5
    };
    let u = 1.0 - t;
    ZoneContext {
        pressure: a.pressure * u + b.pressure * t,
        viscosity: a.viscosity * u + b.viscosity * t,
        temperature_base: a.temperature_base * u + b.temperature_base * t,
        dissipation_mod: a.dissipation_mod * u + b.dissipation_mod * t,
        reactivity_mod: a.reactivity_mod * u + b.reactivity_mod * t,
    }
}

#[inline]
fn zone_context_or_default(field: &EcoBoundaryField, zc: ZoneClass) -> ZoneContext {
    field
        .zone_class_context
        .get(&zc)
        .copied()
        .unwrap_or_else(|| match zc {
            ZoneClass::Void => ZoneContext {
                pressure: 0.0,
                viscosity: 0.0,
                temperature_base: 0.0,
                dissipation_mod: 0.0,
                reactivity_mod: 0.0,
            },
            _ => ZoneContext::default(),
        })
}

#[inline]
fn to_response(ctx: ZoneContext, is_boundary: bool, zone: ZoneClass) -> ContextResponse {
    ContextResponse {
        pressure: ctx.pressure,
        viscosity: ctx.viscosity,
        temperature_base: ctx.temperature_base,
        dissipation_mod: ctx.dissipation_mod,
        reactivity_mod: ctx.reactivity_mod,
        is_boundary,
        zone,
    }
}

/// `EcoBoundaryField` coherente con el grid (mismas dimensiones y vectores alineados).
#[inline]
pub fn eco_field_aligned_with_grid(grid: &EnergyFieldGrid, field: &EcoBoundaryField) -> bool {
    let expected = grid.width as usize * grid.height as usize;
    expected > 0
        && field.markers.len() == expected
        && field.cell_zone_ids.len() == expected
        && field.width == grid.width
        && field.height == grid.height
}

/// Núcleo puro: sin Bevy World (testeable).
pub fn context_at_inner(
    grid: &EnergyFieldGrid,
    field: &EcoBoundaryField,
    climate: Option<&ClimateState>,
    pos: Vec2,
    playfield_margin_cells: u32,
) -> ContextResponse {
    if !eco_field_aligned_with_grid(grid, field) {
        // Degradación segura: un campo stale/bug no debe anular disipación/drag en todo el mundo.
        return context_response_legacy_baseline();
    }

    let Some(idx) = cell_index_for_pos(grid, pos) else {
        return void_context_response();
    };
    let (cx, cy) = cell_xy_from_index(idx, grid.width);
    if is_cell_in_logical_void_margin(cx, cy, grid.width, grid.height, playfield_margin_cells) {
        return void_context_response();
    }
    let Some(marker) = field.markers.get(idx) else {
        return void_context_response();
    };

    let z_cell = grid
        .cell_linear(idx)
        .map(|c| classify_cell(c, grid.cell_size))
        .unwrap_or(ZoneClass::Void);
    let profile: Option<SeasonProfile> = climate.map(|c| c.effective_offsets());

    match marker {
        BoundaryMarker::Interior { zone_id } => {
            let Some(base) = field.zone_contexts.get(zone_id).copied() else {
                return void_context_response();
            };
            let adjusted = profile
                .as_ref()
                .map(|p| apply_season_to_zone_context(base, z_cell, p))
                .unwrap_or(base);
            to_response(adjusted, false, z_cell)
        }
        BoundaryMarker::Boundary {
            zone_a,
            zone_b,
            gradient_factor,
            ..
        } => {
            // ctx_a: parche `center_zone_id` (O(1)). ctx_b: promedio por `ZoneClass` en celdas Interior
            // (no tenemos `zone_id` del vecino en el marcador — trade-off vs diagrama §6.1 literal).
            let center_zid = field.cell_zone_ids.get(idx).copied().unwrap_or(0);
            let ctx_a_base = field
                .zone_contexts
                .get(&center_zid)
                .copied()
                .unwrap_or_else(|| zone_context_or_default(field, *zone_a));
            let ctx_b_base = zone_context_or_default(field, *zone_b);
            let ctx_a = profile
                .as_ref()
                .map(|p| apply_season_to_zone_context(ctx_a_base, *zone_a, p))
                .unwrap_or(ctx_a_base);
            let ctx_b = profile
                .as_ref()
                .map(|p| apply_season_to_zone_context(ctx_b_base, *zone_b, p))
                .unwrap_or(ctx_b_base);
            let lerped = lerp_zone_context(ctx_a, ctx_b, *gradient_factor);
            to_response(lerped, true, *zone_a)
        }
    }
}

/// Zona lógica en la celda (centro = `zone_a` en frontera).
pub fn zone_at_inner(
    grid: &EnergyFieldGrid,
    field: &EcoBoundaryField,
    pos: Vec2,
    playfield_margin_cells: u32,
) -> ZoneClass {
    if !eco_field_aligned_with_grid(grid, field) {
        return ZoneClass::Surface;
    }
    let Some(idx) = cell_index_for_pos(grid, pos) else {
        return ZoneClass::Void;
    };
    let (cx, cy) = cell_xy_from_index(idx, grid.width);
    if is_cell_in_logical_void_margin(cx, cy, grid.width, grid.height, playfield_margin_cells) {
        return ZoneClass::Void;
    }
    let Some(marker) = field.markers.get(idx) else {
        return ZoneClass::Void;
    };
    match marker {
        BoundaryMarker::Boundary { zone_a, .. } => *zone_a,
        BoundaryMarker::Interior { zone_id } => {
            // Mismo contrato que `context_at_inner`: sin fila en `zone_contexts` → Void (campo incompleto).
            if !field.zone_contexts.contains_key(zone_id) {
                return ZoneClass::Void;
            }
            grid.cell_linear(idx)
                .map(|c| classify_cell(c, grid.cell_size))
                .unwrap_or(ZoneClass::Void)
        }
    }
}

pub fn is_boundary_at_inner(
    grid: &EnergyFieldGrid,
    field: &EcoBoundaryField,
    pos: Vec2,
    playfield_margin_cells: u32,
) -> bool {
    if !eco_field_aligned_with_grid(grid, field) {
        return false;
    }
    let Some(idx) = cell_index_for_pos(grid, pos) else {
        return false;
    };
    let (cx, cy) = cell_xy_from_index(idx, grid.width);
    if is_cell_in_logical_void_margin(cx, cy, grid.width, grid.height, playfield_margin_cells) {
        return false;
    }
    matches!(
        field.markers.get(idx),
        Some(BoundaryMarker::Boundary { .. })
    )
}

#[inline]
pub fn is_void_at_inner(
    grid: &EnergyFieldGrid,
    field: &EcoBoundaryField,
    pos: Vec2,
    playfield_margin_cells: u32,
) -> bool {
    zone_at_inner(grid, field, pos, playfield_margin_cells) == ZoneClass::Void
}

/// Interfaz ECS: lectura compartida, sin estado propio.
///
/// `EcoBoundaryField` es opcional: si no existe el resource, `context_at` devuelve baseline legado (E5 sprint).
#[derive(SystemParam)]
pub struct ContextLookup<'w> {
    pub grid: Res<'w, EnergyFieldGrid>,
    pub boundaries: Option<Res<'w, EcoBoundaryField>>,
    pub climate: Option<Res<'w, ClimateState>>,
    pub playfield_margin: Option<Res<'w, EcoPlayfieldMargin>>,
}

impl<'w> ContextLookup<'w> {
    #[inline]
    fn playfield_margin_cells(&self) -> u32 {
        self.playfield_margin.as_ref().map(|m| m.cells).unwrap_or(0)
    }

    #[inline]
    pub fn eco_enabled(&self) -> bool {
        self.boundaries.is_some()
    }

    /// True si hay eco activo y la posición no debe ejecutar catálisis (Void o reactividad nula).
    #[inline]
    pub fn should_skip_catalysis_at(&self, pos: Vec2) -> bool {
        if self.boundaries.is_none() {
            return false;
        }
        let ctx = self.context_at(pos);
        ctx.zone == ZoneClass::Void || ctx.reactivity_mod <= 0.0
    }

    #[inline]
    pub fn context_at(&self, pos: Vec2) -> ContextResponse {
        let Some(field) = self.boundaries.as_ref() else {
            return context_response_legacy_baseline();
        };
        context_at_inner(
            self.grid.as_ref(),
            field.as_ref(),
            self.climate.as_ref().map(|r| r.as_ref()),
            pos,
            self.playfield_margin_cells(),
        )
    }

    #[inline]
    pub fn zone_at(&self, pos: Vec2) -> ZoneClass {
        match self.boundaries.as_ref() {
            None => ZoneClass::Surface,
            Some(field) => zone_at_inner(
                self.grid.as_ref(),
                field.as_ref(),
                pos,
                self.playfield_margin_cells(),
            ),
        }
    }

    #[inline]
    pub fn is_boundary_at(&self, pos: Vec2) -> bool {
        match self.boundaries.as_ref() {
            None => false,
            Some(field) => is_boundary_at_inner(
                self.grid.as_ref(),
                field.as_ref(),
                pos,
                self.playfield_margin_cells(),
            ),
        }
    }

    #[inline]
    pub fn is_void_at(&self, pos: Vec2) -> bool {
        match self.boundaries.as_ref() {
            None => false,
            Some(field) => is_void_at_inner(
                self.grid.as_ref(),
                field.as_ref(),
                pos,
                self.playfield_margin_cells(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eco::boundary_field::aggregate_zone_class_contexts;
    use crate::eco::contracts::BoundaryMarker;
    use crate::eco::zone_classifier::classify_cell;
    use crate::layers::MatterState;
    use bevy::asset::Handle;
    use std::collections::HashMap;

    fn grid_two_zone_ids() -> (EnergyFieldGrid, EcoBoundaryField) {
        let mut grid = EnergyFieldGrid::new(2, 1, 1.0, Vec2::ZERO);
        for x in 0..2 {
            let c = grid.cell_xy_mut(x, 0).unwrap();
            c.accumulated_qe = 4.0;
            c.temperature = 1.5;
            c.matter_state = MatterState::Liquid;
            c.dominant_frequency_hz = 250.0;
        }
        let markers = vec![
            BoundaryMarker::Interior { zone_id: 0 },
            BoundaryMarker::Interior { zone_id: 1 },
        ];
        let zone_contexts: HashMap<u16, ZoneContext> = HashMap::from([
            (
                0,
                ZoneContext {
                    pressure: 1.0,
                    viscosity: 0.5,
                    temperature_base: 10.0,
                    dissipation_mod: 1.0,
                    reactivity_mod: 1.0,
                },
            ),
            (
                1,
                ZoneContext {
                    pressure: 3.0,
                    viscosity: 1.5,
                    temperature_base: 30.0,
                    dissipation_mod: 2.0,
                    reactivity_mod: 0.5,
                },
            ),
        ]);
        let zone_class_context =
            aggregate_zone_class_contexts(&markers, &grid, &zone_contexts, grid.cell_size);
        let field = EcoBoundaryField {
            width: 2,
            height: 1,
            cell_size: 1.0,
            origin: Vec2::ZERO,
            markers,
            cell_zone_ids: vec![0, 1],
            zone_class_context,
            zone_contexts,
            last_seen_grid_generation: 0,
            last_recompute_sim_tick: 0,
        };
        (grid, field)
    }

    #[test]
    fn context_at_interior_es_cache_o1() {
        let (grid, field) = grid_two_zone_ids();
        let r = context_at_inner(&grid, &field, None, Vec2::new(0.5, 0.5), 0);
        assert!(!r.is_boundary);
        assert!((r.pressure - 1.0).abs() < 1e-4);
        assert!((r.viscosity - 0.5).abs() < 1e-4);
        assert!((r.temperature_base - 10.0).abs() < 1e-4);
    }

    #[test]
    fn context_at_frontera_lerpa() {
        let (grid, mut field) = grid_two_zone_ids();
        field.markers[1] = BoundaryMarker::Boundary {
            zone_a: ZoneClass::Surface,
            zone_b: ZoneClass::Surface,
            gradient_factor: 0.5,
            transition_type: crate::eco::contracts::TransitionType::DensityGradient,
        };
        field.zone_class_context = aggregate_zone_class_contexts(
            &field.markers,
            &grid,
            &field.zone_contexts,
            grid.cell_size,
        );
        let r = context_at_inner(&grid, &field, None, Vec2::new(1.5, 0.5), 0);
        assert!(r.is_boundary);
        let mid_p = (1.0 + 3.0) * 0.5;
        assert!((r.pressure - mid_p).abs() < 1e-3, "pressure {}", r.pressure);
    }

    #[test]
    fn context_at_fuera_grid_es_void() {
        let (grid, field) = grid_two_zone_ids();
        let r = context_at_inner(&grid, &field, None, Vec2::new(99.0, 99.0), 0);
        assert_eq!(r.zone, ZoneClass::Void);
        assert_eq!(r.pressure, 0.0);
        assert_eq!(r.reactivity_mod, 0.0);
    }

    #[test]
    fn zone_at_y_is_void_at() {
        let (grid, field) = grid_two_zone_ids();
        let cell0 = grid.cell_xy(0, 0).expect("cell");
        let z0 = classify_cell(cell0, grid.cell_size);
        assert_eq!(zone_at_inner(&grid, &field, Vec2::new(0.5, 0.5), 0), z0);
        assert!(!is_void_at_inner(&grid, &field, Vec2::new(0.5, 0.5), 0));
        let r_void = context_at_inner(&grid, &field, None, Vec2::new(-1.0, 0.0), 0);
        assert_eq!(r_void.zone, ZoneClass::Void);
        assert!(is_void_at_inner(&grid, &field, Vec2::new(-1.0, 0.0), 0));
    }

    #[test]
    fn is_boundary_at_detecta() {
        let (grid, mut field) = grid_two_zone_ids();
        assert!(!is_boundary_at_inner(&grid, &field, Vec2::new(0.5, 0.5), 0));
        field.markers[1] = BoundaryMarker::Boundary {
            zone_a: ZoneClass::Surface,
            zone_b: ZoneClass::Surface,
            gradient_factor: 0.2,
            transition_type: crate::eco::contracts::TransitionType::DensityGradient,
        };
        assert!(is_boundary_at_inner(&grid, &field, Vec2::new(1.5, 0.5), 0));
    }

    #[test]
    fn zone_at_en_frontera_devuelve_zone_a() {
        let (grid, mut field) = grid_two_zone_ids();
        field.markers[1] = BoundaryMarker::Boundary {
            zone_a: ZoneClass::HighAtmosphere,
            zone_b: ZoneClass::Void,
            gradient_factor: 0.25,
            transition_type: crate::eco::contracts::TransitionType::ElementFrontier,
        };
        assert_eq!(
            zone_at_inner(&grid, &field, Vec2::new(1.5, 0.5), 0),
            ZoneClass::HighAtmosphere
        );
    }

    #[test]
    fn sin_climate_perfil_neutro() {
        let (grid, field) = grid_two_zone_ids();
        let r = context_at_inner(&grid, &field, None, Vec2::new(0.5, 0.5), 0);
        assert!((r.temperature_base - 10.0).abs() < 1e-4);
    }

    #[test]
    fn con_climate_aplica_offset_termico() {
        let (grid, field) = grid_two_zone_ids();
        let mut climate = ClimateState::new(Handle::default());
        climate.effective.temperature_offset = 100.0;
        climate.effective.precipitation_factor = 1.0;
        climate.effective.wind_intensity = 1.0;
        let r = context_at_inner(&grid, &field, Some(&climate), Vec2::new(0.5, 0.5), 0);
        assert!((r.temperature_base - 110.0).abs() < 1e-3);
    }

    #[test]
    fn apply_season_zero_no_anula_viscosidad() {
        let base = ZoneContext {
            pressure: 1.0,
            viscosity: 2.0,
            temperature_base: 0.0,
            dissipation_mod: 1.0,
            reactivity_mod: 1.0,
        };
        let out = apply_season_to_zone_context(base, ZoneClass::Surface, &SeasonProfile::ZERO);
        assert!((out.viscosity - 2.0).abs() < 1e-4);
    }

    #[test]
    fn interior_sin_zone_context_zone_at_alineado_con_context_at() {
        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        let c = grid.cell_xy_mut(0, 0).unwrap();
        c.accumulated_qe = 4.0;
        c.temperature = 1.5;
        c.matter_state = MatterState::Liquid;
        c.dominant_frequency_hz = 250.0;
        let markers = vec![BoundaryMarker::Interior { zone_id: 0 }];
        let zone_contexts = HashMap::new();
        let zone_class_context =
            aggregate_zone_class_contexts(&markers, &grid, &zone_contexts, grid.cell_size);
        let field = EcoBoundaryField {
            width: 1,
            height: 1,
            cell_size: 1.0,
            origin: Vec2::ZERO,
            markers,
            cell_zone_ids: vec![0],
            zone_class_context,
            zone_contexts,
            last_seen_grid_generation: 0,
            last_recompute_sim_tick: 0,
        };
        assert_eq!(
            zone_at_inner(&grid, &field, Vec2::new(0.5, 0.5), 0),
            ZoneClass::Void
        );
        assert!(is_void_at_inner(&grid, &field, Vec2::new(0.5, 0.5), 0));
        let r = context_at_inner(&grid, &field, None, Vec2::new(0.5, 0.5), 0);
        assert_eq!(r.zone, ZoneClass::Void);
    }

    #[test]
    fn interior_qe_bajo_is_void_at_y_context_void() {
        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        grid.cell_xy_mut(0, 0).unwrap().accumulated_qe = 0.0;
        let markers = vec![BoundaryMarker::Interior { zone_id: 0 }];
        let zone_contexts = HashMap::new();
        let zone_class_context =
            aggregate_zone_class_contexts(&markers, &grid, &zone_contexts, grid.cell_size);
        let field = EcoBoundaryField {
            width: 1,
            height: 1,
            cell_size: 1.0,
            origin: Vec2::ZERO,
            markers,
            cell_zone_ids: vec![0],
            zone_class_context,
            zone_contexts,
            last_seen_grid_generation: 0,
            last_recompute_sim_tick: 0,
        };
        assert!(is_void_at_inner(&grid, &field, Vec2::new(0.5, 0.5), 0));
        let r = context_at_inner(&grid, &field, None, Vec2::new(0.5, 0.5), 0);
        assert_eq!(r.zone, ZoneClass::Void);
    }

    #[test]
    fn campo_desalineado_degrada_a_baseline() {
        let (grid, mut field) = grid_two_zone_ids();
        field.width = 99;
        assert_eq!(
            zone_at_inner(&grid, &field, Vec2::new(0.5, 0.5), 0),
            ZoneClass::Surface
        );
        assert!(!is_void_at_inner(&grid, &field, Vec2::new(0.5, 0.5), 0));
        assert!(!is_boundary_at_inner(&grid, &field, Vec2::new(1.5, 0.5), 0));
        let r = context_at_inner(&grid, &field, None, Vec2::new(0.5, 0.5), 0);
        assert_eq!(r.zone, ZoneClass::Surface);
        assert!((r.dissipation_mod - 1.0).abs() < 1e-4 && (r.reactivity_mod - 1.0).abs() < 1e-4);
    }

    #[test]
    fn logical_void_margin_marca_solo_borde() {
        assert!(is_cell_in_logical_void_margin(0, 0, 5, 5, 1));
        assert!(is_cell_in_logical_void_margin(2, 0, 5, 5, 1));
        assert!(!is_cell_in_logical_void_margin(2, 2, 5, 5, 1));
    }

    #[test]
    fn legacy_baseline_neutral_para_simulacion() {
        let b = context_response_legacy_baseline();
        assert!((b.dissipation_mod - 1.0).abs() < 1e-5);
        assert!((b.reactivity_mod - 1.0).abs() < 1e-5);
        assert!((b.temperature_base - 0.0).abs() < 1e-5);
        assert!((b.viscosity - 1.0).abs() < 1e-5);
    }
}
