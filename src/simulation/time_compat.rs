use bevy::prelude::*;

/// Delta de simulación: prioriza `Time<Fixed>` si existe (pipeline nominal), si no `Time` (tests mínimos).
#[inline]
pub fn simulation_delta_secs(fixed: Option<Res<Time<Fixed>>>, time: &Time) -> f32 {
    if let Some(ft) = fixed {
        let d = ft.delta_secs();
        if d > 0.0 {
            return d;
        }
    }
    time.delta_secs()
}
