use crate::blueprint::constants;
use crate::layers::BaseEnergy;
use crate::worldgen::field_grid::EnergyFieldGrid;
use bevy::prelude::*;

/// Define la modalidad sensorial para la transducción pura.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensoryModality {
    /// Fotones (ondas electromagnéticas).
    Vision,
    /// Presión mecánica en fluidos/sólidos.
    Audition,
    /// Dispersión térmica.
    Thermo,
}

/// Perfil de un receptor sensorial.
/// Un `ArtefactoReceptor` puede tener múltiples de estos (Polivalencia).
#[derive(Debug, Clone, Copy)]
pub struct SensoryProfile {
    pub modality: SensoryModality,
    pub spectrum_range: (f32, f32), // Hz
    pub activation_threshold: f32,  // min energy
    pub saturation_limit: f32,      // max energy before collapse
}

/// Componente ECS que define un órgano o artefacto capaz de percibir energía.
/// Implementa Polivalencia Sensorial mediante un arreglo de perfiles.
#[derive(Component, Debug, Clone)]
pub struct ArtefactoReceptor {
    pub profiles: Vec<SensoryProfile>,
}

/// Función Pura de Transducción Sensorial (Stateless).
/// Retorna el Nivel de Estímulo en el rango [0.0, 1.0].
#[inline]
pub fn transduce_signal(incident_hz: f32, incident_energy: f32, profile: &SensoryProfile) -> f32 {
    if !incident_hz.is_finite() || !incident_energy.is_finite() {
        return 0.0;
    }

    // Fuera de rango espectral: ciego/sordo a esta frecuencia
    if incident_hz < profile.spectrum_range.0 || incident_hz > profile.spectrum_range.1 {
        return 0.0;
    }

    // Por debajo del umbral de energía: imperceptible
    if incident_energy < profile.activation_threshold {
        return 0.0;
    }

    // Saturación: ceguera temporal o sordera (colapso predictivo local)
    if incident_energy > profile.saturation_limit {
        return 0.0; // Colapso por exceso de energía
    }

    // Estímulo normalizado
    let range = profile.saturation_limit - profile.activation_threshold;
    if range <= f32::EPSILON {
        return 1.0;
    }

    ((incident_energy - profile.activation_threshold) / range).clamp(0.0, 1.0)
}

// AttentionGrid — canonical definition in runtime_platform/contracts/ (DC-4B)
pub use crate::runtime_platform::contracts::AttentionGrid;

/// Sistema de superposición competitiva de atención (\max).
/// Lee el EnergyFieldGrid, evalúa los ArtefactoReceptor de las entidades,
/// y escribe el nivel de atención dominante en el AttentionGrid.
pub fn attention_convergence_system(
    query: Query<(&GlobalTransform, &ArtefactoReceptor)>,
    energy_grid: Option<Res<EnergyFieldGrid>>,
    mut attention: ResMut<AttentionGrid>,
) {
    let Some(energy) = energy_grid else {
        // Sin campo: evitar atención stale de ticks anteriores (consumidores LOD / gating).
        for v in attention.a.iter_mut() {
            *v = 0.0;
        }
        return;
    };

    // Inicializar o limpiar el grid de atención
    let w = energy.width as usize;
    let h = energy.height as usize;
    attention.resize(w, h, energy.cell_size, energy.origin);
    for v in attention.a.iter_mut() {
        *v = 0.0;
    }

    // Superposición competitiva por max()
    for (transform, receptor) in query.iter() {
        let pos = transform.translation();
        let world_xz = Vec2::new(pos.x, pos.z);

        let Some((cx_u, cy_u)) = energy.cell_coords(world_xz) else {
            continue;
        };

        let (local_hz, local_energy) = energy
            .cell_xy(cx_u, cy_u)
            .map(|cell| (cell.dominant_frequency_hz, cell.accumulated_qe))
            .unwrap_or((0.0, 0.0));

        let cx = cx_u as usize;
        let cy = cy_u as usize;
        let cell_idx = attention.idx(cx, cy);

        let mut max_stimulus = 0.0f32;
        for profile in &receptor.profiles {
            let stimulus = transduce_signal(local_hz, local_energy, profile);
            if stimulus > max_stimulus {
                max_stimulus = stimulus;
            }
        }

        // max() consolidado en la celda
        if max_stimulus.is_finite() && max_stimulus > attention.a[cell_idx] {
            attention.a[cell_idx] = max_stimulus;
        }
    }
}

#[cfg(test)]
mod sensory_fix_tests {
    use super::{
        ArtefactoReceptor, AttentionGrid, SensoryModality, SensoryProfile,
        attention_convergence_system, transduce_signal,
    };
    use crate::worldgen::EnergyFieldGrid;
    use bevy::math::Vec2;
    use bevy::prelude::*;

    #[test]
    fn attention_convergence_reads_real_field_energy() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::new(0.0, 0.0));
        if let Some(cell) = grid.cell_xy_mut(1, 1) {
            cell.accumulated_qe = 100.0;
            cell.dominant_frequency_hz = 450.0;
        }
        app.insert_resource(grid);
        app.init_resource::<AttentionGrid>();

        let receptor = ArtefactoReceptor {
            profiles: vec![SensoryProfile {
                modality: SensoryModality::Vision,
                spectrum_range: (400.0, 500.0),
                activation_threshold: 10.0,
                saturation_limit: 200.0,
            }],
        };
        let pos = Vec3::new(2.5, 0.0, 2.5);
        app.world_mut()
            .spawn((GlobalTransform::from_translation(pos), receptor));

        app.add_systems(Update, attention_convergence_system);
        app.update();

        let attention = app.world().resource::<AttentionGrid>();
        let a = attention.a[attention.idx(1, 1)];
        assert!(a > 0.0, "Attention must reflect real field energy, got {a}");
    }

    #[test]
    fn attention_zero_when_cell_empty() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::new(0.0, 0.0));
        app.insert_resource(grid);
        app.init_resource::<AttentionGrid>();

        let receptor = ArtefactoReceptor {
            profiles: vec![SensoryProfile {
                modality: SensoryModality::Vision,
                spectrum_range: (400.0, 500.0),
                activation_threshold: 10.0,
                saturation_limit: 200.0,
            }],
        };
        app.world_mut().spawn((
            GlobalTransform::from_translation(Vec3::new(1.0, 0.0, 1.0)),
            receptor,
        ));

        app.add_systems(Update, attention_convergence_system);
        app.update();

        let attention = app.world().resource::<AttentionGrid>();
        let a = attention.a[attention.idx(0, 0)];
        assert_eq!(a, 0.0, "Empty cell must produce zero attention");
    }

    #[test]
    fn attention_oob_receptor_leaves_grid_cleared() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::new(0.0, 0.0));
        if let Some(cell) = grid.cell_xy_mut(1, 1) {
            cell.accumulated_qe = 999.0;
            cell.dominant_frequency_hz = 450.0;
        }
        app.insert_resource(grid);
        app.init_resource::<AttentionGrid>();

        let receptor = ArtefactoReceptor {
            profiles: vec![SensoryProfile {
                modality: SensoryModality::Vision,
                spectrum_range: (400.0, 500.0),
                activation_threshold: 10.0,
                saturation_limit: 200.0,
            }],
        };
        app.world_mut().spawn((
            GlobalTransform::from_translation(Vec3::new(500.0, 0.0, 500.0)),
            receptor,
        ));

        app.add_systems(Update, attention_convergence_system);
        app.update();

        let attention = app.world().resource::<AttentionGrid>();
        assert!(
            attention.a.iter().all(|&v| v == 0.0),
            "OOB receptor must not clamp attention into edge cells"
        );
    }

    #[test]
    fn transduce_signal_rejects_nonfinite_inputs() {
        let p = SensoryProfile {
            modality: SensoryModality::Vision,
            spectrum_range: (400.0, 500.0),
            activation_threshold: 10.0,
            saturation_limit: 200.0,
        };
        assert_eq!(transduce_signal(f32::NAN, 100.0, &p), 0.0);
        assert_eq!(transduce_signal(450.0, f32::INFINITY, &p), 0.0);
    }

    #[test]
    fn attention_stale_cleared_without_energy_field_grid() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<AttentionGrid>();
        {
            let mut att = app.world_mut().resource_mut::<AttentionGrid>();
            att.resize(2, 2, 1.0, Vec2::ZERO);
            att.a.fill(0.99);
        }

        app.add_systems(Update, attention_convergence_system);
        app.update();

        let attention = app.world().resource::<AttentionGrid>();
        assert!(
            attention.a.iter().all(|&v| v == 0.0),
            "Sin EnergyFieldGrid no debe quedar atención de frames previos"
        );
    }

    #[test]
    fn attention_convergence_with_shifted_grid_origin() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let origin = Vec2::new(-64.0, -64.0);
        let mut grid = EnergyFieldGrid::new(8, 8, 2.0, origin);
        if let Some(cell) = grid.cell_xy_mut(3, 4) {
            cell.accumulated_qe = 80.0;
            cell.dominant_frequency_hz = 450.0;
        }
        app.insert_resource(grid);
        app.init_resource::<AttentionGrid>();

        let receptor = ArtefactoReceptor {
            profiles: vec![SensoryProfile {
                modality: SensoryModality::Vision,
                spectrum_range: (400.0, 500.0),
                activation_threshold: 10.0,
                saturation_limit: 200.0,
            }],
        };
        // Centro celda (3,4): origin + (3.5, 4.5) * cell_size
        let wx = origin.x + 3.5 * 2.0;
        let wz = origin.y + 4.5 * 2.0;
        app.world_mut().spawn((
            GlobalTransform::from_translation(Vec3::new(wx, 0.0, wz)),
            receptor,
        ));

        app.add_systems(Update, attention_convergence_system);
        app.update();

        let attention = app.world().resource::<AttentionGrid>();
        let a = attention.a[attention.idx(3, 4)];
        assert!(
            a > 0.0,
            "Expected attention at (3,4) with shifted origin, got {a}"
        );
    }
}

/// Etiqueta y tracker cronológico para entidades ignoradas por el ECS pesado.
#[derive(Component, Debug, Default, Clone)]
pub struct QuantumSuspension {
    pub suspended_time: f32,       // Acumulador delta-t [segundos]
    pub cached_energy: BaseEnergy, // Copia congelada de la energía
}

/// Congela entidades en celdas con $A \le 0.1$ y transacciona el Colapso Predictivo si despiertan.
/// Usa un patrón Yanagi: remueve de cuajo el componente `BaseEnergy` para que los
/// sistemas posteriores (`ChemicalLayer`, `MetabolicLayer`) simplemente lo ignoren
/// sin tener que reescribir docenas de queries.
pub fn attention_gating_system(
    time: Res<Time>,
    attention: Res<AttentionGrid>,
    mut commands: Commands,
    mut awake_q: Query<(Entity, &GlobalTransform, &BaseEnergy)>,
    mut suspended_q: Query<(Entity, &GlobalTransform, &mut QuantumSuspension)>,
) {
    let dt = time.delta_secs();

    // Umbral Yanagi: Si no te miran y no sos ruidoso, entrás en coma.
    const ATTENTION_WAKE_THRESHOLD: f32 = 0.1;

    // 1. Asfixiar a los que escaparon de la atención
    for (entity, transform, energy) in &mut awake_q {
        let ep = transform.translation();
        let a = attention.get_attention(Vec2::new(ep.x, ep.z)); // mapeo XZ temporal

        if a <= ATTENTION_WAKE_THRESHOLD {
            commands.entity(entity).insert(QuantumSuspension {
                suspended_time: 0.0,
                cached_energy: energy.clone(),
            });
            commands.entity(entity).remove::<BaseEnergy>();
        }
    }

    // 2. Acumular coma y Despertar (Colapso Predictivo)
    for (entity, transform, mut susp) in &mut suspended_q {
        let ep = transform.translation();
        let a = attention.get_attention(Vec2::new(ep.x, ep.z));

        if a > ATTENTION_WAKE_THRESHOLD {
            // Matemática de Interpolación Pura
            // Simulamos una pérdida entrópica base por el tiempo suspendido,
            // resolviendo "de golpe" en vez de tick a tick.
            let entropy_drain_rate = constants::ATTENTION_ENTROPY_DRAIN_RATE;
            let lost_energy = susp.suspended_time * entropy_drain_rate;

            let mut restored = susp.cached_energy.clone();
            if lost_energy > 0.0 {
                restored.drain(lost_energy);
            }

            commands.entity(entity).insert(restored);
            commands.entity(entity).remove::<QuantumSuspension>();
        } else {
            susp.suspended_time += dt;
        }
    }
}
