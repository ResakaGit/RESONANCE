//! Demo inferred world IWG-7: multi-biome map with three fauna archetypes.
//! Body plans emerge from organ manifests processed by the morphogenesis pipeline.
//! `RESONANCE_MAP=inferred_world cargo run`.

use bevy::prelude::*;

use crate::blueprint::ElementId;
use crate::entities::builder::EntityBuilder;
use crate::layers::{LifecycleStage, MatterState, OrganManifest, OrganRole, OrganSpec};

/// Slug for conditional dispatch in plugins.
pub const INFERRED_WORLD_SLUG: &str = "inferred_world";

/// Spawns fauna entities for the inferred world demo.
pub fn spawn_inferred_world_startup_system(mut commands: Commands) {
    // ── Quadruped: terra zone, bilateral with limbs + thorns ────────────
    let mut quadruped_manifest = OrganManifest::new(LifecycleStage::Mature);
    quadruped_manifest.push(OrganSpec::new(OrganRole::Core, 1, 1.0));
    quadruped_manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0));
    quadruped_manifest.push(OrganSpec::new(OrganRole::Limb, 1, 1.0));
    quadruped_manifest.push(OrganSpec::new(OrganRole::Limb, 1, 1.0));
    quadruped_manifest.push(OrganSpec::new(OrganRole::Limb, 1, 1.0));
    quadruped_manifest.push(OrganSpec::new(OrganRole::Limb, 1, 1.0));
    quadruped_manifest.push(OrganSpec::new(OrganRole::Sensory, 1, 1.0));
    quadruped_manifest.push(OrganSpec::new(OrganRole::Thorn, 1, 1.0));
    quadruped_manifest.push(OrganSpec::new(OrganRole::Thorn, 1, 1.0));

    EntityBuilder::new()
        .named("iwg_quadruped")
        .at(Vec2::new(10.0, 10.0))
        .energy(200.0)
        .volume(1.5)
        .wave(ElementId::from_name("Terra"))
        .flow(Vec2::ZERO, 0.04)
        .matter(MatterState::Solid, 600.0, 0.1)
        .with_organ_manifest(quadruped_manifest)
        .with_metabolic_graph_inferred(400.0, 300.0)
        .spawn(&mut commands);

    // ── Fish: aqua zone, streamlined with fins ─────────────────────────
    let mut fish_manifest = OrganManifest::new(LifecycleStage::Mature);
    fish_manifest.push(OrganSpec::new(OrganRole::Core, 1, 1.0));
    fish_manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0));
    fish_manifest.push(OrganSpec::new(OrganRole::Fin, 1, 1.0));
    fish_manifest.push(OrganSpec::new(OrganRole::Fin, 1, 1.0));
    fish_manifest.push(OrganSpec::new(OrganRole::Sensory, 1, 1.0));

    EntityBuilder::new()
        .named("iwg_fish")
        .at(Vec2::new(48.0, 32.0))
        .energy(100.0)
        .volume(0.8)
        .wave(ElementId::from_name("Aqua"))
        .flow(Vec2::new(3.0, 0.0), 0.05)
        .matter(MatterState::Solid, 300.0, 0.08)
        .with_organ_manifest(fish_manifest)
        .with_metabolic_graph_inferred(350.0, 310.0)
        .spawn(&mut commands);

    // ── Star: coast / radial symmetry, limbs only ──────────────────────
    let mut star_manifest = OrganManifest::new(LifecycleStage::Mature);
    star_manifest.push(OrganSpec::new(OrganRole::Core, 1, 1.0));
    star_manifest.push(OrganSpec::new(OrganRole::Limb, 1, 1.0));
    star_manifest.push(OrganSpec::new(OrganRole::Limb, 1, 1.0));
    star_manifest.push(OrganSpec::new(OrganRole::Limb, 1, 1.0));
    star_manifest.push(OrganSpec::new(OrganRole::Limb, 1, 1.0));
    star_manifest.push(OrganSpec::new(OrganRole::Limb, 1, 1.0));

    EntityBuilder::new()
        .named("iwg_star")
        .at(Vec2::new(30.0, 25.0))
        .energy(150.0)
        .volume(1.0)
        .wave(ElementId::from_name("Terra"))
        .flow(Vec2::ZERO, 0.03)
        .matter(MatterState::Solid, 400.0, 0.06)
        .with_organ_manifest(star_manifest)
        .with_metabolic_graph_inferred(380.0, 300.0)
        .spawn(&mut commands);

    info!("IWG demo: spawned 3 fauna entities (quadruped, fish, star)");
}
