//! MG-8G — Integration tests: phenotype table after 10 ticks.
//!
//! Verifica que los tres arquetipos de morfogénesis producen los fenotipos esperados
//! según el balance termodinámico (fineness, albedo, rugosity).

use bevy::prelude::*;
use resonance::entities::archetypes::{
    spawn_aquatic_organism, spawn_desert_plant, spawn_forest_plant,
};
use resonance::layers::{
    InferredAlbedo, MetabolicGraph, MorphogenesisShapeParams, MorphogenesisSurface,
};
use resonance::simulation::metabolic::morphogenesis::{
    albedo_inference_system, entropy_constraint_system, entropy_ledger_system,
    metabolic_graph_step_system, shape_optimization_system, surface_rugosity_system,
};

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(
        Update,
        (
            metabolic_graph_step_system,
            entropy_constraint_system,
            entropy_ledger_system,
            bevy::ecs::schedule::apply_deferred,
            shape_optimization_system,
            surface_rugosity_system,
            albedo_inference_system,
        )
            .chain(),
    );
    app
}

fn run_ticks(app: &mut App, n: u32) {
    for _ in 0..n {
        app.update();
    }
}

fn first_entity_with<T: Component + Clone>(app: &mut App) -> Option<T> {
    app.world_mut()
        .query::<&T>()
        .iter(app.world())
        .next()
        .cloned()
}

// ── MG-8G Test 1: Aquatic organism — high fineness (fusiforme) ─────────────

/// Organismo acuático en medio denso y alta velocidad → converge a forma fusiforme.
/// Con vel=15, viscosity=2.5 y 10 ticks el optimizer debe superar fineness_ratio 2.5.
#[test]
fn aquatic_organism_fineness_ratio_exceeds_threshold() {
    let mut app = make_app();
    {
        let mut commands = app.world_mut().commands();
        spawn_aquatic_organism(&mut commands, Vec2::ZERO);
    }
    run_ticks(&mut app, 13);

    let shape = first_entity_with::<MorphogenesisShapeParams>(&mut app)
        .expect("no MorphogenesisShapeParams found — entity not spawned?");
    assert!(
        shape.fineness_ratio() > 2.5,
        "aquatic organism fineness_ratio expected > 2.5, got {}",
        shape.fineness_ratio(),
    );
}

// ── MG-8G Test 2: Aquatic organism — dark albedo ───────────────────────────

/// Organismo acuático con baja irradiancia en bioma frío pero cuerpo interno caliente
/// (T_core ≈ 699K >> T_env=310K) → balance radiativo domina → albedo bajo (oscuro).
/// photon_density=5, absorbed_fraction=0.3, i_effective=1.5; la disipación radiativa
/// supera el flujo solar → alpha → ALBEDO_MIN extremo; el test pide < 0.4.
#[test]
fn aquatic_organism_albedo_is_dark() {
    let mut app = make_app();
    {
        let mut commands = app.world_mut().commands();
        spawn_aquatic_organism(&mut commands, Vec2::ZERO);
    }
    run_ticks(&mut app, 13);

    let albedo = first_entity_with::<InferredAlbedo>(&mut app)
        .expect("no InferredAlbedo found — albedo_inference_system not running?");
    assert!(
        albedo.albedo() < 0.4,
        "aquatic organism albedo expected < 0.4, got {}",
        albedo.albedo(),
    );
}

// ── MG-8G Test 3: Desert plant — bright albedo ────────────────────────────

/// Planta desértica: T_core << T_env (cuerpo frío, desierto caliente).
/// La disipación superficial es negativa (ambiente calienta) → albedo empuja hacia ALBEDO_MAX.
/// Con photon_density=100 e irradiancia alta, alpha → ALBEDO_MAX (0.95) → > 0.7.
#[test]
fn desert_plant_albedo_is_bright() {
    let mut app = make_app();
    {
        let mut commands = app.world_mut().commands();
        spawn_desert_plant(&mut commands, Vec2::ZERO);
    }
    run_ticks(&mut app, 13);

    let albedo = first_entity_with::<InferredAlbedo>(&mut app).expect("no InferredAlbedo found");
    assert!(
        albedo.albedo() > 0.7,
        "desert plant albedo expected > 0.7, got {}",
        albedo.albedo(),
    );
}

// ── MG-8G Test 4: Desert plant — high rugosity ────────────────────────────

/// Planta desértica: T_core < T_env → h*ΔT negativo → h_dt → ε → rugosity → RUGOSITY_MAX.
/// El test verifica que rugosity > 2.0 (radiadores / espinas).
#[test]
fn desert_plant_rugosity_is_high() {
    let mut app = make_app();
    {
        let mut commands = app.world_mut().commands();
        spawn_desert_plant(&mut commands, Vec2::ZERO);
    }
    run_ticks(&mut app, 13);

    let surface = first_entity_with::<MorphogenesisSurface>(&mut app)
        .expect("no MorphogenesisSurface found — surface_rugosity_system not running?");
    assert!(
        surface.rugosity() > 2.0,
        "desert plant rugosity expected > 2.0, got {}",
        surface.rugosity(),
    );
}

// ── MG-8G Test 5: Forest plant — neutral albedo ───────────────────────────

/// Planta de bosque: irradiancia ≈ 0 (photon=0.001, absorbed=0.001, flux≈1e-6 < eps).
/// Sistema retorna ALBEDO_FALLBACK = 0.5. El test pide albedo ∈ [0.25, 0.55].
#[test]
fn forest_plant_albedo_is_neutral() {
    let mut app = make_app();
    {
        let mut commands = app.world_mut().commands();
        spawn_forest_plant(&mut commands, Vec2::ZERO);
    }
    run_ticks(&mut app, 13);

    let albedo = first_entity_with::<InferredAlbedo>(&mut app).expect("no InferredAlbedo found");
    assert!(
        (0.25..=0.55).contains(&albedo.albedo()),
        "forest plant albedo expected in [0.25, 0.55], got {}",
        albedo.albedo(),
    );
}

// ── MG-8G Test 6: MetabolicGraph present on all MG archetypes ─────────────

/// Verifica que todos los arquetipos MG-8 producen entidades con MetabolicGraph.
#[test]
fn mg8_archetypes_all_have_metabolic_graph() {
    let mut app = make_app();
    {
        let mut commands = app.world_mut().commands();
        spawn_aquatic_organism(&mut commands, Vec2::new(-20.0, 0.0));
        spawn_desert_plant(&mut commands, Vec2::new(20.0, 0.0));
        spawn_forest_plant(&mut commands, Vec2::new(0.0, 20.0));
    }
    app.update();

    let count = app
        .world_mut()
        .query::<&MetabolicGraph>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 3,
        "expected 3 entities with MetabolicGraph, found {count}"
    );
}
