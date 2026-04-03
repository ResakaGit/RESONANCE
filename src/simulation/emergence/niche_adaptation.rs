//! ET-9: Niche adaptation — competitive pressure drives character displacement.
//!
//! Entities with overlapping niches displace apart (Axiom 3: competition as primitive).
//! Uses NicheProfile pub fields: center[4], width[4], displacement_rate.

use bevy::prelude::*;

use crate::blueprint::equations::emergence::niche::{
    character_displacement, competitive_pressure, niche_overlap,
};
use crate::layers::NicheProfile;
use crate::world::SpatialIndex;

use crate::blueprint::constants::emergence::{
    NICHE_OVERLAP_DISPLACEMENT_THRESHOLD, NICHE_SCAN_RADIUS,
};

/// Displaces niche centers when competitors overlap.
pub fn niche_adaptation_system(
    mut query: Query<(Entity, &mut NicheProfile, &Transform)>,
    spatial: Res<SpatialIndex>,
) {
    // Collect niche data for read-only neighbor lookup (avoids aliasing)
    let niche_data: Vec<(Entity, [f32; 4], [f32; 4])> = query
        .iter()
        .map(|(e, niche, _)| (e, niche.center, niche.width))
        .collect();

    for (entity, mut niche, transform) in &mut query {
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        let neighbors = spatial.query_radius(pos, NICHE_SCAN_RADIUS);

        let mut displacement = [0.0f32; 4];
        let mut count = 0u32;

        for neighbor in &neighbors {
            if neighbor.entity == entity {
                continue;
            }
            let Some((_, n_center, n_width)) =
                niche_data.iter().find(|(e, _, _)| *e == neighbor.entity)
            else {
                continue;
            };

            let overlap = niche_overlap(niche.center, niche.width, *n_center, *n_width);
            if overlap < NICHE_OVERLAP_DISPLACEMENT_THRESHOLD {
                continue;
            }

            let pressure = competitive_pressure(overlap, 1.0, 1.0);
            for dim in 0..4 {
                displacement[dim] +=
                    character_displacement(niche.center[dim], n_center[dim], pressure);
            }
            count += 1;
        }

        if count == 0 {
            continue;
        }

        let rate = niche.displacement_rate;
        let mut changed = false;
        for dim in 0..4 {
            let delta = displacement[dim] / count as f32 * rate;
            if delta.abs() > 1e-5 {
                niche.center[dim] += delta;
                changed = true;
            }
        }
        let _ = changed; // mutation guard is per-field above
    }
}
