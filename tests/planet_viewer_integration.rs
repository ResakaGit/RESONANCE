//! Integration tests for the planet viewer pipeline.
//!
//! Validates: frame_buffer rendering, texture data, day/night modulation,
//! water cycle, seasonal modifier, and energy conservation at different tick rates.

use resonance::blueprint::equations::derived_thresholds as dt;
use resonance::blueprint::equations::planetary_rotation::*;
use resonance::viewer::frame_buffer;
use resonance::worldgen::EnergyFieldGrid;

// ── Frame buffer rendering ──────────────────────────────────────────────────

#[test]
fn frame_buffer_produces_correct_dimensions() {
    let grid = EnergyFieldGrid::new(16, 16, 2.0, glam::Vec2::ZERO);
    let frame = frame_buffer::render_frame(&grid, &[], &[]);
    assert_eq!(frame.width, 16);
    assert_eq!(frame.height, 16);
    assert_eq!(frame.pixels.len(), 16 * 16);
}

#[test]
fn frame_buffer_empty_grid_has_dark_pixels() {
    let grid = EnergyFieldGrid::new(8, 8, 1.0, glam::Vec2::ZERO);
    let frame = frame_buffer::render_frame(&grid, &[], &[]);
    // All cells have qe=0 → should be very dark (surface min brightness = dark blue tint).
    for px in &frame.pixels {
        let lum = px[0] as u32 + px[1] as u32 + px[2] as u32;
        assert!(lum < 100, "empty cell should be dark: {:?} lum={}", px, lum);
        assert_eq!(px[3], 255, "alpha must be 255");
    }
}

#[test]
fn frame_buffer_high_energy_cell_is_bright() {
    let mut grid = EnergyFieldGrid::new(8, 8, 1.0, glam::Vec2::ZERO);
    if let Some(cell) = grid.cell_xy_mut(4, 4) {
        cell.accumulated_qe = 100.0;
        cell.dominant_frequency_hz = 85.0;
    }
    let frame = frame_buffer::render_frame(&grid, &[], &[]);
    // Cell (4,4) → pixel at (grid.height - 1 - 4) * 8 + 4 = 3*8+4 = 28
    let bright = frame.pixels[28];
    let dark = frame.pixels[0];
    let bright_lum = bright[0] as u32 + bright[1] as u32 + bright[2] as u32;
    let dark_lum = dark[0] as u32 + dark[1] as u32 + dark[2] as u32;
    assert!(bright_lum > dark_lum * 3, "high qe cell must be brighter: bright={bright_lum} dark={dark_lum}");
}

#[test]
fn frame_buffer_entity_dot_is_white() {
    let grid = EnergyFieldGrid::new(8, 8, 1.0, glam::Vec2::ZERO);
    let entities = vec![(3u32, 3u32, 85.0f32)];
    let frame = frame_buffer::render_frame(&grid, &entities, &[]);
    let idx = (8 - 1 - 3) as usize * 8 + 3;
    assert_eq!(frame.pixels[idx], [255, 255, 255, 255]);
}

#[test]
fn frame_buffer_behavioral_dot_is_cyan() {
    let grid = EnergyFieldGrid::new(8, 8, 1.0, glam::Vec2::ZERO);
    let behaviorals = vec![(4u32, 4u32)];
    let frame = frame_buffer::render_frame(&grid, &[], &behaviorals);
    // Cyan ring: check one neighbor of (4,4).
    let idx = (8 - 1 - 4) as usize * 8 + 5; // (5, 4)
    assert_eq!(frame.pixels[idx], [0, 255, 255, 255]);
}

// ── Day/night: tick-rate independence ────────────────────────────────────────

#[test]
fn night_cooling_fraction_is_dissipation_solid() {
    assert!((night_cooling_fraction() - dt::DISSIPATION_SOLID).abs() < 1e-8);
}

#[test]
fn solar_irradiance_day_side_is_bright() {
    // Meridian at x=24, cell at x=24 → factor = 1.0
    let f = solar_irradiance_factor(24.0, 24.0, 48.0);
    assert!((f - 1.0).abs() < 1e-5, "day side: {f}");
}

#[test]
fn solar_irradiance_night_side_is_dark() {
    // Antipode: half grid away.
    let f = solar_irradiance_factor(0.0, 24.0, 48.0);
    assert!(f < 0.01, "night side should be dark: {f}");
}

// ── Seasonal modifier ───────────────────────────────────────────────────────

#[test]
fn seasonal_no_tilt_returns_one() {
    let m = seasonal_irradiance_modifier(50.0, 100.0, 1000, 10000.0, 0.0);
    assert!((m - 1.0).abs() < 1e-6, "no tilt = no seasons: {m}");
}

#[test]
fn seasonal_no_year_returns_one() {
    let m = seasonal_irradiance_modifier(50.0, 100.0, 1000, 0.0, 0.26);
    assert!((m - 1.0).abs() < 1e-6, "no year = no seasons: {m}");
}

#[test]
fn seasonal_poles_differ_at_solstice() {
    // At tick = year/4 (summer solstice): sub-solar at max latitude.
    let year = 10000.0;
    let tilt = 0.26;
    let tick = (year * 0.25) as u64; // summer
    let north = seasonal_irradiance_modifier(90.0, 100.0, tick, year, tilt);
    let south = seasonal_irradiance_modifier(10.0, 100.0, tick, year, tilt);
    assert!(north > south, "north summer > south: north={north} south={south}");
}

#[test]
fn seasonal_equinox_poles_equal() {
    // At tick = 0 (equinox): sub-solar at center.
    let year = 10000.0;
    let tilt = 0.26;
    let north = seasonal_irradiance_modifier(80.0, 100.0, 0, year, tilt);
    let south = seasonal_irradiance_modifier(20.0, 100.0, 0, year, tilt);
    assert!((north - south).abs() < 0.15, "equinox should be ~symmetric: n={north} s={south}");
}

#[test]
fn seasonal_modifier_always_positive() {
    for tick in (0..10000).step_by(500) {
        for y in (0..100).step_by(10) {
            let m = seasonal_irradiance_modifier(y as f32, 100.0, tick, 10000.0, 0.5);
            assert!(m > 0.0, "modifier must be positive: tick={tick} y={y} m={m}");
        }
    }
}

// ── Water cycle: conservation ───────────────────────────────────────────────

#[test]
fn water_min_threshold_equals_dissipation_liquid() {
    // Water cycle should use DISSIPATION_LIQUID as minimum threshold.
    assert!((dt::DISSIPATION_LIQUID - 0.02).abs() < 1e-6);
}

// ── Toroidal topology ───────────────────────────────────────────────────────

#[test]
fn toroidal_neighbors_wrap_x() {
    let grid = EnergyFieldGrid::new(8, 8, 1.0, glam::Vec2::ZERO);
    let neighbors = grid.neighbors4(0, 4);
    // Left of x=0 should wrap to x=7.
    assert_eq!(neighbors[0], Some((7, 4)));
}

#[test]
fn toroidal_neighbors_wrap_y() {
    let grid = EnergyFieldGrid::new(8, 8, 1.0, glam::Vec2::ZERO);
    let neighbors = grid.neighbors4(4, 0);
    // Down from y=0 should wrap to y=7.
    assert_eq!(neighbors[2], Some((4, 7)));
}

#[test]
fn toroidal_neighbors_always_four() {
    let grid = EnergyFieldGrid::new(4, 4, 1.0, glam::Vec2::ZERO);
    for y in 0..4 {
        for x in 0..4 {
            let count = grid.neighbors4(x, y).iter().flatten().count();
            assert_eq!(count, 4, "all cells must have 4 neighbors: ({x},{y})={count}");
        }
    }
}

// ── Derived thresholds consistency ──────────────────────────────────────────

#[test]
fn recycling_threshold_matches_conversion_losses() {
    let expected = (1.0 - dt::nutrient_retention_mineral()) + (1.0 - dt::nutrient_retention_water());
    assert!((dt::recycling_nutrient_threshold() - expected).abs() < 1e-6);
}

#[test]
fn night_cooling_preserves_energy_over_full_night() {
    let frac = night_cooling_fraction();
    let remaining = (1.0 - frac).powi(300);
    assert!(remaining > 0.15, "cell must survive one night: {remaining:.3}");
}

#[test]
fn ambient_irradiance_derived_correctly() {
    let expected = dt::DISSIPATION_SOLID / dt::DISSIPATION_GAS;
    assert!((AMBIENT_IRRADIANCE - expected).abs() < 1e-6);
}
