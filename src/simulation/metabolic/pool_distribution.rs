//! EC-4: Pool Distribution System — tick de extracción competitiva.
//! Tres sistemas en cadena: intake → distribution → dissipation.
//! Fase: [`crate::simulation::Phase::MetabolicLayer`].
//! Sin HashMap, sin allocations en hot path. Buffer stack-allocated, orden determinista.

use bevy::prelude::*;

use crate::blueprint::constants::{DAMAGE_RATE_DEFAULT, MAX_EXTRACTION_MODIFIERS};
use crate::blueprint::equations::{
    available_for_extraction, dissipation_loss, evaluate_aggressive_extraction,
    evaluate_extraction, scale_extractions_to_available, ExtractionContext, ExtractionProfile,
};
use crate::layers::{BaseEnergy, EnergyPool, ExtractionType, PoolConservationLedger, PoolParentLink};

/// Máximo de hijos por pool (stack buffer por grupo).
pub const MAX_CHILDREN_PER_POOL: usize = 64;
/// Máximo de entradas totales en el buffer de recolección.
const MAX_ENTRIES: usize = 512;

// ─── Buffer interno ───────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct ChildEntry {
    parent_idx: u32,
    parent:     Entity,
    child:      Entity,
    etype:      ExtractionType,
    param:      f32,
}

impl ChildEntry {
    #[inline]
    fn placeholder() -> Self {
        Self {
            parent_idx: u32::MAX,
            parent:     Entity::PLACEHOLDER,
            child:      Entity::PLACEHOLDER,
            etype:      ExtractionType::Proportional,
            param:      0.0,
        }
    }
}

// ─── EC-4A: Intake ────────────────────────────────────────────────────────────

/// Aplica intake al pool antes de la distribución.
pub fn pool_intake_system(mut pools: Query<&mut EnergyPool>) {
    for mut pool in &mut pools {
        let new_pool = (pool.pool() + pool.intake_rate()).min(pool.capacity());
        if pool.pool() != new_pool {
            pool.set_pool(new_pool);
        }
    }
}

// ─── EC-4B: Distribution ─────────────────────────────────────────────────────

/// Distribuye energía de pools padre a hijos según funciones de extracción.
/// Invariante: `Σ extracted ≤ available_for_extraction(pool)` por tick.
pub fn pool_distribution_system(
    mut pools:    Query<&mut EnergyPool>,
    links:        Query<(Entity, &PoolParentLink)>,
    mut energies: Query<&mut BaseEnergy>,
    mut ledgers:  Query<&mut PoolConservationLedger>,
    mut commands: Commands,
) {
    // ── Fase 1: recolectar todos los hijos en buffer stack ────────────────────
    let placeholder = ChildEntry::placeholder();
    let mut buf     = [placeholder; MAX_ENTRIES];
    let mut count   = 0usize;

    for (entity, link) in links.iter() {
        if count < MAX_ENTRIES {
            buf[count] = ChildEntry {
                parent_idx: link.parent().index(),
                parent:     link.parent(),
                child:      entity,
                etype:      link.extraction_type(),
                param:      link.primary_param(),
            };
            count += 1;
        }
    }

    if count == 0 {
        return;
    }

    // Ordenar por parent_idx → grupos contiguos deterministas.
    buf[..count].sort_unstable_by_key(|e| e.parent_idx);

    // ── Fase 2: procesar grupos ───────────────────────────────────────────────
    let mut orphans:      [Entity; MAX_ENTRIES] = [Entity::PLACEHOLDER; MAX_ENTRIES];
    let mut orphan_count: usize = 0;

    let mut i = 0;
    while i < count {
        let parent_idx = buf[i].parent_idx;
        let parent     = buf[i].parent;

        // Encontrar fin del grupo (mismo parent_idx).
        let mut j = i + 1;
        while j < count && buf[j].parent_idx == parent_idx {
            j += 1;
        }
        let group_len = (j - i).min(MAX_CHILDREN_PER_POOL);

        // Verificar que el padre existe.
        let Ok(mut pool) = pools.get_mut(parent) else {
            for k in i..j {
                if orphan_count < MAX_ENTRIES {
                    orphans[orphan_count] = buf[k].child;
                    orphan_count += 1;
                }
            }
            i = j;
            continue;
        };

        // ── Contexto de extracción ────────────────────────────────────────────
        let n_siblings   = group_len as u32;
        let total_fitness: f32 = buf[i..i + group_len]
            .iter()
            .filter(|e| matches!(e.etype, ExtractionType::Competitive))
            .map(|e| e.param)
            .sum();

        let available  = available_for_extraction(pool.pool(), 0.0, pool.dissipation_rate());
        let pool_ratio = pool.pool_ratio();

        let ctx = ExtractionContext { available, pool_ratio, n_siblings, total_fitness };

        // ── Evaluar claimed por hijo ──────────────────────────────────────────
        let mut claimed = [0.0f32; MAX_CHILDREN_PER_POOL];
        for (k, entry) in buf[i..i + group_len].iter().enumerate() {
            let profile = ExtractionProfile {
                base:        entry.etype,
                primary_param: entry.param,
                modifiers:   [None; MAX_EXTRACTION_MODIFIERS],
            };
            claimed[k] = evaluate_extraction(&profile, &ctx);
        }

        // ── Escalar para respetar el pool invariant ───────────────────────────
        scale_extractions_to_available(&mut claimed[..group_len], available);

        let total_claimed: f32 = claimed[..group_len].iter().sum();

        // ── Actualizar pool (distribución + disipación) ───────────────────────
        let loss      = dissipation_loss(pool.pool(), pool.dissipation_rate());
        let new_pool  = (pool.pool() - total_claimed - loss).max(0.0);
        if pool.pool() != new_pool {
            pool.set_pool(new_pool);
        }

        debug_assert!(
            total_claimed <= available + crate::blueprint::constants::POOL_CONSERVATION_EPSILON,
            "Pool invariant violated: claimed={total_claimed} > available={available}"
        );

        // ── EC-6C: Write conservation ledger ────────────────────────────────────
        let new_ledger = PoolConservationLedger::new(
            total_claimed,
            loss,
            pool.intake_rate() - total_claimed - loss,
            group_len as u16,
        );
        if let Ok(mut existing) = ledgers.get_mut(parent) {
            if *existing != new_ledger {
                *existing = new_ledger;
            }
        } else {
            commands.entity(parent).insert(new_ledger);
        }

        // ── Type IV: daño a capacidad (después de distribución) ───────────────
        for entry in buf[i..i + group_len].iter() {
            if entry.etype == ExtractionType::Aggressive {
                let profile = ExtractionProfile {
                    base:          entry.etype,
                    primary_param: entry.param,
                    modifiers:     [None; MAX_EXTRACTION_MODIFIERS],
                };
                let (_, damage) = evaluate_aggressive_extraction(&profile, &ctx, DAMAGE_RATE_DEFAULT);
                pool.degrade_capacity(damage);
            }
        }

        // ── Inyectar energía a hijos ──────────────────────────────────────────
        for (k, entry) in buf[i..i + group_len].iter().enumerate() {
            if let Ok(mut energy) = energies.get_mut(entry.child) {
                energy.inject(claimed[k]);
            }
        }

        i = j;
    }

    // ── Fase 3: remover links huérfanos ───────────────────────────────────────
    for orphan in &orphans[..orphan_count] {
        commands.entity(*orphan).remove::<PoolParentLink>();
    }
}

// ─── EC-4C: Dissipation ───────────────────────────────────────────────────────

/// Aplica disipación a pools que no tienen hijos activos.
/// Pools con hijos ya disiparon en `pool_distribution_system`.
pub fn pool_dissipation_system(
    mut pools:   Query<(Entity, &mut EnergyPool)>,
    child_links: Query<&PoolParentLink>,
) {
    for (pool_entity, mut pool) in &mut pools {
        let has_child = child_links.iter().any(|link| link.parent() == pool_entity);
        if has_child {
            continue; // ya disipó en pool_distribution_system
        }
        let loss     = dissipation_loss(pool.pool(), pool.dissipation_rate());
        let new_pool = (pool.pool() - loss).max(0.0);
        if pool.pool() != new_pool {
            pool.set_pool(new_pool);
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.register_type::<EnergyPool>();
        app.register_type::<PoolParentLink>();
        app.register_type::<BaseEnergy>();
        app
    }

    // ── EC-4A: Intake ────────────────────────────────────────────────────────

    #[test]
    fn pool_intake_system_adds_intake_rate() {
        let mut app = make_app();
        app.add_systems(Update, pool_intake_system);

        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 50.0, 0.01)).id();
        app.update();

        let pool = app.world().get::<EnergyPool>(parent).unwrap();
        assert!((pool.pool() - 1050.0).abs() < 1e-3, "pool={}", pool.pool());
    }

    #[test]
    fn pool_intake_system_clamped_to_capacity() {
        let mut app = make_app();
        app.add_systems(Update, pool_intake_system);

        let parent = app.world_mut().spawn(EnergyPool::new(1990.0, 2000.0, 50.0, 0.01)).id();
        app.update();

        let pool = app.world().get::<EnergyPool>(parent).unwrap();
        assert!((pool.pool() - 2000.0).abs() < 1e-3, "pool={}", pool.pool());
    }

    // ── EC-4B: Distribution ──────────────────────────────────────────────────

    #[test]
    fn pool_distribution_three_proportional_children() {
        let mut app = make_app();
        app.add_systems(Update, (pool_intake_system, pool_distribution_system.after(pool_intake_system)));

        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 0.0, 0.001)).id();
        let child_a = app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
        )).id();
        let child_b = app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
        )).id();
        let child_c = app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
        )).id();

        app.update();

        let ea = app.world().get::<BaseEnergy>(child_a).unwrap().qe();
        let eb = app.world().get::<BaseEnergy>(child_b).unwrap().qe();
        let ec = app.world().get::<BaseEnergy>(child_c).unwrap().qe();

        // Each gets ~1000/3 ≈ 333 (minus small dissipation)
        assert!(ea > 300.0 && ea < 340.0, "ea={ea}");
        assert!((ea - eb).abs() < 1.0, "ea={ea} eb={eb}");
        assert!((eb - ec).abs() < 1.0, "eb={eb} ec={ec}");
    }

    #[test]
    fn pool_distribution_competitive_fitness_split() {
        let mut app = make_app();
        app.add_systems(Update, (pool_intake_system, pool_distribution_system.after(pool_intake_system)));

        // pool=1000, 2 children: fitness 0.7 and 0.3
        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 0.0, 0.001)).id();
        let child_strong = app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Competitive, 0.7),
        )).id();
        let child_weak = app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Competitive, 0.3),
        )).id();

        app.update();

        let e_strong = app.world().get::<BaseEnergy>(child_strong).unwrap().qe();
        let e_weak   = app.world().get::<BaseEnergy>(child_weak).unwrap().qe();

        // strong gets ~70% of available, weak ~30%
        assert!(e_strong > e_weak * 2.0, "strong={e_strong} weak={e_weak}");
    }

    #[test]
    fn pool_distribution_scaling_enforces_pool_invariant() {
        let mut app = make_app();
        app.add_systems(Update, (pool_intake_system, pool_distribution_system.after(pool_intake_system)));

        // pool=100, 2 greedy children each wanting 80 → scaling needed
        let parent = app.world_mut().spawn(EnergyPool::new(100.0, 2000.0, 0.0, 0.001)).id();
        let child_a = app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Greedy, 80.0),
        )).id();
        let child_b = app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Greedy, 80.0),
        )).id();

        app.update();

        let ea = app.world().get::<BaseEnergy>(child_a).unwrap().qe();
        let eb = app.world().get::<BaseEnergy>(child_b).unwrap().qe();
        let pool_left = app.world().get::<EnergyPool>(parent).unwrap().pool();

        // Total extracted must not exceed original pool (approx 100 - dissipation)
        assert!(ea + eb <= 100.0 + 1e-3, "ea={ea} eb={eb} sum={}", ea + eb);
        assert!(ea > 40.0 && ea < 60.0, "ea={ea}");
        assert!((ea - eb).abs() < 1.0, "ea={ea} eb={eb}");
        assert!(pool_left >= 0.0, "pool_left={pool_left}");
    }

    #[test]
    fn pool_distribution_type_iv_degrades_capacity() {
        let mut app = make_app();
        app.add_systems(Update, (pool_intake_system, pool_distribution_system.after(pool_intake_system)));

        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 0.0, 0.001)).id();
        let _child = app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Aggressive, 0.5),
        )).id();

        let cap_before = app.world().get::<EnergyPool>(parent).unwrap().capacity();
        app.update();
        let cap_after = app.world().get::<EnergyPool>(parent).unwrap().capacity();

        assert!(cap_after < cap_before, "capacity should degrade: before={cap_before} after={cap_after}");
    }

    #[test]
    fn pool_distribution_orphan_link_removed_without_panic() {
        let mut app = make_app();
        app.add_systems(Update, (pool_intake_system, pool_distribution_system.after(pool_intake_system)));

        // Create child pointing to a non-existent parent
        let fake_parent = Entity::from_raw(9999);
        let child = app.world_mut().spawn((
            BaseEnergy::new(100.0),
            PoolParentLink::new(fake_parent, ExtractionType::Proportional, 0.0),
        )).id();

        app.update(); // must not panic
        app.update(); // link should be removed after first update's commands are flushed

        // Link should be removed
        assert!(app.world().get::<PoolParentLink>(child).is_none(), "orphan link should be removed");
    }

    #[test]
    fn pool_distribution_pool_invariant_holds() {
        let mut app = make_app();
        app.add_systems(Update, (pool_intake_system, pool_distribution_system.after(pool_intake_system)));

        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 0.0, 0.001)).id();
        // 4 mixed children
        app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(parent, ExtractionType::Proportional, 0.0)));
        app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(parent, ExtractionType::Greedy, 300.0)));
        app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(parent, ExtractionType::Competitive, 0.5)));
        app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(parent, ExtractionType::Regulated, 100.0)));

        for _ in 0..10 {
            app.update();
        }

        let pool = app.world().get::<EnergyPool>(parent).unwrap();
        assert!(pool.pool() >= 0.0, "pool must never go negative");
    }

    // ── EC-4C: Dissipation ───────────────────────────────────────────────────

    #[test]
    fn pool_dissipation_system_applies_to_childless_pool() {
        let mut app = make_app();
        app.add_systems(Update, pool_dissipation_system);

        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 0.0, 0.01)).id();
        app.update();

        let pool = app.world().get::<EnergyPool>(parent).unwrap();
        // loss = 1000 * 0.01 = 10 → pool = 990
        assert!((pool.pool() - 990.0).abs() < 1e-3, "pool={}", pool.pool());
    }

    #[test]
    fn pool_dissipation_system_skips_pool_with_children() {
        let mut app = make_app();
        app.add_systems(Update, pool_dissipation_system);

        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 0.0, 0.01)).id();
        // spawn a child linking to this pool
        app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
        ));

        app.update();

        // dissipation_system should skip this pool (has child) — pool unchanged
        let pool = app.world().get::<EnergyPool>(parent).unwrap();
        assert!((pool.pool() - 1000.0).abs() < 1e-3, "pool should be unchanged: {}", pool.pool());
    }

    // ── EC-4E: Pipeline smoke test ───────────────────────────────────────────

    #[test]
    fn pipeline_minimal_app_runs_without_crash() {
        let mut app = make_app();
        app.add_systems(Update, (
            pool_intake_system,
            pool_distribution_system.after(pool_intake_system),
            pool_dissipation_system.after(pool_distribution_system),
        ));

        let parent = app.world_mut().spawn(EnergyPool::new(500.0, 1000.0, 10.0, 0.01)).id();
        app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
        ));

        for _ in 0..10 {
            app.update();
        }

        // Must not panic and pool must stay valid
        let pool = app.world().get::<EnergyPool>(parent).unwrap();
        assert!(pool.pool() >= 0.0);
        assert!(pool.pool() <= pool.capacity());
    }
}
