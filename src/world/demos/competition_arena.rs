//! Demo competencia energética EC-8D.
//! Bosque + Desierto + Océano con sub-pool (Matryoshka).
//! `RESONANCE_MAP=competition_arena cargo run`.

use bevy::prelude::*;

use crate::entities::archetypes::{spawn_competitor, spawn_environment_pool, spawn_sub_pool};
use crate::layers::ExtractionType;

/// Slug de mapa para dispatch condicional en plugins.
pub const COMPETITION_ARENA_SLUG: &str = "competition_arena";

/// Spawnea el escenario de competencia: pools + organismos.
pub fn spawn_competition_demo_startup_system(mut commands: Commands) {
    // ── Pool Bosque: alta intake, biodiversidad ──────────────────────────────
    let forest = spawn_environment_pool(
        &mut commands, 5000.0, 10000.0, 100.0, Vec3::new(-8.0, -8.0, 0.0),
    );
    spawn_competitor(&mut commands, forest, ExtractionType::Competitive, 0.6, 300.0, Vec3::new(-9.0, -7.0, 0.0));
    spawn_competitor(&mut commands, forest, ExtractionType::Competitive, 0.3, 200.0, Vec3::new(-8.0, -9.0, 0.0));
    spawn_competitor(&mut commands, forest, ExtractionType::Competitive, 0.1, 100.0, Vec3::new(-7.0, -7.0, 0.0));
    spawn_competitor(&mut commands, forest, ExtractionType::Aggressive,  0.4,  50.0, Vec3::new(-8.5, -8.5, 0.0));
    spawn_competitor(&mut commands, forest, ExtractionType::Regulated,  80.0, 150.0, Vec3::new(-7.5, -8.0, 0.0));

    // ── Pool Océano + sub-pool arrecife (Matryoshka) ─────────────────────────
    let ocean = spawn_environment_pool(
        &mut commands, 8000.0, 15000.0, 80.0, Vec3::new(0.0, 8.0, 0.0),
    );
    let reef = spawn_sub_pool(
        &mut commands, ocean, ExtractionType::Competitive, 0.5, 3000.0, 40.0, Vec3::new(1.0, 7.0, 0.0),
    );
    spawn_competitor(&mut commands, reef, ExtractionType::Greedy, 200.0, 100.0, Vec3::new(0.5, 6.5, 0.0));
    spawn_competitor(&mut commands, reef, ExtractionType::Greedy, 150.0,  80.0, Vec3::new(1.5, 6.5, 0.0));

    // ── Pool Desierto: baja intake, pocos competidores ───────────────────────
    let desert = spawn_environment_pool(
        &mut commands, 1000.0, 3000.0, 30.0, Vec3::new(8.0, -8.0, 0.0),
    );
    spawn_competitor(&mut commands, desert, ExtractionType::Proportional, 0.0,  80.0, Vec3::new(8.5, -7.5, 0.0));
    spawn_competitor(&mut commands, desert, ExtractionType::Proportional, 0.0,  60.0, Vec3::new(7.5, -7.5, 0.0));
}
