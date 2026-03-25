//! Integración T9: `AssetEvent` → `terrain_config_loader_system` (mismo orden que `Assets::asset_events`).

use bevy::asset::AssetPlugin;
use bevy::prelude::*;

use resonance::topology::config::{
    TerrainConfig, TerrainConfigAssetState, TerrainConfigRonLoader, TerrainConfigRuntime,
    terrain_config_loader_system,
};

#[test]
fn modified_asset_flushes_event_and_loader_updates_generation() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<TerrainConfig>()
        .init_asset_loader::<TerrainConfigRonLoader>();

    let handle = {
        let world = app.world_mut();
        let mut assets = world.resource_mut::<Assets<TerrainConfig>>();
        assets.add(TerrainConfig::default())
    };

    app.insert_resource(TerrainConfigAssetState {
        initialized: true,
        built: true,
        handle: Some(handle.clone()),
    })
    .insert_resource(TerrainConfigRuntime {
        handle: handle.clone(),
        effective: Some(TerrainConfig::default()),
        generation: 3,
    });

    app.add_systems(
        Update,
        (
            Assets::<TerrainConfig>::asset_events,
            terrain_config_loader_system,
        )
            .chain(),
    );

    {
        let world = app.world_mut();
        let mut assets = world.resource_mut::<Assets<TerrainConfig>>();
        let cfg = assets.get_mut(&handle).expect("terrain config");
        cfg.seed = 55_555;
    }

    app.update();

    let world = app.world();
    let runtime = world.resource::<TerrainConfigRuntime>();
    assert_eq!(runtime.generation, 4);
    assert_eq!(runtime.effective.as_ref().expect("effective").seed, 55_555);
}
