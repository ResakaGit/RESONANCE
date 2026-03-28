# Planetary Simulation

## Overview

Resonance simulates a planet's surface as a 2D toroidal energy field with day/night rotation, seasonal axial tilt, and a closed water cycle. All behavior derives from the 8 foundational axioms and 4 fundamental constants.

## Features

### 1. Toroidal Topology

The grid wraps in both X and Y axes — no edges. Energy diffusion, radiation pressure, and neighbor lookups all wrap cyclically. This models a spherical surface projected onto a torus.

**Implementation:** `field_grid.rs::neighbors4()` returns wrapped coordinates via modulo arithmetic. Propagation diffusion loops use `(x+1) % width` instead of bounds checks.

### 2. Day/Night Cycle

A solar meridian sweeps across the X axis at angular velocity `ω = 2π / period_ticks`. Irradiance follows cosine falloff from the meridian (cylindrical wrapping). Night cooling is proportional (Newton's law): `drain = cell_qe × DISSIPATION_SOLID × shadow_depth`.

**Equations:** `planetary_rotation.rs` — `solar_meridian_x()`, `solar_irradiance_factor()`, `night_cooling_fraction()`.

### 3. Seasonal Modulation

Axial tilt causes the sub-solar latitude to oscillate over the year period:

```
sub_solar_y = center + axial_tilt × half_height × sin(2π × tick / year_period)
seasonal_modifier = 0.5 + 0.5 × cos(π × wrapped_distance_y / grid_height)
```

Result: poles experience winter (reduced irradiance) and summer (increased). Life migrates with the seasons.

**Equation:** `planetary_rotation.rs::seasonal_irradiance_modifier()`.

**Config:** `year_period_ticks` + `axial_tilt` in MapConfig (`.ron` files).

### 4. Water Cycle

Evaporation from hot cells, precipitation on cold cells. Conservation-safe (double-buffered deltas):

```
evaporation = water_norm × DISSIPATION_LIQUID × min(cell_qe / DENSITY_SCALE, 1.0)
→ transferred to coolest neighbor (precipitation)
```

Water moves from warm zones to cool zones, creating emergent river-like patterns.

**System:** `water_cycle_system` in ThermodynamicLayer, after `day_night_modulation_system`.

### 5. Emission Scaling

Nucleus emissions scale with grid area via `emission_scale` in MapConfig:

- `None` = no scaling (backward compatible)
- `Some(x)` = multiply all emissions by `x`

For area-proportional scaling: `emission_scale = new_grid_area / reference_grid_area`.

### 6. Injectable Cosmological Anchor

`self_sustaining_qe` in MapConfig overrides the minimum energy for life (default 20.0). Lower values = easier life, higher = scarcer. All ~40 derived lifecycle constants scale automatically.

## Map Configuration

```ron
(
  width_cells: 128,
  height_cells: 128,
  cell_size: 2.0,
  day_period_ticks: Some(1200.0),
  year_period_ticks: Some(24000.0),  // 20 rotations per year
  axial_tilt: Some(0.26),           // Earth-like 23.5°
  self_sustaining_qe: Some(10.0),
  emission_scale: Some(7.0),        // 128²/48² area ratio
  // ...
)
```

## Axiom Compliance

| Feature | Axiom | Derivation |
|---------|-------|------------|
| Toroidal wrap | 7 (distance attenuation) | Shortest distance on wrapped grid |
| Solar irradiance | 4 (dissipation) | Cosine falloff = energy attenuation |
| Night cooling | 4 | `DISSIPATION_SOLID` = solid ground radiative loss |
| Ambient light | 4 | `DISSIPATION_SOLID / DISSIPATION_GAS` = retention ratio |
| Seasonal tilt | 8 (oscillatory) | Sub-solar latitude oscillates sinusoidally |
| Evaporation | 4 | `DISSIPATION_LIQUID` = liquid→gas phase transition |
| Emission scale | 1 (everything is energy) | Energy flux per area is constant |

## Key Files

- `src/blueprint/equations/planetary_rotation.rs` — all pure math
- `src/worldgen/systems/day_night.rs` — day/night + seasonal system
- `src/worldgen/systems/water_cycle.rs` — evaporation/precipitation
- `src/worldgen/field_grid.rs` — toroidal `neighbors4()`
- `src/worldgen/map_config.rs` — injectable parameters
- `assets/maps/earth_128.ron` — reference planetary map
