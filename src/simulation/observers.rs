//! Observers de lifecycle (Bevy 0.15): hooks inmediatos para add/remove de componentes.
//!
//! Complementan `DeathEvent` y el pipeline por fases; no sustituyen el flujo de gameplay.
//!
//! **Contrato (doble canal):** `DeathEvent` = umbral de existencia (L0 vía `EnergyOps`, gameplay).
//! `OnRemove<BaseEnergy>` = componente retirado (típicamente `despawn`, o `remove` explícito).
//! No son equivalentes: con `qe == 0` el componente puede seguir montado hasta PostPhysics.
//!
//! Ver `docs/sprints/GAMEDEV_PATTERNS/README.md` (G7 cerrado).

use bevy::prelude::*;

use crate::layers::{BaseEnergy, ResonanceLink};

/// Contadores opcionales para tests de integración (no se inserta en bootstrap de producción).
#[derive(Resource, Default, Debug)]
pub struct LifecycleObserverTestHits {
    pub on_remove_base_energy: u32,
    pub on_add_resonance_link: u32,
    pub on_remove_resonance_link: u32,
}

/// Registra observers globales de lifecycle (muerte por retiro de L0, buff L10 add/remove).
pub fn setup_lifecycle_observers(app: &mut App) {
    app.add_observer(on_base_energy_removed);
    app.add_observer(on_resonance_link_added);
    app.add_observer(on_resonance_link_removed);
}

fn on_base_energy_removed(
    trigger: Trigger<OnRemove, BaseEnergy>,
    names: Query<&Name>,
    hits: Option<ResMut<LifecycleObserverTestHits>>,
) {
    if let Some(mut h) = hits {
        h.on_remove_base_energy += 1;
    }
    let label = names
        .get(trigger.entity())
        .map(|n| n.as_str())
        .unwrap_or("<unnamed>");
    debug!("{label}: OnRemove<BaseEnergy> (despawn o remove explícito; no implica solo qe→0)");
}

fn on_resonance_link_added(
    trigger: Trigger<OnAdd, ResonanceLink>,
    links: Query<&ResonanceLink>,
    names: Query<&Name>,
    hits: Option<ResMut<LifecycleObserverTestHits>>,
) {
    if let Some(mut h) = hits {
        h.on_add_resonance_link += 1;
    }
    let Ok(link) = links.get(trigger.entity()) else {
        return;
    };
    let target_label = names
        .get(link.target)
        .map(|n| n.as_str())
        .unwrap_or("<unnamed_target>");
    debug!(
        "ResonanceLink OnAdd: effect {:?} → target {target_label}",
        trigger.entity()
    );
}

fn on_resonance_link_removed(
    trigger: Trigger<OnRemove, ResonanceLink>,
    hits: Option<ResMut<LifecycleObserverTestHits>>,
) {
    if let Some(mut h) = hits {
        h.on_remove_resonance_link += 1;
    }
    debug!("ResonanceLink OnRemove: effect {:?}", trigger.entity());
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;

    use crate::events::{DeathCause, DeathEvent};
    use crate::simulation::post::faction_identity_system;
    use crate::world::Scoreboard;

    fn minimal_app_with_observers() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        setup_lifecycle_observers(&mut app);
        app.insert_resource(LifecycleObserverTestHits::default());
        app
    }

    #[test]
    fn on_remove_base_energy_fires_on_despawn() {
        let mut app = minimal_app_with_observers();
        let e = app
            .world_mut()
            .spawn((Name::new("despawn_probe"), BaseEnergy::new(5.0)))
            .id();
        app.world_mut().flush();
        app.world_mut().entity_mut(e).despawn();
        app.world_mut().flush();

        assert_eq!(
            app.world()
                .resource::<LifecycleObserverTestHits>()
                .on_remove_base_energy,
            1
        );
    }

    #[test]
    fn death_event_then_faction_despawn_triggers_on_remove_base_energy() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        app.init_resource::<Scoreboard>();
        setup_lifecycle_observers(&mut app);
        app.insert_resource(LifecycleObserverTestHits::default());
        app.add_systems(Update, faction_identity_system);

        let e = app
            .world_mut()
            .spawn((Name::new("death_chain"), BaseEnergy::new(1.0)))
            .id();
        app.world_mut().flush();

        app.world_mut()
            .resource_mut::<Events<DeathEvent>>()
            .send(DeathEvent {
                entity: e,
                cause: DeathCause::Destruction,
            });
        app.update();
        app.world_mut().flush();

        assert_eq!(
            app.world()
                .resource::<LifecycleObserverTestHits>()
                .on_remove_base_energy,
            1
        );
        assert!(app.world().get_entity(e).is_err());
    }

    #[test]
    fn on_add_and_remove_resonance_link() {
        let mut app = minimal_app_with_observers();
        let target = app.world_mut().spawn(Name::new("buff_target")).id();
        app.world_mut().flush();

        let effect = app
            .world_mut()
            .spawn((
                Name::new("effect_entity"),
                ResonanceLink {
                    target,
                    modified_field: crate::layers::ModifiedField::VelocityMultiplier,
                    magnitude: 0.5,
                },
            ))
            .id();
        app.world_mut().flush();
        assert_eq!(
            app.world()
                .resource::<LifecycleObserverTestHits>()
                .on_add_resonance_link,
            1
        );

        app.world_mut().entity_mut(effect).remove::<ResonanceLink>();
        app.world_mut().flush();
        assert_eq!(
            app.world()
                .resource::<LifecycleObserverTestHits>()
                .on_remove_resonance_link,
            1
        );
    }
}
