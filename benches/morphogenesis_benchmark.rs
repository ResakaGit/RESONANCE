use bevy::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion};
use resonance::layers::{
    AmbientPressure, BaseEnergy, FlowVector, MetabolicGraphBuilder, MorphogenesisShapeParams,
    OrganRole, SpatialVolume,
};
use resonance::simulation::metabolic::morphogenesis::{
    albedo_inference_system, entropy_constraint_system, entropy_ledger_system,
    metabolic_graph_step_system, shape_optimization_system, surface_rugosity_system,
};

/// Builds a 12-node DAG for benchmark ceiling.
fn build_max_graph() -> resonance::layers::MetabolicGraph {
    MetabolicGraphBuilder::new()
        .add_node(OrganRole::Root,    0.9, 3.0)  // 0
        .add_node(OrganRole::Core,    0.7, 8.0)  // 1
        .add_node(OrganRole::Stem,    0.8, 5.0)  // 2
        .add_node(OrganRole::Leaf,    0.85, 2.0) // 3
        .add_node(OrganRole::Fin,     0.6, 5.0)  // 4
        .add_node(OrganRole::Sensory, 0.5, 4.0)  // 5
        .add_node(OrganRole::Thorn,   0.4, 6.0)  // 6
        .add_node(OrganRole::Shell,   0.3, 7.0)  // 7
        .add_node(OrganRole::Fruit,   0.5, 3.0)  // 8
        .add_node(OrganRole::Bud,     0.6, 2.0)  // 9
        .add_node(OrganRole::Limb,    0.7, 4.0)  // 10
        .add_node(OrganRole::Petal,   0.55, 3.0) // 11
        .add_edge(0, 1, 50.0)
        .add_edge(0, 2, 50.0)
        .add_edge(0, 3, 50.0)
        .add_edge(1, 4, 40.0)
        .add_edge(1, 5, 40.0)
        .add_edge(2, 6, 40.0)
        .add_edge(2, 7, 40.0)
        .add_edge(3, 8, 40.0)
        .add_edge(3, 9, 40.0)
        .add_edge(4, 10, 30.0)
        .add_edge(5, 11, 30.0)
        .build()
        .expect("valid 12-node DAG")
}

/// Benchmark: 100 entities with full MG pipeline (6 systems, no rendering).
fn morphogenesis_pipeline_100_entities(c: &mut Criterion) {
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

    let graph = build_max_graph();
    {
        let mut commands = app.world_mut().commands();
        for i in 0..100u32 {
            let x = (i % 10) as f32 * 5.0;
            let y = (i / 10) as f32 * 5.0;
            commands.spawn((
                Transform::from_translation(Vec3::new(x, y, 0.0)),
                Visibility::default(),
                BaseEnergy::new(500.0),
                SpatialVolume::new(2.0),
                FlowVector::new(Vec2::new(4.0, 0.0), 0.05),
                AmbientPressure::new(0.0, 1000.0),
                graph,
                MorphogenesisShapeParams::default(),
            ));
        }
    }

    // Pre-warm: 3 ticks for EntropyLedger + MorphogenesisSurface + InferredAlbedo insertion.
    app.update();
    app.update();
    app.update();

    c.bench_function("mg_pipeline_100_entities", |b| {
        b.iter(|| {
            app.update();
        });
    });
}

criterion_group!(benches, morphogenesis_pipeline_100_entities);
criterion_main!(benches);
