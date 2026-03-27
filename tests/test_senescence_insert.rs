use bevy::prelude::*;
use resonance::layers::{BaseEnergy, SenescenceProfile, SpatialVolume};

#[test]
fn senescence_profile_inserted_and_queryable() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let e = app.world_mut().spawn((
        BaseEnergy::new(100.0),
        SpatialVolume::new(1.0),
        SenescenceProfile { tick_birth: 42, senescence_coeff: 0.001, max_viable_age: 1000, strategy: 0 },
    )).id();
    let sen = app.world().get::<SenescenceProfile>(e);
    assert!(sen.is_some(), "SenescenceProfile should be queryable");
    assert_eq!(sen.unwrap().tick_birth, 42);
    assert_eq!(sen.unwrap().age(100), 58);
}
