//! Hooks de lifecycle en spawn (por entidad), separados de observers globales en `simulation::observers`.
//!
//! El flag `EntityBuilder::observe_hero_base_energy_spawn` está pensado para arquetipos de héroe;
//! no valida composición MOBA (solo orden observe → insert L0).

use bevy::prelude::*;

use crate::layers::BaseEnergy;

/// Solo tests: contar disparos del hook `OnAdd<BaseEnergy>` (no insertar en bootstrap de producción).
#[derive(Resource, Default, Debug)]
pub struct HeroSpawnLifecycleTestHits {
    pub on_add_base_energy: u32,
}

/// Log cuando la entidad recibe `BaseEnergy` en spawn; el observer debe registrarse **antes** del insert.
pub fn on_hero_base_energy_added(
    trigger: Trigger<OnAdd, BaseEnergy>,
    energies: Query<&BaseEnergy>,
    hits: Option<ResMut<HeroSpawnLifecycleTestHits>>,
) {
    if let Some(mut h) = hits {
        h.on_add_base_energy += 1;
    }
    let Ok(energy) = energies.get(trigger.entity()) else {
        return;
    };
    debug!(
        "Spawn lifecycle OnAdd<BaseEnergy>: {:.2} qe (hook héroe / EntityBuilder)",
        energy.qe()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntityBuilder;
    use bevy::MinimalPlugins;

    #[test]
    fn observe_before_base_energy_insert_triggers_on_add() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(HeroSpawnLifecycleTestHits::default());
        {
            let world = app.world_mut();
            let mut commands = world.commands();
            EntityBuilder::new()
                .named("hero_observer_probe")
                .observe_hero_base_energy_spawn()
                .at(Vec2::ZERO)
                .energy(42.0)
                .spawn(&mut commands);
            world.flush();
        }
        assert_eq!(
            app.world()
                .resource::<HeroSpawnLifecycleTestHits>()
                .on_add_base_energy,
            1
        );
        let mut q = app.world_mut().query::<(&Name, &BaseEnergy)>();
        let (name, energy) = q.iter(app.world()).next().expect("spawned entity");
        assert_eq!(name.as_str(), "hero_observer_probe");
        assert!((energy.qe() - 42.0).abs() < 1e-4);
    }
}
