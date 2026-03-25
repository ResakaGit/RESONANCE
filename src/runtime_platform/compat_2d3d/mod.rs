use bevy::prelude::*;

use crate::events::PathRequestEvent;
use crate::runtime_platform::camera_controller_3d::Camera3dEnabled;
use crate::runtime_platform::camera_controller_3d::Camera3dPlugin;
use crate::runtime_platform::click_to_move::ClickToMovePlugin;
use crate::runtime_platform::core_math_agnostic::DEFAULT_SIM_STANDING_Y;
use crate::runtime_platform::debug_observability::ObservabilityPlugin;
use crate::runtime_platform::input_capture::InputCapturePlugin;
use crate::runtime_platform::intent_projection_3d::{
    ProjectedWillIntent, apply_projected_intent_to_will_system, project_intent_to_resource_system,
};
use crate::runtime_platform::parry_nav_collider::ParryNavCollider;
use crate::runtime_platform::render_bridge_3d::RenderBridge3dPlugin;
use crate::runtime_platform::scenario_isolation::ScenarioIsolationPlugin;
use crate::runtime_platform::simulation_tick::SimulationTickPlugin;
use crate::simulation::InputChannelSet;
use crate::simulation::input::will_input_system;
use crate::simulation::pathfinding::{
    clear_nav_paths_when_no_move_target_system, emit_path_request_on_goal_change_system,
    pathfinding_compute_system,
};
use crate::simulation::states::{GameState, PlayState};
use oxidized_navigation::{NavMeshSettings, OxidizedNavigationPlugin};

/// Perfil de compatibilidad simulación 2D legacy vs. pipeline híbrido / visual 3D.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderCompatProfile {
    /// Solo 2D legacy: sin captura de input extendida ni visual 3D.
    #[default]
    Legacy2dOnly,
    /// Input/tick alineados al runtime moderno, visual principal sigue en 2D.
    Hybrid,
    /// Cámara 3D + render bridge activos.
    Full3dVisual,
}

impl RenderCompatProfile {
    #[inline]
    fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "legacy2d" | "legacy_2d" | "legacy" => Self::Legacy2dOnly,
            "hybrid" => Self::Hybrid,
            "full3d" | "full_3d" | "3d" => Self::Full3dVisual,
            _ => Self::default(),
        }
    }

    fn read_env_raw() -> String {
        std::env::var("RESONANCE_RENDER_COMPAT_PROFILE")
            .or_else(|_| std::env::var("RESONANCE_V6_PROFILE"))
            .unwrap_or_default()
    }

    /// Lee perfil desde `RESONANCE_RENDER_COMPAT_PROFILE`, con fallback a
    /// `RESONANCE_V6_PROFILE` por compatibilidad con despliegues existentes.
    /// Si ambas faltan o vienen vacías → [`Legacy2dOnly`] (compat librerías/tests).
    /// Valores aceptados: `legacy2d`, `hybrid`, `full3d`.
    pub fn from_env() -> Self {
        let raw = Self::read_env_raw();
        if raw.trim().is_empty() {
            return Self::default();
        }
        Self::parse(&raw)
    }

    /// Misma lectura de entorno que [`from_env`], pero si está vacío → [`Full3dVisual`]
    /// (**core demo 3D** del binario `resonance`).
    pub fn from_env_or_core_3d_default() -> Self {
        let raw = Self::read_env_raw();
        if raw.trim().is_empty() {
            Self::Full3dVisual
        } else {
            Self::parse(&raw)
        }
    }

    #[inline]
    pub fn enables_input_capture(self) -> bool {
        !matches!(self, Self::Legacy2dOnly)
    }

    #[inline]
    pub fn enables_visual_3d(self) -> bool {
        matches!(self, Self::Full3dVisual)
    }

    #[inline]
    pub fn enables_observability_plugin(self) -> bool {
        !matches!(self, Self::Legacy2dOnly)
    }

    #[inline]
    pub fn enables_scenario_isolation(self) -> bool {
        !matches!(self, Self::Legacy2dOnly)
    }

    #[cfg(test)]
    fn from_env_or_core_3d_default_logic_for_test(raw: &str) -> Self {
        if raw.trim().is_empty() {
            Self::Full3dVisual
        } else {
            Self::parse(raw)
        }
    }
}

/// Cómo se mapea el plano sim 2D al `Transform` 3D (Y-up Bevy).
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct SimWorldTransformParams {
    /// `true`: pos sim `(sx, sy)` → `Transform(sx, standing_y, sy)`; velocidad → `vec2_to_xz`.
    pub use_xz_ground: bool,
    pub standing_y: f32,
}

impl Default for SimWorldTransformParams {
    fn default() -> Self {
        Self {
            use_xz_ground: false,
            standing_y: DEFAULT_SIM_STANDING_Y,
        }
    }
}

impl SimWorldTransformParams {
    pub fn from_profile(profile: RenderCompatProfile) -> Self {
        if profile.enables_visual_3d() {
            Self {
                use_xz_ground: true,
                standing_y: DEFAULT_SIM_STANDING_Y,
            }
        } else {
            Self::default()
        }
    }

    /// Celda materializada del grid: legacy en XY+Z=0; full3d en XZ con sprite acostado (rot X).
    pub fn materialized_tile_transform(&self, world_pos: Vec2) -> Transform {
        // Por encima del V6GroundPlane para z-fighting mínimo.
        const TILE_EPS_Y: f32 = 0.04;
        if self.use_xz_ground {
            Transform {
                translation: Vec3::new(world_pos.x, self.standing_y + TILE_EPS_Y, world_pos.y),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                scale: Vec3::ONE,
            }
        } else {
            Transform::from_xyz(world_pos.x, world_pos.y, 0.0)
        }
    }
}

/// Escribe el perfil activo como recurso único de compatibilidad 2D/3D.
pub struct Compat2d3dPlugin {
    pub profile: RenderCompatProfile,
}

impl Default for Compat2d3dPlugin {
    fn default() -> Self {
        Self {
            profile: RenderCompatProfile::from_env_or_core_3d_default(),
        }
    }
}

impl Plugin for Compat2d3dPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.profile)
            .insert_resource(SimWorldTransformParams::from_profile(self.profile));
    }
}

/// Registra plugins de plataforma según `RenderCompatProfile`.
pub fn add_compat_plugins_by_profile(app: &mut App) {
    let profile = app
        .world()
        .get_resource::<RenderCompatProfile>()
        .copied()
        .unwrap_or_default();

    if profile.enables_input_capture() {
        app.add_plugins(InputCapturePlugin)
            .init_resource::<ProjectedWillIntent>();

        // Hybrid: `apply_projected_intent_to_will` necesita estos recursos sin cargar ClickToMovePlugin completo.
        if !profile.enables_visual_3d() {
            use crate::runtime_platform::click_to_move::{ClickToMoveConfig, MoveTargetState};
            app.init_resource::<ClickToMoveConfig>()
                .init_resource::<MoveTargetState>();
        }

        let run_playing = in_state(GameState::Playing).and(in_state(PlayState::Active));

        if profile.enables_visual_3d() {
            app.add_event::<PathRequestEvent>()
                .add_plugins(OxidizedNavigationPlugin::<ParryNavCollider>::new(
                    NavMeshSettings::from_agent_and_bounds(0.45, 1.85, 90.0, -4.0),
                ))
                .add_systems(
                    FixedUpdate,
                    (
                        clear_nav_paths_when_no_move_target_system,
                        emit_path_request_on_goal_change_system
                            .after(clear_nav_paths_when_no_move_target_system),
                        pathfinding_compute_system
                            .after(emit_path_request_on_goal_change_system)
                            .after(clear_nav_paths_when_no_move_target_system),
                        project_intent_to_resource_system.after(pathfinding_compute_system),
                        apply_projected_intent_to_will_system
                            .after(project_intent_to_resource_system),
                    )
                        .chain()
                        .in_set(InputChannelSet::PlatformWill)
                        .run_if(run_playing.clone()),
                );
        } else {
            app.add_systems(
                FixedUpdate,
                (
                    project_intent_to_resource_system,
                    apply_projected_intent_to_will_system.after(project_intent_to_resource_system),
                )
                    .chain()
                    .in_set(InputChannelSet::PlatformWill)
                    .run_if(run_playing),
            );
        }
    } else {
        app.add_systems(
            FixedUpdate,
            will_input_system
                .in_set(InputChannelSet::PlatformWill)
                .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
        );
    }
    if profile.enables_visual_3d() {
        app.add_plugins(Camera3dPlugin)
            .add_plugins(ClickToMovePlugin)
            .add_plugins(RenderBridge3dPlugin);
        // `Camera3dPlugin` init `Camera3dEnabled(false)`; sin esto los sistemas del rig no corren y no hay cámara 3D.
        app.insert_resource(Camera3dEnabled(true));
    }
    if profile.enables_scenario_isolation() {
        app.add_plugins(ScenarioIsolationPlugin);
    }
    if profile.enables_observability_plugin() {
        app.add_plugins(ObservabilityPlugin);
    }

    // `SimulationTickPlugin` queda activo en todos los perfiles (contrato de tiempo único).
    app.add_plugins(SimulationTickPlugin);
}

/// Entrada pública: wiring de runtime según perfil 2D/3D.
pub fn add_runtime_platform_plugins_by_profile(app: &mut App) {
    add_compat_plugins_by_profile(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_is_legacy_2d_only() {
        assert_eq!(
            RenderCompatProfile::default(),
            RenderCompatProfile::Legacy2dOnly
        );
    }

    #[test]
    fn from_env_parses_known_profiles() {
        assert_eq!(
            RenderCompatProfile::parse("legacy2d"),
            RenderCompatProfile::Legacy2dOnly
        );
        assert_eq!(
            RenderCompatProfile::parse("hybrid"),
            RenderCompatProfile::Hybrid
        );
        assert_eq!(
            RenderCompatProfile::parse("full3d"),
            RenderCompatProfile::Full3dVisual
        );
    }

    #[test]
    fn helper_flags_match_profile_contract() {
        assert!(!RenderCompatProfile::Legacy2dOnly.enables_input_capture());
        assert!(RenderCompatProfile::Hybrid.enables_input_capture());
        assert!(RenderCompatProfile::Full3dVisual.enables_visual_3d());
    }

    #[test]
    fn core_3d_default_empty_raw_is_full_visual() {
        assert_eq!(
            RenderCompatProfile::from_env_or_core_3d_default_logic_for_test(""),
            RenderCompatProfile::Full3dVisual
        );
        assert_eq!(
            RenderCompatProfile::from_env_or_core_3d_default_logic_for_test("   "),
            RenderCompatProfile::Full3dVisual
        );
        assert_eq!(
            RenderCompatProfile::from_env_or_core_3d_default_logic_for_test("legacy2d"),
            RenderCompatProfile::Legacy2dOnly
        );
        assert_eq!(
            RenderCompatProfile::from_env_or_core_3d_default_logic_for_test("full3d"),
            RenderCompatProfile::Full3dVisual
        );
    }
}
