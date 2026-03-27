//! Headless simulation runner.
//!
//! Runs the full simulation for N ticks without GPU, then dumps
//! the world state as a PPM image (viewable in any image viewer).
//!
//! Usage:
//!   RESONANCE_MAP=genesis_validation cargo run --release --bin headless_sim -- --ticks 10000 --scale 8

use bevy::prelude::*;

use resonance::layers::{BaseEnergy, OscillatorySignature, SpatialVolume};
use resonance::plugins::{LayersPlugin, SimulationPlugin};
use resonance::rendering::quantized_color::PaletteRegistry;
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::runtime_platform::simulation_tick::{SimulationClock, SimulationTickPlugin};
use resonance::worldgen::EnergyFieldGrid;

fn main() {
    let ticks = parse_arg("--ticks").unwrap_or(200);
    let scale = parse_arg("--scale").unwrap_or(4) as usize;
    let out_path = parse_arg_str("--out").unwrap_or_else(|| "world.ppm".to_string());

    eprintln!("=== Resonance Headless Simulator ===");
    eprintln!("ticks: {ticks}  |  scale: {scale}x  |  output: {out_path}");

    // ── Build headless app ──────────────────────────────────────────────────
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<Mesh>();
    app.init_asset::<Image>();
    app.init_asset::<bevy::pbr::StandardMaterial>();
    app.init_resource::<PaletteRegistry>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.add_plugins(SimulationTickPlugin);
    app.insert_resource(SimWorldTransformParams::default());
    app.add_plugins(LayersPlugin);
    app.add_plugins(SimulationPlugin);

    // ── Startup ─────────────────────────────────────────────────────────────
    app.finish();
    app.cleanup();
    app.update(); // Startup schedule: worldgen warmup, state transitions.

    // Run a few more updates to let state transitions settle (Playing + Active).
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        app.update();
    }

    // Debug: check game state.
    {
        let w = app.world();
        let gs = w.get_resource::<State<resonance::simulation::states::GameState>>();
        let ps = w.get_resource::<State<resonance::simulation::states::PlayState>>();
        eprintln!("GameState: {gs:?}  |  PlayState: {ps:?}");
    }
    eprintln!("startup complete — running {ticks} simulation ticks...");

    // ── Tick loop ───────────────────────────────────────────────────────────
    // Sleep 17ms per iteration so Time<Fixed> accumulator hits one step per update.
    // SimulationClock.tick_id is the ground truth for simulation progress.
    let sample_interval = (ticks / 20).max(1) as u64;
    let start = std::time::Instant::now();
    let mut last_printed_tick = 0_u64;
    let mut prev_clk = 0_u64;

    eprintln!("\n{:>6} {:>6} {:>10} {:>8} {:>8} {:>8} {:>6}",
        "tick", "alive", "total_qe", "avg_qe", "avg_rad", "avg_age", "sen");
    eprintln!("{}", "-".repeat(68));

    loop {
        // Sleep enough for one FixedUpdate step (16.6ms at 60Hz).
        std::thread::sleep(std::time::Duration::from_millis(17));
        app.update();

        let clk = app.world()
            .get_resource::<SimulationClock>()
            .map(|c| c.tick_id)
            .unwrap_or(0);

        if clk >= ticks as u64 {
            break;
        }

        if clk > last_printed_tick && clk % sample_interval == 0 {
            last_printed_tick = clk;
            let world = app.world_mut();
            let mut count = 0_u32;
            let mut sen_count = 0_u32;
            let mut sum_qe = 0.0_f32;
            let mut sum_rad = 0.0_f32;
            let mut sum_age = 0.0_f64;
            let mut eq = world.query::<(
                &BaseEnergy,
                &SpatialVolume,
                Option<&resonance::layers::SenescenceProfile>,
            )>();
            for (energy, vol, sen) in eq.iter(world) {
                if energy.is_dead() { continue; }
                count += 1;
                sum_qe += energy.qe();
                sum_rad += vol.radius;
                if let Some(s) = sen {
                    sen_count += 1;
                    sum_age += s.age(clk) as f64;
                }
            }
            let n = count.max(1) as f32;
            let mat_count = world.query::<&resonance::worldgen::Materialized>().iter(world).count();
            let avg_age = if sen_count > 0 { sum_age / sen_count as f64 } else { 0.0 };
            eprintln!("{:>6} {:>6} {:>10.1} {:>8.2} {:>8.3} {:>8.0} {:>6}  mat={}",
                clk, count, sum_qe, sum_qe / n, sum_rad / n, avg_age, sen_count, mat_count);
        }
    }
    let total_time = start.elapsed().as_secs_f32();
    let final_clk = app.world()
        .get_resource::<SimulationClock>()
        .map(|c| c.tick_id)
        .unwrap_or(0);
    eprintln!("{}", "-".repeat(68));
    eprintln!("sim ticks: {final_clk}  |  wall: {total_time:.1}s  |  {:.0} sim-ticks/s\n",
        final_clk as f32 / total_time);

    // ── Collect grid data ───────────────────────────────────────────────────
    let world = app.world_mut();
    let Some(grid) = world.get_resource::<EnergyFieldGrid>() else {
        eprintln!("no EnergyFieldGrid — empty world?");
        return;
    };
    let gw = grid.width;
    let gh = grid.height;
    let total_qe = grid.total_qe();
    let grid_origin = grid.origin;
    let grid_cell_size = grid.cell_size;

    let mut max_qe: f32 = 1.0;
    let mut max_freq: f32 = 1.0;
    struct CellData { qe: f32, freq: f32, purity: f32 }
    let mut cell_data: Vec<CellData> = Vec::with_capacity((gw * gh) as usize);
    for y in 0..gh {
        for x in 0..gw {
            if let Some(cell) = grid.cell_xy(x, y) {
                max_qe = max_qe.max(cell.accumulated_qe);
                max_freq = max_freq.max(cell.dominant_frequency_hz);
                cell_data.push(CellData { qe: cell.accumulated_qe, freq: cell.dominant_frequency_hz, purity: cell.purity });
            } else {
                cell_data.push(CellData { qe: 0.0, freq: 0.0, purity: 0.0 });
            }
        }
    }

    let w = gw as usize;
    let h = gh as usize;
    let mut field_buf = vec![(0u8, 0u8, 0u8); w * h];
    let mut cells_with_energy = 0_u32;
    for y in 0..gh {
        for x in 0..gw {
            let ci = y as usize * w + x as usize;
            let cd = &cell_data[ci];
            if cd.qe > 0.1 { cells_with_energy += 1; }
            let intensity = (cd.qe / max_qe).sqrt();
            let hue = if max_freq > 0.0 { cd.freq / max_freq } else { 0.0 };
            let boosted = (intensity * 1.5).min(1.0);
            let sat = cd.purity.max(0.3);
            let idx = (gh - 1 - y) as usize * w + x as usize;
            field_buf[idx] = hsv_to_rgb(hue, sat, boosted);
        }
    }

    // ── Entities overlay ────────────────────────────────────────────────────
    struct EntityDot { grid_x: u32, grid_y: u32, freq: f32 }
    let mut entities: Vec<EntityDot> = Vec::new();
    let xz_ground = world
        .get_resource::<SimWorldTransformParams>()
        .map(|p| p.use_xz_ground)
        .unwrap_or(true);
    let mut eq = world.query::<(&Transform, &BaseEnergy, &SpatialVolume, &OscillatorySignature)>();
    for (tr, _energy, _vol, osc) in eq.iter(world) {
        let pos = if xz_ground {
            bevy::math::Vec2::new(tr.translation.x, tr.translation.z)
        } else {
            bevy::math::Vec2::new(tr.translation.x, tr.translation.y)
        };
        let rel = pos - grid_origin;
        if rel.x >= 0.0 && rel.y >= 0.0 {
            let cx = (rel.x / grid_cell_size).floor() as u32;
            let cy = (rel.y / grid_cell_size).floor() as u32;
            if cx < gw && cy < gh {
                entities.push(EntityDot { grid_x: cx, grid_y: cy, freq: osc.frequency_hz() });
            }
        }
    }

    eprintln!("grid: {w}x{h}  |  total_qe: {total_qe:.1}  |  max_cell_qe: {max_qe:.1}");
    eprintln!("cells with energy: {cells_with_energy}/{}  |  entities: {}", w * h, entities.len());

    for ent in &entities {
        let idx = (gh - 1 - ent.grid_y) as usize * w + ent.grid_x as usize;
        if idx < field_buf.len() {
            field_buf[idx] = (255, 255, 255);
            let hue = if max_freq > 0.0 { ent.freq / max_freq } else { 0.0 };
            let ring_color = hsv_to_rgb(hue, 1.0, 1.0);
            for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = ent.grid_x as i32 + dx;
                let ny = ent.grid_y as i32 + dy;
                if nx >= 0 && ny >= 0 && nx < gw as i32 && ny < gh as i32 {
                    let ri = (gh as i32 - 1 - ny) as usize * w + nx as usize;
                    if ri < field_buf.len() { field_buf[ri] = ring_color; }
                }
            }
        }
    }

    // ── Upscale + write PPM ─────────────────────────────────────────────────
    let out_w = w * scale;
    let out_h = h * scale;
    let mut pixels = Vec::with_capacity(out_w * out_h * 3);
    for oy in 0..out_h {
        for ox in 0..out_w {
            let src = (oy / scale) * w + (ox / scale);
            let (r, g, b) = field_buf[src];
            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
        }
    }
    let header = format!("P6\n{out_w} {out_h}\n255\n");
    let mut file_data = header.into_bytes();
    file_data.extend_from_slice(&pixels);
    std::fs::write(&out_path, &file_data).expect("failed to write PPM");
    eprintln!("wrote {out_path} ({out_w}x{out_h}, {:.1} KB)", file_data.len() as f32 / 1024.0);
    eprintln!("=== done ===");
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h.clamp(0.0, 1.0) * 6.0;
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (((r + m) * 255.0) as u8, ((g + m) * 255.0) as u8, ((b + m) * 255.0) as u8)
}

fn parse_arg(flag: &str) -> Option<u32> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok())
}

fn parse_arg_str(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1).cloned())
}
